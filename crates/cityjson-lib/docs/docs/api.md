# Using cjlib

## Naming conventions

| CityJSON        | cjlib       |
|-----------------|-------------|
| CityJSON object | CityModel   |
| CityObject      | CityObject  |
| CityJSONFeature | CityFeature |


## Creating CityModels

Create a new, empty instance of a `CityModel`.

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
    }"#
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

### Reading a stream of CityJSONFeatures

Parse a stream of text sequence into [`CityJSONFeature`s](https://www.cityjson.org/specs/1.1.2/#text-sequences-and-streaming-with-cityjsonfeature).
While this approach does not need access to `CityModel`, we only recommend it in the case when you process and discard the features one by one, because the semantic and appearance objects are duplicated across the features.

=== "Rust"

    ```rust
    use serde_json::Deserializer;

    let features_sequence = r#"
        {"type":"CityJSONFeature"}
        {"type":"CityJSONFeature"}
    "#;
    let stream = Deserializer::from_str(features_sequence).into_iter::<CityFeature>();
    let transform_properties = Transform::new()
        .scale(1.0, 1.0, 1.0)
        .translate(0.0, 0.0, 0.0);

    while let Some(feature) = stream.next() {
        let parent_cityobject: String = feature.id;
        for (coid, co) in feature.cityobjects.iter_mut() {
            println!("CityObject id: {}", coid);
            co.transform(&transform_properties);
            // process the CityObject
        }
    }
    ```

=== "Python"

    ```python
    from io import StringIO

    features_sequence = """
        {"type":"CityJSONFeature"}
        {"type":"CityJSONFeature"}
    """.strip("\n").strip()
    stream = StringIO(features_sequence)
    
    transform_properties = Transform(
        scale=(1.0, 1.0, 1.0),
        translate=(0.0, 0.0, 0.0)
    )

    for cityjsonfeature_str in stream:
        if cityjsonfeature_str is None or cityjsonfeature_str == "":
            break
        else:
            cityfeature = CityFeature.from_str(cityjsonfeature_str)
        for (coid, co) in cityfeature.cityobjects:
            print(f"CityObject id: {coid}")
            co.transform(transform_properties)
            # process the CityObject
    ```

Normally, a `CityJSONFeature` stream will contain a single `CityJSON` object as the first item.
This `CityJSON` object contains metadata about the city model, but also the transformation properties that are required for converting the compressed `CityObject` vertices into real-world coordinates.
We expect that the first item is a `CityJSON` object.
If this is not the case, you can also create an empty `CityModel` and set the transformation properties for the `CityJSONFeature`s in the stream.


=== "Rust"

    ```rust
    use std::io::BufReader;

    let features_sequence = r#"
        {"type":"CityJSON"}
        {"type":"CityJSONFeature"}
        {"type":"CityJSONFeature"}
    "#;
    let stream_iter = BufReader::new(&features_sequence).lines();

    let mut cm: CityModel; // (1)
    if let Some(cityjson_str) = stream.next() // (2) {
        cm = CityModel::from_str(&cityjson_str);
    }

    while let Some(cityjsonfeature_str) = stream.next() {

        let cf: CityFeature = CityFeature::from_str(&cityjsonfeature_str)
            .with(&mut cm); // (3)

        for (coid, co) in cf.cityobjects.iter() {
            println!("CityObject id: {}", coid);
        }

        // Additionally, you can the CityFeature to the CityModel.
        // The CityObjects from the CityFeature are added to the 
        // CityObjects of the CityModel.
        cm.cityobjects.insert(cf);
    }
    ```
    
    1. We need a `mut`able `CityModel`, because its semantics and appearances will be populated from the `CityJSONFeature`s.
    
    2. We expect that the first item in the stream is a `CityJSON` object.
        This first `CityJSON` object is converted to a `CityModel`, which is then used for parsing the `CityJSONFeature`.
    
    3. Parse the `CityJSONFeature` into a `CityFeature`, with the information from the CityModel `cm` (transformation properties etc.). 
        Since the appearance and semantic objects of the `CityObject`s are stored on the `CityModel`, we need a mutable reference to it.

=== "Python"

    ```python
    from io import StringIO

    features_sequence = """
        {"type":"CityJSON"}
        {"type":"CityJSONFeature"}
        {"type":"CityJSONFeature"}
    """.strip("\n").strip()
    stream = StringIO(features_sequence)

    citymodel_str = stream.readline()
    cm = CityModel.from_str(citymodel_str)
    
    for cityjsonfeature_str in stream:
        if cityjsonfeature_str is None or cityjsonfeature_str == "":
            break
        else:
            cf = CityFeature.from_str(cityjsonfeature_str, citymodel=cm)

        # Additionally, you can the CityFeature to the CityModel.
        # The CityObjects from the CityFeature are added to the
        # CityObjects of the CityModel.
        cm.cityobjects.insert(cf)
    ```


## Writing a CityJSON document

Convert a `CityModel` to a CityJSON string.

=== "Rust"

    ```rust
    let cm = CityModel::new();
    let cityjson = cm.to_string(&cm)?;
    ```

=== "Python"

    ```python
    cm = CityModel()
    cityjson = cm.to_str(cm)
    ```

Convert a `CityModel` to a CityJSON file.

=== "Rust"

    ```rust
    let cm = CityModel::new();

    let mut new_file = File::create("mynew.city.json")?;
    let cityjson = cm.to_writer(&new_file)?;
    ```

=== "Python"

    ```python
    cm = CityModel()

    cityjson = cm.to_file("mynew.city.json")
    ```

??? info "Vertex transformation"

    The CityJSON specifications require that the vertices of the city model are transformed. The transformation is 
    coordinate scaling and translation, so that we end up with small(er) integers instead of floating point numbers  
    as coordinates. Thus, the vertex transformation reduces the size of the CityJSON document.
    Besides doing the actual coordinate transformation, we need to store the transformation properties in the 
    [Transform Object](https://www.cityjson.org/specs/1.1.2/#transform-object) in the CityJSON document.

    Internally, *cjlib* stores the vertices with their true, untransformed coordinates. 

    When a CityJSON document is loaded, its transformation properties are preserved in the `CityModel`, and these same 
    properties are used by default when the `CityModel` is written to a CityJSON document. However, you can override the 
    transformation properties by providing new ones.

Set new transformation properties for the CityJSON document. 

!!! info

    Internally, *cjlib* stores the vertices with their true, untransformed coordinates, thus the transformation 
    properties are only applied when the `CityModel` is converted to a CityJSON document.

=== "Rust"

    ```rust
    let cm = CityModel::new();

    cm.set_transform(Transform());

    let mut new_file = File::create("mynew.city.json")?;
    let cityjson = cm.to_writer(&new_file)?;
    ```

=== "Python"

    ```python
    cm = CityModel()

    cm.set_transform(Transform())

    cityjson = cm.to_file("mynew.city.json")
    ```

## CityJSON Extensions

[CityJSON Extensions](https://www.cityjson.org/specs/1.1.2/#extensions) are first-class citizen in cjlib.

!!! note "Dev note"
    
    Can we make an API for creating and modifying extensions? So that is possible (and easier) to create extensions 
    interactively? This would enable to create an simple web UI for creating extensions, 
    eg. [like this one but prettier and specific to CityJSON](https://bjdash.github.io/JSON-Schema-Builder/).

If you load a document that contains extensions, the extensions are automatically loaded from their `url`.
You don't need to do anything special.
Currently, only the `http(s)://` and `file://` protocols are supported for loading extensions.

=== "Rust"

    ```rust
    let cityjson_with_extension = r#"{
        "type": "CityJSON",
        "version": "1.1",
        "extensions": {
            "Noise": {
                "url" : "https://someurl.org/noise.json",
                "version": "2.0"
            },
            "Solar_Potential": {
                "url" : "http://otherurl.org/solar.json",
                "version": "0.8"
            }
        },
        "CityObjects": {},
        "vertices": []
    }"#
    let cm = CityModel::from_str(&cityjson_with_extension);
    ```

=== "Python"

    ```python
    cityjson_with_extension = """{
        "type": "CityJSON",
        "version": "1.1",
        "extensions": {
            "Noise": {
                "url" : "https://someurl.org/noise.json",
                "version": "2.0"
            },
            "Solar_Potential": {
                "url" : "http://otherurl.org/solar.json",
                "version": "0.8"
            }
        },
        "CityObjects": {},
        "vertices": []
    }"""
    cm = CityModel.from_str(cityjson_with_extension)
    ```

You can override the extension `url` by passing the new `url` along with the extension name when loading the document.
The parameter is a collection of tuples, where the first value of the tuple is the extension name, the second value is the new `url`.

=== "Rust"

    ```rust
    let extensions_uris = [("Noise", "file:///mydirectory/noise.json"), ];
    let cm = CityModel::from_str(&cityjson_with_extension, &extensions_uris);
    ```

=== "Python"

    ```python
    extensions_uris = [("Noise", "file:///mydirectory/noise.json"), ]
    cm = CityModel.from_str(cityjson_with_extension, extensions_uris=extensions_uris)
    ```