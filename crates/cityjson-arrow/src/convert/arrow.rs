#![allow(clippy::wildcard_imports)]

use super::*;
use num_traits::ToPrimitive;

pub(super) fn optional_batch_from<F>(is_empty: bool, build: F) -> Result<Option<RecordBatch>>
where
    F: FnOnce() -> Result<RecordBatch>,
{
    if is_empty {
        Ok(None)
    } else {
        build().map(Some)
    }
}

pub(super) struct SchemaFieldLookup<'a> {
    schema: &'a Arc<::arrow::datatypes::Schema>,
    fields: HashMap<String, FieldRef>,
}

impl<'a> SchemaFieldLookup<'a> {
    pub(super) fn new(schema: &'a Arc<::arrow::datatypes::Schema>) -> Self {
        Self {
            schema,
            fields: HashMap::new(),
        }
    }

    pub(super) fn field(&mut self, name: &str) -> Result<FieldRef> {
        if let Some(field) = self.fields.get(name) {
            return Ok(field.clone());
        }
        let field = Arc::new(self.schema.field_with_name(name)?.clone());
        self.fields.insert(name.to_string(), field.clone());
        Ok(field)
    }
}

pub(super) fn field_from_schema(
    schema: &Arc<::arrow::datatypes::Schema>,
    name: &str,
) -> Result<FieldRef> {
    Ok(Arc::new(schema.field_with_name(name)?.clone()))
}

pub(super) fn fixed_size_f64_array<const N: usize>(
    field: &FieldRef,
    size: i32,
    rows: Vec<Option<[f64; N]>>,
) -> Result<FixedSizeListArray> {
    let mut flat = Vec::with_capacity(rows.len() * N);
    let mut validity = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(values) = row {
            flat.extend(values);
            validity.push(true);
        } else {
            flat.extend(std::iter::repeat_n(0.0, N));
            validity.push(false);
        }
    }
    let values: ArrayRef = Arc::new(Float64Array::from(flat));
    let nulls = if validity.iter().all(|item| *item) {
        None
    } else {
        Some(NullBuffer::from(validity))
    };
    FixedSizeListArray::try_new(fixed_list_child_field(field)?, size, values, nulls)
        .map_err(Error::from)
}

pub(super) fn list_f64_array(field: &FieldRef, rows: Vec<Option<Vec<f64>>>) -> Result<ListArray> {
    let mut offsets = vec![0_i32];
    let mut flat: Vec<f64> = Vec::new();
    let mut validity = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(values) = row {
            flat.extend(&values);
            offsets.push(usize_to_i32(flat.len(), "list offset")?);
            validity.push(true);
        } else {
            offsets.push(usize_to_i32(flat.len(), "list offset")?);
            validity.push(false);
        }
    }
    let values: ArrayRef = Arc::new(Float64Array::from(flat));
    let nulls = if validity.iter().all(|item| *item) {
        None
    } else {
        Some(NullBuffer::from(validity))
    };
    ListArray::try_new(
        list_child_field(field)?,
        OffsetBuffer::new(ScalarBuffer::from(offsets)),
        values,
        nulls,
    )
    .map_err(Error::from)
}

pub(super) fn fixed_list_child_field(field: &FieldRef) -> Result<FieldRef> {
    match field.data_type() {
        DataType::FixedSizeList(child, _) => Ok(child.clone()),
        other => Err(Error::Conversion(format!(
            "expected fixed size list field, found {other:?}"
        ))),
    }
}

pub(super) fn list_child_field(field: &FieldRef) -> Result<FieldRef> {
    match field.data_type() {
        DataType::List(child) => Ok(child.clone()),
        other => Err(Error::Conversion(format!(
            "expected list field, found {other:?}"
        ))),
    }
}

pub(super) fn point_of_contact_array(
    field: &FieldRef,
    contact: Option<&MetadataContactRow>,
    address_spec: Option<&ProjectedStructSpec>,
) -> Result<ArrayRef> {
    let address_field = if address_spec.is_some() {
        Some(match field.data_type() {
            DataType::Struct(fields) => fields
                .iter()
                .find(|candidate| candidate.name() == "address")
                .cloned()
                .ok_or_else(|| {
                    Error::Conversion(
                        "point_of_contact.address field missing from schema".to_string(),
                    )
                })?,
            other => {
                return Err(Error::Conversion(format!(
                    "expected point_of_contact struct field, found {other:?}"
                )));
            }
        })
    } else {
        None
    };
    let address_rows = [contact.and_then(|value| value.address.as_ref())];
    let address_array = address_spec
        .zip(address_field.as_ref())
        .map(|(spec, field)| projected_struct_array_from_attributes(field, spec, &address_rows))
        .transpose()?;

    let fields = match field.data_type() {
        DataType::Struct(fields) => fields.clone(),
        other => {
            return Err(Error::Conversion(format!(
                "expected point_of_contact struct field, found {other:?}"
            )));
        }
    };
    let arrays: Vec<ArrayRef> = vec![
        Arc::new(LargeStringArray::from(vec![
            contact.map(|value| value.contact_name.clone()),
        ])),
        Arc::new(LargeStringArray::from(vec![
            contact.map(|value| value.email_address.clone()),
        ])),
        Arc::new(StringArray::from(vec![
            contact.and_then(|value| value.role.clone()),
        ])),
        Arc::new(LargeStringArray::from(vec![
            contact.and_then(|value| value.website.clone()),
        ])),
        Arc::new(StringArray::from(vec![
            contact.and_then(|value| value.contact_type.clone()),
        ])),
        Arc::new(LargeStringArray::from(vec![
            contact.and_then(|value| value.phone.clone()),
        ])),
        Arc::new(LargeStringArray::from(vec![
            contact.and_then(|value| value.organization.clone()),
        ])),
    ];
    let mut arrays = arrays;
    if let Some(array) = address_array {
        arrays.push(array);
    }
    let nulls = contact.is_none();
    Ok(Arc::new(StructArray::try_new(
        fields,
        arrays,
        if nulls {
            Some(NullBuffer::from(vec![false]))
        } else {
            None
        },
    )?))
}

pub(super) fn projected_struct_array_from_attributes(
    field: &FieldRef,
    spec: &ProjectedStructSpec,
    rows: &[Option<&cityjson::v2_0::OwnedAttributes>],
) -> Result<ArrayRef> {
    let child_fields = match field.data_type() {
        DataType::Struct(fields) => fields.clone(),
        other => {
            return Err(Error::Conversion(format!(
                "expected projected struct field, found {other:?}"
            )));
        }
    };
    let mut child_arrays = Vec::with_capacity(spec.fields.len());
    for (index, child_spec) in spec.fields.iter().enumerate() {
        let child_values = rows
            .iter()
            .map(|row| row.and_then(|attributes| attributes.get(&child_spec.name)))
            .collect::<Vec<_>>();
        let child_field = child_fields
            .get(index)
            .cloned()
            .ok_or_else(|| Error::Conversion("projected struct field missing".to_string()))?;
        child_arrays.push(projected_value_array(
            &child_field,
            &child_spec.value,
            &child_values,
        )?);
    }

    let validity = rows.iter().map(Option::is_some).collect::<Vec<_>>();
    let nulls = if validity.iter().all(|item| *item) {
        None
    } else {
        Some(NullBuffer::from(validity))
    };
    Ok(Arc::new(StructArray::try_new(child_fields, child_arrays, nulls)?) as ArrayRef)
}

pub(super) fn projected_map_from_array(
    spec: &ProjectedStructSpec,
    array: &StructArray,
    row: usize,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<HashMap<String, OwnedAttributeValue>> {
    let mut attributes = HashMap::with_capacity(spec.fields.len());
    if array.is_null(row) {
        return Ok(attributes);
    }
    for (index, field_spec) in spec.fields.iter().enumerate() {
        let value = projected_value_from_array(
            array.column(index).as_ref(),
            &field_spec.value,
            row,
            geometry_handles,
        )?;
        if !matches!(value, AttributeValue::Null) {
            attributes.insert(field_spec.name.clone(), value);
        }
    }
    Ok(attributes)
}

pub(super) fn projected_value_array(
    field: &FieldRef,
    spec: &ProjectedValueSpec,
    values: &[Option<&OwnedAttributeValue>],
) -> Result<ArrayRef> {
    match spec {
        ProjectedValueSpec::Null => projected_null_array(values),
        ProjectedValueSpec::Boolean => projected_boolean_array(values),
        ProjectedValueSpec::UInt64 => projected_u64_array(values),
        ProjectedValueSpec::Int64 => projected_i64_array(values),
        ProjectedValueSpec::Float64 => projected_f64_array(values),
        ProjectedValueSpec::Utf8 => projected_utf8_array(values),
        ProjectedValueSpec::Json => projected_json_array(values),
        ProjectedValueSpec::GeometryRef => projected_geometry_ref_array(values),
        ProjectedValueSpec::List { item, .. } => projected_list_array(field, item, values),
        ProjectedValueSpec::Struct(spec) => projected_struct_value_array(spec, values),
    }
}

pub(super) fn projected_null_array(values: &[Option<&OwnedAttributeValue>]) -> Result<ArrayRef> {
    for value in values {
        if let Some(value) = value
            && !matches!(value, AttributeValue::Null)
        {
            return Err(Error::Conversion(format!(
                "expected null projected value, found {value}"
            )));
        }
    }
    Ok(Arc::new(NullArray::new(values.len())) as ArrayRef)
}

pub(super) fn projected_boolean_array(values: &[Option<&OwnedAttributeValue>]) -> Result<ArrayRef> {
    Ok(Arc::new(BooleanArray::from(
        values
            .iter()
            .map(|value| match value {
                None | Some(AttributeValue::Null) => Ok(None),
                Some(AttributeValue::Bool(value)) => Ok(Some(*value)),
                Some(other) => Err(Error::Conversion(format!(
                    "expected bool projected value, found {other}"
                ))),
            })
            .collect::<Result<Vec<_>>>()?,
    )) as ArrayRef)
}

pub(super) fn projected_u64_array(values: &[Option<&OwnedAttributeValue>]) -> Result<ArrayRef> {
    Ok(Arc::new(UInt64Array::from(
        values
            .iter()
            .map(|value| match value {
                None | Some(AttributeValue::Null) => Ok(None),
                Some(AttributeValue::Unsigned(value)) => Ok(Some(*value)),
                Some(other) => Err(Error::Conversion(format!(
                    "expected u64 projected value, found {other}"
                ))),
            })
            .collect::<Result<Vec<_>>>()?,
    )) as ArrayRef)
}

pub(super) fn projected_i64_array(values: &[Option<&OwnedAttributeValue>]) -> Result<ArrayRef> {
    Ok(Arc::new(Int64Array::from(
        values
            .iter()
            .map(|value| match value {
                None | Some(AttributeValue::Null) => Ok(None),
                Some(AttributeValue::Integer(value)) => Ok(Some(*value)),
                // Numeric widening: UInt64 promoted to Int64 (values > i64::MAX saturate)
                Some(AttributeValue::Unsigned(value)) => {
                    Ok(Some(i64::try_from(*value).unwrap_or(i64::MAX)))
                }
                Some(other) => Err(Error::Conversion(format!(
                    "expected i64 projected value, found {other}"
                ))),
            })
            .collect::<Result<Vec<_>>>()?,
    )) as ArrayRef)
}

pub(super) fn projected_f64_array(values: &[Option<&OwnedAttributeValue>]) -> Result<ArrayRef> {
    Ok(Arc::new(Float64Array::from(
        values
            .iter()
            .map(|value| match value {
                None | Some(AttributeValue::Null) => Ok(None),
                Some(AttributeValue::Float(value)) => Ok(Some(*value)),
                // Numeric widening: integers promoted to Float64
                Some(AttributeValue::Unsigned(value)) => {
                    Ok(Some(value.to_f64().ok_or_else(|| {
                        Error::Conversion(
                            "unsigned integer value cannot be represented as f64".to_string(),
                        )
                    })?))
                }
                Some(AttributeValue::Integer(value)) => {
                    Ok(Some(value.to_f64().ok_or_else(|| {
                        Error::Conversion("integer value cannot be represented as f64".to_string())
                    })?))
                }
                Some(other) => Err(Error::Conversion(format!(
                    "expected f64 projected value, found {other}"
                ))),
            })
            .collect::<Result<Vec<_>>>()?,
    )) as ArrayRef)
}

pub(super) fn projected_utf8_array(values: &[Option<&OwnedAttributeValue>]) -> Result<ArrayRef> {
    Ok(Arc::new(LargeStringArray::from(
        values
            .iter()
            .map(|value| match value {
                None | Some(AttributeValue::Null) => Ok(None),
                Some(AttributeValue::String(value)) => Ok(Some(value.clone())),
                Some(other) => Err(Error::Conversion(format!(
                    "expected string projected value, found {other}"
                ))),
            })
            .collect::<Result<Vec<_>>>()?,
    )) as ArrayRef)
}

pub(super) fn projected_json_array(values: &[Option<&OwnedAttributeValue>]) -> Result<ArrayRef> {
    Ok(Arc::new(LargeStringArray::from(
        values
            .iter()
            .map(|value| match value {
                None | Some(AttributeValue::Null) => Ok(None),
                Some(value) => serde_json::to_string(&attribute_value_to_json(value))
                    .map(Some)
                    .map_err(|err| {
                        Error::Conversion(format!("JSON fallback serialization failed: {err}"))
                    }),
            })
            .collect::<Result<Vec<_>>>()?,
    )) as ArrayRef)
}

fn attribute_value_to_json(value: &OwnedAttributeValue) -> serde_json::Value {
    match value {
        AttributeValue::Bool(b) => serde_json::Value::Bool(*b),
        AttributeValue::Unsigned(u) => serde_json::Value::Number((*u).into()),
        AttributeValue::Integer(i) => serde_json::Value::Number((*i).into()),
        AttributeValue::Float(f) => serde_json::Number::from_f64(*f)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        AttributeValue::String(s) => serde_json::Value::String(s.clone()),
        AttributeValue::Vec(items) => {
            serde_json::Value::Array(items.iter().map(attribute_value_to_json).collect())
        }
        AttributeValue::Map(map) => serde_json::Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), attribute_value_to_json(v)))
                .collect(),
        ),
        _ => serde_json::Value::Null,
    }
}

fn json_to_attribute_value(value: serde_json::Value) -> OwnedAttributeValue {
    match value {
        serde_json::Value::Null => AttributeValue::Null,
        serde_json::Value::Bool(b) => AttributeValue::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                AttributeValue::Unsigned(u)
            } else if let Some(i) = n.as_i64() {
                AttributeValue::Integer(i)
            } else {
                AttributeValue::Float(n.as_f64().unwrap_or(f64::NAN))
            }
        }
        serde_json::Value::String(s) => AttributeValue::String(s),
        serde_json::Value::Array(arr) => {
            AttributeValue::Vec(arr.into_iter().map(json_to_attribute_value).collect())
        }
        serde_json::Value::Object(map) => AttributeValue::Map(
            map.into_iter()
                .map(|(k, v)| (k, json_to_attribute_value(v)))
                .collect(),
        ),
    }
}

pub(super) fn projected_geometry_ref_array(
    values: &[Option<&OwnedAttributeValue>],
) -> Result<ArrayRef> {
    Ok(Arc::new(UInt64Array::from(
        values
            .iter()
            .map(|value| match value {
                None | Some(AttributeValue::Null) => Ok(None),
                Some(AttributeValue::Geometry(handle)) => Ok(Some(raw_id_from_handle(*handle))),
                Some(other) => Err(Error::Conversion(format!(
                    "expected geometry reference projected value, found {other}"
                ))),
            })
            .collect::<Result<Vec<_>>>()?,
    )) as ArrayRef)
}

pub(super) fn projected_list_array(
    field: &FieldRef,
    item: &ProjectedValueSpec,
    values: &[Option<&OwnedAttributeValue>],
) -> Result<ArrayRef> {
    let mut offsets = vec![0_i32];
    let mut flattened = Vec::new();
    let mut validity = Vec::with_capacity(values.len());
    for value in values {
        match value {
            None | Some(AttributeValue::Null) => {
                offsets.push(usize_to_i32(flattened.len(), "projected list offset")?);
                validity.push(false);
            }
            Some(AttributeValue::Vec(items)) => {
                flattened.extend(items.iter().map(Some));
                offsets.push(usize_to_i32(flattened.len(), "projected list offset")?);
                validity.push(true);
            }
            Some(other) => {
                return Err(Error::Conversion(format!(
                    "expected list projected value, found {other}"
                )));
            }
        }
    }

    let child_field = list_child_field(field)?;
    let child_values = projected_value_array(&child_field, item, &flattened)?;
    let nulls = if validity.iter().all(|item| *item) {
        None
    } else {
        Some(NullBuffer::from(validity))
    };
    Ok(Arc::new(ListArray::try_new(
        child_field,
        OffsetBuffer::new(ScalarBuffer::from(offsets)),
        child_values,
        nulls,
    )?) as ArrayRef)
}

pub(super) fn projected_struct_value_array(
    spec: &ProjectedStructSpec,
    values: &[Option<&OwnedAttributeValue>],
) -> Result<ArrayRef> {
    let child_fields = spec.to_arrow_fields();
    let mut child_arrays = Vec::with_capacity(spec.fields.len());
    for child_spec in &spec.fields {
        let child_values = projected_struct_child_values(values, &child_spec.name)?;
        child_arrays.push(projected_value_array(
            &Arc::new(child_spec.to_arrow_field()),
            &child_spec.value,
            &child_values,
        )?);
    }

    let validity = values
        .iter()
        .map(|value| matches!(value, Some(AttributeValue::Map(_))))
        .collect::<Vec<_>>();
    let nulls = if validity.iter().all(|item| *item) {
        None
    } else {
        Some(NullBuffer::from(validity))
    };
    Ok(Arc::new(StructArray::try_new(child_fields, child_arrays, nulls)?) as ArrayRef)
}

pub(super) fn projected_struct_child_values<'a>(
    values: &[Option<&'a OwnedAttributeValue>],
    field_name: &str,
) -> Result<Vec<Option<&'a OwnedAttributeValue>>> {
    values
        .iter()
        .map(|value| match value {
            None | Some(AttributeValue::Null) => Ok(None),
            Some(AttributeValue::Map(map)) => Ok(map.get(field_name)),
            Some(other) => Err(Error::Conversion(format!(
                "expected struct projected value, found {other}"
            ))),
        })
        .collect()
}

pub(super) fn projected_attributes_from_array(
    spec: Option<&ProjectedStructSpec>,
    array: Option<&StructArray>,
    row: usize,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<cityjson::v2_0::OwnedAttributes> {
    let mut attributes = cityjson::v2_0::OwnedAttributes::default();
    let (Some(spec), Some(array)) = (spec, array) else {
        return Ok(attributes);
    };
    if array.is_null(row) {
        return Ok(attributes);
    }
    for (index, field_spec) in spec.fields.iter().enumerate() {
        let value = projected_value_from_array(
            array.column(index).as_ref(),
            &field_spec.value,
            row,
            geometry_handles,
        )?;
        if !matches!(value, AttributeValue::Null) {
            attributes.insert(field_spec.name.clone(), value);
        }
    }
    Ok(attributes)
}

pub(super) fn projected_value_from_array(
    array: &dyn Array,
    spec: &ProjectedValueSpec,
    row: usize,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<OwnedAttributeValue> {
    if array.is_null(row) {
        return Ok(AttributeValue::Null);
    }

    Ok(match spec {
        ProjectedValueSpec::Null => AttributeValue::Null,
        ProjectedValueSpec::Boolean => {
            AttributeValue::Bool(required_downcast::<BooleanArray>(array, "bool")?.value(row))
        }
        ProjectedValueSpec::UInt64 => {
            AttributeValue::Unsigned(required_downcast::<UInt64Array>(array, "u64")?.value(row))
        }
        ProjectedValueSpec::Int64 => {
            AttributeValue::Integer(required_downcast::<Int64Array>(array, "i64")?.value(row))
        }
        ProjectedValueSpec::Float64 => {
            AttributeValue::Float(required_downcast::<Float64Array>(array, "f64")?.value(row))
        }
        ProjectedValueSpec::Utf8 => AttributeValue::String(
            required_downcast::<LargeStringArray>(array, "large_utf8")?
                .value(row)
                .to_string(),
        ),
        ProjectedValueSpec::Json => {
            let s = required_downcast::<LargeStringArray>(array, "large_utf8 (json)")?.value(row);
            let json: serde_json::Value = serde_json::from_str(s).map_err(|err| {
                Error::Conversion(format!("JSON fallback deserialization failed: {err}"))
            })?;
            json_to_attribute_value(json)
        }
        ProjectedValueSpec::GeometryRef => {
            let id = required_downcast::<UInt64Array>(array, "geometry_ref")?.value(row);
            AttributeValue::Geometry(*geometry_handles.get(&id).ok_or_else(|| {
                Error::Conversion(format!(
                    "missing geometry handle for projected geometry id {id}"
                ))
            })?)
        }
        ProjectedValueSpec::List { item, .. } => {
            let list = required_downcast::<ListArray>(array, "list")?;
            let offsets = list.value_offsets();
            let start = usize::try_from(offsets[row]).expect("offset fits into usize");
            let end = usize::try_from(offsets[row + 1]).expect("offset fits into usize");
            let values = (start..end)
                .map(|index| {
                    projected_value_from_array(
                        list.values().as_ref(),
                        item,
                        index,
                        geometry_handles,
                    )
                })
                .collect::<Result<Vec<_>>>()?;
            AttributeValue::Vec(values)
        }
        ProjectedValueSpec::Struct(spec) => AttributeValue::Map(projected_map_from_array(
            spec,
            required_downcast::<StructArray>(array, "struct")?,
            row,
            geometry_handles,
        )?),
    })
}

pub(super) fn required_downcast<'a, T: 'static>(
    array: &'a dyn Array,
    expected: &str,
) -> Result<&'a T> {
    array
        .as_any()
        .downcast_ref::<T>()
        .ok_or_else(|| Error::Conversion(format!("expected projected array type {expected}")))
}

pub(super) fn required_struct_column<'a>(
    array: &'a StructArray,
    name: &str,
) -> Result<&'a dyn Array> {
    array
        .column_by_name(name)
        .map(ArrayRef::as_ref)
        .ok_or_else(|| Error::Conversion(format!("missing point_of_contact.{name} column")))
}

pub(super) fn read_optional_projected_attributes(
    batch: &RecordBatch,
    column_name: &str,
    spec: Option<&ProjectedStructSpec>,
    row: usize,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<Option<cityjson::v2_0::OwnedAttributes>> {
    let array = spec
        .map(|_| downcast_required::<StructArray>(batch, column_name))
        .transpose()?;
    let attributes = projected_attributes_from_array(spec, array, row, geometry_handles)?;
    Ok((!attributes.is_empty()).then_some(attributes))
}

pub(super) fn read_metadata_point_of_contact(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
    geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<Option<MetadataContactRow>> {
    let array = downcast_required::<StructArray>(batch, "point_of_contact")?;
    if array.is_null(0) {
        return Ok(None);
    }

    let address_array = projection
        .metadata_point_of_contact_address
        .as_ref()
        .map(|_| {
            required_downcast::<StructArray>(required_struct_column(array, "address")?, "struct")
        })
        .transpose()?;
    let address = projected_attributes_from_array(
        projection.metadata_point_of_contact_address.as_ref(),
        address_array,
        0,
        geometry_handles,
    )?;

    Ok(Some(MetadataContactRow {
        contact_name: required_downcast::<LargeStringArray>(
            required_struct_column(array, "contact_name")?,
            "large_utf8",
        )?
        .value(0)
        .to_string(),
        email_address: required_downcast::<LargeStringArray>(
            required_struct_column(array, "email_address")?,
            "large_utf8",
        )?
        .value(0)
        .to_string(),
        role: read_named_string_field(array, "role", 0)?,
        website: read_named_large_string_field(array, "website", 0)?,
        contact_type: read_named_string_field(array, "contact_type", 0)?,
        phone: read_named_large_string_field(array, "phone", 0)?,
        organization: read_named_large_string_field(array, "organization", 0)?,
        address: (!address.is_empty()).then_some(address),
    }))
}

pub(super) fn read_metadata_row(
    batch: &RecordBatch,
    projection: &ProjectionLayout,
) -> Result<MetadataRow> {
    let empty_geometry_handles = HashMap::new();
    Ok(MetadataRow {
        citymodel_id: read_large_string_scalar(batch, "citymodel_id", 0)?,
        cityjson_version: read_string_scalar(batch, "cityjson_version", 0)?,
        citymodel_kind: read_string_scalar(batch, "citymodel_kind", 0)?,
        feature_root_id: read_large_string_optional(batch, "feature_root_id", 0)?,
        identifier: read_large_string_optional(batch, "identifier", 0)?,
        title: read_large_string_optional(batch, "title", 0)?,
        reference_system: read_large_string_optional(batch, "reference_system", 0)?,
        geographical_extent: read_fixed_size_f64_optional::<6>(batch, "geographical_extent", 0)?,
        reference_date: read_string_optional(batch, "reference_date", 0)?,
        default_material_theme: read_string_optional(batch, "default_material_theme", 0)?,
        default_texture_theme: read_string_optional(batch, "default_texture_theme", 0)?,
        point_of_contact: read_metadata_point_of_contact(
            batch,
            projection,
            &empty_geometry_handles,
        )?,
        root_extra: read_optional_projected_attributes(
            batch,
            "root_extra",
            projection.root_extra.as_ref(),
            0,
            &empty_geometry_handles,
        )?,
        metadata_extra: read_optional_projected_attributes(
            batch,
            "metadata_extra",
            projection.metadata_extra.as_ref(),
            0,
            &empty_geometry_handles,
        )?,
    })
}

pub(super) fn read_transform_row(batch: &RecordBatch) -> Result<TransformRow> {
    Ok(TransformRow {
        scale: read_fixed_size_f64_required::<3>(batch, "scale", 0)?,
        translate: read_fixed_size_f64_required::<3>(batch, "translate", 0)?,
    })
}

pub(super) fn apply_metadata_row(
    model: &mut OwnedCityModel,
    row: &MetadataRow,
    _geometry_handles: &HashMap<u64, cityjson::prelude::GeometryHandle>,
) -> Result<()> {
    if let Some(identifier) = &row.identifier {
        model
            .metadata_mut()
            .set_identifier(CityModelIdentifier::new(identifier.clone()));
    }
    if let Some(title) = &row.title {
        model.metadata_mut().set_title(title.clone());
    }
    if let Some(reference_system) = &row.reference_system {
        model
            .metadata_mut()
            .set_reference_system(CRS::new(reference_system.clone()));
    }
    if let Some(extent) = row.geographical_extent {
        model
            .metadata_mut()
            .set_geographical_extent(BBox::from(extent));
    }

    if let Some(reference_date) = &row.reference_date {
        model
            .metadata_mut()
            .set_reference_date(cityjson::v2_0::Date::new(reference_date.clone()));
    }
    if let Some(theme) = &row.default_material_theme {
        model.set_default_material_theme(Some(ThemeName::new(theme.clone())));
    }
    if let Some(theme) = &row.default_texture_theme {
        model.set_default_texture_theme(Some(ThemeName::new(theme.clone())));
    }
    if let Some(value) = &row.point_of_contact {
        let mut contact = Contact::new();
        contact.set_contact_name(value.contact_name.clone());
        contact.set_email_address(value.email_address.clone());
        contact.set_role(value.role.as_deref().map(parse_contact_role).transpose()?);
        contact.set_website(value.website.clone());
        contact.set_contact_type(
            value
                .contact_type
                .as_deref()
                .map(parse_contact_type)
                .transpose()?,
        );
        contact.set_phone(value.phone.clone());
        contact.set_organization(value.organization.clone());
        contact.set_address(value.address.clone());
        model.metadata_mut().set_point_of_contact(Some(contact));
    }
    if let Some(extra) = &row.root_extra {
        for (key, value) in extra.iter() {
            model.extra_mut().insert(key.clone(), value.clone());
        }
    }
    if let Some(extra) = &row.metadata_extra {
        for (key, value) in extra.iter() {
            model
                .metadata_mut()
                .extra_mut()
                .insert(key.clone(), value.clone());
        }
    }

    Ok(())
}

pub(super) fn parse_image_type(value: &str) -> Result<ImageType> {
    Ok(match value {
        "PNG" => ImageType::Png,
        "JPG" => ImageType::Jpg,
        other => return Err(Error::Conversion(format!("unsupported image type {other}"))),
    })
}

pub(super) fn parse_wrap_mode(value: &str) -> Result<WrapMode> {
    Ok(match value {
        "wrap" => WrapMode::Wrap,
        "mirror" => WrapMode::Mirror,
        "clamp" => WrapMode::Clamp,
        "border" => WrapMode::Border,
        "none" => WrapMode::None,
        other => return Err(Error::Conversion(format!("unsupported wrap mode {other}"))),
    })
}

pub(super) fn parse_texture_mapping_type(value: &str) -> Result<TextureType> {
    Ok(match value {
        "unknown" => TextureType::Unknown,
        "specific" => TextureType::Specific,
        "typical" => TextureType::Typical,
        other => {
            return Err(Error::Conversion(format!(
                "unsupported texture type {other}"
            )));
        }
    })
}

pub(super) fn parse_semantic_type(
    value: &str,
) -> SemanticType<cityjson::prelude::OwnedStringStorage> {
    match value {
        "Default" => SemanticType::Default,
        "RoofSurface" => SemanticType::RoofSurface,
        "GroundSurface" => SemanticType::GroundSurface,
        "WallSurface" => SemanticType::WallSurface,
        "ClosureSurface" => SemanticType::ClosureSurface,
        "OuterCeilingSurface" => SemanticType::OuterCeilingSurface,
        "OuterFloorSurface" => SemanticType::OuterFloorSurface,
        "Window" => SemanticType::Window,
        "Door" => SemanticType::Door,
        "InteriorWallSurface" => SemanticType::InteriorWallSurface,
        "CeilingSurface" => SemanticType::CeilingSurface,
        "FloorSurface" => SemanticType::FloorSurface,
        "WaterSurface" => SemanticType::WaterSurface,
        "WaterGroundSurface" => SemanticType::WaterGroundSurface,
        "WaterClosureSurface" => SemanticType::WaterClosureSurface,
        "TrafficArea" => SemanticType::TrafficArea,
        "AuxiliaryTrafficArea" => SemanticType::AuxiliaryTrafficArea,
        "TransportationMarking" => SemanticType::TransportationMarking,
        "TransportationHole" => SemanticType::TransportationHole,
        other if other.starts_with('+') => SemanticType::Extension(other.to_string()),
        other => SemanticType::Extension(other.to_string()),
    }
}

pub(super) fn parse_contact_role(value: &str) -> Result<ContactRole> {
    Ok(match value {
        "Author" => ContactRole::Author,
        "CoAuthor" => ContactRole::CoAuthor,
        "Processor" => ContactRole::Processor,
        "PointOfContact" => ContactRole::PointOfContact,
        "Owner" => ContactRole::Owner,
        "User" => ContactRole::User,
        "Distributor" => ContactRole::Distributor,
        "Originator" => ContactRole::Originator,
        "Custodian" => ContactRole::Custodian,
        "ResourceProvider" => ContactRole::ResourceProvider,
        "RightsHolder" => ContactRole::RightsHolder,
        "Sponsor" => ContactRole::Sponsor,
        "PrincipalInvestigator" => ContactRole::PrincipalInvestigator,
        "Stakeholder" => ContactRole::Stakeholder,
        "Publisher" => ContactRole::Publisher,
        other => {
            return Err(Error::Conversion(format!(
                "unsupported metadata contact role {other}"
            )));
        }
    })
}

pub(super) fn parse_contact_type(value: &str) -> Result<ContactType> {
    Ok(match value {
        "Individual" => ContactType::Individual,
        "Organization" => ContactType::Organization,
        other => {
            return Err(Error::Conversion(format!(
                "unsupported metadata contact type {other}"
            )));
        }
    })
}

pub(super) fn read_large_string_scalar(
    batch: &RecordBatch,
    name: &str,
    row: usize,
) -> Result<String> {
    let array = downcast_required::<LargeStringArray>(batch, name)?;
    Ok(array.value(row).to_string())
}

pub(super) fn read_large_string_optional(
    batch: &RecordBatch,
    name: &str,
    row: usize,
) -> Result<Option<String>> {
    let array = downcast_required::<LargeStringArray>(batch, name)?;
    Ok((!array.is_null(row)).then(|| array.value(row).to_string()))
}

pub(super) fn read_string_scalar(batch: &RecordBatch, name: &str, row: usize) -> Result<String> {
    let array = downcast_required::<StringArray>(batch, name)?;
    Ok(array.value(row).to_string())
}

pub(super) fn read_string_optional(
    batch: &RecordBatch,
    name: &str,
    row: usize,
) -> Result<Option<String>> {
    let array = downcast_required::<StringArray>(batch, name)?;
    Ok((!array.is_null(row)).then(|| array.value(row).to_string()))
}

pub(super) fn read_named_large_string_field(
    array: &StructArray,
    name: &str,
    row: usize,
) -> Result<Option<String>> {
    let column = array
        .column_by_name(name)
        .ok_or_else(|| Error::Conversion(format!("missing struct field {name}")))?;
    let column = required_downcast::<LargeStringArray>(column.as_ref(), "large_utf8")?;
    Ok((!column.is_null(row)).then(|| column.value(row).to_string()))
}

pub(super) fn read_named_string_field(
    array: &StructArray,
    name: &str,
    row: usize,
) -> Result<Option<String>> {
    let column = array
        .column_by_name(name)
        .ok_or_else(|| Error::Conversion(format!("missing struct field {name}")))?;
    let column = required_downcast::<StringArray>(column.as_ref(), "utf8")?;
    Ok((!column.is_null(row)).then(|| column.value(row).to_string()))
}

pub(super) fn read_large_string_array_optional(
    array: Option<&LargeStringArray>,
    row: usize,
) -> Option<String> {
    array.and_then(|array| (!array.is_null(row)).then(|| array.value(row).to_string()))
}

pub(super) fn read_f64_array_optional(array: Option<&Float64Array>, row: usize) -> Option<f64> {
    array.and_then(|array| (!array.is_null(row)).then(|| array.value(row)))
}

pub(super) fn read_bool_array_optional(
    array: Option<&::arrow::array::BooleanArray>,
    row: usize,
) -> Option<bool> {
    array.and_then(|array| (!array.is_null(row)).then(|| array.value(row)))
}

pub(super) fn read_list_f64_array_optional<const N: usize>(
    array: Option<&ListArray>,
    row: usize,
) -> Result<Option<[f64; N]>> {
    let Some(array) = array else {
        return Ok(None);
    };
    if array.is_null(row) {
        return Ok(None);
    }
    let values = array.value(row);
    let values = values
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| Error::Conversion("list child is not f64".to_string()))?;
    let slice = values.values().as_ref();
    Ok(Some(slice.try_into().map_err(|_| {
        Error::Conversion(format!("list does not have length {N}"))
    })?))
}

pub(super) fn read_fixed_size_f64_required<const N: usize>(
    batch: &RecordBatch,
    name: &str,
    row: usize,
) -> Result<[f64; N]> {
    read_fixed_size_f64_optional::<N>(batch, name, row)?
        .ok_or_else(|| Error::Conversion(format!("missing required fixed-size list {name}")))
}

pub(super) fn read_fixed_size_f64_optional<const N: usize>(
    batch: &RecordBatch,
    name: &str,
    row: usize,
) -> Result<Option<[f64; N]>> {
    let array = downcast_required::<FixedSizeListArray>(batch, name)?;
    if array.is_null(row) {
        return Ok(None);
    }
    let values = array.value(row);
    let values = values
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| Error::Conversion(format!("fixed-size list {name} does not contain f64")))?;
    let slice = values.values().as_ref();
    Ok(Some(slice.try_into().map_err(|_| {
        Error::Conversion(format!("fixed-size list {name} does not have length {N}"))
    })?))
}

pub(super) fn read_fixed_size_list_array_optional<const N: usize>(
    array: &FixedSizeListArray,
    name: &str,
    row: usize,
) -> Result<Option<[f64; N]>> {
    if array.is_null(row) {
        return Ok(None);
    }
    let values = array.value(row);
    let values = values
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| Error::Conversion(format!("fixed-size list {name} does not contain f64")))?;
    let slice = values.values().as_ref();
    Ok(Some(slice.try_into().map_err(|_| {
        Error::Conversion(format!("fixed-size list {name} does not have length {N}"))
    })?))
}

pub(super) fn ensure_strictly_increasing_u64(
    previous: Option<u64>,
    current: u64,
    field_name: &str,
) -> Result<()> {
    if let Some(previous) = previous
        && current <= previous
    {
        return Err(Error::Conversion(format!(
            "{field_name} must be strictly increasing in canonical order, found {current} after {previous}"
        )));
    }
    Ok(())
}

pub(super) fn bind_vertex_columns<'a>(
    batch: &'a RecordBatch,
    id_name: &str,
) -> Result<VertexColumns<'a>> {
    Ok(VertexColumns {
        vertex_id: downcast_required::<UInt64Array>(batch, id_name)?,
        x: downcast_required::<Float64Array>(batch, "x")?,
        y: downcast_required::<Float64Array>(batch, "y")?,
        z: downcast_required::<Float64Array>(batch, "z")?,
    })
}

pub(super) fn bind_uv_columns(batch: &RecordBatch) -> Result<UvColumns<'_>> {
    Ok(UvColumns {
        uv_id: downcast_required::<UInt64Array>(batch, "uv_id")?,
        u: downcast_required::<Float32Array>(batch, "u")?,
        v: downcast_required::<Float32Array>(batch, "v")?,
    })
}

pub(super) fn bind_semantic_columns<'a>(
    batch: &'a RecordBatch,
    projection: &ProjectionLayout,
) -> Result<SemanticColumns<'a>> {
    Ok(SemanticColumns {
        semantic_id: downcast_required::<UInt64Array>(batch, "semantic_id")?,
        semantic_type: downcast_required::<StringArray>(batch, "semantic_type")?,
        parent_semantic_id: downcast_required::<UInt64Array>(batch, "parent_semantic_id")?,
        attributes: projection
            .semantic_attributes
            .as_ref()
            .map(|_| downcast_required::<StructArray>(batch, "attributes"))
            .transpose()?,
    })
}

pub(super) fn bind_material_columns<'a>(
    batch: &'a RecordBatch,
    projection: &ProjectionLayout,
) -> Result<MaterialColumns<'a>> {
    Ok(MaterialColumns {
        material_id: downcast_required::<UInt64Array>(batch, "material_id")?,
        name: downcast_required::<LargeStringArray>(batch, FIELD_MATERIAL_NAME)?,
        ambient_intensity: has_projection_field(
            projection.material_payload.as_ref(),
            FIELD_MATERIAL_AMBIENT_INTENSITY,
        )
        .then(|| downcast_required::<Float64Array>(batch, FIELD_MATERIAL_AMBIENT_INTENSITY))
        .transpose()?,
        diffuse_color: has_projection_field(
            projection.material_payload.as_ref(),
            FIELD_MATERIAL_DIFFUSE_COLOR,
        )
        .then(|| downcast_required::<ListArray>(batch, FIELD_MATERIAL_DIFFUSE_COLOR))
        .transpose()?,
        emissive_color: has_projection_field(
            projection.material_payload.as_ref(),
            FIELD_MATERIAL_EMISSIVE_COLOR,
        )
        .then(|| downcast_required::<ListArray>(batch, FIELD_MATERIAL_EMISSIVE_COLOR))
        .transpose()?,
        specular_color: has_projection_field(
            projection.material_payload.as_ref(),
            FIELD_MATERIAL_SPECULAR_COLOR,
        )
        .then(|| downcast_required::<ListArray>(batch, FIELD_MATERIAL_SPECULAR_COLOR))
        .transpose()?,
        shininess: has_projection_field(
            projection.material_payload.as_ref(),
            FIELD_MATERIAL_SHININESS,
        )
        .then(|| downcast_required::<Float64Array>(batch, FIELD_MATERIAL_SHININESS))
        .transpose()?,
        transparency: has_projection_field(
            projection.material_payload.as_ref(),
            FIELD_MATERIAL_TRANSPARENCY,
        )
        .then(|| downcast_required::<Float64Array>(batch, FIELD_MATERIAL_TRANSPARENCY))
        .transpose()?,
        is_smooth: has_projection_field(
            projection.material_payload.as_ref(),
            FIELD_MATERIAL_IS_SMOOTH,
        )
        .then(|| downcast_required::<::arrow::array::BooleanArray>(batch, FIELD_MATERIAL_IS_SMOOTH))
        .transpose()?,
    })
}

pub(super) fn bind_texture_columns<'a>(
    batch: &'a RecordBatch,
    projection: &ProjectionLayout,
) -> Result<TextureColumns<'a>> {
    Ok(TextureColumns {
        texture_id: downcast_required::<UInt64Array>(batch, "texture_id")?,
        image_uri: downcast_required::<LargeStringArray>(batch, "image_uri")?,
        image_type: downcast_required::<LargeStringArray>(batch, FIELD_TEXTURE_IMAGE_TYPE)?,
        wrap_mode: has_projection_field(
            projection.texture_payload.as_ref(),
            FIELD_TEXTURE_WRAP_MODE,
        )
        .then(|| downcast_required::<LargeStringArray>(batch, FIELD_TEXTURE_WRAP_MODE))
        .transpose()?,
        texture_type: has_projection_field(
            projection.texture_payload.as_ref(),
            FIELD_TEXTURE_TEXTURE_TYPE,
        )
        .then(|| downcast_required::<LargeStringArray>(batch, FIELD_TEXTURE_TEXTURE_TYPE))
        .transpose()?,
        border_color: has_projection_field(
            projection.texture_payload.as_ref(),
            FIELD_TEXTURE_BORDER_COLOR,
        )
        .then(|| downcast_required::<ListArray>(batch, FIELD_TEXTURE_BORDER_COLOR))
        .transpose()?,
    })
}

pub(super) fn bind_template_geometry_columns(
    batch: &RecordBatch,
) -> Result<TemplateGeometryColumns<'_>> {
    Ok(TemplateGeometryColumns {
        template_geometry_id: downcast_required::<UInt64Array>(batch, "template_geometry_id")?,
        geometry_type: downcast_required::<StringArray>(batch, "geometry_type")?,
        lod: downcast_required::<StringArray>(batch, "lod")?,
    })
}

pub(super) fn bind_geometry_columns(batch: &RecordBatch) -> Result<GeometryColumns<'_>> {
    Ok(GeometryColumns {
        geometry_id: downcast_required::<UInt64Array>(batch, "geometry_id")?,
        cityobject_ix: downcast_required::<UInt64Array>(batch, "cityobject_ix")?,
        geometry_ordinal: downcast_required::<UInt32Array>(batch, "geometry_ordinal")?,
        geometry_type: downcast_required::<StringArray>(batch, "geometry_type")?,
        lod: downcast_required::<StringArray>(batch, "lod")?,
    })
}

pub(super) fn bind_geometry_instance_columns(
    batch: &RecordBatch,
) -> Result<GeometryInstanceColumns<'_>> {
    Ok(GeometryInstanceColumns {
        geometry_id: downcast_required::<UInt64Array>(batch, "geometry_id")?,
        cityobject_ix: downcast_required::<UInt64Array>(batch, "cityobject_ix")?,
        geometry_ordinal: downcast_required::<UInt32Array>(batch, "geometry_ordinal")?,
        lod: downcast_required::<StringArray>(batch, "lod")?,
        template_geometry_id: downcast_required::<UInt64Array>(batch, "template_geometry_id")?,
        reference_point_vertex_id: downcast_required::<UInt64Array>(
            batch,
            "reference_point_vertex_id",
        )?,
        transform_matrix: downcast_required::<FixedSizeListArray>(batch, "transform_matrix")?,
    })
}

pub(super) fn bind_cityobject_columns<'a>(
    batch: &'a RecordBatch,
    projection: &ProjectionLayout,
) -> Result<CityObjectColumns<'a>> {
    Ok(CityObjectColumns {
        cityobject_id: downcast_required::<LargeStringArray>(batch, "cityobject_id")?,
        cityobject_ix: downcast_required::<UInt64Array>(batch, "cityobject_ix")?,
        object_type: downcast_required::<StringArray>(batch, "object_type")?,
        geographical_extent: downcast_required::<FixedSizeListArray>(batch, "geographical_extent")?,
        attributes: projection
            .cityobject_attributes
            .as_ref()
            .map(|_| downcast_required::<StructArray>(batch, "attributes"))
            .transpose()?,
        extra: projection
            .cityobject_extra
            .as_ref()
            .map(|_| downcast_required::<StructArray>(batch, "extra"))
            .transpose()?,
    })
}

pub(super) fn downcast_required<'a, T: Array + 'static>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a T> {
    batch
        .column_by_name(name)
        .ok_or_else(|| Error::MissingField(name.to_string()))?
        .as_any()
        .downcast_ref::<T>()
        .ok_or_else(|| Error::Conversion(format!("field {name} has unexpected array type")))
}
