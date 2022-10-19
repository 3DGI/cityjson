# Using cjlib

## Naming conventions

| CityJSON        | cjlib       |
|-----------------|-------------|
| CityJSON object | CityModel   |
| CityObject      | CityObject  |
| CityJSONFeature | CityFeature |


## Creating CityModels

Instantiate a new, empty `CityModel`.

=== "Rust"

    ```rust
    let cm = CityModel::new(); // (1)
    ```

    1. Although, most likely you'll want to create `cm` as `mut`able and fill it up with content later.

=== "Python"

    ```python
    cm = CityModel()
    ```

Create a `CityModel` from a CityJSON string.

=== "Rust"

    ```rust
    let cityjson_str = r#"{
        "type": "CityJSON",
        "version": "1.1",
        "transform": {
            "scale": [1.0, 1.0, 1.0],
            "translate": [0.0, 0.0, 0.0]
        },
        "CityObjects": {},
        "vertices": []
    }"
    let cm = CityModel::from_str(&cityjson_str);
    ```

=== "Python"

    ```python
    cityjson_str = """{
        "type": "CityJSON",
        "version": "1.1",
        "transform": {
            "scale": [1.0, 1.0, 1.0],
            "translate": [0.0, 0.0, 0.0]
        },
        "CityObjects": {},
        "vertices": []
    }"""
    cm = CityModel.from_str(cityjson_str)
    ```

Create a `CityModel` from a CityJSON file.

=== "Rust"
    
    ```rust
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open("myfile.city.json").expect("Couldn't open CityJSON file");
    let reader = BufReader::new(&file);
    let cm = CityModel::from_reader(reader);
    ```

=== "Python"

    ```python
    cm = CityModel.from_file("myfile.city.json")
    ```

Parse a stream of text sequence into [`CityJSONFeature`s](https://www.cityjson.org/specs/1.1.2/#text-sequences-and-streaming-with-cityjsonfeature).

=== "Rust"

    ```rust
    use serde_json::Deserializer;

    let features_sequence = r#"{"type":"CityJSONFeature"}{"type":"CityJSONFeature"}";

    let stream = Deserializer::from_str(features_sequence).into_iter::<CityFeature>();
    for feature in stream {
        let parent_cityobject: String = feature.id;
        for (coid, co) in feature.cityobjects.iter() {
            println!("CityObject id: {}", coid);
        }
    }
    ```

=== "Python"

    ```python

    features_sequence = ["{\"type\":\"CityJSONFeature\"}, {\"type\":\"CityJSONFeature\"}"]
    for feature in feature_sequence:
        cityfeature = cjlib.readline(feature)
    ```