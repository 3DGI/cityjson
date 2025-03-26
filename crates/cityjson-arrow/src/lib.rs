use arrow::array::{
    ArrayRef, FixedSizeListBuilder, Float64Builder, Int64Builder, RecordBatch, StructArray,
};
use arrow::datatypes::{DataType, Field, Schema};
use cityjson::prelude::{QuantizedCoordinate, TransformTrait};
use cityjson::v2_0::Transform;
use std::error::Error;
use std::sync::Arc;

// todo: create specific error and conversion for crate
pub fn transform_to_arrow(transform: &Transform) -> Result<RecordBatch, Box<dyn Error>> {
    let scale_builder = Float64Builder::new();
    let translate_builder = Float64Builder::new();
    // Wrap the f64 arrays in FixedSizeListArrays.
    let mut scale_array_builder = FixedSizeListBuilder::with_capacity(scale_builder, 3, 1)
        .with_field(Field::new("item", DataType::Float64, false));
    let mut translate_array_builder = FixedSizeListBuilder::with_capacity(translate_builder, 3, 1)
        .with_field(Field::new("item", DataType::Float64, false));
    // Build the Lists
    transform
        .scale()
        .into_iter()
        .for_each(|v| scale_array_builder.values().append_value(v));
    scale_array_builder.append(true);
    let scale_list = scale_array_builder.finish();
    transform
        .translate()
        .into_iter()
        .for_each(|v| translate_array_builder.values().append_value(v));
    translate_array_builder.append(true);
    let translate_list = translate_array_builder.finish();

    let schema = Schema::new(vec![
        Field::new(
            "scale",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float64, false)), 3),
            false,
        ),
        Field::new(
            "translate",
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float64, false)), 3),
            false,
        ),
    ]);

    // Create a RecordBatch with a single row.
    let batch = RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(scale_list) as ArrayRef,
            Arc::new(translate_list) as ArrayRef,
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
    fn extend<I: IntoIterator<Item = &'a QuantizedCoordinate>>(&mut self, iter: I) {
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
