use cityjson::CityModelType;
use serde_json::json;

use serde_cityjson::{
    as_json, from_str_borrowed, from_str_owned, to_string, BorrowedCityModel, Error, OwnedCityModel,
};

fn baseline_json() -> serde_json::Value {
    json!({
        "type": "CityJSON",
        "version": "2.0",
        "transform": {
            "scale": [0.5, 1.0, 2.0],
            "translate": [10.0, 20.0, 30.0]
        },
        "metadata": {
            "identifier": "dataset-1",
            "title": "Minimal adapter fixture",
            "pointOfContact": {
                "contactName": "Example Org",
                "emailAddress": "hello@example.com",
                "role": "author",
                "contactType": "organization",
                "address": {
                    "city": "Den Haag"
                }
            },
            "nospec_description": "kept as metadata extra"
        },
        "extensions": {
            "Noise": {
                "url": "https://example.com/noise.json",
                "version": "1.0"
            }
        },
        "CityObjects": {
            "parent": {
                "type": "Building",
                "children": ["child"],
                "attributes": {
                    "floors": 3
                },
                "children_roles": ["main part"]
            },
            "child": {
                "type": "BuildingPart",
                "parents": ["parent"],
                "geographicalExtent": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                "attributes": {
                    "name": "annex"
                }
            }
        },
        "vertices": [
            [0, 1, 2],
            [4, 5, 6]
        ],
        "+stats": {
            "count": 2
        }
    })
}

fn parse_owned(value: &serde_json::Value) -> OwnedCityModel {
    from_str_owned(&value.to_string()).unwrap()
}

fn parse_borrowed(value: &serde_json::Value) -> BorrowedCityModel<'_> {
    let input = value.to_string();
    let leaked: &'static str = Box::leak(input.into_boxed_str());
    from_str_borrowed(leaked).unwrap()
}

#[test]
fn owned_roundtrip_for_supported_v2_0_slice() {
    let input = baseline_json();
    let model = parse_owned(&input);

    assert_eq!(model.type_citymodel(), CityModelType::CityJSON);
    assert_eq!(model.cityobjects().len(), 2);
    assert_eq!(model.vertices().len(), 2);

    let serialized = serde_json::to_value(as_json(&model)).unwrap();
    assert_eq!(serialized, input);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&to_string(&model).unwrap()).unwrap(),
        input
    );
}

#[test]
fn borrowed_and_owned_parity_match() {
    let input = baseline_json();
    let owned = parse_owned(&input);
    let borrowed = parse_borrowed(&input);

    let owned_json = serde_json::to_value(as_json(&owned)).unwrap();
    let borrowed_json = serde_json::to_value(as_json(&borrowed)).unwrap();

    assert_eq!(owned_json, borrowed_json);
    assert_eq!(owned_json, input);
}

#[test]
fn unsupported_version_is_rejected() {
    let err = from_str_owned(
        &json!({
            "type": "CityJSON",
            "version": "1.1",
            "CityObjects": {},
            "vertices": []
        })
        .to_string(),
    )
    .unwrap_err();

    assert!(matches!(err, Error::UnsupportedVersion(version) if version == "1.1"));
}

#[test]
fn unresolved_cityobject_reference_is_rejected() {
    let err = from_str_owned(
        &json!({
            "type": "CityJSON",
            "version": "2.0",
            "CityObjects": {
                "child": {
                    "type": "BuildingPart",
                    "parents": ["missing"]
                }
            },
            "vertices": []
        })
        .to_string(),
    )
    .unwrap_err();

    assert!(matches!(
        err,
        Error::UnresolvedCityObjectReference {
            source_id,
            target_id,
            relation
        } if source_id == "child" && target_id == "missing" && relation == "parent"
    ));
}

#[test]
fn geometry_import_is_explicitly_rejected_for_now() {
    let err = from_str_owned(
        &json!({
            "type": "CityJSON",
            "version": "2.0",
            "CityObjects": {
                "building": {
                    "type": "Building",
                    "geometry": [{
                        "type": "MultiSurface",
                        "lod": "2",
                        "boundaries": [[[0, 1, 2]]]
                    }]
                }
            },
            "vertices": [[0, 0, 0], [1, 0, 0], [0, 1, 0]]
        })
        .to_string(),
    )
    .unwrap_err();

    assert!(
        matches!(err, Error::UnsupportedFeature(feature) if feature == "geometry import is not implemented yet")
    );
}
