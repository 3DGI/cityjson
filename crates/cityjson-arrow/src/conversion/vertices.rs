use crate::error::{Error, Result};
use arrow::array::{ArrayRef, Int64Array, Int64Builder, RecordBatch, StructArray};
use arrow::datatypes::{DataType, Field};
use cityjson::prelude::{QuantizedCoordinate, VertexRef, Vertices};
use std::sync::Arc;

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

/// Converts an Arrow RecordBatch containing vertex data into a cityjson-rs Vertices collection.
///
/// # Parameters
///
/// * `batch` - A RecordBatch containing vertex coordinate data
///
/// # Returns
///
/// A Result containing either the converted Vertices object or an error
///
/// # Errors
///
/// Will return an error if:
/// - The batch doesn't contain exactly one column
/// - The column isn't a StructArray with x, y, z fields
/// - Any required field is missing or of the wrong type
/// - Adding vertices to the container fails
pub fn batch_to_vertices<VR: VertexRef>(
    batch: &RecordBatch,
) -> Result<Vertices<VR, QuantizedCoordinate>> {
    if batch.num_columns() != 3 {
        return Err(Error::Conversion(format!(
            "Expected 3 columns in vertices batch, got {}",
            batch.num_columns()
        )));
    }

    // Extract x, y, z arrays
    let x_array = batch
        .column_by_name("x")
        .ok_or_else(|| Error::Conversion("Missing 'x' field in vertices".to_string()))?
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| Error::Conversion("Expected Int64Array for x coordinates".to_string()))?;

    let y_array = batch
        .column_by_name("y")
        .ok_or_else(|| Error::Conversion("Missing 'y' field in vertices".to_string()))?
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| Error::Conversion("Expected Int64Array for y coordinates".to_string()))?;

    let z_array = batch
        .column_by_name("z")
        .ok_or_else(|| Error::Conversion("Missing 'z' field in vertices".to_string()))?
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| Error::Conversion("Expected Int64Array for z coordinates".to_string()))?;

    // Create a new Vertices container with the appropriate capacity
    let num_vertices = x_array.len();
    let mut vertices = Vertices::<VR, QuantizedCoordinate>::with_capacity(num_vertices);

    // Convert each set of x, y, z values to a QuantizedCoordinate and add to the container
    for i in 0..num_vertices {
        let coord = QuantizedCoordinate::new(x_array.value(i), y_array.value(i), z_array.value(i));
        vertices.push(coord)?;
    }

    Ok(vertices)
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use cityjson::prelude::VertexIndex;

    #[test]
    fn test_vertices_to_batch() {
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

    #[test]
    fn test_batch_to_vertices() {
        // Create sample coordinates
        let coordinates = vec![
            QuantizedCoordinate::new(10, 20, 30),
            QuantizedCoordinate::new(40, 50, 60),
            QuantizedCoordinate::new(70, 80, 90),
        ];

        // Convert to Arrow batch
        let batch = vertices_to_batch(&coordinates);

        // Convert back to Vertices
        let vertices: Vertices<u32, QuantizedCoordinate> = batch_to_vertices(&batch).unwrap();

        // Verify results
        assert_eq!(vertices.len(), 3);
        assert_eq!(vertices.get(VertexIndex::new(0)).unwrap().x(), 10);
        assert_eq!(vertices.get(VertexIndex::new(0)).unwrap().y(), 20);
        assert_eq!(vertices.get(VertexIndex::new(0)).unwrap().z(), 30);
        assert_eq!(vertices.get(VertexIndex::new(1)).unwrap().x(), 40);
        assert_eq!(vertices.get(VertexIndex::new(1)).unwrap().y(), 50);
        assert_eq!(vertices.get(VertexIndex::new(1)).unwrap().z(), 60);
        assert_eq!(vertices.get(VertexIndex::new(2)).unwrap().x(), 70);
        assert_eq!(vertices.get(VertexIndex::new(2)).unwrap().y(), 80);
        assert_eq!(vertices.get(VertexIndex::new(2)).unwrap().z(), 90);
    }
}
