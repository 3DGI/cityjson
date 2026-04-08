#[path = "support/shared_corpus.rs"]
mod shared_corpus;

use cityarrow::{ModelDecoder, ModelEncoder};

#[test]
fn arrow_roundtrip_preserves_curated_shared_corpus_transport_cases() {
    for case in shared_corpus::load_transport_roundtrip_cases() {
        let mut bytes = Vec::new();
        ModelEncoder
            .encode(&case.model, &mut bytes)
            .unwrap_or_else(|err| {
                panic!("failed to encode shared corpus case '{}': {err}", case.id)
            });
        let decoded = ModelDecoder.decode(bytes.as_slice()).unwrap_or_else(|err| {
            panic!("failed to decode shared corpus case '{}': {err}", case.id)
        });

        assert_eq!(
            shared_corpus::normalized_json(&case.model),
            shared_corpus::normalized_json(&decoded),
            "shared corpus case '{}'",
            case.id
        );
    }
}
