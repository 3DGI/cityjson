# Design

## Deserialization

Parsing a `CityJSON` document is split into two sequential phases.

**Phase 1 - root preparation (`parse_root`).**
The document is read once by a handwritten `serde` visitor that fills a
`PreparedRoot<'de>` struct. Well-known sections with bounded size (transform,
metadata, appearance, geometry-templates, extensions) are deserialized eagerly.
The `CityObjects` map, which may be arbitrarily large, is kept as a borrowed
`&RawValue` slice pointing into the original input bytes. Nothing is allocated
for it yet.

**Phase 2 - model construction (`build_model`).**
The prepared root is used to initialize the `CityModel`. Appearance, geometry
templates, and vertices are imported first, establishing handles that the
`CityObjects` import can reference. The `CityObjects` slice is then
deserialized once more, but streamed entry by entry directly into the model
instead of materializing a full intermediate object graph. Parent and child
relations are resolved in a follow-up pass after all objects have been inserted.

**Geometry.**
Each geometry object is parsed by a streaming visitor that reads the `type`,
`lod`, and `boundaries` fields manually. Boundaries are parsed by a specialized
flat parser that scans the raw bytes and writes vertex indices and offset
vectors directly into the shapes the `cityjson` backend expects
(`Boundary<u32>`). There is no intermediate nested boundary tree. Finished
geometry parts are inserted through the backend's trusted raw API
(`add_geometry_unchecked`) which skips the authoring-time validation that
`GeometryDraft::insert_into` performs.

**Attributes.**
Attributes and extra properties are deserialized directly into the backend
`AttributeValue<SS>` and `Attributes<SS>` types via
`AttributeValueSeed` / `AttributesSeed` / `OptionalAttributesSeed`. There is no
temporary `RawAttribute` tree: the `CityObject` visitor produces final values in
a single pass.

**Owned and borrowed storage.**
The single `ParseStringStorage<'de>` trait controls whether string values are
heap-allocated (`OwnedStringStorage`) or zero-copy borrowed from the input
(`BorrowedStringStorage`). Borrowed mode fails on strings that contain JSON
escape sequences because those cannot be represented without allocation.

## Serialization

**Direct streaming.**
The serializer writes the `CityModel` directly through `serde::Serialize`
without first constructing an intermediate `serde_json::Value` DOM. Each
section of the document is a dedicated serializer struct that borrows from the
model and emits JSON fields on demand.

**Shared write context.**
Before any field is written, a `WriteContext` is built once for the entire
serialization. It precomputes four lookup maps:

- city object handle -> JSON id string
- geometry template handle -> dense array index
- material handle -> dense array index
- texture handle -> dense array index

All nested serializers borrow the same context, so handle-to-index lookups are
O(1) hash-map reads with no repeated work.

**Transform-aware vertex quantization.**
When a transform is present, vertex coordinates are quantized by applying the
inverse transform `(x - translate) / scale` before serialization and then
rounded to the nearest integer. Without a transform, coordinates are written as
floating-point values. The same quantization applies when writing
`CityJSONSeq` streams.

**Material compaction.**
When all surfaces of a geometry in a given material theme share the same
non-null material index, the serializer writes the compact `{"value": N}` form
instead of an explicit `{"values": [...]}` array.

**Validation policy.**
`write_model` and `to_vec` serialize without pre-flight checks by default.
Setting `WriteOptions::validate_default_themes` enables the
`validate_default_themes` pass before serialization, confirming that the
default material and texture theme names reference themes that actually exist
in the appearance section.

**`CityJSONSeq` stream writing.**
`write_feature_stream(writer, features, options)` writes one `CityJSON` header
followed by `CityJSONFeature` items. The header is derived from the shared root
state of the feature models, while each feature item suppresses metadata,
extensions, appearance, and geometry templates. By default the transform
translation is derived from the bounding box of all feature vertices; use
`FeatureStreamTransform::Explicit` to supply an explicit transform instead.
