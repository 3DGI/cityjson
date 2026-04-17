# Binding API

This page shows the common user-facing API across Rust, Python, and C++.

The wrappers are aligned around the same core ideas:

- parse explicit document or feature payloads
- inspect summary and metadata
- run explicit cleanup, append, and extract workflows
- serialize back to document, feature, or feature-stream bytes

The wasm adapter is still work in progress and is not covered here.

## Parse And Inspect

=== "Rust"
    ```rust
    use cityjson_lib::{json, query};

    let model = json::from_file("amsterdam.city.json")?;
    let summary = query::summary(&model);
    assert!(summary.cityobject_count >= 1);
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    from cityjson_lib import CityModel

    model = CityModel.parse_document_bytes(open("amsterdam.city.json", "rb").read())
    summary = model.summary()
    assert summary.cityobject_count >= 1
    ```

=== "C++"
    ```cpp
    #include <cityjson_lib/cityjson_lib.hpp>
    #include <stdexcept>

    const auto bytes = read_file_bytes("amsterdam.city.json");
    auto model = cityjson_lib::Model::parse_document(bytes);
    const auto summary = model.summary();
    if (summary.cityobject_count == 0U) {
      throw std::runtime_error("expected at least one cityobject");
    }
    ```

## Cleanup And Extract

=== "Rust"
    ```rust
    use cityjson_lib::{json, ops};

    let model = json::from_file("amsterdam.city.json")?;
    let cleaned = ops::cleanup(&model)?;
    let subset = ops::extract(&cleaned, ["building-1"])?;
    # let _ = subset;
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    model.cleanup()
    subset = model.extract_cityobjects(["building-1"])
    ```

=== "C++"
    ```cpp
    model.cleanup();
    const std::array<std::string_view, 1> ids{"building-1"};
    const auto subset = model.extract_cityobjects(ids);
    ```

## Append Or Merge

=== "Rust"
    ```rust
    use std::fs::File;
    use std::io::BufReader;

    use cityjson_lib::{json, ops};

    let reader = BufReader::new(File::open("tests/data/v2_0/stream.city.jsonl")?);
    let models = json::read_feature_stream(reader)?
        .collect::<cityjson_lib::Result<Vec<_>>>()?;
    let merged = ops::merge(models)?;
    # let _ = merged;
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    from cityjson_lib import merge_feature_stream_bytes

    left.append_model(right)
    merged = merge_feature_stream_bytes(open("tiles.city.jsonl", "rb").read())
    ```

=== "C++"
    ```cpp
    left.append_model(right);
    auto merged = cityjson_lib::Model::merge_feature_stream(stream_bytes);
    ```

## Serialize

=== "Rust"
    ```rust
    use cityjson_lib::json;

    let model = json::from_file("amsterdam.city.json")?;
    let document = json::to_vec(&model)?;
    let feature = json::to_feature_vec(&model)?;
    # let _ = (document, feature);
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    document_bytes = model.serialize_document_bytes()
    feature_bytes = model.serialize_feature_bytes()
    ```

=== "C++"
    ```cpp
    const auto document_bytes = model.serialize_document_bytes();
    const auto feature_bytes = model.serialize_feature_bytes();
    ```
