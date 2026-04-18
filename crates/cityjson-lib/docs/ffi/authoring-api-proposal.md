# FFI Authoring API Proposal

Status: draft for review only. This document proposes a replacement write-side
API for the shared FFI core and the C++ wrapper. Nothing on this page is
implemented yet.

## Goals

- Make the shared core expressive enough to build the full
  `cityjson_fake_complete.city.json` fixture without raw JSON fragments.
- Replace the current patchwork write API with a small number of consistent,
  typed authoring concepts.
- Keep the low-level FFI ABI generic and binding-friendly.
- Make the C++ wrapper feel like a first-class typed authoring API rather than
  a thin collection of one-off mutators.

## Non-Goals

- Preserve backwards compatibility with the current write-side FFI API.
- Expose every Rust implementation detail 1:1.
- Optimize for handwritten C ergonomics. The core stays C ABI-safe first; the
  C++ wrapper provides the user-facing authoring experience.

## Design Summary

The proposed design has four pillars:

1. The shared core exposes typed model-owned resources and typed draft handles.
2. Recursive CityJSON attribute trees are represented by a typed `Value` API,
   not by raw JSON strings.
3. Geometry authoring is centered on draft objects that match the Rust
   `GeometryDraft` shape: points, linestrings, rings, surfaces, shells, solids,
   and instances.
4. The C++ wrapper provides value types, strong resource IDs, and RAII builder
   objects over the shared core.

## Core FFI Surface

### Core Principles

- Opaque handles own all non-trivial mutable state:
  - `cj_model_t`
  - `cj_value_t`
  - `cj_contact_t`
  - `cj_cityobject_draft_t`
  - `cj_geometry_draft_t`
  - `cj_ring_draft_t`
  - `cj_surface_draft_t`
  - `cj_shell_draft_t`
- Model-owned resources return stable typed IDs:
  - `cj_cityobject_id_t`
  - `cj_geometry_id_t`
  - `cj_geometry_template_id_t`
  - `cj_semantic_id_t`
  - `cj_material_id_t`
  - `cj_texture_id_t`
- Small leaf data stays POD:
  - `cj_vertex_t`
  - `cj_uv_t`
  - `cj_transform_t`
  - `cj_bbox_t`
  - `cj_rgb_t`
  - `cj_rgba_t`
  - `cj_affine_transform_4x4_t`

### New Shared Enums and Plain Structs

```c
typedef uint32_t cj_cityobject_id_t;
typedef uint32_t cj_geometry_id_t;
typedef uint32_t cj_geometry_template_id_t;
typedef uint32_t cj_semantic_id_t;
typedef uint32_t cj_material_id_t;
typedef uint32_t cj_texture_id_t;

typedef enum cj_value_kind_t {
  CJ_VALUE_NULL = 0,
  CJ_VALUE_BOOL = 1,
  CJ_VALUE_INT64 = 2,
  CJ_VALUE_FLOAT64 = 3,
  CJ_VALUE_STRING = 4,
  CJ_VALUE_ARRAY = 5,
  CJ_VALUE_OBJECT = 6,
  CJ_VALUE_GEOMETRY_REF = 7
} cj_value_kind_t;

typedef struct cj_bbox_t {
  double min_x, min_y, min_z;
  double max_x, max_y, max_z;
} cj_bbox_t;

typedef struct cj_rgb_t { float r, g, b; } cj_rgb_t;
typedef struct cj_rgba_t { float r, g, b, a; } cj_rgba_t;

typedef struct cj_affine_transform_4x4_t {
  double elements[16];
} cj_affine_transform_4x4_t;
```

### Value API

The `Value` API is the single generic mechanism for:

- CityObject attributes
- CityObject extra members such as `address` and `children_roles`
- metadata extra members such as `nospec_description`
- root extra members such as `+census`
- contact address objects

It is intentionally typed, recursive, and independent from JSON text.

```c
cj_status_t cj_value_new_null(cj_value_t **out_value);
cj_status_t cj_value_new_bool(bool value, cj_value_t **out_value);
cj_status_t cj_value_new_int64(int64_t value, cj_value_t **out_value);
cj_status_t cj_value_new_float64(double value, cj_value_t **out_value);
cj_status_t cj_value_new_string(cj_string_view_t value, cj_value_t **out_value);
cj_status_t cj_value_new_array(cj_value_t **out_value);
cj_status_t cj_value_new_object(cj_value_t **out_value);
cj_status_t cj_value_new_geometry_ref(cj_geometry_id_t id, cj_value_t **out_value);

cj_status_t cj_value_array_push(cj_value_t *array_value, cj_value_t *element);
cj_status_t cj_value_object_insert(
    cj_value_t *object_value,
    cj_string_view_t key,
    cj_value_t *member_value);

cj_status_t cj_value_free(cj_value_t *value);
```

Notes:

- `cj_value_array_push` and `cj_value_object_insert` transfer ownership of the
  child value into the parent.
- `CJ_VALUE_GEOMETRY_REF` is required because CityJSON attributes can embed
  geometry references, as in the fixture `address.location`.

### Metadata and Root API

The current one-field-at-a-time metadata setters are replaced by grouped,
typed operations.

```c
cj_status_t cj_contact_new(cj_contact_t **out_contact);
cj_status_t cj_contact_set_name(cj_contact_t *contact, cj_string_view_t value);
cj_status_t cj_contact_set_email(cj_contact_t *contact, cj_string_view_t value);
cj_status_t cj_contact_set_role(cj_contact_t *contact, cj_contact_role_t value);
cj_status_t cj_contact_set_website(cj_contact_t *contact, cj_string_view_t value);
cj_status_t cj_contact_set_type(cj_contact_t *contact, cj_contact_type_t value);
cj_status_t cj_contact_set_phone(cj_contact_t *contact, cj_string_view_t value);
cj_status_t cj_contact_set_organization(cj_contact_t *contact, cj_string_view_t value);
cj_status_t cj_contact_set_address(cj_contact_t *contact, cj_value_t *object_value);
cj_status_t cj_contact_free(cj_contact_t *contact);

cj_status_t cj_model_set_metadata_geographical_extent(cj_model_t *model, cj_bbox_t bbox);
cj_status_t cj_model_set_metadata_identifier(cj_model_t *model, cj_string_view_t value);
cj_status_t cj_model_set_metadata_reference_date(cj_model_t *model, cj_string_view_t value);
cj_status_t cj_model_set_metadata_reference_system(cj_model_t *model, cj_string_view_t value);
cj_status_t cj_model_set_metadata_title(cj_model_t *model, cj_string_view_t value);
cj_status_t cj_model_set_metadata_contact(cj_model_t *model, cj_contact_t *contact);
cj_status_t cj_model_set_metadata_extra(
    cj_model_t *model,
    cj_string_view_t key,
    cj_value_t *value);

cj_status_t cj_model_set_root_extra(
    cj_model_t *model,
    cj_string_view_t key,
    cj_value_t *value);

cj_status_t cj_model_add_extension(
    cj_model_t *model,
    cj_string_view_t name,
    cj_string_view_t url,
    cj_string_view_t version);
```

### Appearance Resource API

Appearance resources are model-owned and referenced by typed IDs.

```c
cj_status_t cj_model_add_semantic(
    cj_model_t *model,
    cj_string_view_t semantic_type,
    cj_semantic_id_t *out_id);
cj_status_t cj_model_set_semantic_parent(
    cj_model_t *model,
    cj_semantic_id_t semantic,
    cj_semantic_id_t parent);
cj_status_t cj_model_semantic_set_extra(
    cj_model_t *model,
    cj_semantic_id_t semantic,
    cj_string_view_t key,
    cj_value_t *value);

cj_status_t cj_model_add_material(
    cj_model_t *model,
    cj_string_view_t name,
    cj_material_id_t *out_id);
cj_status_t cj_model_material_set_ambient_intensity(
    cj_model_t *model,
    cj_material_id_t material,
    float value);
cj_status_t cj_model_material_set_diffuse_color(
    cj_model_t *model,
    cj_material_id_t material,
    cj_rgb_t value);
cj_status_t cj_model_material_set_emissive_color(
    cj_model_t *model,
    cj_material_id_t material,
    cj_rgb_t value);
cj_status_t cj_model_material_set_specular_color(
    cj_model_t *model,
    cj_material_id_t material,
    cj_rgb_t value);
cj_status_t cj_model_material_set_shininess(
    cj_model_t *model,
    cj_material_id_t material,
    float value);
cj_status_t cj_model_material_set_transparency(
    cj_model_t *model,
    cj_material_id_t material,
    float value);
cj_status_t cj_model_material_set_is_smooth(
    cj_model_t *model,
    cj_material_id_t material,
    bool value);

cj_status_t cj_model_add_texture(
    cj_model_t *model,
    cj_string_view_t image,
    cj_image_type_t image_type,
    cj_texture_id_t *out_id);
cj_status_t cj_model_texture_set_wrap_mode(
    cj_model_t *model,
    cj_texture_id_t texture,
    cj_wrap_mode_t value);
cj_status_t cj_model_texture_set_texture_type(
    cj_model_t *model,
    cj_texture_id_t texture,
    cj_texture_type_t value);
cj_status_t cj_model_texture_set_border_color(
    cj_model_t *model,
    cj_texture_id_t texture,
    cj_rgba_t value);

cj_status_t cj_model_add_uv_coordinate(
    cj_model_t *model,
    cj_uv_t uv,
    uint32_t *out_uv_index);
cj_status_t cj_model_set_default_material_theme(cj_model_t *model, cj_string_view_t theme);
cj_status_t cj_model_set_default_texture_theme(cj_model_t *model, cj_string_view_t theme);
```

### CityObject Draft API

CityObjects are authored as drafts, inserted into the model, then linked by
typed IDs.

```c
cj_status_t cj_cityobject_draft_new(
    cj_string_view_t id,
    cj_string_view_t cityobject_type,
    cj_cityobject_draft_t **out_draft);
cj_status_t cj_cityobject_draft_set_geographical_extent(
    cj_cityobject_draft_t *draft,
    cj_bbox_t bbox);
cj_status_t cj_cityobject_draft_set_attribute(
    cj_cityobject_draft_t *draft,
    cj_string_view_t key,
    cj_value_t *value);
cj_status_t cj_cityobject_draft_set_extra(
    cj_cityobject_draft_t *draft,
    cj_string_view_t key,
    cj_value_t *value);
cj_status_t cj_model_add_cityobject(
    cj_model_t *model,
    cj_cityobject_draft_t *draft,
    cj_cityobject_id_t *out_id);

cj_status_t cj_model_cityobject_add_geometry(
    cj_model_t *model,
    cj_cityobject_id_t cityobject,
    cj_geometry_id_t geometry);
cj_status_t cj_model_cityobject_add_parent(
    cj_model_t *model,
    cj_cityobject_id_t child,
    cj_cityobject_id_t parent);

cj_status_t cj_cityobject_draft_free(cj_cityobject_draft_t *draft);
```

Notes:

- `cj_model_cityobject_add_parent` updates both parent and child relations.
- Explicit `children_roles` stays an extra member authored through `Value`.

### Geometry Draft API

Geometry authoring follows the Rust draft shape, not the current
boundary-columnar shortcut.

```c
cj_status_t cj_ring_draft_new(cj_ring_draft_t **out_ring);
cj_status_t cj_ring_draft_push_vertex_index(cj_ring_draft_t *ring, uint32_t vertex_index);
cj_status_t cj_ring_draft_push_vertex(cj_ring_draft_t *ring, cj_vertex_t vertex);
cj_status_t cj_ring_draft_add_texture(
    cj_ring_draft_t *ring,
    cj_string_view_t theme,
    cj_texture_id_t texture,
    const uint32_t *uv_indices,
    size_t uv_index_count);

cj_status_t cj_surface_draft_new(
    cj_ring_draft_t *outer,
    cj_surface_draft_t **out_surface);
cj_status_t cj_surface_draft_add_inner_ring(
    cj_surface_draft_t *surface,
    cj_ring_draft_t *inner);
cj_status_t cj_surface_draft_set_semantic(
    cj_surface_draft_t *surface,
    cj_semantic_id_t semantic);
cj_status_t cj_surface_draft_add_material(
    cj_surface_draft_t *surface,
    cj_string_view_t theme,
    cj_material_id_t material);

cj_status_t cj_shell_draft_new(cj_shell_draft_t **out_shell);
cj_status_t cj_shell_draft_add_surface(
    cj_shell_draft_t *shell,
    cj_surface_draft_t *surface);

cj_status_t cj_geometry_draft_new_multi_point(
    cj_string_view_t lod,
    cj_geometry_draft_t **out_draft);
cj_status_t cj_geometry_draft_new_multi_line_string(
    cj_string_view_t lod,
    cj_geometry_draft_t **out_draft);
cj_status_t cj_geometry_draft_new_multi_surface(
    cj_string_view_t lod,
    cj_geometry_draft_t **out_draft);
cj_status_t cj_geometry_draft_new_composite_surface(
    cj_string_view_t lod,
    cj_geometry_draft_t **out_draft);
cj_status_t cj_geometry_draft_new_solid(
    cj_string_view_t lod,
    cj_geometry_draft_t **out_draft);
cj_status_t cj_geometry_draft_new_multi_solid(
    cj_string_view_t lod,
    cj_geometry_draft_t **out_draft);
cj_status_t cj_geometry_draft_new_composite_solid(
    cj_string_view_t lod,
    cj_geometry_draft_t **out_draft);
cj_status_t cj_geometry_draft_new_instance(
    cj_geometry_template_id_t template_id,
    uint32_t reference_vertex_index,
    cj_affine_transform_4x4_t transform,
    cj_geometry_draft_t **out_draft);

cj_status_t cj_geometry_draft_add_point_vertex_index(
    cj_geometry_draft_t *draft,
    uint32_t vertex_index,
    const cj_semantic_id_t *semantic_or_null);
cj_status_t cj_geometry_draft_add_linestring(...);
cj_status_t cj_geometry_draft_add_surface(...);
cj_status_t cj_geometry_draft_add_shell(...);
cj_status_t cj_geometry_draft_add_solid(...);

cj_status_t cj_model_add_geometry(
    cj_model_t *model,
    cj_geometry_draft_t *draft,
    cj_geometry_id_t *out_id);
cj_status_t cj_model_add_geometry_template(
    cj_model_t *model,
    cj_geometry_draft_t *draft,
    cj_geometry_template_id_t *out_id);

cj_status_t cj_geometry_draft_free(cj_geometry_draft_t *draft);
cj_status_t cj_ring_draft_free(cj_ring_draft_t *draft);
cj_status_t cj_surface_draft_free(cj_surface_draft_t *draft);
cj_status_t cj_shell_draft_free(cj_shell_draft_t *draft);
```

Notes:

- Mixed authoring is allowed where it is useful: a ring can accept existing
  vertex indices or new vertices.
- UVs are model-owned, so ring texture assignment references UV indices.
- The old `cj_model_add_geometry_from_boundary` path is removed.

## C++ Wrapper Surface

### Wrapper Goals

- Hide raw C handle management completely.
- Use value types and strong IDs in public signatures.
- Make the common path read like direct model authoring, not ABI plumbing.

### Core Public Types

```cpp
namespace cityjson_lib {

struct BBox { double min_x, min_y, min_z, max_x, max_y, max_z; };
struct Rgb { float r, g, b; };
struct Rgba { float r, g, b, a; };
struct Transform { std::array<double, 3> scale, translate; };
struct AffineTransform4x4 { std::array<double, 16> elements; };

class Value;
class Contact;
class CityObjectDraft;
class RingDraft;
class SurfaceDraft;
class ShellDraft;
class GeometryDraft;

struct CityObjectId { std::uint32_t value; };
struct GeometryId { std::uint32_t value; };
struct GeometryTemplateId { std::uint32_t value; };
struct SemanticId { std::uint32_t value; };
struct MaterialId { std::uint32_t value; };
struct TextureId { std::uint32_t value; };

}  // namespace cityjson_lib
```

### Proposed `Value` API

```cpp
class Value final {
 public:
  static Value null();
  static Value boolean(bool value);
  static Value integer(std::int64_t value);
  static Value number(double value);
  static Value string(std::string value);
  static Value geometry(GeometryId value);
  static Value array();
  static Value object();

  Value& push(Value value);
  Value& insert(std::string key, Value value);
};
```

### Proposed `Model` Authoring API

```cpp
class Model final {
 public:
  static Model create(ModelType type);

  void reserve_import(const ModelCapacities& capacities);
  std::uint32_t add_vertex(const Vertex& vertex);
  std::uint32_t add_template_vertex(const Vertex& vertex);
  std::uint32_t add_uv_coordinate(const UV& uv);

  void set_transform(const Transform& transform);
  void clear_transform();

  void set_metadata_geographical_extent(const BBox& bbox);
  void set_metadata_identifier(std::string_view value);
  void set_metadata_reference_date(std::string_view value);
  void set_metadata_reference_system(std::string_view value);
  void set_metadata_title(std::string_view value);
  void set_metadata_contact(Contact contact);
  void set_metadata_extra(std::string key, Value value);

  void set_root_extra(std::string key, Value value);
  void add_extension(std::string name, std::string url, std::string version);

  SemanticId add_semantic(std::string semantic_type);
  void set_semantic_parent(SemanticId semantic, SemanticId parent);
  void set_semantic_extra(SemanticId semantic, std::string key, Value value);

  MaterialId add_material(std::string name);
  void set_material_ambient_intensity(MaterialId id, float value);
  void set_material_diffuse_color(MaterialId id, Rgb value);
  void set_material_emissive_color(MaterialId id, Rgb value);
  void set_material_specular_color(MaterialId id, Rgb value);
  void set_material_shininess(MaterialId id, float value);
  void set_material_transparency(MaterialId id, float value);
  void set_material_is_smooth(MaterialId id, bool value);

  TextureId add_texture(std::string image, ImageType image_type);
  void set_texture_wrap_mode(TextureId id, WrapMode value);
  void set_texture_type(TextureId id, TextureType value);
  void set_texture_border_color(TextureId id, Rgba value);

  void set_default_material_theme(std::string theme);
  void set_default_texture_theme(std::string theme);

  GeometryId add_geometry(GeometryDraft draft);
  GeometryTemplateId add_geometry_template(GeometryDraft draft);

  CityObjectId add_cityobject(CityObjectDraft draft);
  void add_cityobject_geometry(CityObjectId cityobject, GeometryId geometry);
  void add_cityobject_parent(CityObjectId child, CityObjectId parent);
};
```

### Proposed Draft Types

```cpp
class Contact final {
 public:
  Contact& set_name(std::string value);
  Contact& set_email(std::string value);
  Contact& set_role(ContactRole value);
  Contact& set_website(std::string value);
  Contact& set_type(ContactType value);
  Contact& set_phone(std::string value);
  Contact& set_organization(std::string value);
  Contact& set_address(Value object_value);
};

class CityObjectDraft final {
 public:
  CityObjectDraft(std::string id, std::string type);
  CityObjectDraft& set_geographical_extent(const BBox& bbox);
  CityObjectDraft& set_attribute(std::string key, Value value);
  CityObjectDraft& set_extra(std::string key, Value value);
};

class RingDraft final {
 public:
  RingDraft& push_vertex_index(std::uint32_t index);
  RingDraft& push_vertex(Vertex vertex);
  RingDraft& add_texture(std::string theme, TextureId texture, std::vector<std::uint32_t> uv_indices);
};

class SurfaceDraft final {
 public:
  explicit SurfaceDraft(RingDraft outer);
  SurfaceDraft& add_inner_ring(RingDraft inner);
  SurfaceDraft& set_semantic(SemanticId semantic);
  SurfaceDraft& add_material(std::string theme, MaterialId material);
};

class ShellDraft final {
 public:
  ShellDraft& add_surface(SurfaceDraft surface);
};

class GeometryDraft final {
 public:
  static GeometryDraft multi_point(std::optional<std::string> lod = std::nullopt);
  static GeometryDraft multi_line_string(std::optional<std::string> lod = std::nullopt);
  static GeometryDraft multi_surface(std::optional<std::string> lod = std::nullopt);
  static GeometryDraft composite_surface(std::optional<std::string> lod = std::nullopt);
  static GeometryDraft solid(std::optional<std::string> lod = std::nullopt);
  static GeometryDraft multi_solid(std::optional<std::string> lod = std::nullopt);
  static GeometryDraft composite_solid(std::optional<std::string> lod = std::nullopt);
  static GeometryDraft instance(
      GeometryTemplateId template_id,
      std::uint32_t reference_vertex_index,
      AffineTransform4x4 transform);

  GeometryDraft& add_point(std::uint32_t vertex_index, std::optional<SemanticId> semantic = std::nullopt);
  GeometryDraft& add_linestring(std::vector<std::uint32_t> vertex_indices, std::optional<SemanticId> semantic = std::nullopt);
  GeometryDraft& add_surface(SurfaceDraft surface);
  GeometryDraft& add_shell(ShellDraft shell);
  GeometryDraft& add_solid(ShellDraft outer, std::vector<ShellDraft> inner_shells = {});
};
```

## Example Authoring Style

The target C++ example should be able to read naturally at the model level:

```cpp
auto model = cityjson_lib::Model::create(CJ_MODEL_TYPE_CITY_JSON);

model.set_metadata_title("Complete and fake CityJSON");
model.set_root_extra(
    "+census",
    cityjson_lib::Value::object()
        .insert("percent_men", cityjson_lib::Value::number(49.5))
        .insert("percent_women", cityjson_lib::Value::number(51.5)));

const auto roof = model.add_semantic("RoofSurface");
const auto patio = model.add_semantic("+PatioDoor");
model.set_semantic_parent(patio, roof);
model.set_semantic_extra(roof, "surfaceAttribute", cityjson_lib::Value::boolean(true));

const auto irradiation = model.add_material("irradiation");
const auto winter = model.add_texture(
    "http://www.someurl.org/filename.jpg",
    cityjson_lib::ImageType::Png);

auto solid = cityjson_lib::GeometryDraft::solid("2.1");
// ... add shells, surfaces, semantics, materials, and textured rings ...
const auto building_geometry = model.add_geometry(std::move(solid));

auto building = cityjson_lib::CityObjectDraft("id-1", "BuildingPart");
building.set_attribute("measuredHeight", cityjson_lib::Value::number(22.3));
building.set_extra("address", /* recursive typed value tree */);

const auto building_id = model.add_cityobject(std::move(building));
model.add_cityobject_geometry(building_id, building_geometry);
```

## Explicit Breaking Changes

The current write-side FFI surface is proposed to be replaced, not extended.

Removed or superseded concepts:

- boundary-only geometry insertion as the primary authoring path
- isolated metadata title/identifier setters as the main metadata model
- cityobject mutation by string ID as the main construction mechanism
- ad hoc one-off mutators that do not compose into full model authoring

Retained concepts:

- parse, inspect, cleanup, append, extract, and serialize workflows
- RAII `Model` ownership in C++
- explicit write options and feature-stream helpers

## Review Questions

- Is the value-tree API sufficiently typed, or should geometry references inside
  attribute values be modeled differently?
- Is handle-based cityobject linking acceptable at the core layer, with the C++
  wrapper providing the higher-level ergonomics?
- Should LoD remain string-based at the FFI boundary, or be normalized into a
  dedicated enum/value type before implementation?
