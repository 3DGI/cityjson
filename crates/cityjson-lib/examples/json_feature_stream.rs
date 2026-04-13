use std::fs::File;
use std::io::BufReader;

use cityjson_lib::json;

fn main() -> cityjson_lib::Result<()> {
    let reader = BufReader::new(File::open("tests/data/v2_0/stream.city.jsonl")?);
    let models = json::read_feature_stream(reader)?.collect::<cityjson_lib::Result<Vec<_>>>()?;
    println!("loaded {} feature models", models.len());
    Ok(())
}
