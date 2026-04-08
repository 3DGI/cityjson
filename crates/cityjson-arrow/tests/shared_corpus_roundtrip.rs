#[path = "support/shared_corpus.rs"]
mod shared_corpus;

use cityarrow::{ModelDecoder, ModelEncoder};

#[test]
fn arrow_roundtrip_preserves_all_normative_shared_corpus_conformance_cases() {
    let mut failures = Vec::new();

    for case in shared_corpus::load_normative_conformance_cases() {
        let mut bytes = Vec::new();
        if let Err(err) = ModelEncoder.encode(&case.model, &mut bytes) {
            failures.push(format!("{}: encode failed: {err}", case.id));
            continue;
        }

        let decoded = match ModelDecoder.decode(bytes.as_slice()) {
            Ok(decoded) => decoded,
            Err(err) => {
                failures.push(format!("{}: decode failed: {err}", case.id));
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
        "normative conformance roundtrip failures:\n{}",
        failures.join("\n")
    );
}
