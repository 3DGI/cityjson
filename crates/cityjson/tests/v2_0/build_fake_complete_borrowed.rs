//! Golden fake-complete coverage for `BorrowedStringStorage`.

use cityjson::error::Result;
use cityjson::resources::storage::BorrowedStringStorage;

use super::fake_complete_shared::{
    assert_model_matches_fixture, build_model_from_fixture, load_fixture,
};

#[test]
fn build_fake_complete_borrowed() -> Result<()> {
    let fixture = load_fixture();
    let model = build_model_from_fixture::<BorrowedStringStorage<'_>>(&fixture)?;
    assert_model_matches_fixture(&model, &fixture);
    Ok(())
}
