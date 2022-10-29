# Using cjlib

## Differences between the CityJSON and cjlib data model

CityJSON uses JSON as an encoding format for *exchanging* semantic 3D city models.
Therefore, the data model of CityJSON is optimized for storage and simplicity, so that a CityJSON document has a small size, it is easy extract the information from it, and it is easy to create.
On the other hand, *cjlib* needs a data model that is optimized for computation, so that operations on city models can be done as efficiently as possible.
The two goals are not completely orthogonal and there are several parts where cjlib follows CityJSON completely, but there are parts where it differs from it.
Normally you don't need to be aware of these differences in order to use cjlib, but for the sake of completeness they are described here.

1) **Geometry boundaries store the vertex coordinates instead of vertex indices.**

In a CityJSON document the vertex coordinates are stored in the `"vertices"` array at the document root.
The Geometry boundaries are then [arrays of different depth](https://www.cityjson.org/specs/1.1.2/#arrays-to-represent-boundaries) (depending on the Geometry type), containing array-indices, each pointing to a vertex in the `"vertices"` array.

```json
// The global vertex array, containing a collection of vertices 
// with their [x, y, z] coordinates.
"vertices": [
    [102, 103, 1],
    [11, 910, 43],
    [25, 744, 22],
    [8523, 487, 22],
    ...
]

// A Geometry Object's boundary stores array-indices, each pointing 
// to a vertex in the "vertices" array.
"boundaries": [
    [[0, 3, 2, 1]], [[...]], ...
]
```

On the contrary, *cjlib* stores the vertex coordinates directly in the geometry boundaries, and there is no global vertex array.
If we translate the CityJSON snippet above to *cjlib*'s data model, we get something similar to what you can see below.
*But read on to the next point!*

```json
"boundaries": [
  [[ [102, 103, 1], [8523, 487, 22], [25, 744, 22], [11, 910, 43] ]], [[...]], ...
]
```

2) **Coordinates are un-transformed**

CityJSON requires that the vertex coordinates in the document are transformed.
*Transformation* means scaling and translation by the values set in the [Transform object](https://www.cityjson.org/specs/1.1.2/#transform-object) in the document.

On the contrary, *cjlib* stores the true (un-transformed) coordinate values, by reversing the coordinate transformation when the CityJSON document is parsed.


3) **Naming conventions**

The table below shows the names of objects in CityJSON and their equivalent in *cjlib*.

| CityJSON        | cjlib       |
|-----------------|-------------|
| CityJSON object | CityModel   |
| CityJSONFeature | CityFeature |


## Creating CityModels

- [x] Create a new blank instance of a `CityModel`.

=== "Rust"

    ```rust
    let cm = CityModel::new(); // (1)
    ```

    1. Although, most likely you'll want to create `cm` as `mut`able and fill it up with content later, using the `set_*` methods.

=== "Python"

    ```python
    cm = CityModel()
    ```

!!! note "Dev note"

    Build a `CityModel` with parameters. 
    In Rust this makes sense, however, in Python we can just keyword parameters.

    === "Rust"

        ```rust
        let cm = CityModel::builder()
            .transform()
            .title()
            .cityobjects()
            .identifier()
            .extension("A", Extension)
            .extension("B", Extension)
            .version()
            .build()
            .unwrap();
        ```

    === "Python"
    
        ```python
        cm = CityModel(
            transform=None, 
            title=None, 
            cityobjects=None, 
            *others
        )
        ```

    I'm still debating if a builder is necessary. The user can still configure a blank 
    CityModel by calling the constructor and any of the setters afterwards. For instance,

    ```rust
    let mut cm = CityModel::new();
    cm.set_extension("A", Extension);
    cm.set_extension("B", Extension);
    cm.set_title();
    cm.set_transform();
    cm.set_identifier();
    cm.set_cityobjects();
    cm.set_version();
    ```
    Also, because calling at least one of the setters might be inevitable, because we only 
    have the information later on.

    ```rust
    let mut cm = CityModel::builder()
        .title()
        .identifier("id_1")
        .extension("A", Extension)
        .extension("B", Extension)
        .version()
        .build()
        .unwrap();
    // do some processing here
    let cityobjects = SomeData;
    cm.set_cityobjects(cityobjects);
    // change the id on the model
    cm.set_identifier("id_2");
    // set the transform for writing out the citymodel
    cm.set_transform();
    ```

- [x] Create a `CityModel` from a CityJSON string.

!!! note "Dev note"

    Consider merging the `from_str` and `from_reader` functions into a `from_document`, that accepts any string and path.
    See [video/article/slides](https://www.youtube.com/watch?v=6-8-9ZV-2WQ&list=WL&index=1) on how to do this.
    Or maybe at least make `from_reader` --> `from_file`.
    Although, maybe all this is too much abstraction.

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
    }"#;
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

- [x] Create a `CityModel` from a CityJSON file.
The file can be either a regular CityJSON file, or a [JSON Lines text](https://jsonlines.org/) file, containing [`CityJSONFeature`s](https://www.cityjson.org/specs/1.1.2/#text-sequences-and-streaming-with-cityjsonfeature).
The parsing method is based on the file extension.
A file with an extension of `.city.json`, `.cityjson` or `.json` is expected to contain only a single CityJSON object at the root level.
A file with an extension of `.city.jsonl` or `.jsonl` a newline-delimited sequence of `CityJSON` and `CityJSONFeature` objects, where the first object in the file is a `CityJSON` object and all subsequent objects are `CityJSONFeature`s.

=== "Rust"
    
    ```rust
    let cm = CityModel::from_file("myfile.city.json");
    let cm_from_features = CityModel::from_file("myfile_with_features.city.jsonl");
    ```

=== "Python"

    ```python
    cm = CityModel.from_file("myfile.city.json")
    cm_from_features = CityModel.from_file("myfile_with_features.city.jsonl")
    ```

### Reading a stream of CityJSONFeatures

- [ ] Stream with only CityJSONFeatures

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

- [x] If you want to collect the whole stream into a single `CityModel`, use the `from_stream` method.

=== "Rust"

    ```rust
    use std::io::{BufRead, Cursor};

    let feature_sequence = r#"{"type":"CityJSON","version":"1.1","transform":{"scale":[0.1,0.1,0.1],"translate":[0.0,0.0,0.0]},"CityObjects":{},"vertices":[]}
        {"type":"CityJSONFeature","id":"id-1","CityObjects":{},"vertices":[]}
        {"type":"CityJSONFeature","id":"id-2","CityObjects":{},"vertices":[]}"#;
    let mut stream = Cursor::new(feature_sequence);

    let cm = CityModel::from_stream(stream);
    ```

=== "Python"

    ```python
    from io import StringIO

    features_sequence = """{"type":"CityJSON"}
        {"type":"CityJSONFeature"}
        {"type":"CityJSONFeature"}
    """.strip("\n").strip()
    stream = StringIO(features_sequence)

    cm = CityModel.from_stream(stream)
    ```

- [ ] You can also process and discard the features as you iterate over the stream, instead of collecting them into a `CityModel`.


=== "Rust"

    ```rust
    use std::io::{BufRead, Cursor};

    let feature_sequence = r#"{"type":"CityJSON","version":"1.1","transform":{"scale":[0.1,0.1,0.1],"translate":[0.0,0.0,0.0]},"CityObjects":{},"vertices":[]}
        {"type":"CityJSONFeature","id":"id-1","CityObjects":{},"vertices":[]}
        {"type":"CityJSONFeature","id":"id-2","CityObjects":{},"vertices":[]}"#;
    let mut stream_iter = Cursor::new(feature_sequence).lines();

    let mut cm: CityModel; // (1)
    if let Some(res) = stream_iter.next() // (2) {
        let cityjson_str = res.expect("Failed to read object from the sequence.");
        cm = CityModel::from_str(&cityjson_str);
    }

    for res in stream_iter {
        let cityjsonfeature_str = res.expect("Failed to read item from the stream.");

        let cf = CityFeature::from_str(&cityjsonfeature_str).with(&mut cm); // (3)

        for (coid, co) in cf.cityobjects.iter() {
            println!("CityObject id: {}", coid);
        }

        // Additionally, you can insert the CityObjects from 
        // the CityFeature to the CityModel.
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

        # Additionally, you can insert the CityObjects from 
        # the CityFeature to the CityModel.
        cm.cityobjects.insert(cf)
    ```


## Writing a CityJSON document

- [x] Convert a `CityModel` to a CityJSON string.

=== "Rust"

    ```rust
    let cm = CityModel::new();
    let cityjson = cm.to_string()?;
    ```

=== "Python"

    ```python
    cm = CityModel()
    cityjson = cm.to_str(cm)
    ```

- [x] Convert a `CityModel` to a CityJSON file.

=== "Rust"

    ```rust
    let cm = CityModel::new();
    let cityjson = cm.to_file("mynew.city.json")?;
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

- [x] Set new transformation properties for the CityJSON document. 

!!! info

    Internally, *cjlib* stores the vertices with their true, untransformed coordinates, thus the transformation 
    properties are only applied when the `CityModel` is converted to a CityJSON document.

=== "Rust"

    ```rust
    let cm = CityModel::new();
    cm.set_transform(Transform::default());
    let cityjson = cm.to_file("mynew.city.json")?;
    ```

=== "Python"

    ```python
    cm = CityModel()
    cm.set_transform(Transform())
    cityjson = cm.to_file("mynew.city.json")
    ```

!!! note "Dev note"

    The default transformation is `scale: [1.0, 1.0, 1.0]`.
    Maybe it could be `scale: [0.001, 0.001, 0.001]`, which is a sensible default for projected CRS-es with meters as units?
    And probably most citymodels are like that.
    Or, we could just set `scale: [0.001, 0.001, 0.001]` as default only when writing a CityJSON and keep `scale: [1.0, 1.0, 1.0]` as default in the Transform constructor..

## CityJSON Extensions

[CityJSON Extensions](https://www.cityjson.org/specs/1.1.2/#extensions) are first-class citizen in cjlib.

!!! note "Dev note"
    
    Can we make an API for creating and modifying extensions? So that is possible (and easier) to create extensions 
    interactively? This would enable to create an simple web UI for creating extensions, 
    eg. [like this one but prettier and specific to CityJSON](https://bjdash.github.io/JSON-Schema-Builder/).

- [ ] If you load a document that contains extensions, the extensions are automatically loaded from their `url`.
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

- [ ] You can override the extension `url` by passing the new `url` along with the extension name when loading the document.
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