# Reading CityJSON Data

A walk-through of the common read path: load a document, inspect its contents, and access
geometry. Each section shows the same operation across all four APIs.

## Load a Document

The simplest entry point is a file path or a byte slice already in memory.

=== "Rust"
```rust
use cityjson_lib::json;
use cityjson_lib::CityModel;

// from a file path
let model = json::from_file("amsterdam.city.json")?;

// or from bytes already in memory
let bytes = std::fs::read("amsterdam.city.json")?;
let model = json::from_slice(&bytes)?;
    ```

=== "C++"
    ```cpp
    #include <cityjson_lib/cityjson_lib.hpp>
    #include <fstream>
    #include <iterator>

    std::ifstream file("amsterdam.city.json", std::ios::binary);
    std::vector<uint8_t> bytes(std::istreambuf_iterator<char>(file), {});

    auto model = cityjson_lib::Model::parse_document(bytes);
    ```

=== "Python"
    ```python
    from cityjson_lib import CityModel

    data = open("amsterdam.city.json", "rb").read()
    model = CityModel.parse_document_bytes(data)
    ```

=== "WASM"
    ```c
    cj_model_t *handle = NULL;
    cj_status_t status = cj_model_parse_document_bytes(data, data_len, &handle);
    /* check status != CJ_STATUS_SUCCESS for errors */
    ```

## Inspect Basic Counts

Query summary fields without descending into the geometry tree.

=== "Rust"
    ```rust
    let inner = model.as_inner();
    println!("{} city objects", inner.cityobjects().len());
    println!("{} vertices",     inner.vertices().len());
    ```

=== "C++"
    ```cpp
    cityjson_lib::ModelSummary s = model.summary();
    printf("%zu city objects\n", s.cityobject_count);
    printf("%zu vertices\n",     s.vertex_count);
    printf("title: %s\n",        model.metadata_title().c_str());
    ```

=== "Python"
    ```python
    s = model.summary()
    print(f"{s.cityobject_count} city objects")
    print(f"{s.vertex_count} vertices")
    print("title:", model.metadata_title())
    ```

=== "WASM"
    ```c
    cj_model_summary_t s = {0};
    cj_model_get_summary(handle, &s);
    printf("%zu city objects\n", s.cityobject_count);
    printf("%zu vertices\n",     s.vertex_count);
    ```

## List City Object IDs

=== "Rust"
    ```rust
    for (_, obj) in model.as_inner().cityobjects().iter() {
        println!("{}", obj.id());
    }
    ```

=== "C++"
    ```cpp
    for (const std::string& id : model.cityobject_ids()) {
        std::cout << id << "\n";
    }
    ```

=== "Python"
    ```python
    for id in model.cityobject_ids():
        print(id)
    ```

=== "WASM"
    ```c
    cj_model_summary_t s = {0};
    cj_model_get_summary(handle, &s);

    for (size_t i = 0; i < s.cityobject_count; i++) {
        cj_bytes_t id = {0};
        cj_model_get_cityobject_id(handle, i, &id);
        /* id.data points to id.len bytes of UTF-8 */
        cj_bytes_free(id);
    }
    ```

## Export Arrow Transport

=== "Rust"
    ```rust
    let batches = cityjson_lib::arrow::export_batches(&model)?;
    println!("{} geometry rows", batches.geometries.num_rows());
    ```

=== "C++"
    ```cpp
    const auto arrow_bytes = model.serialize_arrow_bytes();
    printf("%zu Arrow bytes\n", arrow_bytes.size());
    ```

=== "Python"
    ```python
    arrow_bytes = model.serialize_arrow_bytes()
    print(len(arrow_bytes))
    ```

=== "WASM"
    ```c
    cj_vertices_t verts = {0};
    cj_model_copy_vertices(handle, &verts);

    for (size_t i = 0; i < verts.len; i++) {
        printf("%.3f  %.3f  %.3f\n",
               verts.data[i].x, verts.data[i].y, verts.data[i].z);
    }
    cj_vertices_free(verts);
    ```

## Read a Feature Stream

CityJSONSeq (`.city.jsonl`) files start with a base `CityJSON` document line followed by one
`CityJSONFeature` line per tile or chunk. Use the feature-stream API to iterate features
one at a time or merge them into a single model.

=== "Rust"
    ```rust
    use std::fs::File;
    use std::io::BufReader;

    let reader = BufReader::new(File::open("tiles.city.jsonl")?);
    for result in cityjson_lib::json::read_feature_stream(reader)? {
        let model = result?;
        println!("{} objects in tile", model.as_inner().cityobjects().len());
    }
    ```

=== "C++"
    ```cpp
    // read the .city.jsonl into a byte buffer, then merge all features at once
    std::ifstream file("tiles.city.jsonl", std::ios::binary);
    std::vector<uint8_t> stream_bytes(std::istreambuf_iterator<char>(file), {});

    auto merged = cityjson_lib::Model::merge_feature_stream(stream_bytes);
    printf("%zu objects after merge\n", merged.summary().cityobject_count);
    ```

=== "Python"
    ```python
    from cityjson_lib import merge_feature_stream_bytes

    data = open("tiles.city.jsonl", "rb").read()
    merged = merge_feature_stream_bytes(data)
    print(f"{merged.summary().cityobject_count} objects after merge")
    ```

=== "WASM"
    ```c
    cj_model_t *merged = NULL;
    cj_model_parse_feature_stream_merge_bytes(data, data_len, &merged);

    cj_model_summary_t s = {0};
    cj_model_get_summary(merged, &s);
    printf("%zu objects after merge\n", s.cityobject_count);

    cj_model_free(merged);
    ```
