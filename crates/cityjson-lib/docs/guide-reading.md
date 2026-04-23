# Reading Data

This page shows the common read path across the published Rust, Python, and
C++ surfaces.

## Load A Document

=== "Rust"
    ```rust
    use cityjson_lib::json;

    let model = json::from_file("amsterdam.city.json")?;

    let bytes = std::fs::read("amsterdam.city.json")?;
    let model = json::from_slice(&bytes)?;
    # let _ = model;
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    from cityjson_lib import CityModel

    data = open("amsterdam.city.json", "rb").read()
    model = CityModel.parse_document_bytes(data)
    ```

=== "C++"
    ```cpp
    #include <cityjson_lib/cityjson_lib.hpp>
    #include <fstream>
    #include <iterator>

    std::ifstream file("amsterdam.city.json", std::ios::binary);
    std::vector<std::uint8_t> bytes(std::istreambuf_iterator<char>(file), {});

    auto model = cityjson_lib::Model::parse_document(bytes);
    ```

## Probe Before Parsing

=== "Rust"
    ```rust
    use cityjson_lib::{json, CityJSONVersion};

    let bytes = std::fs::read("amsterdam.city.json")?;
    let probe = json::probe(&bytes)?;
    assert_eq!(probe.kind(), json::RootKind::CityJSON);
    assert_eq!(probe.version(), Some(CityJSONVersion::V2_0));
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    from cityjson_lib import RootKind, Version, probe_bytes

    data = open("amsterdam.city.json", "rb").read()
    probe = probe_bytes(data)
    assert probe.root_kind == RootKind.CJ_ROOT_KIND_CITY_JSON
    assert probe.version == Version.CJ_VERSION_2_0
    ```

=== "C++"
    ```cpp
    const auto probe = cityjson_lib::Model::probe(bytes);
    if (probe.root_kind != CJ_ROOT_KIND_CITY_JSON) {
      throw std::runtime_error("expected CityJSON document");
    }
    ```

## Inspect Summary Data

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
    summary = model.summary()
    print(summary.cityobject_count)
    print(summary.vertex_count)
    print(model.metadata_title())
    ```

=== "C++"
    ```cpp
    const auto summary = model.summary();
    std::printf("%zu\n", summary.cityobject_count);
    std::printf("%zu\n", summary.vertex_count);
    std::puts(model.metadata_title().c_str());
    ```

## Read Or Merge A Feature Stream

CityJSONSeq stays explicit.

=== "Rust"
    ```rust
    use std::fs::File;
    use std::io::BufReader;

    use cityjson_lib::json;

    let reader = BufReader::new(File::open("tiles.city.jsonl")?);
    let models = json::read_feature_stream(reader)?
        .collect::<cityjson_lib::Result<Vec<_>>>()?;
    # let _ = models;
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    from cityjson_lib import merge_feature_stream_bytes

    data = open("tiles.city.jsonl", "rb").read()
    merged = merge_feature_stream_bytes(data)
    print(merged.summary().cityobject_count)
    ```

=== "C++"
    ```cpp
    std::ifstream file("tiles.city.jsonl", std::ios::binary);
    std::vector<std::uint8_t> bytes(std::istreambuf_iterator<char>(file), {});

    auto merged = cityjson_lib::Model::merge_feature_stream(bytes);
    std::printf("%zu\n", merged.summary().cityobject_count);
    ```

The wasm adapter is still being shaped and is not part of the published docs
surface yet.

## Read From Arrow

Arrow I/O requires the `arrow` feature (`features = ["arrow"]` in `Cargo.toml`).
The Python and C++ bindings always include it.

=== "Rust"
    ```rust
    # #[cfg(feature = "arrow")]
    # {
    use cityjson_lib::arrow;

    let bytes = std::fs::read("model.cjarrow")?;
    let model = arrow::from_bytes(&bytes)?;

    let model = arrow::from_file("model.cjarrow")?;
    # let _ = model;
    # }
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    from cityjson_lib import CityModel

    model = CityModel.parse_arrow_bytes(open("model.cjarrow", "rb").read())
    ```

=== "C++"
    ```cpp
    std::ifstream file("model.cjarrow", std::ios::binary);
    std::vector<std::uint8_t> bytes(std::istreambuf_iterator<char>(file), {});

    auto model = cityjson_lib::Model::parse_arrow(bytes);
    ```

## Read From Parquet

Parquet I/O requires the `parquet` feature.
Two layouts are supported: a self-contained package file and a bare dataset
directory.

=== "Rust"
    ```rust
    # #[cfg(feature = "parquet")]
    # {
    use cityjson_lib::parquet;

    let model = parquet::from_file("city.cityjson-parquet")?;
    let model = parquet::from_dir("city.dataset")?;
    # let _ = model;
    # }
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    from cityjson_lib import CityModel

    model = CityModel.parse_parquet_file("city.cityjson-parquet")
    model = CityModel.parse_parquet_dataset_dir("city.dataset")
    ```

=== "C++"
    ```cpp
    auto model = cityjson_lib::Model::parse_parquet_file("city.cityjson-parquet");
    auto model = cityjson_lib::Model::parse_parquet_dataset_dir("city.dataset");
    ```
