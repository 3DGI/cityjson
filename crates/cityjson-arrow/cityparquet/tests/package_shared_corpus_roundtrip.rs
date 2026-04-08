#[path = "../../tests/support/shared_corpus.rs"]
mod shared_corpus;

use cityparquet::{PackageReader, PackageWriter};
use tempfile::tempdir;

#[test]
fn package_roundtrip_preserves_curated_shared_corpus_transport_cases() {
    let dir = tempdir().unwrap();

    for case in shared_corpus::load_transport_roundtrip_cases() {
        let path = dir.path().join(format!("{}.cityarrow", case.id));
        PackageWriter
            .write_file(&path, &case.model)
            .unwrap_or_else(|err| {
                panic!("failed to write shared corpus case '{}': {err}", case.id)
            });
        let decoded = PackageReader
            .read_file(&path)
            .unwrap_or_else(|err| panic!("failed to read shared corpus case '{}': {err}", case.id));

        assert_eq!(
            shared_corpus::normalized_json(&case.model),
            shared_corpus::normalized_json(&decoded),
            "shared corpus case '{}'",
            case.id
        );
    }
}
