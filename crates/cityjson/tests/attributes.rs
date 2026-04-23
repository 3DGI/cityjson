//! Tests for attribute functionality.

const FLOAT_EPSILON: f64 = 1.0e-12;

fn assert_f64_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= FLOAT_EPSILON,
        "expected {expected}, got {actual} (epsilon {FLOAT_EPSILON})"
    );
}

mod value_types {
    //! Tests for attribute value types.
    //! Verifies that it is possible to create and retrieve attributes of all the
    //! possible value types.

    use super::assert_f64_eq;
    use cityjson::v2_0::*;
    use std::collections::HashMap;

    /// Can we insert and retrieve scalar attribute values?
    #[test]
    fn test_scalar() {
        let mut attrs = OwnedAttributes::new();

        // Insert all values
        attrs.insert("float".to_string(), OwnedAttributeValue::Float(42.5));
        attrs.insert(
            "string".to_string(),
            OwnedAttributeValue::String("string".to_string()),
        );
        attrs.insert("integer".to_string(), OwnedAttributeValue::Integer(10));
        attrs.insert("bool".to_string(), OwnedAttributeValue::Bool(true));
        attrs.insert("unsigned".to_string(), OwnedAttributeValue::Unsigned(2026));
        attrs.insert("null".to_string(), OwnedAttributeValue::Null);

        // Verify values
        if let Some(OwnedAttributeValue::Null) = attrs.get("null") {
            // Expected behavior
        } else {
            panic!("Expected null value");
        }

        if let Some(OwnedAttributeValue::Float(h)) = attrs.get("float") {
            assert_f64_eq(*h, 42.5);
        } else {
            panic!("Expected float value");
        }

        if let Some(OwnedAttributeValue::String(n)) = attrs.get("string") {
            assert_eq!(n, "string");
        } else {
            panic!("Expected string value");
        }

        if let Some(OwnedAttributeValue::Unsigned(u)) = attrs.get("unsigned") {
            assert_eq!(*u, 2026);
        } else {
            panic!("Expected unsigned value");
        }

        if let Some(OwnedAttributeValue::Integer(i)) = attrs.get("integer") {
            assert_eq!(*i, 10);
        } else {
            panic!("Expected integer value");
        }

        if let Some(OwnedAttributeValue::Bool(b)) = attrs.get("bool") {
            assert!(*b);
        } else {
            panic!("Expected bool value");
        }
    }

    /// Can we create an attribute that is a map?
    /// Tests "attributes": { "address": { "street": "Main St", "number": 123 } }
    #[test]
    fn test_map() {
        let mut attrs = OwnedAttributes::new();

        // Create nested map (like address)
        let mut address_map = HashMap::new();
        address_map.insert(
            "street".to_string(),
            OwnedAttributeValue::String("Main St".to_string()),
        );
        address_map.insert("number".to_string(), OwnedAttributeValue::Integer(123));

        attrs.insert("address".to_string(), OwnedAttributeValue::Map(address_map));

        // Verify nested access
        if let Some(OwnedAttributeValue::Map(addr)) = attrs.get("address")
            && let Some(street_val) = addr.get("street")
            && let OwnedAttributeValue::String(s) = street_val
        {
            assert_eq!(s, "Main St");
        }
    }

    /// Can we create an attribute that is a vector?
    /// Tests "attributes": { "materials": ["concrete", 1, null] }
    #[test]
    fn test_vector() {
        let mut attrs = OwnedAttributes::new();

        // Create a vector of Box<AttributeValue> to store the value array
        let materials = vec![
            OwnedAttributeValue::String("concrete".to_string()),
            OwnedAttributeValue::Integer(1),
            OwnedAttributeValue::Null,
        ];
        // Add the vector to the attributes
        attrs.insert("materials".to_string(), OwnedAttributeValue::Vec(materials));

        // Verify vector access
        if let Some(OwnedAttributeValue::Vec(mats)) = attrs.get("materials") {
            assert_eq!(mats.len(), 3);

            if let OwnedAttributeValue::String(first) = &mats[0] {
                assert_eq!(first, "concrete");
            }
            if let OwnedAttributeValue::Integer(first) = &mats[1] {
                assert_eq!(*first, 1);
            }
            assert!(OwnedAttributeValue::Null == mats[2], "Expected null value");
        }
    }
}

mod operations {
    //! Tests for attribute operations.
    //! Verifies that all supported attribute operations are working correctly.

    use cityjson::resources::GeometryHandle;
    use cityjson::v2_0::*;
    use std::collections::HashMap;

    /// Can we display attributes in a human-readable format?
    #[test]
    fn test_display() {
        let mut attrs = OwnedAttributes::new();
        let values: Vec<(&str, OwnedAttributeValue, &str)> = vec![
            ("null", AttributeValue::Null, "null"),
            ("bool", AttributeValue::Bool(true), "true"),
            ("unsigned", AttributeValue::Unsigned(2026), "2026"),
            ("integer", AttributeValue::Integer(-42), "-42"),
            (
                "float",
                AttributeValue::Float(std::f64::consts::PI),
                "3.141592653589793",
            ),
            (
                "string",
                AttributeValue::String("test".to_string()),
                "\"test\"",
            ),
            (
                "vec",
                AttributeValue::Vec(vec![
                    AttributeValue::String("a".to_string()),
                    AttributeValue::Integer(1),
                    AttributeValue::Null,
                ]),
                "[\"a\", 1, null]",
            ),
            (
                "map",
                AttributeValue::Map(HashMap::from([(
                    "inner".to_string(),
                    AttributeValue::Unsigned(7),
                )])),
                "{\"inner\": 7}",
            ),
            (
                "geometry",
                AttributeValue::Geometry(GeometryHandle::default()),
                "Geometry(GeometryHandle)",
            ),
        ];

        let mut expected_entries: Vec<String> = Vec::with_capacity(values.len());
        for (key, value, expected_display) in values {
            assert_eq!(format!("{value}"), expected_display);
            expected_entries.push(format!("\"{key}\": {expected_display}"));
            attrs.insert(key.to_string(), value);
        }

        let display_str = format!("{attrs}");
        for expected_entry in expected_entries {
            assert!(display_str.contains(&expected_entry));
        }
    }

    /// Does default create an empty attributes container?
    #[test]
    fn test_default() {
        let attrs: OwnedAttributes = OwnedAttributes::default();

        assert!(attrs.is_empty());
        assert_eq!(attrs.len(), 0);
    }

    /// Can we clear all attributes?
    #[test]
    fn test_clear() {
        let mut attrs = OwnedAttributes::new();

        attrs.insert("a".to_string(), OwnedAttributeValue::Integer(1));
        attrs.insert("b".to_string(), OwnedAttributeValue::Integer(2));
        assert_eq!(attrs.len(), 2);

        attrs.clear();

        assert!(attrs.is_empty());
        assert_eq!(attrs.len(), 0);
        assert_eq!(attrs.get("a"), None);
        assert_eq!(attrs.get("b"), None);
    }

    /// Can we check if a key exists?
    #[test]
    fn test_contains_key() {
        let mut attrs = OwnedAttributes::new();

        attrs.insert("height".to_string(), OwnedAttributeValue::Float(25.5));

        assert!(attrs.contains_key("height"));
        assert!(!attrs.contains_key("missing"));
    }

    /// Can we iterate keys only?
    #[test]
    fn test_keys() {
        let mut attrs = OwnedAttributes::new();

        attrs.insert("a".to_string(), OwnedAttributeValue::Integer(1));
        attrs.insert("b".to_string(), OwnedAttributeValue::Integer(2));
        attrs.insert("c".to_string(), OwnedAttributeValue::Integer(3));

        assert!(attrs.keys().any(|k| k == "a"));
        assert!(attrs.keys().any(|k| k == "b"));
        assert!(attrs.keys().any(|k| k == "c"));
    }

    /// Does len report the number of entries reliably?
    #[test]
    fn test_len() {
        let mut attrs = OwnedAttributes::new();
        assert_eq!(attrs.len(), 0);

        attrs.insert("a".to_string(), OwnedAttributeValue::Integer(1));
        attrs.insert("b".to_string(), OwnedAttributeValue::Integer(2));
        assert_eq!(attrs.len(), 2);
    }

    /// Can we modify attributes in place?
    #[test]
    fn test_modification() {
        let mut attrs = OwnedAttributes::new();

        attrs.insert("count".to_string(), OwnedAttributeValue::Integer(10));

        // Modify via get_mut
        if let Some(OwnedAttributeValue::Integer(c)) = attrs.get_mut("count") {
            *c += 5;
        }

        // Verify modification
        if let Some(OwnedAttributeValue::Integer(c)) = attrs.get("count") {
            assert_eq!(*c, 15);
        }
    }

    /// Can we remove attributes and return the removed value?
    /// Does emptiness check work reliably?
    #[test]
    fn test_removal_and_empty() {
        let mut attrs = OwnedAttributes::new();
        assert!(attrs.is_empty());

        attrs.insert("temp".to_string(), OwnedAttributeValue::Bool(true));
        assert_eq!(attrs.len(), 1);

        let removed = attrs.remove("temp");
        assert!(matches!(removed, Some(OwnedAttributeValue::Bool(true))));
        assert!(attrs.is_empty());
    }

    /// Can we iterate over attributes, immutable and mutable?
    #[test]
    fn test_iteration() {
        let mut attrs = OwnedAttributes::new();

        attrs.insert("a".to_string(), OwnedAttributeValue::Integer(1));
        attrs.insert("b".to_string(), OwnedAttributeValue::Integer(2));
        attrs.insert("c".to_string(), OwnedAttributeValue::Integer(3));

        // Iterate
        let mut sum = 0i64;
        for (_, value) in attrs.iter() {
            if let OwnedAttributeValue::Integer(n) = value {
                sum += n;
            }
        }
        assert_eq!(sum, 6);

        // Mutate during iteration
        for (_, value) in attrs.iter_mut() {
            if let OwnedAttributeValue::Integer(n) = value {
                *n += 1;
            }
        }
        // Sum using an iterator pipeline
        sum = attrs
            .values()
            .filter_map(|v| match v {
                OwnedAttributeValue::Integer(n) => Some(*n),
                _ => None,
            })
            .sum();
        assert_eq!(sum, 9);
    }
}

mod containers {
    //! Tests types that can contain attributes.
    //! This is either a direct containment under an "attributes" member, or indirect
    //! via a member that stores an `Attributes` instance (e.g for allowing extra
    //! properties).

    use super::assert_f64_eq;
    use cityjson::resources::storage::OwnedStringStorage;
    use cityjson::v2_0::*;

    /// Can we insert and retrieve arbitrary `CityModel` properties so that we can have
    /// Extensions?
    /// Tests { "type": "`CityModel`", "+mandatoryProperty": "string", ... }
    #[test]
    fn test_citymodel_extra() {
        let mut city_model: CityModel = CityModel::new(CityModelType::CityJSON);

        // Add extra properties (extension properties)
        city_model.extra_mut().insert(
            "+mandatoryProperty".to_string(),
            OwnedAttributeValue::String("string".to_string()),
        );

        // Retrieve and verify
        let extra = city_model.extra().unwrap();

        if let Some(OwnedAttributeValue::String(v)) = extra.get("+mandatoryProperty") {
            assert_eq!(v, "string");
        }
    }

    /// Can we insert and retrieve arbitrary Metadata root properties and an arbitrary
    /// address definition in the pointOfContact member?
    /// Tests { "nonMandatoryProperty": "string", "pointOfContact": { "address": { ... } }
    #[test]
    fn test_metadata() {
        use std::collections::HashMap;

        let mut metadata = Metadata::<OwnedStringStorage>::default();

        // Add extra property to metadata root
        metadata.extra_mut().insert(
            "nonMandatoryProperty".to_string(),
            OwnedAttributeValue::String("string".to_string()),
        );

        // Create pointOfContact with address
        let mut contact = Contact::<OwnedStringStorage>::default();
        let mut address_map = HashMap::new();
        address_map.insert(
            "street".to_string(),
            OwnedAttributeValue::String("Main St".to_string()),
        );
        address_map.insert("number".to_string(), OwnedAttributeValue::Integer(123));
        contact
            .address_mut()
            .insert("address".to_string(), OwnedAttributeValue::Map(address_map));
        metadata.set_point_of_contact(Some(contact));

        // Verify metadata extra property
        let extra = metadata.extra().unwrap();
        if let Some(OwnedAttributeValue::String(v)) = extra.get("nonMandatoryProperty") {
            assert_eq!(v, "string");
        } else {
            panic!("Expected nonMandatoryProperty");
        }

        // Verify pointOfContact address
        if let Some(contact) = metadata.point_of_contact()
            && let Some(contact_address) = contact.address()
            && let Some(OwnedAttributeValue::Map(addr)) = contact_address.get("address")
        {
            if let Some(OwnedAttributeValue::String(s)) = addr.get("street") {
                assert_eq!(s, "Main St");
            }
            if let Some(OwnedAttributeValue::Integer(n)) = addr.get("number") {
                assert_eq!(*n, 123);
            }
        }
    }

    /// Can we insert and retrieve `CityObject` attributes and extension properties?
    /// Tests:
    /// { "type": "Building", "attributes": { "height": 25.5 }, "nonMandatoryProperty": "string", "+mandatoryExtensionProperty": "string" }
    #[test]
    fn test_cityobject() {
        let mut city_model: CityModel = CityModel::new(CityModelType::CityJSON);

        // Create a building with attributes and extension properties
        let mut building = CityObject::new(
            CityObjectIdentifier::new("building-001".to_string()),
            CityObjectType::Building,
        );
        building
            .attributes_mut()
            .insert("height".to_string(), OwnedAttributeValue::Float(25.5));
        building.extra_mut().insert(
            "nonMandatoryProperty".to_string(),
            OwnedAttributeValue::String("string".to_string()),
        );
        building.extra_mut().insert(
            "+mandatoryExtensionProperty".to_string(),
            OwnedAttributeValue::String("string".to_string()),
        );

        let building_ref = city_model.cityobjects_mut().add(building).unwrap();

        // Retrieve and verify attributes
        let retrieved = city_model.cityobjects().get(building_ref).unwrap();
        let attrs = retrieved.attributes().unwrap();

        if let Some(OwnedAttributeValue::Float(h)) = attrs.get("height") {
            assert_f64_eq(*h, 25.5);
        }

        // Retrieve and verify extension properties
        let extra = retrieved.extra().unwrap();

        if let Some(OwnedAttributeValue::String(v)) = extra.get("nonMandatoryProperty") {
            assert_eq!(v, "string");
        }
        if let Some(OwnedAttributeValue::String(v)) = extra.get("+mandatoryExtensionProperty") {
            assert_eq!(v, "string");
        }
    }

    /// Can we insert and retrieve Semantic surface attributes?
    /// Tests { "type": "`RoofSurface`", "material": "tile" }
    #[test]
    fn test_semantic() {
        // Create semantic with attributes
        let mut roof: Semantic<OwnedStringStorage> = Semantic::new(SemanticType::RoofSurface);

        roof.attributes_mut().insert(
            "material".to_string(),
            OwnedAttributeValue::String("tile".to_string()),
        );

        // Retrieve and verify
        let attrs = roof.attributes().unwrap();

        if let Some(OwnedAttributeValue::String(m)) = attrs.get("material") {
            assert_eq!(m.as_str(), "tile");
        }
    }
}
