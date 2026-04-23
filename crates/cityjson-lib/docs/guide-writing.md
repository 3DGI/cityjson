# Writing Data

The shared FFI write path now centers on typed values, typed resource ids, and
draft objects for nested geometry authoring.

The published C++ and Python surfaces expose that authoring model directly.

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

=== "Python"
    ```python
    from cityjson_lib import CityModel, ModelType

    model = CityModel.create(model_type=ModelType.CITY_JSON)
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

=== "Python"
    ```python
    from cityjson_lib import BBox, Contact, ContactRole

    model.set_metadata_title("My Dataset")
    model.set_metadata_identifier("my-dataset-001")
    model.set_metadata_geographical_extent(
        BBox(min_x=0.0, min_y=0.0, min_z=0.0, max_x=1.0, max_y=1.0, max_z=1.0)
    )
    model.set_metadata_contact(
        Contact()
        .set_name("Author")
        .set_email("author@example.com")
        .set_role(ContactRole.AUTHOR)
    )
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

=== "Python"
    ```python
    from cityjson_lib import CityObjectDraft, GeometryDraft, RingDraft, SurfaceDraft, Value, Vertex

    v0 = model.add_vertex(Vertex(10.0, 20.0, 0.0))
    v1 = model.add_vertex(Vertex(11.0, 20.0, 0.0))
    v2 = model.add_vertex(Vertex(11.0, 21.0, 0.0))
    v3 = model.add_vertex(Vertex(10.0, 21.0, 0.0))

    ring = RingDraft().push_vertex_index(v0).push_vertex_index(v1).push_vertex_index(v2).push_vertex_index(v3)
    draft = GeometryDraft.multi_surface("2.2").add_surface(SurfaceDraft(ring))

    building = CityObjectDraft("building-1", "Building")
    building.set_attribute("height", Value.number(12.5))

    geometry_id = model.add_geometry(draft)
    building_id = model.add_cityobject(building)
    model.add_cityobject_geometry(building_id, geometry_id)
    ```

## Full Fixture Example

The full reference examples live in `ffi/cpp/examples/fake_complete.cpp` and
`ffi/python/examples/fake_complete.py`. Both build the equivalent of the
complete fake CityJSON fixture through the typed authoring API and are
exercised in the automated test suite.

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

## Serialize To Arrow

Arrow I/O requires the `arrow` feature (`features = ["arrow"]` in `Cargo.toml`).
The Python and C++ bindings always include it.

=== "Rust"
    ```rust
    # #[cfg(feature = "arrow")]
    # {
    use cityjson_lib::{arrow, json};

    let model = json::from_file("amsterdam.city.json")?;
    let bytes = arrow::to_vec(&model)?;
    arrow::to_file("model.cjarrow", &model)?;
    # let _ = bytes;
    # }
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    arrow_bytes = model.serialize_arrow_bytes()
    model.serialize_parquet_file("model.cjarrow")  # via file path
    ```

=== "C++"
    ```cpp
    const auto arrow_bytes = model.serialize_arrow_bytes();
    model.serialize_parquet_file("model.cjarrow");
    ```

## Serialize To Parquet

Parquet I/O requires the `parquet` feature.
Two layouts are supported: a self-contained package file and a bare dataset
directory.

=== "Rust"
    ```rust
    # #[cfg(feature = "parquet")]
    # {
    use cityjson_lib::{json, parquet};

    let model = json::from_file("amsterdam.city.json")?;
    let manifest = parquet::to_file("city.cityjson-parquet", &model)?;
    let dataset_manifest = parquet::to_dir("city.dataset", &model)?;
    # let _ = (manifest, dataset_manifest);
    # }
    # Ok::<(), cityjson_lib::Error>(())
    ```

=== "Python"
    ```python
    model.serialize_parquet_file("city.cityjson-parquet")
    model.serialize_parquet_dataset_dir("city.dataset")
    ```

=== "C++"
    ```cpp
    model.serialize_parquet_file("city.cityjson-parquet");
    model.serialize_parquet_dataset_dir("city.dataset");
    ```

The wasm adapter remains work in progress and is intentionally omitted from the
published write guide.
