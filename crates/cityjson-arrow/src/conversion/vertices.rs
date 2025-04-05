use arrow::array::{ArrayRef, Int64Builder, RecordBatch, StructArray};
use arrow::datatypes::{DataType, Field};
use cityjson::prelude::QuantizedCoordinate;
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

#[cfg(test)]
mod tests {
    use super::*;
    use cityjson::prelude::*;

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
}
