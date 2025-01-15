use crate::common::DATA_DIR;
use cjlib::{
    Attributes, BBox, CityJSONVersion, CityModel, CityModelIdentifier, CityObject, CityObjectType,
    Contact, ContactRole, ContactType, Date, Extension, ExtensionName, Extensions, Transform, CRS,
};
use serde_cityjson::CityModelType;
use std::collections::HashMap;

mod common;

#[test]
fn init_citymodel() {
    let _cm = CityModel::new(CityModelType::CityJSON);
    let _cm2 = CityModel::default();
}

#[test]
fn citymodel_from_str_minimal() {
    let cityjson_str = r#"{
      "type": "CityJSON",
      "version": "1.1",
      "extensions": {},
      "transform": {
        "scale": [ 1.0, 1.0, 1.0 ],
        "translate": [ 0.0, 0.0, 0.0 ]
      },
      "metadata": {},
      "CityObjects": {},
      "vertices": [],
      "appearance": {},
      "geometry-templates": {
        "templates": [],
        "vertices-templates": []
      }
    }"#;
    assert!(CityModel::from_str(cityjson_str).is_ok());
}

#[test]
fn citymodel_from_str_dummy() {
    let cityjson_str = std::fs::read_to_string(DATA_DIR.join("cityjson_dummy_complete.city.json"))
        .expect("Failed to read the file");
    let cm = CityModel::from_str(cityjson_str.as_str()).unwrap();
    println!("\nDummy:\n{}", &cm);
}

#[test]
fn debug_citymodel() {
    let cm = CityModel::new(CityModelType::CityJSON);
    println!("\nDebug CityModel:\n{:?}", cm);
}

#[test]
fn display_citymodel() {
    let cm = CityModel::new(CityModelType::CityJSON);
    println!("\nDisplay CityModel:\n{}", cm);
}

#[test]
fn test_version() {
    let mut cm = CityModel::default();
    assert_eq!(cm.version(), &Some(CityJSONVersion::default()));

    // Test modification through mutable reference
    *cm.version_mut() = CityJSONVersion::V1_0;
    assert_eq!(cm.version(), &Some(CityJSONVersion::V1_0));
}

#[test]
fn test_transform() {
    // Test Transform construction and getters
    let scale = [1.0, 1.0, 1.0];
    let translate = [0.0, 0.0, 0.0];
    let mut transform = Transform::new(scale, translate);
    assert_eq!(transform.scale(), &scale);
    assert_eq!(transform.translate(), &translate);

    // Test setters
    let new_scale = [2.0, 2.0, 2.0];
    let new_translate = [5.0, 5.0, 5.0];
    transform.set_scale(new_scale);
    transform.set_translate(new_translate);
    assert_eq!(transform.scale(), &new_scale);
    assert_eq!(transform.translate(), &new_translate);

    // Test Transform in CityModel
    let mut cm = CityModel::default();
    assert!(cm.transform().is_none());

    let cm_transform = cm.transform_mut();
    cm_transform.set_scale(new_scale);
    cm_transform.set_translate(new_translate);

    assert_eq!(cm.transform().unwrap().scale(), &new_scale);
    assert_eq!(cm.transform().unwrap().translate(), &new_translate);
}

#[test]
fn test_citymodel_extensions() {
    let mut cm = CityModel::new(CityModelType::CityJSON);
    assert!(cm.extensions().is_none());

    let ext_name = "test";
    let ext_name_2 = ExtensionName::from("test_2");
    let ext = Extension::new("https://example.com/ext".to_string(), "1.0".to_string());
    cm.extensions_mut().insert(ext_name, ext.clone());
    cm.extensions_mut().insert(ext_name_2, ext.clone());
    assert_eq!(cm.extensions().as_ref().unwrap().get(ext_name), Some(&ext));

    let ext_removed = cm.extensions_mut().remove(ext_name).unwrap();
    assert_eq!(ext, ext_removed);
}

#[test]
fn test_extensions() {
    let mut extensions = Extensions::new();
    let ext = Extension::new("https://example.com/ext".to_string(), "1.0".to_string());
    let name = "test".to_string();

    // Insert and retrieve
    extensions.insert(&name, ext.clone());
    assert!(extensions.contains(&name));
    assert_eq!(extensions.get(&name), Some(&ext));

    // Remove
    assert_eq!(extensions.remove(&name), Some(ext));
    assert!(!extensions.contains(&name));

    // Iteration
    let mut extensions = Extensions::new();

    extensions.insert(
        &"ext1".to_string(),
        Extension::new("https://example.com/ext1".to_string(), "1.0".to_string()),
    );
    extensions.insert(
        &"ext2".to_string(),
        Extension::new("https://example.com/ext2".to_string(), "2.0".to_string()),
    );

    // Test iter()
    let mut count = 0;
    for (name, ext) in extensions.iter() {
        assert!(["ext1", "ext2"].contains(&name.as_str()));
        assert!(ext.url().starts_with("https://example.com/"));
        count += 1;
    }
    assert_eq!(count, 2);

    // Test iter_mut()
    for (_, ext) in extensions.iter_mut() {
        ext.set_version("3.0".to_string());
    }

    // Verify all versions were updated
    for (_, ext) in &extensions {
        assert_eq!(ext.version(), "3.0");
    }
}

#[test]
fn test_extension() {
    let url = "https://example.com/ext".to_string();
    let version = "1.0".to_string();
    let mut ext = Extension::new(url.clone(), version.clone());

    // Check initial values
    assert_eq!(ext.url(), url);
    assert_eq!(ext.version(), version);

    // Update values
    let new_url = "https://example.com/ext2".to_string();
    let new_version = "2.0".to_string();
    ext.set_url(new_url.clone());
    ext.set_version(new_version.clone());

    assert_eq!(ext.url(), new_url);
    assert_eq!(ext.version(), new_version);
}

#[test]
fn test_extra_root_properties() {
    let mut cm = CityModel::default();

    // Initially, extra root properties should be None
    assert!(cm.extra_root_properties().is_none());

    // Getting mutable reference should create empty Attributes
    let extra = cm.extra_root_properties_mut();
    assert!(extra.is_null()); // Default is Null

    // Create a map with various types
    let mut map = HashMap::new();
    map.insert("string".to_string(), Attributes::String("test".to_string()));
    map.insert("number".to_string(), Attributes::Integer(42));
    map.insert("boolean".to_string(), Attributes::Bool(true));
    map.insert("float".to_string(), Attributes::Float(3.14));

    *extra = Attributes::Map(map);

    // Test reading values through immutable reference
    if let Some(extra) = cm.extra_root_properties() {
        if let Some(map) = extra.as_map() {
            assert_eq!(map.get("string").unwrap().as_str(), Some("test"));
            assert_eq!(map.get("number").unwrap().as_integer(), Some(42));
            assert_eq!(map.get("boolean").unwrap().as_bool(), Some(true));
            assert_eq!(map.get("float").unwrap().as_float(), Some(3.14));
        } else {
            panic!("Expected Map variant");
        }
    } else {
        panic!("Extra root properties should exist");
    }

    // Test modification through mutable reference
    if let Some(map) = cm.extra_root_properties_mut().as_map_mut() {
        map.insert(
            "new_value".to_string(),
            Attributes::String("added later".to_string()),
        );
    }

    // Verify all values including the new one
    if let Some(map) = cm.extra_root_properties().unwrap().as_map() {
        assert_eq!(map.get("string").unwrap().as_str(), Some("test"));
        assert_eq!(map.get("number").unwrap().as_integer(), Some(42));
        assert_eq!(map.get("boolean").unwrap().as_bool(), Some(true));
        assert_eq!(map.get("float").unwrap().as_float(), Some(3.14));
        assert_eq!(map.get("new_value").unwrap().as_str(), Some("added later"));
    } else {
        panic!("Expected Map variant");
    }
}

#[test]
fn test_metadata() {
    let mut cm = CityModel::default();

    // Initially, metadata should be None
    assert!(cm.metadata().is_none());

    // Getting mutable reference should create default Metadata
    let metadata = cm.metadata_mut();

    // Test setting and getting geographical extent
    let bbox = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    metadata.set_geographical_extent(bbox);
    assert_eq!(metadata.geographical_extent(), Some(&bbox));

    // Test setting and getting identifier
    metadata.set_identifier("test-id");
    assert_eq!(
        metadata.identifier(),
        Some(&CityModelIdentifier::from("test-id"))
    );

    // Test setting and getting title
    metadata.set_title("Test Title");
    assert_eq!(metadata.title(), Some(&"Test Title".to_string()));

    // Test setting and getting reference system
    metadata.set_reference_system("EPSG:4326");
    assert_eq!(metadata.reference_system(), Some(&CRS::from("EPSG:4326")));

    // Test setting and getting reference date
    metadata.set_reference_date("2023-01-01");
    assert_eq!(metadata.reference_date(), Some(&Date::from("2023-01-01")));

    // Test setting and getting point of contact
    let mut contact = Contact::new();
    contact.set_contact_name("John Doe");
    contact.set_email_address("john@example.com");
    contact.set_role(ContactRole::Author);
    contact.set_organization("Test Org");
    metadata.set_point_of_contact(contact);

    // Verify contact information
    if let Some(contact) = metadata.point_of_contact() {
        assert_eq!(contact.contact_name(), "John Doe");
        assert_eq!(contact.email_address(), "john@example.com");
        assert_eq!(contact.role(), Some(&ContactRole::Author));
        assert_eq!(contact.organization(), Some("Test Org"));
    } else {
        panic!("Expected contact information");
    }

    // Test modifying contact through mutable reference
    let contact = metadata.point_of_contact_mut();
    contact.set_website("https://example.com");
    contact.set_phone("+1234567890");
    contact.set_contact_type(ContactType::Individual);

    // Verify modified contact information
    if let Some(contact) = metadata.point_of_contact() {
        assert_eq!(contact.website(), Some("https://example.com"));
        assert_eq!(contact.phone(), Some("+1234567890"));
        assert_eq!(contact.contact_type(), Some(&ContactType::Individual));
    }

    // Test extra attributes
    let mut map = HashMap::new();
    map.insert("key1".to_string(), Attributes::String("value1".to_string()));
    metadata.set_extra(Attributes::Map(map));

    if let Some(Attributes::Map(map)) = metadata.extra() {
        assert_eq!(map.get("key1").unwrap().as_str(), Some("value1"));
    } else {
        panic!("Expected Map variant for extra attributes");
    }

    // Test Display implementation
    println!();
    println!("Metadata: {}", metadata);
    println!("Contact: {}", metadata.point_of_contact().unwrap());
}

#[test]
fn test_cityobject_creation() {
    let co = CityObject::new(CityObjectType::Building);

    // Test initial state
    assert_eq!(co.type_co(), &CityObjectType::Building);
    assert!(co.attributes().is_none());
    assert!(co.geographical_extent().is_none());
    assert!(co.children().is_none());
    assert!(co.parents().is_none());
    assert!(co.extra().is_none());
}

#[test]
fn test_cityobject_setters() {
    let mut co = CityObject::new(CityObjectType::Building);

    // Test attributes
    let mut attrs = HashMap::new();
    attrs.insert("height".to_string(), Attributes::Float(10.5));
    attrs.insert("year".to_string(), Attributes::Integer(2023));
    co.set_attributes(Attributes::Map(attrs));

    // Test geographical extent
    let bbox: BBox = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    co.set_geographical_extent(bbox);

    // Test children and parents
    let children = vec!["child1".to_string(), "child2".to_string()];
    let parents = vec!["parent1".to_string(), "parent2".to_string()];
    co.set_children(children.clone());
    co.set_parents(parents.clone());

    // Test extra attributes
    let mut extra = HashMap::new();
    extra.insert("note".to_string(), Attributes::String("test".to_string()));
    co.set_extra(Attributes::Map(extra));

    // Verify all values
    if let Some(Attributes::Map(map)) = co.attributes() {
        assert_eq!(map.get("height").unwrap().as_float(), Some(10.5));
        assert_eq!(map.get("year").unwrap().as_integer(), Some(2023));
    } else {
        panic!("Expected Map variant for attributes");
    }

    assert_eq!(co.geographical_extent(), Some(&bbox));
    assert_eq!(co.children(), Some(&children));
    assert_eq!(co.parents(), Some(&parents));

    if let Some(Attributes::Map(map)) = co.extra() {
        assert_eq!(map.get("note").unwrap().as_str(), Some("test"));
    } else {
        panic!("Expected Map variant for extra");
    }
}

#[test]
fn test_cityobject_mutable_access() {
    let mut co = CityObject::new(CityObjectType::Building);

    // Test attributes modification
    let mut map = HashMap::new();
    map.insert("height".to_string(), Attributes::Float(10.5));
    *co.attributes_mut() = Some(Attributes::Map(map));

    // Test geographical extent modification
    let bbox: BBox = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    *co.geographical_extent_mut() = Some(bbox);

    // Test children and parents modification
    let children = vec!["child1".to_string()];
    let parents = vec!["parent1".to_string()];
    *co.children_mut() = Some(children.clone());
    *co.parents_mut() = Some(parents.clone());

    // Verify modifications
    if let Some(Attributes::Map(map)) = co.attributes() {
        assert_eq!(map.get("height").unwrap().as_float(), Some(10.5));
    } else {
        panic!("Expected Map variant for attributes");
    }

    assert_eq!(co.geographical_extent(), Some(&bbox));
    assert_eq!(co.children(), Some(&children));
    assert_eq!(co.parents(), Some(&parents));
}
