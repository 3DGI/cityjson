//! Golden fake-complete coverage for `OwnedStringStorage`.

use cityjson_types::error::Result;
use cityjson_types::resources::storage::OwnedStringStorage;

use super::fake_complete_shared::{
    assert_model_matches_fixture, build_model_from_fixture, load_fixture,
};

#[test]
fn build_fake_complete_owned() -> Result<()> {
    let fixture = load_fixture();
    let model = build_model_from_fixture::<OwnedStringStorage>(&fixture)?;
    assert_model_matches_fixture(&model, &fixture);
    Ok(())
}
