use std::fs;

#[path = "support/mod.rs"]
mod support;

use cityarrow::schema::PackageTableEncoding;
use serde_cityjson::from_str_owned;

fn minimal_fixture_model() -> cityjson::v2_0::OwnedCityModel {
    let input_path = support::sibling_serde_cityjson_root()
        .join("tests/data/v2_0/cityjson_minimal_complete.city.json");
    let input_json = fs::read_to_string(&input_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", input_path.display()));

    from_str_owned(&input_json).unwrap_or_else(|error| {
        panic!(
            "serde_cityjson failed for {}: {error}",
            input_path.display()
        )
    })
}

#[test]
fn minimal_serde_cityjson_fixture_preserves_canonical_parts_through_parquet_package() {
    support::assert_package_roundtrip_parts_integrity(
        minimal_fixture_model(),
        PackageTableEncoding::Parquet,
    );
}

#[test]
fn minimal_serde_cityjson_fixture_preserves_canonical_parts_through_ipc_package() {
    support::assert_package_roundtrip_parts_integrity(
        minimal_fixture_model(),
        PackageTableEncoding::ArrowIpcFile,
    );
}
