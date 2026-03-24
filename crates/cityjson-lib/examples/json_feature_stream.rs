use std::fs::File;
use std::io::BufReader;

use cjlib::CityModel;

fn main() -> cjlib::Result<()> {
    let reader = BufReader::new(File::open("tests/data/v2_0/stream.city.jsonl")?);
    let model = CityModel::from_stream(reader)?;
    println!("loaded {} CityObjects", model.cityobjects().len());
    Ok(())
}
