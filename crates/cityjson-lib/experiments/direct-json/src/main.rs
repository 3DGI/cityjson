use serde_json;

fn main() -> Result<(), serde_json::Error> {
    let path = "../data/cluster.city.json";
    let str_dataset = std::fs::read_to_string(&path)
        .expect("Couldn't read CityJSON file");
    let j: serde_json::Value = serde_json::from_str(&str_dataset)?;
    let cos = j.get("CityObjects").unwrap().as_object().unwrap();
    for coid in cos.keys() {
        println!("CityObject {} is of type {}", coid, j["CityObjects"][coid]["type"])
    }
    Ok(())
}
