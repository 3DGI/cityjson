mod attributes_to_arrow;

use arrow::array::{
    ArrayData, ArrayRef, DictionaryArray, FixedSizeListArray, Int64Builder,
    Int8Array, RecordBatch, StringArray, StructArray,
};
use arrow::buffer::Buffer;
use arrow::datatypes::{DataType, Field, Fields, Int8Type, Schema};
use arrow::error::ArrowError;
use cityjson::prelude::{
    BBoxTrait, QuantizedCoordinate, ResourceRef, StringStorage, TransformTrait,
};
use cityjson::v2_0::{Contact, Metadata, Transform};
use std::sync::Arc;

use crate::attributes_to_arrow::attributes_to_arrow;

pub fn metadata_to_arrow<SS: StringStorage, RR: ResourceRef>(
    metadata: &Metadata<SS, RR>,
) -> Result<StructArray, ArrowError> {
    let mut fields = Vec::with_capacity(7);
    let mut arrays = Vec::with_capacity(7);

    if let Some(geographical_extent) = metadata.geographical_extent() {
        let field_geographical_extent = Field::new(
            "geographical_extent",
            DataType::FixedSizeList(Arc::new(Field::new_list_field(DataType::Float64, false)), 6),
            true,
        );
        fields.push(field_geographical_extent);

        let geographical_extent_data = ArrayData::builder(DataType::Float64)
            .len(6)
            .add_buffer(Buffer::from_slice_ref(geographical_extent.as_slice()))
            .build()?;
        let list_data_type =
            DataType::FixedSizeList(Arc::new(Field::new_list_field(DataType::Float64, false)), 6);
        let list_data = ArrayData::builder(list_data_type)
            .len(1)
            .add_child_data(geographical_extent_data)
            .build()?;
        let geographical_extent_array = FixedSizeListArray::from(list_data);
        arrays.push(Arc::new(geographical_extent_array) as ArrayRef);
    }

    if let Some(identifier) = metadata.identifier() {
        let field_identifier = Field::new("identifier", DataType::Utf8, true);
        fields.push(field_identifier);

        let identifier_array = StringArray::from(vec![identifier.to_string()]);
        arrays.push(Arc::new(identifier_array) as ArrayRef);
    }

    if let Some(point_of_contact) = metadata.point_of_contact() {
        let contact_array = contact_to_arrow(point_of_contact)?;
        let field_point_of_contact = Field::new(
            "point_of_contact",
            DataType::Struct(contact_array.fields().clone()),
            true,
        );
        fields.push(field_point_of_contact);
        arrays.push(Arc::new(contact_array) as ArrayRef);
    }

    if let Some(reference_date) = metadata.reference_date() {
        let field_reference_date = Field::new("reference_date", DataType::Utf8, true);
        fields.push(field_reference_date);

        let reference_date_array = StringArray::from(vec![reference_date.to_string()]);
        arrays.push(Arc::new(reference_date_array) as ArrayRef);
    }

    if let Some(reference_system) = metadata.reference_system() {
        let field_reference_system = Field::new("reference_system", DataType::Utf8, true);
        fields.push(field_reference_system);

        let reference_system_array = StringArray::from(vec![reference_system.to_string()]);
        arrays.push(Arc::new(reference_system_array) as ArrayRef);
    }

    if let Some(title) = metadata.title() {
        let field_title = Field::new("title", DataType::Utf8, true);
        fields.push(field_title);

        let title_array = StringArray::from(vec![title.to_string()]);
        arrays.push(Arc::new(title_array) as ArrayRef);
    }

    if let Some(extra) = metadata.extra() {
        let (schema, map_array) = attributes_to_arrow(extra, "extra")?;
        fields.push(schema.field(0).clone());
        arrays.push(Arc::new(map_array) as ArrayRef);
    }

    StructArray::try_new(Fields::from(fields), arrays, None)
}

pub fn contact_to_arrow<SS: StringStorage, RR: ResourceRef>(
    contact: &Contact<SS, RR>,
) -> Result<StructArray, ArrowError> {
    let mut fields = Vec::with_capacity(8);
    let mut arrays = Vec::with_capacity(8);

    let field_contact_name = Field::new("contact_name", DataType::Utf8, true);
    fields.push(field_contact_name);

    let contact_name_array = StringArray::from(vec![contact.contact_name().to_string()]);
    arrays.push(Arc::new(contact_name_array) as ArrayRef);

    let field_email_address = Field::new("email_address", DataType::Utf8, true);
    fields.push(field_email_address);

    let email_address_array = StringArray::from(vec![contact.email_address().to_string()]);
    arrays.push(Arc::new(email_address_array) as ArrayRef);

    if let Some(role) = contact.role() {
        let field_role = Field::new(
            "role",
            DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
            true,
        );
        fields.push(field_role);

        let role_key = vec![role as i8];
        let role_value = vec![role.to_string()];
        let role_array = DictionaryArray::<Int8Type>::try_new(
            Int8Array::from(role_key),
            Arc::new(StringArray::from(role_value)),
        )?;

        arrays.push(Arc::new(role_array) as ArrayRef);
    }

    if let Some(contact_type) = contact.contact_type() {
        let field_contact_type = Field::new(
            "contact_type",
            DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
            true,
        );
        fields.push(field_contact_type);

        let contact_type_key = vec![contact_type as i8];
        let contact_type_value = vec![contact_type.to_string()];
        let contact_type_array = DictionaryArray::<Int8Type>::try_new(
            Int8Array::from(contact_type_key),
            Arc::new(StringArray::from(contact_type_value)),
        )?;

        arrays.push(Arc::new(contact_type_array) as ArrayRef);
    }

    if let Some(website) = contact.website() {
        let field_website = Field::new("website", DataType::Utf8, true);
        fields.push(field_website);

        let website_array = StringArray::from(vec![website.to_string()]);
        arrays.push(Arc::new(website_array) as ArrayRef);
    }

    if let Some(organization) = contact.organization() {
        let field_organization = Field::new("organization", DataType::Utf8, true);
        fields.push(field_organization);

        let organization_array = StringArray::from(vec![organization.to_string()]);
        arrays.push(Arc::new(organization_array) as ArrayRef);
    }

    if let Some(phone) = contact.phone() {
        let field_phone = Field::new("phone", DataType::Utf8, true);
        fields.push(field_phone);

        let phone_array = StringArray::from(vec![phone.to_string()]);
        arrays.push(Arc::new(phone_array) as ArrayRef);
    }

    // todo: add extra fields

    Ok(StructArray::try_new(Fields::from(fields), arrays, None)?)
}

// todo: create specific error and conversion for crate
pub fn transform_to_arrow(transform: &Transform) -> Result<RecordBatch, ArrowError> {
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
    use arrow::array::{Array, FixedSizeListArray, StringArray, StructArray};
    use cityjson::prelude::*;
    use cityjson::v2_0::*;

    #[test]
    fn test_metadata_to_arrow() {
        // Create a test metadata object with all fields populated
        let mut metadata = Metadata::<OwnedStringStorage, ResourceId32>::new();

        // Set geographic extent (bounding box)
        metadata.set_geographical_extent(BBox::new(10.0, 20.0, 30.0, 40.0, 50.0, 60.0));

        // Set identifier
        metadata.set_identifier(CityModelIdentifier::new("test-dataset-id".to_string()));

        // Set reference date
        metadata.set_reference_date(Date::new("2024-04-05".to_string()));

        // Set reference system
        metadata.set_reference_system(CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/4326".to_string(),
        ));

        // Set title
        metadata.set_title("Test City Model");

        // Set point of contact
        metadata.set_contact_name("Test User");
        metadata.set_email_address("test@example.com");
        metadata.set_role(ContactRole::Author);
        metadata.set_website("https://example.com");
        metadata.set_contact_type(ContactType::Individual);
        metadata.set_organization("Test Organization");
        metadata.set_phone("+1-555-1234");

        // Set extra attributes
        let mut extra = OwnedAttributes::new();
        extra.insert(
            "version".to_string(),
            AttributeValue::String("1.0".to_string()),
        );
        extra.insert(
            "created".to_string(),
            AttributeValue::String("2024-04-05".to_string()),
        );
        metadata.extra_mut().replace(extra);

        // Convert metadata to Arrow
        let arrow_struct =
            metadata_to_arrow(&metadata).expect("Failed to convert metadata to Arrow");

        // Verify the result is a StructArray with the expected fields
        assert_eq!(arrow_struct.fields().len(), 7); // All fields should be present

        // Check geographical_extent
        let geo_extent_field = arrow_struct
            .column_by_name("geographical_extent")
            .expect("geographical_extent field missing");
        let geo_extent = geo_extent_field
            .as_any()
            .downcast_ref::<FixedSizeListArray>()
            .expect("geographical_extent should be a FixedSizeListArray");
        assert_eq!(geo_extent.len(), 1);

        // Check identifier
        let identifier_field = arrow_struct
            .column_by_name("identifier")
            .expect("identifier field missing");
        let identifier = identifier_field
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("identifier should be a StringArray");
        assert_eq!(identifier.value(0), "test-dataset-id");

        // Check reference_date
        let ref_date_field = arrow_struct
            .column_by_name("reference_date")
            .expect("reference_date field missing");
        let ref_date = ref_date_field
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("reference_date should be a StringArray");
        assert_eq!(ref_date.value(0), "2024-04-05");

        // Check reference_system
        let ref_system_field = arrow_struct
            .column_by_name("reference_system")
            .expect("reference_system field missing");
        let ref_system = ref_system_field
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("reference_system should be a StringArray");
        assert_eq!(
            ref_system.value(0),
            "https://www.opengis.net/def/crs/EPSG/0/4326"
        );

        // Check title
        let title_field = arrow_struct
            .column_by_name("title")
            .expect("title field missing");
        let title = title_field
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("title should be a StringArray");
        assert_eq!(title.value(0), "Test City Model");

        // Check point_of_contact
        let contact_field = arrow_struct
            .column_by_name("point_of_contact")
            .expect("point_of_contact field missing");
        let contact = contact_field
            .as_any()
            .downcast_ref::<StructArray>()
            .expect("point_of_contact should be a StructArray");

        // Verify contact fields
        let contact_name = contact
            .column_by_name("contact_name")
            .expect("contact_name field missing")
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("contact_name should be a StringArray");
        assert_eq!(contact_name.value(0), "Test User");

        let email = contact
            .column_by_name("email_address")
            .expect("email_address field missing")
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("email_address should be a StringArray");
        assert_eq!(email.value(0), "test@example.com");

        // Check extra attributes
        let _extra_field = arrow_struct
            .column_by_name("extra")
            .expect("extra field missing");

        // Successfully converted all metadata fields to Arrow format
        println!(
            "Successfully converted metadata to Arrow structure with {} fields",
            arrow_struct.fields().len()
        );
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
}
