use cityjson::prelude::*;
use cityjson::v1_1::*;

fn main() {
    let _ = Metadata::<OwnedStringStorage>::new();
    let _ = BBox::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
}
