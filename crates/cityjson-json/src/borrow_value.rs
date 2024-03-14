use std::env;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];
    let mut file = File::open(file_path).map_err(|e| e.to_string())?;
    let mut cityjson_json = Vec::new();
    file.read_to_end(&mut cityjson_json)
        .map_err(|e| e.to_string())?;
    let cm: serde_json_borrow::Value =
        serde_json::from_slice(&cityjson_json).map_err(|e| e.to_string())?;
    println!("{:?}", &cm.get("version"));
    Ok(())
}
