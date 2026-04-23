use arrow::buffer::NullBuffer;
use arrow_array::{
    Array, ArrayRef, FixedSizeListArray, Float64Array, ListArray, RecordBatch, StructArray,
    builder::{Float64Builder, ListBuilder},
};
use arrow_schema::{DataType, Field, FieldRef, Schema, SchemaRef};
use cityjson::v2_0::OwnedCityModel;
use cityjson_arrow::error::{Error, Result};
use cityjson_arrow::internal::{
    CanonicalTable, CanonicalTableSink, IncrementalDecoder, canonical_table_order,
    canonical_table_position, concat_record_batches, emit_tables, schema_for_table,
};
use cityjson_arrow::schema::{
    CityArrowHeader, CityArrowPackageVersion, ProjectionLayout, canonical_schema_set,
};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::arrow_writer::ArrowWriter;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const DATASET_MANIFEST: &str = "manifest.json";
const TABLE_DIR: &str = "tables";
const PARQUET_EXTENSION: &str = "parquet";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParquetDatasetTableRef {
    pub name: String,
    pub path: PathBuf,
    pub rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParquetDatasetManifest {
    pub package_schema: CityArrowPackageVersion,
    pub cityjson_version: String,
    pub citymodel_id: String,
    pub projection: ProjectionLayout,
    pub tables: Vec<ParquetDatasetTableRef>,
}

impl ParquetDatasetManifest {
    #[must_use]
    pub fn new(
        citymodel_id: impl Into<String>,
        cityjson_version: impl Into<String>,
        projection: ProjectionLayout,
    ) -> Self {
        Self {
            package_schema: CityArrowPackageVersion::V3Alpha3,
            cityjson_version: cityjson_version.into(),
            citymodel_id: citymodel_id.into(),
            projection,
            tables: Vec::new(),
        }
    }
}

impl From<&ParquetDatasetManifest> for CityArrowHeader {
    fn from(value: &ParquetDatasetManifest) -> Self {
        Self::new(
            value.package_schema,
            value.citymodel_id.clone(),
            value.cityjson_version.clone(),
        )
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ParquetDatasetWriter;

impl ParquetDatasetWriter {
    /// Writes one native Parquet file per canonical table under `path`.
    ///
    /// # Errors
    ///
    /// Returns an error when model conversion or Parquet serialization fails.
    pub fn write_dir(
        &self,
        path: impl AsRef<Path>,
        model: &OwnedCityModel,
    ) -> Result<ParquetDatasetManifest> {
        write_parquet_dataset_model_dir(path, model)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ParquetDatasetReader;

impl ParquetDatasetReader {
    /// Reads a native Parquet canonical-table dataset into a city model.
    ///
    /// # Errors
    ///
    /// Returns an error when the dataset manifest or table files cannot be
    /// read or decoded.
    pub fn read_dir(&self, path: impl AsRef<Path>) -> Result<OwnedCityModel> {
        read_parquet_dataset_model_dir(path)
    }

    /// Reads only the dataset manifest.
    ///
    /// # Errors
    ///
    /// Returns an error when the manifest cannot be read.
    pub fn read_manifest(&self, path: impl AsRef<Path>) -> Result<ParquetDatasetManifest> {
        read_parquet_dataset_manifest(path)
    }
}

struct DatasetSink {
    root: PathBuf,
    manifest: Option<ParquetDatasetManifest>,
}

impl DatasetSink {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            manifest: None,
        }
    }

    fn finish(self) -> Result<ParquetDatasetManifest> {
        let manifest = self
            .manifest
            .ok_or_else(|| Error::Conversion("dataset manifest was not initialized".to_string()))?;
        validate_dataset_manifest(&manifest)?;
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
        fs::write(self.root.join(DATASET_MANIFEST), manifest_bytes)?;
        Ok(manifest)
    }
}

impl CanonicalTableSink for DatasetSink {
    fn start(&mut self, header: &CityArrowHeader, projection: &ProjectionLayout) -> Result<()> {
        fs::create_dir_all(self.root.join(TABLE_DIR))?;
        self.manifest = Some(ParquetDatasetManifest::new(
            header.citymodel_id.clone(),
            header.cityjson_version.clone(),
            projection.clone(),
        ));
        Ok(())
    }

    fn push_batch(&mut self, table: CanonicalTable, batch: RecordBatch) -> Result<()> {
        let relative_path = table_path(table);
        let absolute_path = self.root.join(&relative_path);
        let file = File::create(&absolute_path)?;
        write_parquet_batch(file, &batch)?;
        self.manifest
            .as_mut()
            .ok_or_else(|| Error::Conversion("dataset manifest was not initialized".to_string()))?
            .tables
            .push(ParquetDatasetTableRef {
                name: table.as_str().to_string(),
                path: relative_path,
                rows: batch.num_rows(),
            });
        Ok(())
    }
}

#[doc(hidden)]
pub fn write_parquet_dataset_model_dir(
    path: impl AsRef<Path>,
    model: &OwnedCityModel,
) -> Result<ParquetDatasetManifest> {
    let root = path.as_ref();
    prepare_dataset_root(root)?;
    let mut sink = DatasetSink::new(root.to_path_buf());
    emit_tables(model, &mut sink)?;
    sink.finish()
}

#[doc(hidden)]
pub fn read_parquet_dataset_model_dir(path: impl AsRef<Path>) -> Result<OwnedCityModel> {
    let root = path.as_ref();
    let manifest = read_parquet_dataset_manifest(root)?;
    validate_dataset_manifest(&manifest)?;
    let schemas = canonical_schema_set(&manifest.projection);
    let mut decoder = IncrementalDecoder::new(
        CityArrowHeader::from(&manifest),
        manifest.projection.clone(),
    )?;

    for table_ref in &manifest.tables {
        let table = CanonicalTable::parse(&table_ref.name)?;
        let batch = read_parquet_table(
            &root.join(&table_ref.path),
            schema_for_table(&schemas, table),
            table,
            table_ref.rows,
        )?;
        decoder.push_batch(table, &batch)?;
    }

    decoder.finish()
}

/// Reads only the manifest from a native Parquet dataset.
///
/// # Errors
///
/// Returns an error when the dataset manifest cannot be read or parsed.
pub fn read_parquet_dataset_manifest(path: impl AsRef<Path>) -> Result<ParquetDatasetManifest> {
    let manifest_path = path.as_ref().join(DATASET_MANIFEST);
    let manifest_bytes = fs::read(manifest_path)?;
    serde_json::from_slice(&manifest_bytes).map_err(Error::from)
}

fn prepare_dataset_root(root: &Path) -> Result<()> {
    if root.exists() && !root.is_dir() {
        return Err(Error::Unsupported(format!(
            "{} exists but is not a directory",
            root.display()
        )));
    }
    fs::create_dir_all(root.join(TABLE_DIR))?;
    Ok(())
}

fn table_path(table: CanonicalTable) -> PathBuf {
    PathBuf::from(TABLE_DIR).join(format!("{}.{}", table.as_str(), PARQUET_EXTENSION))
}

fn write_parquet_batch(file: File, batch: &RecordBatch) -> Result<()> {
    let batch = parquet_batch_from_arrow(batch)?;
    let mut writer = ArrowWriter::try_new(file, batch.schema(), None)?;
    writer.write(&batch)?;
    writer.close()?;
    Ok(())
}

fn read_parquet_table(
    path: &Path,
    expected_schema: &SchemaRef,
    table: CanonicalTable,
    expected_rows: usize,
) -> Result<RecordBatch> {
    if !path.is_file() {
        return Err(Error::Unsupported(format!(
            "{} table path {} is missing",
            table.as_str(),
            path.display()
        )));
    }

    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    validate_parquet_schema(expected_schema, builder.schema(), table)?;
    let reader = builder.build()?;
    let mut batches = Vec::new();
    for batch in reader {
        batches.push(batch?);
    }

    let batch = if batches.is_empty() {
        RecordBatch::new_empty(expected_schema.clone())
    } else if batches.len() == 1 {
        batches.remove(0)
    } else {
        concat_record_batches(expected_schema, &batches)?
    };

    let columns = batch
        .columns()
        .iter()
        .zip(expected_schema.fields())
        .map(|(column, field)| canonicalize_array(column, field))
        .collect::<Result<Vec<_>>>()?;
    let batch = RecordBatch::try_new(expected_schema.clone(), columns)?;

    if batch.num_rows() != expected_rows {
        return Err(Error::Conversion(format!(
            "{} table declared {expected_rows} rows but decoded {} rows",
            table.as_str(),
            batch.num_rows()
        )));
    }
    Ok(batch)
}

fn validate_parquet_schema(
    expected: impl AsRef<Schema>,
    actual: impl AsRef<Schema>,
    table: CanonicalTable,
) -> Result<()> {
    let expected = expected.as_ref();
    let actual = actual.as_ref();
    if expected.fields().len() != actual.fields().len() {
        return Err(Error::SchemaMismatch {
            expected: format!("{}: {expected:?}", table.as_str()),
            found: format!("{}: {actual:?}", table.as_str()),
        });
    }

    for (expected_field, actual_field) in expected.fields().iter().zip(actual.fields()) {
        if !parquet_field_matches(expected_field, actual_field) {
            return Err(Error::SchemaMismatch {
                expected: format!("{}: {expected:?}", table.as_str()),
                found: format!("{}: {actual:?}", table.as_str()),
            });
        }
    }

    Ok(())
}

fn canonicalize_array(array: &ArrayRef, expected_field: &FieldRef) -> Result<ArrayRef> {
    match expected_field.data_type() {
        DataType::FixedSizeList(expected_child, size) => {
            if let Some(actual) = array.as_any().downcast_ref::<FixedSizeListArray>() {
                let values = canonicalize_array(actual.values(), expected_child)?;
                Ok(Arc::new(FixedSizeListArray::try_new(
                    expected_child.clone(),
                    *size,
                    values,
                    actual.nulls().cloned(),
                )?))
            } else if let Some(actual) = array.as_any().downcast_ref::<ListArray>() {
                parquet_list_to_fixed_size_list(actual, expected_field, expected_child, *size)
            } else {
                Err(Error::Conversion(format!(
                    "expected {} to be a list array",
                    expected_field.name()
                )))
            }
        }
        DataType::List(expected_child) => {
            let actual = array.as_any().downcast_ref::<ListArray>().ok_or_else(|| {
                Error::Conversion(format!(
                    "expected {} to be a list array",
                    expected_field.name()
                ))
            })?;
            let values = canonicalize_array(actual.values(), expected_child)?;
            Ok(Arc::new(ListArray::try_new(
                expected_child.clone(),
                actual.offsets().clone(),
                values,
                actual.nulls().cloned(),
            )?))
        }
        DataType::Struct(expected_fields) => {
            let actual = array
                .as_any()
                .downcast_ref::<StructArray>()
                .ok_or_else(|| {
                    Error::Conversion(format!(
                        "expected {} to be a struct array",
                        expected_field.name()
                    ))
                })?;
            let columns = actual
                .columns()
                .iter()
                .zip(expected_fields.iter())
                .map(|(column, field)| canonicalize_array(column, field))
                .collect::<Result<Vec<_>>>()?;
            Ok(Arc::new(StructArray::try_new(
                expected_fields.clone(),
                columns,
                actual.nulls().cloned(),
            )?))
        }
        _ => Ok(array.clone()),
    }
}

fn parquet_field_matches(expected: &Field, actual: &Field) -> bool {
    expected.name() == actual.name()
        && expected.is_nullable() == actual.is_nullable()
        && parquet_data_type_matches(expected.data_type(), actual.data_type())
}

fn parquet_data_type_matches(expected: &DataType, actual: &DataType) -> bool {
    match (expected, actual) {
        (
            DataType::List(expected_child) | DataType::FixedSizeList(expected_child, _),
            DataType::List(actual_child),
        ) => {
            expected_child.is_nullable() == actual_child.is_nullable()
                && parquet_data_type_matches(expected_child.data_type(), actual_child.data_type())
        }
        (
            DataType::FixedSizeList(expected_child, expected_len),
            DataType::FixedSizeList(actual_child, actual_len),
        ) => {
            expected_len == actual_len
                && expected_child.is_nullable() == actual_child.is_nullable()
                && parquet_data_type_matches(expected_child.data_type(), actual_child.data_type())
        }
        (DataType::Struct(expected_fields), DataType::Struct(actual_fields)) => {
            expected_fields.len() == actual_fields.len()
                && expected_fields.iter().zip(actual_fields.iter()).all(
                    |(expected_field, actual_field)| {
                        parquet_field_matches(expected_field, actual_field)
                    },
                )
        }
        _ => expected == actual,
    }
}

fn parquet_batch_from_arrow(batch: &RecordBatch) -> Result<RecordBatch> {
    let schema = Arc::new(Schema::new(
        batch
            .schema()
            .fields()
            .iter()
            .map(|field| parquet_field_from_arrow(field))
            .collect::<Vec<_>>(),
    ));
    let columns = batch
        .columns()
        .iter()
        .zip(batch.schema().fields())
        .zip(schema.fields())
        .map(|((column, arrow_field), parquet_field)| {
            parquet_array_from_arrow(column, arrow_field, parquet_field)
        })
        .collect::<Result<Vec<_>>>()?;

    RecordBatch::try_new(schema, columns).map_err(Error::from)
}

fn parquet_field_from_arrow(field: &Field) -> Field {
    field
        .clone()
        .with_data_type(parquet_data_type_from_arrow(field.data_type()))
}

fn parquet_data_type_from_arrow(data_type: &DataType) -> DataType {
    match data_type {
        DataType::FixedSizeList(child, _) => {
            DataType::List(Arc::new(parquet_field_from_arrow(child.as_ref())))
        }
        DataType::List(child) => DataType::List(Arc::new(parquet_field_from_arrow(child.as_ref()))),
        DataType::Struct(fields) => DataType::Struct(
            fields
                .iter()
                .map(|field| Arc::new(parquet_field_from_arrow(field)))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn parquet_array_from_arrow(
    array: &ArrayRef,
    arrow_field: &FieldRef,
    parquet_field: &FieldRef,
) -> Result<ArrayRef> {
    match arrow_field.data_type() {
        DataType::FixedSizeList(_, size) => {
            let actual = array
                .as_any()
                .downcast_ref::<FixedSizeListArray>()
                .ok_or_else(|| {
                    Error::Conversion(format!(
                        "expected {} to be a fixed-size list array",
                        arrow_field.name()
                    ))
                })?;
            fixed_size_list_to_parquet_list(actual, parquet_field, *size)
        }
        DataType::Struct(arrow_fields) => {
            let actual = array
                .as_any()
                .downcast_ref::<StructArray>()
                .ok_or_else(|| {
                    Error::Conversion(format!(
                        "expected {} to be a struct array",
                        arrow_field.name()
                    ))
                })?;
            let DataType::Struct(parquet_fields) = parquet_field.data_type() else {
                return Err(Error::Conversion(format!(
                    "expected {} parquet field to be a struct",
                    parquet_field.name()
                )));
            };
            let columns = actual
                .columns()
                .iter()
                .zip(arrow_fields.iter())
                .zip(parquet_fields.iter())
                .map(|((column, arrow_field), parquet_field)| {
                    parquet_array_from_arrow(column, arrow_field, parquet_field)
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(Arc::new(StructArray::try_new(
                parquet_fields.clone(),
                columns,
                actual.nulls().cloned(),
            )?))
        }
        _ => Ok(array.clone()),
    }
}

fn fixed_size_list_to_parquet_list(
    array: &FixedSizeListArray,
    parquet_field: &FieldRef,
    size: i32,
) -> Result<ArrayRef> {
    let DataType::List(list_child) = parquet_field.data_type() else {
        return Err(Error::Conversion(format!(
            "expected {} parquet field to be a list",
            parquet_field.name()
        )));
    };
    let size = usize::try_from(size).map_err(|_| {
        Error::Conversion(format!(
            "{} has a negative fixed-size list length",
            parquet_field.name()
        ))
    })?;
    let mut builder = ListBuilder::new(Float64Builder::new());
    for row_index in 0..array.len() {
        if array.is_null(row_index) {
            builder.append_null();
            continue;
        }

        let value = array.value(row_index);
        let values = value
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| {
                Error::Conversion(format!(
                    "expected {} fixed-size list values to be Float64",
                    parquet_field.name()
                ))
            })?;
        if values.len() != size {
            return Err(Error::Conversion(format!(
                "{} fixed-size list row {row_index} has {} values, expected {size}",
                parquet_field.name(),
                values.len()
            )));
        }
        for value_index in 0..size {
            builder.values().append_value(values.value(value_index));
        }
        builder.append(true);
    }

    let list = builder.finish();
    Ok(Arc::new(ListArray::try_new(
        list_child.clone(),
        list.offsets().clone(),
        list.values().clone(),
        list.nulls().cloned(),
    )?))
}

fn parquet_list_to_fixed_size_list(
    array: &ListArray,
    expected_field: &FieldRef,
    expected_child: &FieldRef,
    size: i32,
) -> Result<ArrayRef> {
    let size = usize::try_from(size).map_err(|_| {
        Error::Conversion(format!(
            "{} has a negative fixed-size list length",
            expected_field.name()
        ))
    })?;
    let offsets = array.value_offsets();
    let values = array
        .values()
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| {
            Error::Conversion(format!(
                "expected {} list values to be Float64",
                expected_field.name()
            ))
        })?;
    let mut flat = Vec::with_capacity(array.len() * size);
    let mut validity = Vec::with_capacity(array.len());
    for row_index in 0..array.len() {
        if array.is_null(row_index) {
            flat.extend(std::iter::repeat_n(0.0, size));
            validity.push(false);
            continue;
        }

        let start = usize::try_from(offsets[row_index]).map_err(|_| {
            Error::Conversion(format!(
                "{} row {row_index} has a negative list offset",
                expected_field.name()
            ))
        })?;
        let end = usize::try_from(offsets[row_index + 1]).map_err(|_| {
            Error::Conversion(format!(
                "{} row {row_index} has a negative list offset",
                expected_field.name()
            ))
        })?;
        if end - start != size {
            return Err(Error::Conversion(format!(
                "{} row {row_index} has {} values, expected {size}",
                expected_field.name(),
                end - start
            )));
        }
        for value_index in start..end {
            if values.is_null(value_index) {
                return Err(Error::Conversion(format!(
                    "{} row {row_index} contains a null fixed-list item",
                    expected_field.name()
                )));
            }
            flat.push(values.value(value_index));
        }
        validity.push(true);
    }

    let nulls = if validity.iter().all(|item| *item) {
        None
    } else {
        Some(NullBuffer::from(validity))
    };
    Ok(Arc::new(FixedSizeListArray::try_new(
        expected_child.clone(),
        i32::try_from(size).expect("fixed-size list length came from i32"),
        Arc::new(Float64Array::from(flat)),
        nulls,
    )?))
}

fn validate_dataset_manifest(manifest: &ParquetDatasetManifest) -> Result<()> {
    let mut last_position = None;
    let mut required = canonical_table_order()
        .iter()
        .filter(|table| table.is_required())
        .copied()
        .collect::<Vec<_>>();

    for table_ref in &manifest.tables {
        let table = CanonicalTable::parse(&table_ref.name)?;
        let position = canonical_table_position(table);
        if let Some(last_position) = last_position
            && position <= last_position
        {
            return Err(Error::Unsupported(
                "dataset manifest tables are not in canonical order".to_string(),
            ));
        }
        last_position = Some(position);
        required.retain(|required_table| *required_table != table);

        if table_ref.path.is_absolute() || table_ref.path.components().any(is_parent_component) {
            return Err(Error::Unsupported(format!(
                "{} table path {} is not a safe relative path",
                table.as_str(),
                table_ref.path.display()
            )));
        }
    }

    if let Some(missing) = required.first() {
        return Err(Error::Unsupported(format!(
            "dataset manifest is missing required {} table",
            missing.as_str()
        )));
    }

    Ok(())
}

fn is_parent_component(component: std::path::Component<'_>) -> bool {
    matches!(component, std::path::Component::ParentDir)
}
