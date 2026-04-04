# ADR 0011: Strict CityJSONSeq Writer With Canonical Root And Explicit Transform

## Status

Proposed.

## Context

The current JSON boundary has an asymmetry:

- strict `CityJSONSeq` reading already exists
- strict `CityJSONSeq` writing does not

Today `serde_cityjson::v2_0::read_feature_stream(...)` expects a stream whose
first item is a `CityJSON` root object and whose later items are
`CityJSONFeature` objects. That is the strict stream shape already enforced by
the reader.

On the write side, `cjlib::json::write_feature_stream(...)` currently writes
newline-delimited self-contained `CityJSONFeature` payloads only. That is a
useful loose JSONL contract, but it is not a strict `CityJSONSeq` writer
because it does not emit:

- a leading `CityJSON` root object
- one canonical file-level `transform`
- one canonical set of global root sections such as `metadata`, `extensions`,
  and `geometry-templates`

This became a real pipeline issue in the 3DBAG intermediate stages.
Reconstruction can produce self-contained feature packages that each carry
their own local root state and transform. Later stages such as `party_walls`
and `floors_estimation` regroup those feature packages into tile-level output
files. Once multiple features are written into one strict `CityJSONSeq` file,
the file has exactly one root object and therefore exactly one root-level
`transform`.

That means regrouping cannot preserve every source package's original
root-level transform verbatim. The writer must choose one canonical output
transform and serialize every feature against it.

The architecture also needs a clear rule for global root sections:

- `metadata`
- `extensions`
- `geometry-templates`
- and future root-level shared sections such as `appearance` or top-level
  extras

These sections belong to the stream root. They should not be inferred
opportunistically from arbitrary per-feature packages during writing.

## Decision

Add a strict `CityJSONSeq` writer API that is built around:

1. one canonical `CityJSON` base root
2. a stream of self-contained feature-sized `CityModel` packages
3. one explicit output transform for the whole file

The design split is:

- `cityjson-rs` remains the semantic model crate
- `serde_cityjson` owns strict `CityJSON` / `CityJSONSeq` wire-format writing
- `cjlib::json` exposes the strict writer as the stable facade

The primitive write path is explicit-transform writing.
Automatic-transform writing is a convenience wrapper around it.

### 1. Canonical Base Root

The strict writer takes a `CityJSON` model as the canonical base root.

That base root:

- must be `CityJSON`
- must have empty `CityObjects`
- must have empty root `vertices`
- may carry global root sections such as `metadata`, `extensions`,
  `geometry-templates`, `appearance`, and top-level extra members

This is preferred over passing `metadata`, `extensions`, and
`geometry-templates` as separate arguments because:

- it keeps the API aligned with the actual `CityJSON` root shape
- it leaves room for future shared root sections without widening the function
  signature again
- it mirrors the existing staged read APIs that already rehydrate feature
  packages against a base `CityJSON` root

### 2. Explicit Transform Is The Primitive

The primary writer API takes the output transform explicitly.

This is the primitive because extent alone does not define a complete
quantization policy. Extent can help determine translation and resulting
metadata extent, but scale is a precision choice. That choice should be
explicit at the boundary.

### 3. Automatic Transform Is A Convenience Wrapper

The convenience API may compute an output transform from the complete extent of
all input features, but it still requires an explicit precision policy.

So the automatic-transform path:

- collects the feature packages
- computes one overall real-world extent
- chooses translation from that extent
- takes scale from explicit options
- delegates to the explicit-transform writer

### 4. Keep Loose Feature-Only JSONL Writing

The existing `cjlib::json::write_feature_stream(...)` contract remains valid
and unchanged.

It continues to mean:

- newline-delimited `CityJSONFeature` payloads
- no leading `CityJSON` root object
- no strict stream-level global-root contract

The new strict writer is a separate API. It must not silently change the
meaning of the existing loose writer.

## API Shape

### `serde_cityjson`

`serde_cityjson` should add strict writer APIs under `v2_0` that write:

- one `CityJSON` header item
- followed by `CityJSONFeature` items

The intended shape is:

```rust
pub struct CityJSONSeqWriteOptions {
    pub validate_default_themes: bool,
    pub trailing_newline: bool,
    pub update_metadata_geographical_extent: bool,
}

pub struct AutoTransformOptions {
    pub scale: [f64; 3],
    pub validate_default_themes: bool,
    pub trailing_newline: bool,
    pub update_metadata_geographical_extent: bool,
}

pub struct CityJSONSeqWriteReport {
    pub transform: cityjson::v2_0::Transform,
    pub geographical_extent: Option<cityjson::v2_0::BBox>,
    pub feature_count: usize,
    pub cityobject_count: usize,
}

pub fn write_cityjsonseq_with_transform_refs<'a, W, I, VR, SS>(
    writer: W,
    base_root: &CityModel<VR, SS>,
    features: I,
    transform: &cityjson::v2_0::Transform,
    options: CityJSONSeqWriteOptions,
) -> Result<CityJSONSeqWriteReport>
where
    W: std::io::Write,
    I: IntoIterator<Item = &'a CityModel<VR, SS>>,
    VR: VertexRef + serde::Serialize + 'a,
    SS: StringStorage + 'a;

pub fn write_cityjsonseq_auto_transform_refs<'a, W, I, VR, SS>(
    writer: W,
    base_root: &CityModel<VR, SS>,
    features: I,
    options: AutoTransformOptions,
) -> Result<CityJSONSeqWriteReport>
where
    W: std::io::Write,
    I: IntoIterator<Item = &'a CityModel<VR, SS>>,
    VR: VertexRef + serde::Serialize + 'a,
    SS: StringStorage + 'a;
```

### `cjlib`

`cjlib::json` should expose facade-level wrappers over the strict writer:

```rust
pub struct CityJSONSeqWriteOptions {
    pub validate_default_themes: bool,
    pub trailing_newline: bool,
    pub update_metadata_geographical_extent: bool,
}

pub struct AutoTransformOptions {
    pub scale: [f64; 3],
    pub validate_default_themes: bool,
    pub trailing_newline: bool,
    pub update_metadata_geographical_extent: bool,
}

pub struct CityJSONSeqWriteReport {
    pub transform: cityjson::v2_0::Transform,
    pub geographical_extent: Option<cityjson::v2_0::BBox>,
    pub feature_count: usize,
    pub cityobject_count: usize,
}

pub fn write_cityjsonseq_refs<'a, I, W>(
    writer: W,
    base_root: &crate::CityModel,
    features: I,
    transform: &cityjson::v2_0::Transform,
    options: CityJSONSeqWriteOptions,
) -> crate::Result<CityJSONSeqWriteReport>
where
    I: IntoIterator<Item = &'a crate::CityModel>,
    W: std::io::Write;

pub fn write_cityjsonseq_auto_transform_refs<'a, I, W>(
    writer: W,
    base_root: &crate::CityModel,
    features: I,
    options: AutoTransformOptions,
) -> crate::Result<CityJSONSeqWriteReport>
where
    I: IntoIterator<Item = &'a crate::CityModel>,
    W: std::io::Write;
```

Owned-value convenience overloads may be added in `cjlib`, but reference-based
variants are the core shape because many write paths already hold borrowed model
handles.

## Validation Rules

The strict writer must reject invalid stream assembly rather than trying to
guess what the caller intended.

### Required Rules

- `base_root` must be `CityJSON`
- each item in `features` must be `CityJSONFeature`
- duplicate `CityObject` ids across feature packages are rejected
- feature packages may not carry conflicting stream-level root state
- the output root `transform` is always taken from the writer input, not from
  per-feature packages

### Shared Root State

The base root is authoritative for shared root sections.

Feature packages may be self-contained, but when writing a strict
`CityJSONSeq`:

- `metadata` comes from the base root, optionally with recomputed
  `geographicalExtent`
- `extensions` come from the base root
- `geometry-templates` come from the base root
- future shared root sections such as `appearance` or top-level extras also
  come from the base root

Per-feature packages must not override those sections during stream writing.

### Metadata Extent

If `update_metadata_geographical_extent` is enabled, the writer recomputes the
overall real-world extent from all features and writes it into the root
metadata.

This extent remains floating-point metadata.
It is not quantized.

## Serialization Rules

### Quantization

`cityjson-rs` remains the real-world semantic model.
Quantization happens at write time.

The strict writer serializes:

- the root header with the chosen output transform
- each feature's root vertices quantized against that transform

Sections that are not quantized today should stay non-quantized:

- metadata extents
- template vertices
- texture vertices
- geometry-instance affine transforms

This preserves the existing serializer rule already enforced by
`serde_cityjson`.

### Root Header

The first emitted item must be a `CityJSON` object.

That object contains:

- `type = "CityJSON"`
- `version`
- chosen output `transform`
- shared root sections from the canonical base root
- empty `CityObjects`
- empty `vertices`

The header is authoritative for the stream.

### Feature Items

Each later emitted item must be a `CityJSONFeature` object.

Feature items:

- carry their own `CityObjects`
- carry their own quantized local `vertices` arrays
- do not duplicate the stream-level shared root sections

## Consequences

Positive:

- aligns strict write semantics with the strict read contract already enforced
  today
- gives regrouping pipelines a correct place to choose one canonical output
  transform
- keeps `CityJSONFeature` as a wire-format boundary while still using
  self-contained `CityModel` packages as the semantic unit
- prevents silent loss or accidental duplication of global root sections
- leaves the current loose JSONL feature writer intact for callers that really
  want feature-only streams

Trade-offs:

- strict `CityJSONSeq` writing is more explicit and less permissive than the
  current loose writer
- Option A necessarily buffers or pre-scans features to compute extent
- callers that want automatic transforms must still choose a scale policy
- some future cases, especially shared appearance or template-pool
  reconciliation across incompatible packages, still require semantic
  remapping or localization work rather than serializer inference

## Rejected Alternatives

### Reuse `write_feature_stream(...)` for strict `CityJSONSeq`

Rejected because the current function already means loose newline-delimited
feature bytes only.
Changing its meaning would silently break callers and blur two distinct
contracts.

### Infer Global Root Sections From Arbitrary Feature Packages

Rejected because stream-level root state should be authoritative and explicit.
Inferring it opportunistically from feature packages is fragile and makes
conflict handling ambiguous.

### Make Automatic Extent-Based Transform Selection The Primitive

Rejected because extent does not fully define a quantization policy.
Scale is a precision choice and should remain explicit.

### Hide This Inside `cityjson-rs`

Rejected because strict `CityJSONSeq` writing is a JSON wire-format concern.
`cityjson-rs` should remain the semantic model crate, not the transport
boundary crate.

## Implementation Notes

The likely implementation sequence is:

1. add strict writer serializers in `serde_cityjson`
2. add facade wrappers in `cjlib::json`
3. keep `write_feature_stream(...)` as the loose writer
4. later expose the strict writer through the shared FFI once the Rust surface
   has stabilized

The current `cjlib::ops` merge and append helpers remain separate concerns.
They may later provide semantic remap or localization workflows, but the strict
writer itself should not hide those semantics behind implicit repair logic.
