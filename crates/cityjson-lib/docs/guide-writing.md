# Writing CityJSON Data

A walk-through of the build path: create a model, add a city object with geometry, then
serialize to JSON. Each section shows the same operation across all four APIs.

## Create an Empty Model

=== "Rust"
    ```rust
    use cityjson_lib::CityModel;
    use cityjson_lib::cityjson::CityModelType;

    let mut model = CityModel::new(CityModelType::CityJSON);
    ```

=== "C++"
    ```cpp
    #include <cityjson_lib/cityjson_lib.hpp>

    auto model = cityjson_lib::Model::create(CJ_MODEL_TYPE_CITY_JSON);
    ```

=== "Python"
    ```python
    from cityjson_lib import CityModel, ModelType

    model = CityModel.create(model_type=ModelType.CJ_MODEL_TYPE_CITY_JSON)
    ```

=== "WASM"
    ```c
    cj_model_t *handle = NULL;
    cj_status_t status = cj_model_create(CJ_MODEL_TYPE_CITY_JSON, &handle);
    /* check status != CJ_STATUS_SUCCESS for errors */
    ```

## Set Metadata

Dataset-level metadata is optional but useful for any real file.

=== "Rust"
    ```rust
    // mutation goes through the cityjson-rs inner model
    model.as_inner_mut().metadata_mut().set_title("My Dataset".to_string());
    ```

=== "C++"
    ```cpp
    model.set_metadata_title("My Dataset");
    model.set_metadata_identifier("my-dataset-001");
    ```

=== "Python"
    ```python
    model.set_metadata_title("My Dataset")
    model.set_metadata_identifier("my-dataset-001")
    ```

=== "WASM"
    ```c
    cj_string_view_t title = {
        .data = (const uint8_t *)"My Dataset",
        .len  = 10,
    };
    cj_model_set_metadata_title(handle, title);
    ```

## Add Vertices

Vertices are stored once at the model level and referenced by index in each geometry boundary.
These four points form a square footprint in the XY plane.

=== "Rust"
    ```rust
    use cityjson_lib::cityjson::v2_0::RealWorldCoordinate;

    let inner = model.as_inner_mut();
    inner.add_vertex(RealWorldCoordinate::new(10.0, 20.0, 0.0)).unwrap();
    inner.add_vertex(RealWorldCoordinate::new(11.0, 20.0, 0.0)).unwrap();
    inner.add_vertex(RealWorldCoordinate::new(11.0, 21.0, 0.0)).unwrap();
    inner.add_vertex(RealWorldCoordinate::new(10.0, 21.0, 0.0)).unwrap();
    ```

=== "C++"
    ```cpp
    model.add_vertex({10.0, 20.0, 0.0});
    model.add_vertex({11.0, 20.0, 0.0});
    model.add_vertex({11.0, 21.0, 0.0});
    model.add_vertex({10.0, 21.0, 0.0});
    ```

=== "Python"
    ```python
    from cityjson_lib import Vertex

    model.add_vertex(Vertex(10.0, 20.0, 0.0))
    model.add_vertex(Vertex(11.0, 20.0, 0.0))
    model.add_vertex(Vertex(11.0, 21.0, 0.0))
    model.add_vertex(Vertex(10.0, 21.0, 0.0))
    ```

=== "WASM"
    ```c
    cj_vertex_t vertices[] = {
        {10.0, 20.0, 0.0},
        {11.0, 20.0, 0.0},
        {11.0, 21.0, 0.0},
        {10.0, 21.0, 0.0},
    };
    for (size_t i = 0; i < 4; i++) {
        size_t idx = 0;
        cj_model_add_vertex(handle, vertices[i], &idx);
    }
    ```

## Add a City Object and Geometry

Add the city object first, then build a geometry boundary and attach it by index.

=== "Rust"
    ```rust
    use cityjson_lib::cityjson::v2_0::{
        CityObject, CityObjectIdentifier, CityObjectType,
        GeometryDraft, LoD, RingDraft, SurfaceDraft,
    };

    let ring = RingDraft::new([
        [10.0, 20.0, 0.0],
        [11.0, 20.0, 0.0],
        [11.0, 21.0, 0.0],
        [10.0, 21.0, 0.0],
    ]);
    let surface = SurfaceDraft::new(ring, []);
    let geom = GeometryDraft::multi_surface(Some(LoD::LoD2), [surface])
        .insert_into(model.as_inner_mut())
        .unwrap();

    let mut building = CityObject::new(
        CityObjectIdentifier::new("building-1".to_string()),
        CityObjectType::Building,
    );
    building.add_geometry(geom);
    model.as_inner_mut().cityobjects_mut().add(building).unwrap();
    ```

=== "C++"
    ```cpp
    model.add_cityobject("building-1", "Building");

    cityjson_lib::GeometryBoundary boundary{
        .geometry_type   = CJ_GEOMETRY_TYPE_MULTI_SURFACE,
        .vertex_indices  = {0, 1, 2, 3, 0},
        .ring_offsets    = {0},
        .surface_offsets = {0},
    };
    auto geom_idx = model.add_geometry_from_boundary(boundary, "2.2");
    model.attach_geometry_to_cityobject("building-1", geom_idx);
    ```

=== "Python"
    ```python
    from cityjson_lib import GeometryBoundary, GeometryType

    model.add_cityobject("building-1", "Building")

    boundary = GeometryBoundary(
        geometry_type   = GeometryType.CJ_GEOMETRY_TYPE_MULTI_SURFACE,
        has_boundaries  = True,
        vertex_indices  = [0, 1, 2, 3, 0],
        ring_offsets    = [0],
        surface_offsets = [0],
        shell_offsets   = [],
        solid_offsets   = [],
    )
    geom_idx = model.add_geometry_from_boundary(boundary, lod="2.2")
    model.attach_geometry_to_cityobject("building-1", geom_idx)
    ```

=== "WASM"
    ```c
    cj_string_view_t id   = {(const uint8_t *)"building-1", 10};
    cj_string_view_t type = {(const uint8_t *)"Building",   8};
    cj_model_add_cityobject(handle, id, type);

    size_t vi[]  = {0, 1, 2, 3, 0};
    size_t ri[]  = {0};
    size_t si[]  = {0};

    cj_geometry_boundary_view_t boundary = {
        .geometry_type   = CJ_GEOMETRY_TYPE_MULTI_SURFACE,
        .vertex_indices  = {vi, 5},
        .ring_offsets    = {ri, 1},
        .surface_offsets = {si, 1},
        .shell_offsets   = {NULL, 0},
        .solid_offsets   = {NULL, 0},
    };
    cj_string_view_t lod = {(const uint8_t *)"2.2", 3};

    size_t geom_idx = 0;
    cj_model_add_geometry_from_boundary(handle, boundary, lod, &geom_idx);
    cj_model_attach_geometry_to_cityobject(handle, id, geom_idx);
    ```

## Serialize to JSON

Call `cleanup()` to strip unreferenced vertices before writing. Pass `WriteOptions` to
control pretty-printing.

=== "Rust"
    ```rust
    use cityjson_lib::json::WriteOptions;

    // compact
    let json = cityjson_lib::json::to_string(&model)?;

    // pretty-printed
    let json = cityjson_lib::json::to_string_with_options(
        &model,
        WriteOptions { pretty: true, ..Default::default() },
    )?;
    println!("{json}");
    ```

=== "C++"
    ```cpp
    model.cleanup();
    std::string json = model.serialize_document(
        cityjson_lib::WriteOptions{.pretty = true}
    );
    printf("%s\n", json.c_str());
    ```

=== "Python"
    ```python
    from cityjson_lib import WriteOptions

    model.cleanup()
    json_str = model.serialize_document(WriteOptions(pretty=True))
    print(json_str)
    ```

=== "WASM"
    ```c
    cj_model_cleanup(handle);

    cj_json_write_options_t opts = {
        .pretty                  = true,
        .validate_default_themes = true,
    };
    cj_bytes_t out = {0};
    cj_model_serialize_document_with_options(handle, opts, &out);

    /* out.data points to out.len bytes of UTF-8 JSON */
    cj_bytes_free(out);
    cj_model_free(handle);
    ```
