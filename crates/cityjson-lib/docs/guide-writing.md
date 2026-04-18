# Writing Data

The shared FFI write path now centers on typed values, typed resource ids, and
draft objects for nested geometry authoring.

The published C++ surface exposes that model directly. The Python binding keeps
the stable parse/inspect/stream workflows, but its write-side API is still
catching up to the new draft-based authoring layer.

## Create A Model

=== "Rust"
    ```rust
    use cityjson_lib::cityjson::{self, v2_0::OwnedCityModel};

    let model = OwnedCityModel::new(cityjson::CityModelType::CityJSON);
    # let _ = model;
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

=== "C++"
    ```cpp
    model.set_metadata_title("My Dataset");
    model.set_metadata_identifier("my-dataset-001");
    model.set_metadata_geographical_extent({
        .min_x = 0.0,
        .min_y = 0.0,
        .min_z = 0.0,
        .max_x = 1.0,
        .max_y = 1.0,
        .max_z = 1.0,
    });
    model.set_metadata_contact(
        cityjson_lib::Contact{}
            .set_name("Author")
            .set_email("author@example.com")
            .set_role(CJ_CONTACT_ROLE_AUTHOR));
    ```

## Add Geometry Through Drafts

=== "C++"
    ```cpp
    const auto v0 = model.add_vertex({10.0, 20.0, 0.0});
    const auto v1 = model.add_vertex({11.0, 20.0, 0.0});
    const auto v2 = model.add_vertex({11.0, 21.0, 0.0});
    const auto v3 = model.add_vertex({10.0, 21.0, 0.0});

    auto ring = cityjson_lib::RingDraft{};
    ring.push_vertex_index(v0)
        .push_vertex_index(v1)
        .push_vertex_index(v2)
        .push_vertex_index(v3);

    auto draft = cityjson_lib::GeometryDraft::multi_surface("2.2");
    draft.add_surface(cityjson_lib::SurfaceDraft(std::move(ring)));

    auto building = cityjson_lib::CityObjectDraft("building-1", "Building");
    building.set_attribute("height", cityjson_lib::Value::number(12.5));

    const auto geometry_id = model.add_geometry(std::move(draft));
    const auto building_id = model.add_cityobject(std::move(building));
    model.add_cityobject_geometry(building_id, geometry_id);
    ```

## Full Fixture Example

The full reference example lives in `ffi/cpp/examples/fake_complete.cpp`. It
builds the equivalent of the complete fake CityJSON fixture through the
typed C++ API and is exercised in the automated test suite.

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
