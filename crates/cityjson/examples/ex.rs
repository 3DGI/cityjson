use cityjson::v1_1::metadata;
use cityjson::resources::storage::OwnedStringStorage;
fn main() {
    let _ = metadata::Metadata::<OwnedStringStorage>::new();
    let _ = metadata::BBox::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
}