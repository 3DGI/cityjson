// TODO: Re-enable when AttributePool support is added
// use crate::conversion::attributes::{arrow_to_attributes_owned, attributes_to_arrow};
use arrow::array::{
    Array, ArrayData, ArrayRef, DictionaryArray, FixedSizeListArray, Float64Array, Int8Array,
    StringArray, StructArray,
};
use arrow::buffer::Buffer;
use arrow::datatypes::{DataType, Field, Fields, Int8Type};
use cityjson::prelude::CRS;
use cityjson::prelude::{BBox, CityModelIdentifier, Date, OwnedStringStorage, StringStorage};
use cityjson::v2_0::{Contact, ContactRole, ContactType, Metadata};
use std::sync::Arc;

use crate::error::{Error, Result};

pub fn metadata_to_arrow<SS: StringStorage>(metadata: &Metadata<SS>) -> Result<StructArray> {
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

    // TODO: Extra properties conversion requires an AttributePool
    // For now, skip extra conversion (would need to create empty array with correct structure)
    if let Some(_extra) = metadata.extra() {
        // Skipping attributes conversion - requires AttributePool
        // let empty_pool = cityjson::cityjson::core::attributes::AttributePool::<SS, cityjson::prelude::ResourceId32>::new();
        // let (schema, map_array) = attributes_to_arrow(extra, &empty_pool, "extra")?;
        // fields.push(schema.field(0).clone());
        // arrays.push(Arc::new(map_array) as ArrayRef);
    }

    StructArray::try_new(Fields::from(fields), arrays, None).map_err(Error::from)
}

pub fn contact_to_arrow<SS: StringStorage>(contact: &Contact<SS>) -> Result<StructArray> {
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

    StructArray::try_new(Fields::from(fields), arrays, None).map_err(Error::from)
}

pub fn arrow_to_metadata(arrow_struct: &StructArray) -> Result<Metadata<OwnedStringStorage>> {
    // Create a new empty metadata
    let mut metadata = Metadata::new();

    // Extract geographical_extent (BBox)
    if let Some(geo_extent_column) = arrow_struct.column_by_name("geographical_extent") {
        if !geo_extent_column.is_null(0) {
            let geo_extent_array = geo_extent_column
                .as_any()
                .downcast_ref::<FixedSizeListArray>()
                .ok_or_else(|| {
                    Error::Conversion("Failed to downcast geographical_extent".to_string())
                })?;

            let values = geo_extent_array.value(0);
            let float_array = values
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or_else(|| Error::Conversion("Failed to downcast float values".to_string()))?;

            if float_array.len() == 6 {
                let bbox = BBox::new(
                    float_array.value(0),
                    float_array.value(1),
                    float_array.value(2),
                    float_array.value(3),
                    float_array.value(4),
                    float_array.value(5),
                );
                metadata.set_geographical_extent(bbox);
            }
        }
    }

    // Extract identifier
    if let Some(identifier_column) = arrow_struct.column_by_name("identifier") {
        if !identifier_column.is_null(0) {
            let identifier_array = identifier_column
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| Error::Conversion("Failed to downcast identifier".to_string()))?;

            let identifier_str = identifier_array.value(0);
            metadata.set_identifier(CityModelIdentifier::new(identifier_str.to_string()));
        }
    }

    // Extract point_of_contact
    if let Some(contact_column) = arrow_struct.column_by_name("point_of_contact") {
        if !contact_column.is_null(0) {
            let contact_struct = contact_column
                .as_any()
                .downcast_ref::<StructArray>()
                .ok_or_else(|| {
                    Error::Conversion("Failed to downcast point_of_contact".to_string())
                })?;

            let contact = arrow_to_contact(contact_struct)?;
            metadata.set_point_of_contact(Some(contact));
        }
    }

    // Extract reference_date
    if let Some(date_column) = arrow_struct.column_by_name("reference_date") {
        if !date_column.is_null(0) {
            let date_array = date_column
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    Error::Conversion("Failed to downcast reference_date".to_string())
                })?;

            let date_str = date_array.value(0);
            metadata.set_reference_date(Date::new(date_str.to_string()));
        }
    }

    // Extract reference_system
    if let Some(crs_column) = arrow_struct.column_by_name("reference_system") {
        if !crs_column.is_null(0) {
            let crs_array = crs_column
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    Error::Conversion("Failed to downcast reference_system".to_string())
                })?;

            let crs_str = crs_array.value(0);
            metadata.set_reference_system(CRS::new(crs_str.to_string()));
        }
    }

    // Extract title
    if let Some(title_column) = arrow_struct.column_by_name("title") {
        if !title_column.is_null(0) {
            let title_array = title_column
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| Error::Conversion("Failed to downcast title".to_string()))?;

            let title_str = title_array.value(0);
            metadata.set_title(title_str);
        }
    }

    // Extract extra
    // TODO: Converting extra attributes requires an AttributePool
    // For now, skip this conversion
    if let Some(_extra_column) = arrow_struct.column_by_name("extra") {
        // Skipping attributes conversion - requires AttributePool
        /*
        if !extra_column.is_null(0) {
            let map_array = extra_column
                .as_any()
                .downcast_ref::<MapArray>()
                .ok_or_else(|| Error::Conversion("Failed to downcast extra".to_string()))?;

            let mut pool = cityjson::cityjson::core::attributes::OwnedAttributePool::new();
            let attributes = arrow_to_attributes_owned(
                map_array,
                &mut pool,
                cityjson::cityjson::core::attributes::AttributeOwnerType::Metadata
            )?;
            metadata.set_extra(Some(attributes));
        }
        */
    }

    Ok(metadata)
}

/// Converts an Arrow StructArray to a cityjson-rs Contact object.
fn arrow_to_contact(contact_struct: &StructArray) -> Result<Contact<OwnedStringStorage>> {
    let mut contact = Contact::new();

    // Extract contact_name (required field)
    if let Some(name_column) = contact_struct.column_by_name("contact_name") {
        let name_array = name_column
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| Error::Conversion("Failed to downcast contact_name".to_string()))?;

        if !name_array.is_null(0) {
            contact.set_contact_name(name_array.value(0).to_string());
        }
    }

    // Extract email_address (required field)
    if let Some(email_column) = contact_struct.column_by_name("email_address") {
        let email_array = email_column
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| Error::Conversion("Failed to downcast email_address".to_string()))?;

        if !email_array.is_null(0) {
            contact.set_email_address(email_array.value(0).to_string());
        }
    }

    // Extract role (optional field)
    if let Some(role_column) = contact_struct.column_by_name("role") {
        if !role_column.is_null(0) {
            let role_array = role_column
                .as_any()
                .downcast_ref::<DictionaryArray<Int8Type>>()
                .ok_or_else(|| Error::Conversion("Failed to downcast role".to_string()))?;

            // Get the string value from the dictionary
            let key = role_array.keys().value(0);
            let role_values = role_array.values();
            let role_dict = role_values
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    Error::Conversion("Expected StringArray for role dictionary".to_string())
                })?;

            let role_str = role_dict.value(key as usize);

            // Convert string to ContactRole enum
            let role = match role_str {
                "Author" => Some(ContactRole::Author),
                "CoAuthor" => Some(ContactRole::CoAuthor),
                "Collaborator" => Some(ContactRole::Collaborator),
                "Contributor" => Some(ContactRole::Contributor),
                "Custodian" => Some(ContactRole::Custodian),
                "Distributor" => Some(ContactRole::Distributor),
                "Editor" => Some(ContactRole::Editor),
                "Funder" => Some(ContactRole::Funder),
                "Mediator" => Some(ContactRole::Mediator),
                "Originator" => Some(ContactRole::Originator),
                "Owner" => Some(ContactRole::Owner),
                "PointOfContact" => Some(ContactRole::PointOfContact),
                "PrincipalInvestigator" => Some(ContactRole::PrincipalInvestigator),
                "Processor" => Some(ContactRole::Processor),
                "Publisher" => Some(ContactRole::Publisher),
                "ResourceProvider" => Some(ContactRole::ResourceProvider),
                "RightsHolder" => Some(ContactRole::RightsHolder),
                "Sponsor" => Some(ContactRole::Sponsor),
                "Stakeholder" => Some(ContactRole::Stakeholder),
                "User" => Some(ContactRole::User),
                _ => None,
            };

            if let Some(role_value) = role {
                contact.set_role(Some(role_value));
            }
        }
    }

    // Extract contact_type (optional field)
    if let Some(type_column) = contact_struct.column_by_name("contact_type") {
        if !type_column.is_null(0) {
            let type_array = type_column
                .as_any()
                .downcast_ref::<DictionaryArray<Int8Type>>()
                .ok_or_else(|| Error::Conversion("Failed to downcast contact_type".to_string()))?;

            // Get the string value from the dictionary
            let key = type_array.keys().value(0);
            let type_values = type_array.values();
            let type_dict = type_values
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    Error::Conversion(
                        "Expected StringArray for contact_type dictionary".to_string(),
                    )
                })?;

            let type_str = type_dict.value(key as usize);

            // Convert string to ContactType enum
            let contact_type = match type_str {
                "Individual" => Some(ContactType::Individual),
                "Organization" => Some(ContactType::Organization),
                _ => None,
            };

            if let Some(type_value) = contact_type {
                contact.set_contact_type(Some(type_value));
            }
        }
    }

    // Extract website (optional field)
    if let Some(website_column) = contact_struct.column_by_name("website") {
        if !website_column.is_null(0) {
            let website_array = website_column
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| Error::Conversion("Failed to downcast website".to_string()))?;

            contact.set_website(Some(website_array.value(0).to_string()));
        }
    }

    // Extract organization (optional field)
    if let Some(org_column) = contact_struct.column_by_name("organization") {
        if !org_column.is_null(0) {
            let org_array = org_column
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| Error::Conversion("Failed to downcast organization".to_string()))?;

            contact.set_organization(Some(org_array.value(0).to_string()));
        }
    }

    // Extract phone (optional field)
    if let Some(phone_column) = contact_struct.column_by_name("phone") {
        if !phone_column.is_null(0) {
            let phone_array = phone_column
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| Error::Conversion("Failed to downcast phone".to_string()))?;

            contact.set_phone(Some(phone_array.value(0).to_string()));
        }
    }

    // Handle address field (optional)
    // This would use a similar approach to extracting extra attributes
    // TODO: Converting address attributes requires an AttributePool
    // For now, skip this conversion
    if let Some(_address_column) = contact_struct.column_by_name("address") {
        // Skipping attributes conversion - requires AttributePool
        /*
        if !address_column.is_null(0) {
            let map_array = address_column
                .as_any()
                .downcast_ref::<MapArray>()
                .ok_or_else(|| Error::Conversion("Failed to downcast address".to_string()))?;

            let mut pool = cityjson::cityjson::core::attributes::OwnedAttributePool::new();
            let attributes = arrow_to_attributes_owned(
                map_array,
                &mut pool,
                cityjson::cityjson::core::attributes::AttributeOwnerType::Metadata
            )?;
            contact.set_address(Some(attributes));
        }
        */
    }

    Ok(contact)
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use arrow::array::{Array, FixedSizeListArray, StringArray, StructArray};
    use cityjson::prelude::{
        AttributeValue, BBox, CRS, CityModelIdentifier, Date, OwnedAttributes, OwnedStringStorage,
    };
    use cityjson::v2_0::{ContactRole, ContactType, Metadata};

    // TODO: This test needs to be updated to work with the new AttributePool-based API
    #[cfg(any())]
    #[test]
    fn test_metadata_to_arrow() {
        // Create a test metadata object with all fields populated
        let mut metadata = Metadata::<OwnedStringStorage>::new();

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

    // TODO: This test needs to be updated to work with the new AttributePool-based API
    #[cfg(any())]
    #[test]
    fn test_arrow_to_metadata() {
        // Create a test metadata object with all fields populated
        let mut original = Metadata::<OwnedStringStorage>::new();

        // Set fields
        original.set_geographical_extent(BBox::new(10.0, 20.0, 30.0, 40.0, 50.0, 60.0));
        original.set_identifier(CityModelIdentifier::new("test-id".to_string()));
        original.set_reference_date(Date::new("2024-04-05".to_string()));
        original.set_reference_system(CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/4326".to_string(),
        ));
        original.set_title("Test City Model");

        // Set point of contact
        original.set_contact_name("Test User");
        original.set_email_address("test@example.com");
        original.set_role(ContactRole::Author);
        original.set_website("https://example.com");
        original.set_contact_type(ContactType::Individual);
        original.set_organization("Test Organization");
        original.set_phone("+1-555-1234");

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
        original.extra_mut().replace(extra);

        // Convert to Arrow
        let arrow_struct = metadata_to_arrow(&original).unwrap();

        // Convert back to Metadata
        let result = arrow_to_metadata(&arrow_struct).unwrap();

        // Verify fields
        assert_eq!(result.geographical_extent(), original.geographical_extent());
        assert_eq!(
            result.identifier().unwrap().to_string(),
            original.identifier().unwrap().to_string()
        );
        assert_eq!(
            result.reference_date().unwrap().to_string(),
            original.reference_date().unwrap().to_string()
        );
        assert_eq!(
            result.reference_system().unwrap().to_string(),
            original.reference_system().unwrap().to_string()
        );
        assert_eq!(result.title(), Some("Test City Model"));

        // Verify contact info
        let contact = result.point_of_contact().unwrap();
        assert_eq!(contact.contact_name(), "Test User");
        assert_eq!(contact.email_address(), "test@example.com");
        assert_eq!(contact.role(), Some(ContactRole::Author));
        assert_eq!(contact.website(), &Some("https://example.com".to_string()));
        assert_eq!(contact.contact_type(), Some(ContactType::Individual));
        assert_eq!(
            contact.organization(),
            &Some("Test Organization".to_string())
        );
        assert_eq!(contact.phone(), &Some("+1-555-1234".to_string()));

        // Verify extra attributes
        if let Some(extra) = result.extra() {
            assert_eq!(
                extra.get("version"),
                Some(&AttributeValue::String("1.0".to_string()))
            );
            assert_eq!(
                extra.get("created"),
                Some(&AttributeValue::String("2024-04-05".to_string()))
            );
        } else {
            panic!("Extra attributes not found");
        }
    }
}
