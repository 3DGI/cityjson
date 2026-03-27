use std::fs::File;
use std::io::BufReader;

use cjlib::json;

fn main() -> cjlib::Result<()> {
    let reader = BufReader::new(File::open("tests/data/v2_0/stream.city.jsonl")?);
    let models = json::read_feature_stream(reader)?.collect::<cjlib::Result<Vec<_>>>()?;
    println!("loaded {} feature models", models.len());
    Ok(())
}
