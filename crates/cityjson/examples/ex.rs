use cityjson::prelude::*;
use cityjson::v1_1::*;

fn main() {
    let _ = Metadata::<ResourceId32, OwnedStringStorage>::new();
    let _ = BBox::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
}
