use cityjson_lib::json;
use cityjson_lib::ops;

fn main() -> cityjson_lib::Result<()> {
    let model = json::from_file("tests/data/v2_0/minimal.city.json")?;
    let _ = ops::merge([model]);

    Ok(())
}
