//! Benchmark the memory allocations with the datasize module.
//! Run 'just download' first to download the data files.
use std::path::PathBuf;
use std::hint::black_box;

use serde_cityjson::datasize::SerdeCityJSONDataSize;

fn main() -> Result<(), String> {
    let downloaded_dir = PathBuf::from("resources").join("data").join("downloaded");
    let filename = "10-356-724";
    let filepath = downloaded_dir.join(filename).with_extension("city.json");
    let ds = SerdeCityJSONDataSize::new(None);
    let _ = black_box(ds
        .run("3DBAG", filename, filepath)
        .map_err(|e| e.to_string())?);

    let filename = "30gz1_04";
    let filepath = downloaded_dir.join(filename).with_extension("json");
    let ds = SerdeCityJSONDataSize::new(None);
    let _ = black_box(ds
        .run("3D Basisvoorziening", filename, filepath)
        .map_err(|e| e.to_string())?);
    Ok(())
}
