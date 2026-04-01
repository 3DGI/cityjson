#[path = "support/mod.rs"]
mod support;

use cityarrow::schema::PackageTableEncoding;
use std::sync::Mutex;

static CORPUS_GATE: Mutex<()> = Mutex::new(());

#[test]
fn shared_conformance_index_contains_expected_sentinel_cases() {
    let case_ids = support::conformance_case_ids();

    assert!(case_ids.contains(&"cityjson_minimal_complete"));
    assert!(case_ids.contains(&"cityjson_fake_complete"));
    assert!(case_ids.contains(&"spec_complete_omnibus"));
}

#[test]
fn shared_conformance_cases_roundtrip_exactly_through_parquet_packages() {
    let _gate = CORPUS_GATE.lock().expect("corpus gate lock");
    for case_id in support::conformance_case_ids() {
        support::assert_conformance_case_roundtrip_with_encoding(
            case_id,
            PackageTableEncoding::Parquet,
        );
    }
}

#[test]
fn shared_conformance_cases_roundtrip_exactly_through_ipc_packages() {
    let _gate = CORPUS_GATE.lock().expect("corpus gate lock");
    for case_id in support::conformance_case_ids() {
        support::assert_conformance_case_roundtrip_with_encoding(
            case_id,
            PackageTableEncoding::ArrowIpcFile,
        );
    }
}
