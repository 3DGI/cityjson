use arrow::array::{ArrayData, ArrayRef, FixedSizeListArray, Int64Builder, RecordBatch, StructArray};
use arrow::buffer::Buffer;
use arrow::datatypes::{DataType, Field, Schema, UnionFields, UnionMode};
use arrow::error::ArrowError;
use cityjson::prelude::{QuantizedCoordinate, ResourceRef, StringStorage, TransformTrait};
use cityjson::v2_0::{Metadata, Transform};
use std::sync::Arc;

pub trait ToArrowDataType {
    fn to_arrow_data_type(&self) -> DataType;
}

pub fn metadata_to_arrow<SS: StringStorage, RR: ResourceRef>(metadata: &Metadata<SS, RR>) -> Result<RecordBatch, ArrowError> {
    // 1. Define the union type for all possible value types in metadata
    let value_union_type = DataType::Union(
        // Fields for each possible type
        UnionFields::new(
            vec![0, 1, 2, 3, 4, 5, 6],
            vec![
                Field::new("string", DataType::Utf8, true),
                Field::new("bbox", DataType::FixedSizeList(
                    Arc::new(Field::new_list_field(DataType::Float64, false)),
                    6,
                ), true),
                Field::new("contact", create_contact_data_type(), true),
                Field::new("date", DataType::Utf8, true),
                Field::new("crs", DataType::Utf8, true),
                Field::new("attributes", create_attributes_data_type(), true),
                Field::new("null", DataType::Null, true),
            ])
        // Dense union mode is more efficient for our use case
        UnionMode::Dense,
    );
    let batch = RecordBatch::try_new(
        Arc::default(),
        vec![],
    )?;
    Ok(batch)
}

DataType::Map(
Arc::new(Field::new("keys", DataType::Utf8, false)),
Arc::new(Field::new("values", DataType::Union(
UnionMode::Dense,
vec![0, 1, 2, 3, 4, 5, 6],
UnionFields::new(vec![
    Field::new("string", DataType::Utf8, false),
    Field::new("role", DataType::Dictionary(
        Box::new(DataType::Int8),
        Box::new(DataType::Utf8),
    ), true),
    Field::new("contact_type", DataType::Dictionary(
        Box::new(DataType::Int8),
        Box::new(DataType::Utf8),
    ), true),
    Field::new("attributes", create_attributes_data_type(), true),
    Field::new("null", DataType::Null, true),
]),
), true)),
false
)

pub fn create_contact_data_type() -> DataType {
    DataType::Map(
        Arc::new(
            Field::new()
        ),
        true
    )
}

struct ContactBuilder {

}


// todo: create specific error and conversion for crate
pub fn transform_to_arrow(transform: &Transform) -> Result<RecordBatch, ArrowError> {
    // Create arrays of values
    let scale_value_data = ArrayData::builder(DataType::Float64).len(3).add_buffer(Buffer::from_slice_ref(transform.scale())).build()?;
    let translate_value_data = ArrayData::builder(DataType::Float64).len(3).add_buffer(Buffer::from_slice_ref(transform.translate())).build()?;

    let scale_list_data_type = DataType::FixedSizeList(
        Arc::new(Field::new_list_field(DataType::Float64, false)),
        3,
    );
    let translate_list_data_type = DataType::FixedSizeList(
        Arc::new(Field::new_list_field(DataType::Float64, false)),
        3,
    );

    let scale_list_data = ArrayData::builder(scale_list_data_type.clone()).len(1).add_child_data(scale_value_data).build()?;
    let translate_list_data = ArrayData::builder(translate_list_data_type.clone()).len(1).add_child_data(translate_value_data).build()?;

    // Wrap the f64 arrays in FixedSizeListArrays.

    let scale_listarray = FixedSizeListArray::from(scale_list_data);
    let translate_listarray = FixedSizeListArray::from(translate_list_data);

    let schema = Schema::new(vec![
        Field::new("scale", scale_list_data_type.clone(), false),
        Field::new("translate", translate_list_data_type.clone(), false)
    ]);

    // Create a RecordBatch with a single row.
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(scale_listarray) as ArrayRef,
            Arc::new(translate_listarray) as ArrayRef,
        ],
    )?;
    Ok(batch)
}

#[test]
fn test_transform() {
    // Create a Transform with known values
    let mut transform = Transform::new();
    transform.set_scale([0.1, 0.2, 0.3]);
    transform.set_translate([10.0, 20.0, 30.0]);

    // Convert to Arrow RecordBatch
    let batch = transform_to_arrow(&transform).unwrap();

    // Verify the batch structure
    assert_eq!(batch.num_rows(), 1);
    assert_eq!(batch.num_columns(), 2);
    assert_eq!(batch.schema().field(0).name(), "scale");
    assert_eq!(batch.schema().field(1).name(), "translate");
    dbg!(batch.column(0).as_any().downcast_ref::<FixedSizeListArray>().unwrap());
    dbg!(batch.column(1).as_any().downcast_ref::<FixedSizeListArray>().unwrap());
}

#[derive(Debug, Default)]
pub struct VerticesBuilder {
    x: Int64Builder,
    y: Int64Builder,
    z: Int64Builder,
}

impl VerticesBuilder {
    pub fn append(&mut self, coordinate: &QuantizedCoordinate) {
        self.x.append_value(coordinate.x());
        self.y.append_value(coordinate.y());
        self.z.append_value(coordinate.z());
    }

    pub fn finish(&mut self) -> StructArray {
        let x = Arc::new(self.x.finish()) as ArrayRef;
        let x_field = Arc::new(Field::new("x", DataType::Int64, false));
        let y = Arc::new(self.y.finish()) as ArrayRef;
        let y_field = Arc::new(Field::new("y", DataType::Int64, false));
        let z = Arc::new(self.z.finish()) as ArrayRef;
        let z_field = Arc::new(Field::new("z", DataType::Int64, false));

        StructArray::from(vec![(x_field, x), (y_field, y), (z_field, z)])
    }
}

impl<'a> Extend<&'a QuantizedCoordinate> for VerticesBuilder {
    fn extend<I: IntoIterator<Item=&'a QuantizedCoordinate>>(&mut self, iter: I) {
        iter.into_iter()
            .for_each(|coordinate| self.append(coordinate));
    }
}

pub fn vertices_to_batch(vertices: &[QuantizedCoordinate]) -> RecordBatch {
    let mut builder = VerticesBuilder::default();
    builder.extend(vertices);
    RecordBatch::from(&builder.finish())
}

#[test]
fn test_vertices() {
    use rand::Rng;

    // Create a random number generator
    let mut rng = rand::rng();

    // Create 1000 random QuantizedCoordinate instances
    let mut vertices = Vec::with_capacity(1000);
    for _ in 0..1000 {
        let x = rng.random_range(-1000..=300000);
        let y = rng.random_range(-20000..=400000);
        let z = rng.random_range(-100..=300);

        // Create a QuantizedCoordinate with random values
        let coordinate = QuantizedCoordinate::new(x, y, z);
        vertices.push(coordinate);
    }

    // Convert vertices to a RecordBatch
    let batch = vertices_to_batch(&vertices);

    // Verify the batch has 1000 rows
    assert_eq!(batch.num_rows(), 1000);
}
