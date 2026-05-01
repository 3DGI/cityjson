//! Compile-time assertions that the public API types implement Send + Sync.

use cityjson_types::prelude::*;
use cityjson_types::v2_0::*;

fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn city_model_is_send_sync() {
    assert_send_sync::<OwnedCityModel>();
    assert_send_sync::<BorrowedCityModel<'static>>();
}

#[test]
fn city_object_is_send_sync() {
    assert_send_sync::<CityObject<OwnedStringStorage>>();
}

#[test]
fn geometry_is_send_sync() {
    assert_send_sync::<Geometry<u32, OwnedStringStorage>>();
}

#[test]
fn semantic_is_send_sync() {
    assert_send_sync::<Semantic<OwnedStringStorage>>();
}

#[test]
fn appearance_is_send_sync() {
    assert_send_sync::<Material<OwnedStringStorage>>();
    assert_send_sync::<Texture<OwnedStringStorage>>();
}

#[test]
fn attribute_value_is_send_sync() {
    assert_send_sync::<OwnedAttributeValue>();
}

#[test]
fn handles_are_send_sync() {
    assert_send_sync::<CityObjectHandle>();
    assert_send_sync::<GeometryHandle>();
    assert_send_sync::<SemanticHandle>();
    assert_send_sync::<MaterialHandle>();
    assert_send_sync::<TextureHandle>();
}

#[test]
fn cityjson_enum_is_send_sync() {
    assert_send_sync::<CityJSON<u32, OwnedStringStorage>>();
}
