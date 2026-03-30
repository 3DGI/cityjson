#[path = "support/mod.rs"]
mod support;

#[test]
fn manifest_layout_matches_serde_cityjson_real_cases() {
    let cases = support::acceptance_cases();
    let ids = cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<Vec<_>>();

    assert!(ids.contains(&"3DBAG"));
    assert!(ids.contains(&"3D Basisvoorziening"));

    for case in &cases {
        assert!(
            case.source.is_some(),
            "real acceptance case {} should have a source path",
            case.id
        );
        assert!(
            case.description.len() > 10,
            "case {} should keep the serde_cityjson description",
            case.id
        );
    }
}

#[test]
#[ignore = "expensive real-data acceptance gate"]
fn real_datasets_validate_with_serde_cityjson_cityarrow_and_cjval() {
    for case in support::acceptance_cases() {
        support::assert_case_roundtrip(&case);
    }
}
