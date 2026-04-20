# Binding API

This page shows the common user-facing API across Rust, Python, and C++.

The wrappers are aligned around the same core ideas:

- parse explicit document or feature payloads
- inspect summary and metadata
- author typed values, resources, and geometry drafts
- run explicit cleanup, append, and extract workflows
- serialize back to document, feature, or feature-stream bytes

The shared C ABI itself is owned by `ffi/core`; this page documents the
user-facing contracts layered on top of it.

The typed write-side authoring flow is documented separately in
[Writing Data](../guide-writing.md). The full end-to-end references live in
`ffi/cpp/examples/fake_complete.cpp` and `ffi/python/examples/fake_complete.py`.

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

## Typed Authoring

=== "Python"
    ```python
    from cityjson_lib import CityModel, CityObjectDraft, GeometryDraft, ModelType, RingDraft, SurfaceDraft, Value, Vertex

    model = CityModel.create(model_type=ModelType.CITY_JSON)
    v0 = model.add_vertex(Vertex(10.0, 20.0, 0.0))
    v1 = model.add_vertex(Vertex(11.0, 20.0, 0.0))
    v2 = model.add_vertex(Vertex(11.0, 21.0, 0.0))
    v3 = model.add_vertex(Vertex(10.0, 21.0, 0.0))

    draft = GeometryDraft.multi_surface("2.2").add_surface(
        SurfaceDraft(
            RingDraft()
            .push_vertex_index(v0)
            .push_vertex_index(v1)
            .push_vertex_index(v2)
            .push_vertex_index(v3)
        )
    )
    building = CityObjectDraft("building-1", "Building")
    building.set_attribute("height", Value.number(12.5))

    geometry_id = model.add_geometry(draft)
    building_id = model.add_cityobject(building)
    model.add_cityobject_geometry(building_id, geometry_id)
    ```

=== "C++"
    ```cpp
    auto model = cityjson_lib::Model::create(CJ_MODEL_TYPE_CITY_JSON);
    const auto v0 = model.add_vertex({10.0, 20.0, 0.0});
    const auto v1 = model.add_vertex({11.0, 20.0, 0.0});
    const auto v2 = model.add_vertex({11.0, 21.0, 0.0});
    const auto v3 = model.add_vertex({10.0, 21.0, 0.0});

    auto draft = cityjson_lib::GeometryDraft::multi_surface("2.2");
    draft.add_surface(cityjson_lib::SurfaceDraft(
        cityjson_lib::RingDraft{}
            .push_vertex_index(v0)
            .push_vertex_index(v1)
            .push_vertex_index(v2)
            .push_vertex_index(v3)));

    auto building = cityjson_lib::CityObjectDraft("building-1", "Building");
    building.set_attribute("height", cityjson_lib::Value::number(12.5));

    const auto geometry_id = model.add_geometry(std::move(draft));
    const auto building_id = model.add_cityobject(std::move(building));
    model.add_cityobject_geometry(building_id, geometry_id);
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

## Arrow I/O

The `arrow` feature must be enabled for Rust (`features = ["arrow"]`).
Python and C++ bindings always include it.

=== "Rust"
    ```rust
    # #[cfg(feature = "arrow")]
    # {
    use cityjson_lib::{arrow, json};

    let model = json::from_file("amsterdam.city.json")?;

    let bytes = arrow::to_vec(&model)?;
    let roundtrip = arrow::from_bytes(&bytes)?;

    arrow::to_file("model.cjarrow", &model)?;
    let from_file = arrow::from_file("model.cjarrow")?;
    # let _ = (roundtrip, from_file);
    # }
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    from cityjson_lib import CityModel

    arrow_bytes = model.serialize_arrow_bytes()
    roundtrip = CityModel.parse_arrow_bytes(arrow_bytes)
    ```

=== "C++"
    ```cpp
    const auto arrow_bytes = model.serialize_arrow_bytes();
    auto roundtrip = cityjson_lib::Model::parse_arrow(arrow_bytes);
    ```

## Parquet I/O

The `parquet` feature must be enabled for Rust (`features = ["parquet"]`).
Two layouts are supported: a self-contained package file and a bare dataset
directory.

=== "Rust"
    ```rust
    # #[cfg(feature = "parquet")]
    # {
    use cityjson_lib::{json, parquet};

    let model = json::from_file("amsterdam.city.json")?;

    parquet::to_file("city.cityjson-parquet", &model)?;
    let from_pkg = parquet::from_file("city.cityjson-parquet")?;

    parquet::to_dir("city.dataset", &model)?;
    let from_dir = parquet::from_dir("city.dataset")?;
    # let _ = (from_pkg, from_dir);
    # }
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    from cityjson_lib import CityModel

    model.serialize_parquet_file("city.cityjson-parquet")
    from_pkg = CityModel.parse_parquet_file("city.cityjson-parquet")

    model.serialize_parquet_dataset_dir("city.dataset")
    from_dir = CityModel.parse_parquet_dataset_dir("city.dataset")
    ```

=== "C++"
    ```cpp
    model.serialize_parquet_file("city.cityjson-parquet");
    auto from_pkg = cityjson_lib::Model::parse_parquet_file("city.cityjson-parquet");

    model.serialize_parquet_dataset_dir("city.dataset");
    auto from_dir = cityjson_lib::Model::parse_parquet_dataset_dir("city.dataset");
    ```
