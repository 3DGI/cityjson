# Writing Data

This page shows the common write path across the published Rust, Python, and
C++ surfaces.

## Create A Model

=== "Rust"
    ```rust
    use cityjson_lib::cityjson::{self, v2_0::OwnedCityModel};

    let model = OwnedCityModel::new(cityjson::CityModelType::CityJSON);
    # let _ = model;
    ```

=== "Python"
    ```python
    from cityjson_lib import CityModel, ModelType

    model = CityModel.create(model_type=ModelType.CJ_MODEL_TYPE_CITY_JSON)
    ```

=== "C++"
    ```cpp
    #include <cityjson_lib/cityjson_lib.hpp>

    auto model = cityjson_lib::Model::create(CJ_MODEL_TYPE_CITY_JSON);
    ```

## Set Metadata

=== "Rust"
    ```rust
    use cityjson_lib::cityjson::{self, v2_0::OwnedCityModel};

    let mut model = OwnedCityModel::new(cityjson::CityModelType::CityJSON);
    model.metadata_mut().set_title("My Dataset".to_string());
    # let _ = model;
    ```

=== "Python"
    ```python
    model.set_metadata_title("My Dataset")
    model.set_metadata_identifier("my-dataset-001")
    ```

=== "C++"
    ```cpp
    model.set_metadata_title("My Dataset");
    model.set_metadata_identifier("my-dataset-001");
    ```

## Add Geometry Through The Binding Surface

The non-Rust bindings expose a direct geometry-boundary builder path.

=== "Python"
    ```python
    from cityjson_lib import GeometryBoundary, GeometryType, Vertex

    model.add_vertex(Vertex(10.0, 20.0, 0.0))
    model.add_vertex(Vertex(11.0, 20.0, 0.0))
    model.add_vertex(Vertex(11.0, 21.0, 0.0))
    model.add_vertex(Vertex(10.0, 21.0, 0.0))
    model.add_cityobject("building-1", "Building")

    boundary = GeometryBoundary(
        geometry_type=GeometryType.CJ_GEOMETRY_TYPE_MULTI_SURFACE,
        has_boundaries=True,
        vertex_indices=[0, 1, 2, 3, 0],
        ring_offsets=[0],
        surface_offsets=[0],
        shell_offsets=[],
        solid_offsets=[],
    )
    geom_index = model.add_geometry_from_boundary(boundary, lod="2.2")
    model.attach_geometry_to_cityobject("building-1", geom_index)
    ```

=== "C++"
    ```cpp
    model.add_vertex({10.0, 20.0, 0.0});
    model.add_vertex({11.0, 20.0, 0.0});
    model.add_vertex({11.0, 21.0, 0.0});
    model.add_vertex({10.0, 21.0, 0.0});
    model.add_cityobject("building-1", "Building");

    cityjson_lib::GeometryBoundary boundary{
        .geometry_type = CJ_GEOMETRY_TYPE_MULTI_SURFACE,
        .has_boundaries = true,
        .vertex_indices = {0, 1, 2, 3, 0},
        .ring_offsets = {0},
        .surface_offsets = {0},
        .shell_offsets = {},
        .solid_offsets = {},
    };
    const auto geom_index = model.add_geometry_from_boundary(boundary, "2.2");
    model.attach_geometry_to_cityobject("building-1", geom_index);
    ```

## Serialize A Document

=== "Rust"
    ```rust
    use cityjson_lib::json::{self, WriteOptions};

    let model = json::from_file("amsterdam.city.json")?;
    let compact = json::to_string(&model)?;
    let pretty = json::to_string_with_options(
        &model,
        WriteOptions { pretty: true, ..Default::default() },
    )?;
    # let _ = (compact, pretty);
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    from cityjson_lib import WriteOptions

    model.cleanup()
    text = model.serialize_document(WriteOptions(pretty=True))
    ```

=== "C++"
    ```cpp
    model.cleanup();
    const auto text = model.serialize_document(cityjson_lib::WriteOptions{.pretty = true});
    ```

## Serialize A Feature Stream

=== "Rust"
    ```rust
    use std::fs::File;
    use std::io::BufReader;

    use cityjson_lib::json;

    let reader = BufReader::new(File::open("tests/data/v2_0/stream.city.jsonl")?);
    let models = json::read_feature_stream(reader)?
        .collect::<cityjson_lib::Result<Vec<_>>>()?;

    let mut out = Vec::new();
    json::write_feature_stream(&mut out, models)?;
    # let _ = out;
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    from cityjson_lib import CityModel, serialize_feature_stream_bytes

    feature_a = CityModel.parse_feature_bytes(open("feature-a.city.json", "rb").read())
    feature_b = CityModel.parse_feature_bytes(open("feature-b.city.json", "rb").read())
    payload = serialize_feature_stream_bytes([feature_a, feature_b])
    ```

=== "C++"
    ```cpp
    std::array<const cityjson_lib::Model*, 2> models{&feature_a, &feature_b};
    const auto payload = cityjson_lib::Model::serialize_feature_stream(models);
    ```

The wasm adapter remains work in progress and is intentionally omitted from the
published write guide.
