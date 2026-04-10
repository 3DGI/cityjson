#[path = "../../tests/support/shared_corpus.rs"]
mod shared_corpus;

use cityparquet::{PackageReader, PackageWriter};
use tempfile::tempdir;

#[test]
fn package_roundtrip_preserves_all_shared_corpus_conformance_cases() {
    let dir = tempdir().unwrap();
    let mut failures = Vec::new();

    for case in shared_corpus::load_conformance_cases() {
        let path = dir.path().join(format!("{}.cityarrow", case.id));
        if let Err(err) = PackageWriter.write_file(&path, &case.model) {
            failures.push(format!("{}: write failed: {err}", case.id));
            continue;
        }

        let decoded = match PackageReader.read_file(&path) {
            Ok(decoded) => decoded,
            Err(err) => {
                failures.push(format!("{}: read failed: {err}", case.id));
                continue;
            }
        };

        let expected = shared_corpus::normalized_json(&case.model);
        let actual = shared_corpus::normalized_json(&decoded);
        if expected != actual {
            failures.push(format!("{}: roundtrip JSON mismatch", case.id));
        }
    }

    assert!(
        failures.is_empty(),
        "conformance package roundtrip failures:\n{}",
        failures.join("\n")
    );
}
