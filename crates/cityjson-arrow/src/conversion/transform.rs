use arrow::array::{ArrayData, ArrayRef, FixedSizeListArray, RecordBatch};
use arrow::buffer::Buffer;
use arrow::datatypes::{DataType, Field, Schema};
use cityjson::prelude::TransformTrait;
use cityjson::v2_0::Transform;
use std::sync::Arc;

use crate::error::{Error, Result};

pub fn transform_to_arrow(transform: &Transform) -> Result<RecordBatch> {
    // Create arrays of values
    let scale_value_data = ArrayData::builder(DataType::Float64)
        .len(3)
        .add_buffer(Buffer::from_slice_ref(transform.scale()))
        .build()?;
    let translate_value_data = ArrayData::builder(DataType::Float64)
        .len(3)
        .add_buffer(Buffer::from_slice_ref(transform.translate()))
        .build()?;

    let scale_list_data_type =
        DataType::FixedSizeList(Arc::new(Field::new_list_field(DataType::Float64, false)), 3);
    let translate_list_data_type =
        DataType::FixedSizeList(Arc::new(Field::new_list_field(DataType::Float64, false)), 3);

    let scale_list_data = ArrayData::builder(scale_list_data_type.clone())
        .len(1)
        .add_child_data(scale_value_data)
        .build()?;
    let translate_list_data = ArrayData::builder(translate_list_data_type.clone())
        .len(1)
        .add_child_data(translate_value_data)
        .build()?;

    // Wrap the f64 arrays in FixedSizeListArrays.

    let scale_listarray = FixedSizeListArray::from(scale_list_data);
    let translate_listarray = FixedSizeListArray::from(translate_list_data);

    let schema = Schema::new(vec![
        Field::new("scale", scale_list_data_type.clone(), false),
        Field::new("translate", translate_list_data_type.clone(), false),
    ]);

    // Create a RecordBatch with a single row.
    RecordBatch::try_new(
        Arc::new(schema),
        vec![
            Arc::new(scale_listarray) as ArrayRef,
            Arc::new(translate_listarray) as ArrayRef,
        ],
    )
    .map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::FixedSizeListArray;
    use cityjson::v2_0::Transform;

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
        dbg!(
            batch
                .column(0)
                .as_any()
                .downcast_ref::<FixedSizeListArray>()
                .unwrap()
        );
        dbg!(
            batch
                .column(1)
                .as_any()
                .downcast_ref::<FixedSizeListArray>()
                .unwrap()
        );
    }
}
