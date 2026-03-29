# Roofer And `cjlib` Harmonization Plan

## Premise

We own both `roofer` and `cjlib`.

Breaking changes, rewrites, and deletion of legacy abstractions are acceptable
if they produce a cleaner architecture.

The goal is not to preserve Roofer's current writer seam and hide Rust behind
it.
The goal is to delete Roofer's local CityJSON authoring layer and make `cjlib`
the only CityJSON boundary.

## Decision

Replace Roofer's current CityJSON writer stack with a `cjlib` foreign boundary
that has three layers:

- `cjlib` Rust core
- `cjlib-c` stable C ABI
- `cjlib-cpp` first-class C++ API over that C ABI

The public foreign-language integration surface for Roofer should be
`cjlib-cpp`, not a bag of raw C structs.

That means:

- delete [CityJsonWriter.cpp](/home/balazs/Development/roofer/src/extra/io/CityJsonWriter.cpp#L36)
- delete or radically shrink [CityJsonWriter.hpp](/home/balazs/Development/roofer/include/roofer/io/CityJsonWriter.hpp)
- stop constructing CityJSON with `nlohmann::json`
- stop passing `std::ostream` into the export boundary
- stop freezing a Roofer-specific C DTO schema as the long-term API
- let C++ callers construct full `CityModel` values from standard C++ types
- keep the canonical semantic model in Rust

Important distinction:

- stable C++ API: yes
- stable native C++ ABI: no

The stable binary boundary should be C.
The stable C++ API should be a wrapper over that boundary.

## Why The Current Writer Should Not Survive

Roofer's current writer is carrying too much format logic.

It currently owns:

- vertex dedup and topology assembly: [CityJsonWriter.cpp](/home/balazs/Development/roofer/src/extra/io/CityJsonWriter.cpp#L37)
- Roofer face-label to CityJSON semantic mapping: [CityJsonWriter.cpp](/home/balazs/Development/roofer/src/extra/io/CityJsonWriter.cpp#L98)
- attribute-to-JSON normalization: [CityJsonWriter.cpp](/home/balazs/Development/roofer/src/extra/io/CityJsonWriter.cpp#L172)
- metadata document construction: [CityJsonWriter.cpp](/home/balazs/Development/roofer/src/extra/io/CityJsonWriter.cpp#L350)
- feature construction and manual quantization: [CityJsonWriter.cpp](/home/balazs/Development/roofer/src/extra/io/CityJsonWriter.cpp#L419)

Those responsibilities belong on the CityJSON side of the system, not in
Roofer's reconstruction codebase.

The current writer also has design smells that should be removed rather than
wrapped:

- separate LoD maps with mismatched part ids, patched with `try/catch`: [CityJsonWriter.cpp](/home/balazs/Development/roofer/src/extra/io/CityJsonWriter.cpp#L261)
- root object id inferred by "object without parents": [CityJsonWriter.cpp](/home/balazs/Development/roofer/src/extra/io/CityJsonWriter.cpp#L435)

Those are not stable concepts worth preserving.

## Target Architecture

### Roofer

Roofer should stop thinking in CityJSON terms at the export boundary.

It can keep its own internal reconstruction representation.
There will always be a conversion step.
The important design choice is what Roofer converts into.

Roofer should convert into generic `cjlib-cpp` model inputs and builders, not
into:

- ad hoc JSON
- Roofer-specific FFI DTO arrays
- a bespoke second CityJSON implementation

Key breaking changes inside Roofer still make sense:

- replace `unordered_map<int, Mesh>` per LoD with one stable part model keyed once
- replace raw label ints `0/1/2/3` with a semantic enum
- make the root feature id explicit
- pass real-world coordinates across the seam
- remove all CityJSON-specific JSON assembly from Roofer

### `cjlib`

`cjlib` should remain generic.
Do not put Roofer-specific types into the main public crate.

The crate and module shape should become:

- `cjlib`
  - canonical Rust semantic model wrapper
  - Rust read and write APIs
- `cjlib-c`
  - stable C ABI
  - opaque handles
  - explicit ownership and error rules
- `cjlib-cpp`
  - RAII wrappers
  - standard C++ types
  - builders, readers, and writers

That keeps the core crate aligned with its current direction:

- [implementation-plan.md](/home/balazs/Development/cjlib/docs/implementation-plan.md#L28)
- [public-api.md](/home/balazs/Development/cjlib/docs/public-api.md#L44)

Inside Rust, the export path should still build `cityjson-rs` models directly.
That is already feasible through drafts and model mutation:

- geometry authoring: [builder.rs](/home/balazs/Development/cityjson-rs/benches/builder.rs#L26)
- vertex insertion: [citymodel.rs](/home/balazs/Development/cityjson-rs/src/v2_0/citymodel.rs#L451)
- metadata mutation: [metadata.rs](/home/balazs/Development/cityjson-rs/src/v2_0/metadata.rs#L87)

## Best Boundary Shape

The best long-term boundary is:

- public C++ API: `cjlib-cpp`
- stable binary substrate: `cjlib-c`
- canonical semantics and serialization: Rust

This is better than both extremes:

- better than a raw C-only DTO surface for Roofer
- better than pretending a native exported C++ ABI is stable

The foreign API should expose three workflow families:

- read
- build
- write

The build API should be able to express a full CityJSON v2.0 model from
standard C++ types.
That does not mean the C++ layer becomes the canonical semantic
implementation.
It means C++ callers can describe the full model in ordinary C++ values, and
`cjlib` lowers that into the canonical Rust model.

## Coordinate Contract

Choose one coordinate contract and enforce it everywhere.

The seam should use real-world coordinates only.

Roofer owns:

- reconstruction
- CRS conversion if needed

`cjlib` owns:

- `transform`
- quantization
- serialization

That matches `cityjson-rs` and `serde_cityjson`, which internally store
real-world coordinates and quantize on write when a transform exists:

- [citymodel.rs](/home/balazs/Development/serde_cityjson/src/ser/citymodel.rs#L64)
- [citymodel.rs](/home/balazs/Development/serde_cityjson/src/ser/citymodel.rs#L406)

## Required `cjlib` Addition Exposed By Roofer

Roofer's current stream shape is:

- one metadata `CityJSON` root
- then many `CityJSONFeature` roots
- features do not carry their own `transform`

Current `serde_cityjson` writes `transform` if the model has one:

- [citymodel.rs](/home/balazs/Development/serde_cityjson/src/ser/citymodel.rs#L64)

So Roofer dogfooding will likely force one new explicit write mode in Rust:

- quantize feature vertices using an external transform
- optionally omit `transform` from feature roots

That is a good addition.
It is not Roofer-specific behavior.
It is a real boundary mode already present conceptually in the wider CityJSON
metadata-plus-feature stream workflow.

## Concrete C++ API Sketch

### Design Rules

The C++ layer should satisfy all of these:

- C++ callers can build a full `CityModel` from standard C++ types
- C++ callers can read and write documents and features
- the canonical semantic model still lives in Rust
- ownership is explicit and RAII-friendly
- unsupported parts of CityJSON fail explicitly rather than silently disappearing

### Core Value Types

```cpp
namespace cj {

enum class ModelKind {
  CityJSON,
  CityJSONFeature,
};

enum class GeometryKind {
  MultiPoint,
  MultiLineString,
  MultiSurface,
  CompositeSurface,
  Solid,
  MultiSolid,
  CompositeSolid,
  GeometryInstance,
};

enum class SemanticSurface {
  RoofSurface,
  WallSurface,
  GroundSurface,
  ClosureSurface,
  OuterCeilingSurface,
  OuterFloorSurface,
};

struct Vec2d {
  double x;
  double y;
};

struct Vec3d {
  double x;
  double y;
  double z;
};

struct Transform {
  std::array<double, 3> scale;
  std::array<double, 3> translate;
};

class AttributeValue {
 public:
  static AttributeValue null();
  static AttributeValue boolean(bool value);
  static AttributeValue integer(std::int64_t value);
  static AttributeValue number(double value);
  static AttributeValue string(std::string value);
  static AttributeValue array(std::vector<AttributeValue> values);
  static AttributeValue object(std::map<std::string, AttributeValue> values);
};

using Attributes = std::map<std::string, AttributeValue>;

struct MetadataInput {
  std::optional<std::string> identifier;
  std::optional<std::string> reference_system;
  std::optional<std::array<double, 6>> geographical_extent;
  Attributes extra;
};

struct RingInput {
  std::vector<Vec3d> vertices;
};

struct SurfaceInput {
  std::vector<RingInput> rings;
  std::optional<SemanticSurface> semantic;
};

struct ShellInput {
  std::vector<SurfaceInput> surfaces;
};

struct GeometryInput {
  GeometryKind kind;
  std::optional<std::string> lod;
  std::vector<ShellInput> shells;
  Attributes extra;
};

struct CityObjectInput {
  std::string id;
  std::string type;
  Attributes attributes;
  std::vector<std::string> parents;
  std::vector<std::string> children;
  std::vector<GeometryInput> geometries;
};

struct AppearanceInput;
struct GeometryTemplatesInput;

struct ModelInput {
  ModelKind kind = ModelKind::CityJSON;
  std::string version = "2.0";
  std::optional<Transform> transform;
  std::optional<MetadataInput> metadata;
  std::vector<CityObjectInput> cityobjects;
  std::optional<AppearanceInput> appearance;
  std::optional<GeometryTemplatesInput> geometry_templates;
  Attributes extra_root_members;
};

class Error;

template <typename T>
using Result = std::expected<T, Error>;

}  // namespace cj
```

Notes:

- the public API should use `std::string`, `std::vector`, `std::optional`,
  `std::array`, `std::map`, and `std::filesystem::path`
- `AppearanceInput` and `GeometryTemplatesInput` must exist in the real API even
  if Roofer v1 does not use them
- "full CityModel from C++ std types" means the API shape must be capable of
  representing full CityJSON v2.0, not just Roofer's first subset

### Build API

Two build styles make sense:

- a one-shot value-based constructor
- an incremental builder API

One-shot:

```cpp
namespace cj {

class Model;

Result<Model> build_model(const ModelInput& input);

}  // namespace cj
```

Incremental:

```cpp
namespace cj {

class SurfaceBuilder {
 public:
  void add_ring(std::vector<Vec3d> ring);
  void set_semantic(SemanticSurface semantic);
};

class ShellBuilder {
 public:
  SurfaceBuilder add_surface();
};

class GeometryBuilder {
 public:
  ShellBuilder add_shell();
  void set_extra(std::string key, AttributeValue value);
};

class CityObjectBuilder {
 public:
  void set_attribute(std::string key, AttributeValue value);
  void add_parent(std::string id);
  void add_child(std::string id);
  GeometryBuilder add_geometry(
      GeometryKind kind,
      std::optional<std::string> lod = {});
};

class ModelBuilder {
 public:
  explicit ModelBuilder(ModelKind kind = ModelKind::CityJSON);
  void set_transform(Transform transform);
  void set_metadata(MetadataInput metadata);
  void set_root_member(std::string key, AttributeValue value);
  CityObjectBuilder add_cityobject(std::string id, std::string type);
  Result<Model> build() &&;
};

}  // namespace cj
```

That gives C++ callers a clean way to construct a complete model without
manually flattening everything into C arrays.

### Read API

```cpp
namespace cj {

class Model {
 public:
  Model(Model&&) noexcept;
  Model& operator=(Model&&) noexcept;
  ~Model();

  ModelKind kind() const;
  std::string version() const;
  std::vector<std::string> cityobject_ids() const;

  Result<ModelInput> to_input() const;
  Result<std::string> to_json() const;
  Result<std::string> to_feature_json() const;
};

Result<Model> read_json(const std::filesystem::path& path);
Result<Model> read_feature_json(const std::filesystem::path& path);
Result<Model> read_feature_json(
    const std::filesystem::path& feature_path,
    const Model& base_document);

class FeatureStreamReader {
 public:
  static Result<FeatureStreamReader> open(const std::filesystem::path& path);
  Result<std::optional<Model>> next();
};

}  // namespace cj
```

`Model::to_input()` matters.
If the C++ API can build a full model from standard types, it should also be
able to materialize that same standard-type representation back out when a C++
caller wants to inspect or transform it without depending on Rust internals.

### Write API

```cpp
namespace cj {

Result<void> write_json(
    const std::filesystem::path& path,
    const Model& model);

Result<void> write_feature_json(
    const std::filesystem::path& path,
    const Model& model);

struct FeatureStreamWriterOptions {
  std::optional<Transform> external_transform;
  bool omit_feature_transform = false;
};

class FeatureStreamWriter {
 public:
  static Result<FeatureStreamWriter> open(
      const std::filesystem::path& path,
      const Model& metadata_document,
      FeatureStreamWriterOptions options = {});

  Result<void> write(const Model& feature_model);
  Result<void> close();
};

}  // namespace cj
```

That write surface directly covers Roofer's stream use case.

### Roofer-Side Usage Sketch

```cpp
cj::ModelBuilder feature_builder(cj::ModelKind::CityJSONFeature);
feature_builder.add_cityobject("building-1", "Building");

auto feature_model = feature_builder.build();

cj::ModelBuilder metadata_builder(cj::ModelKind::CityJSON);
metadata_builder.set_transform(tile_transform);
metadata_builder.set_metadata(tile_metadata);

auto metadata_model = metadata_builder.build();

auto writer = cj::FeatureStreamWriter::open(
    output_path,
    metadata_model.value(),
    {.external_transform = tile_transform, .omit_feature_transform = true});

writer.value().write(feature_model.value());
writer.value().close();
```

Roofer would still convert from its own reconstruction types first.
The difference is that the conversion target is now a generic C++ `cjlib`
surface instead of ad hoc JSON or raw C FFI structs.

## Backing C ABI Sketch

The C ABI is not the ergonomic surface.
It is the stable substrate that `cjlib-cpp` wraps.

Representative shape:

```c
typedef struct cj_model cj_model;
typedef struct cj_model_builder cj_model_builder;
typedef struct cj_feature_stream_reader cj_feature_stream_reader;
typedef struct cj_feature_stream_writer cj_feature_stream_writer;

typedef struct cj_error {
    int code;
    const char* message;
} cj_error;

typedef enum cj_model_kind {
    CJ_MODEL_CITYJSON = 0,
    CJ_MODEL_CITYJSONFEATURE = 1,
} cj_model_kind;

cj_status cj_model_builder_new(cj_model_kind kind, cj_model_builder** out);
cj_status cj_model_builder_set_transform(
    cj_model_builder* builder,
    const cj_transform* transform);
cj_status cj_model_builder_add_cityobject(
    cj_model_builder* builder,
    const cj_cityobject_desc* object_desc);
cj_status cj_model_builder_build(
    cj_model_builder* builder,
    cj_model** out);

cj_status cj_read_json_file(const char* path, cj_model** out);
cj_status cj_write_json_file(const char* path, const cj_model* model);

cj_status cj_feature_stream_writer_open(
    const char* path,
    const cj_model* metadata_document,
    const cj_feature_stream_writer_options* options,
    cj_feature_stream_writer** out);
cj_status cj_feature_stream_writer_write(
    cj_feature_stream_writer* writer,
    const cj_model* feature_model);
cj_status cj_feature_stream_writer_close(
    cj_feature_stream_writer* writer);
```

The actual C API can stay more granular and less pleasant than the C++ API.
That is acceptable because the C++ wrapper is the real caller-facing foreign
surface for Roofer.

## Alternatives

### 1. Stable C ABI Plus First-Class C++ API

Best overall architecture.
Recommended.

### 2. Direct `cxx` Bridge

Good only if the integration remains tightly in-tree and source-coupled.
Less compelling as a reusable long-term boundary.

### 3. C ABI Only

Technically viable.
Not recommended for Roofer.
It forces Roofer into low-level flattening code for little architectural gain.

### 4. Native Exported C++ ABI

Not recommended.
That is not a stable binary boundary across toolchains.

### 5. Preserve Current Writer Interface And Swap Internals

Not recommended.
Too many current design problems survive.

## What Not To Do

- do not keep `CityJsonWriterInterface` as the long-term boundary
- do not let Roofer keep constructing CityJSON-shaped JSON
- do not expose raw C DTOs as the primary C++ integration API
- do not export a native C++ ABI and call it "stable"
- do not create a second canonical CityJSON semantic implementation in C++
- do not keep per-LoD maps with sid mismatch handling
- do not pass quantized coordinates from Roofer into Rust
- do not add Roofer-specific helpers to the `cjlib` root API

## Concrete Rewrite Plan

### Phase 1: Freeze The Public Foreign Surface

- define the `cjlib-cpp` value types for full-model construction
- define the `cjlib-cpp` builder, reader, and writer API
- define the backing `cjlib-c` handle model and ownership rules
- decide the exact support level for appearance, templates, and extras in v1

### Phase 2: Add The Rust Build And Write Path

- implement `ModelInput -> CityModel` lowering in Rust
- implement document and feature writing
- implement feature stream writing with external-transform mode
- add parity tests against representative Roofer output

### Phase 3: Add The C ABI

- add `cjlib-c`
- expose opaque model, builder, and writer handles
- expose read, build, and write operations
- make ownership and error propagation explicit

### Phase 4: Add The C++ Wrapper

- add `cjlib-cpp`
- wrap the C ABI in RAII types
- expose standard C++ containers and value types
- support full-model construction from C++ std types

### Phase 5: Migrate Roofer

- replace `CityJsonWriter.cpp` usage with `cjlib-cpp`
- convert Roofer reconstruction output to `cjlib-cpp` model inputs
- simplify Roofer call sites to build models and write streams through `cjlib`

### Phase 6: Delete Roofer's Legacy Writer

- remove `CityJsonWriter.cpp`
- remove writer-specific JSON helpers
- remove the writer-oriented `std::ostream` seam
- remove Roofer-local CityJSON assembly code

## Immediate Next Step

If this becomes the next dogfooding task, the immediate deliverable should be:

1. freeze the `cjlib-cpp` `ModelInput` and `FeatureStreamWriterOptions` types
2. prototype `build_model(const ModelInput&)`
3. prototype `FeatureStreamWriter::open` and `write`
4. migrate one Roofer feature path end to end
5. add a parity test for Roofer's metadata-plus-feature stream shape
