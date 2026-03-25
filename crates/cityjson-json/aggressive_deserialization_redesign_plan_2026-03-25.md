# Aggressive Deserialization Redesign Plan

## Goal

Replace the current split owned/borrowed deserialization architecture with a single staged parser pipeline that:

- parses the root once into a small borrowed shell
- uses `&serde_json::value::RawValue` only at declared section boundaries
- parses each section from `&RawValue` into typed borrowed section structs
- builds either `CityModel<u32, OwnedStringStorage>` or `CityModel<u32, BorrowedStringStorage<'de>>` from the same import code
- removes root-level special cases for `appearance` and `geometry-templates`
- minimizes duplicate code and schema mirroring
- is easy to reason about, test, and extend

This plan assumes that `cityjson-rs` metadata is fully `StringStorage`-generic, including `Metadata`, `Contact`, and related helper types.

## Architecture Decision

### Current problem

The current implementation forks too early:

1. it defines separate owned and borrowed raw input trees
2. some borrowed fields are typed borrowed structs, while others are untyped JSON blobs
3. `appearance` and `geometry-templates` are parsed as untyped borrowed blobs and then only the owned path reparses them into typed owned structs
4. the borrowed path therefore stops exactly where the code stopped using a typed borrowed representation

This is why borrowed deserialization "worked for other string-bearing types" but stopped at appearance/templates. The distinction was architectural, not semantic.

### New design

Use one staged deserialization pipeline:

1. deserialize the input into a thin borrowed root shell
2. keep major sections as `&RawValue`
3. deserialize each section from its `&RawValue` into typed borrowed section structs
4. convert those typed section structs into `CityModel<u32, SS>` with a small storage adapter trait

The storage adapter is the only owned-vs-borrowed switch.

This is a two-stage parser by design, but not a messy hybrid. It is governed by one strict rule:

- `RawValue` is allowed only at declared parser boundaries
- inside a parser boundary, parsing is fully typed

This means:

- no duplicated raw owned/borrowed trees
- no `serde_json::Value` or `serde_json_borrow::Value` for schema-defined sections
- no reparsing JSON subtrees through owned intermediate JSON values
- no "borrowed mode unsupported" special cases for appearance/templates
- no need to mirror the entire CityJSON schema in one giant borrowed AST

## Public API Redesign

Break the API aggressively and make the generic parser the primary interface.

### New primary API

```rust
pub fn from_str<'de, SS>(input: &'de str) -> Result<CityModel<u32, SS>>
where
    SS: ParseStringStorage<'de>;
```

Optional convenience wrappers may remain, but they are not the conceptual API:

```rust
pub fn from_str_owned(input: &str) -> Result<OwnedCityModel> {
    from_str::<OwnedStringStorage>(input)
}

pub fn from_str_borrowed<'de>(input: &'de str) -> Result<BorrowedCityModel<'de>> {
    from_str::<BorrowedStringStorage<'de>>(input)
}
```

### API removals and renames

Remove or demote the following:

- root-level implementation split between `from_str_owned` and `from_str_borrowed`
- any internal API that is specialized only because of owned vs borrowed string storage
- duplicated raw owned/borrowed helper functions and structs

### Optional stronger break

If maximum cleanliness is preferred, remove `from_str_owned` and `from_str_borrowed` entirely from the crate root and expose only:

```rust
serde_cityjson::v2_0::from_str::<OwnedStringStorage>(...)
serde_cityjson::v2_0::from_str::<BorrowedStringStorage<'_>>(...)
```

This is the cleanest shape, but it is less ergonomic. If ergonomics still matter, keep wrappers as thin aliases.

## Core Internal Abstraction

Add a local trait in the deserialization module:

```rust
pub trait ParseStringStorage<'de>: StringStorage {
    fn store(value: &'de str) -> Self::String;
}

impl<'de> ParseStringStorage<'de> for OwnedStringStorage {
    fn store(value: &'de str) -> Self::String {
        value.to_owned()
    }
}

impl<'de> ParseStringStorage<'de> for BorrowedStringStorage<'de> {
    fn store(value: &'de str) -> Self::String {
        value
    }
}
```

This trait is intentionally tiny. It should only solve string storage. It should not become a second parsing framework.

## Staged Parser Redesign

## Rule

`RawValue` is allowed only at declared parser boundaries.

Inside a parser boundary, the section is parsed into typed borrowed structs.

Do not use `RawValue` opportunistically for convenience. Do not mix typed fields and raw blobs inside the same section unless the blob is itself a declared child boundary.

## Declared parser boundaries

The recommended section boundaries are:

- root shell
- `metadata`
- `extensions`
- `appearance`
- `geometry-templates`
- `CityObjects`

Within `CityObjects`, city objects and geometry entries should be parsed with typed borrowed structs, not left as raw blobs unless a future refactor deliberately introduces a second-level boundary.

## Root shell

The root shell should remain small and only hold:

- root scalars and header fields
- transform
- vertices
- section boundaries as `&RawValue`
- root extra properties if they remain generic

Suggested shape:

```rust
#[derive(Deserialize)]
struct RawRoot<'de> {
    #[serde(rename = "type", borrow)]
    type_name: &'de str,
    #[serde(default, borrow)]
    version: Option<&'de str>,
    #[serde(default)]
    transform: Option<RawTransform>,
    vertices: Vec<[f64; 3]>,
    #[serde(default, borrow)]
    metadata: Option<&'de RawValue>,
    #[serde(default, borrow)]
    extensions: Option<&'de RawValue>,
    #[serde(rename = "CityObjects", borrow)]
    cityobjects: &'de RawValue,
    #[serde(default, borrow)]
    appearance: Option<&'de RawValue>,
    #[serde(rename = "geometry-templates", default, borrow)]
    geometry_templates: Option<&'de RawValue>,
    #[serde(flatten, borrow)]
    extra: HashMap<&'de str, RawAttribute<'de>>,
}
```

This keeps the root compact while still borrowing from the input.

## Typed section structs

Each declared parser boundary gets its own typed borrowed section structs.

Recommended families:

- `RawMetadataSection<'de>`
- `RawContact<'de>`
- `RawExtensionsSection<'de>`
- `RawExtension<'de>`
- `RawAppearanceSection<'de>`
- `RawMaterial<'de>`
- `RawTexture<'de>`
- `RawGeometryTemplatesSection<'de>`
- `RawCityObject<'de>`
- `RawGeometry<'de>`
- `RawSemantics<'de>`
- `RawSemanticSurface<'de>`

This avoids a full-crate raw mirror while still keeping each section strongly typed.

## Section parsing policy

Every section parser must:

1. accept `&RawValue`
2. deserialize directly into typed borrowed section structs
3. hand those structs to the generic importer

Example:

```rust
fn parse_appearance<'de>(raw: &'de RawValue) -> Result<RawAppearanceSection<'de>> {
    RawAppearanceSection::deserialize(raw).map_err(Error::from)
}
```

This is the core simplification `RawValue` provides. It lets the code defer parsing without losing access to borrowed substrings from the original input.

## Attribute Representation

Replace the current mixed `serde_json::Value` / `serde_json_borrow::Value` handling with one typed recursive enum:

```rust
#[derive(Deserialize)]
#[serde(untagged)]
enum RawAttribute<'de> {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(&'de str),
    Array(Vec<RawAttribute<'de>>),
    Object(HashMap<&'de str, RawAttribute<'de>>),
}
```

Then convert with:

```rust
fn attribute_value<'de, SS>(raw: RawAttribute<'de>) -> Result<AttributeValue<SS>>
where
    SS: ParseStringStorage<'de>;
```

This removes:

- `serde_json_borrow`
- `BorrowedJsonValue`
- most use of `OwnedJsonValue`
- re-parsing through owned JSON values
- leak-based `Cow` fallback logic

The current `Box::leak` strategy in borrowed attribute conversion should be considered unacceptable in the new design and removed completely.

## Deserialization Pipeline

Implement the pipeline as:

```rust
pub fn from_str<'de, SS>(input: &'de str) -> Result<CityModel<u32, SS>>
where
    SS: ParseStringStorage<'de>,
{
    let raw: RawRoot<'de> = serde_json::from_str(input)?;
    build_model::<SS>(raw)
}
```

## Build model

```rust
fn build_model<'de, SS>(raw: RawRoot<'de>) -> Result<CityModel<u32, SS>>
where
    SS: ParseStringStorage<'de>;
```

This function should:

1. parse and validate the root header
2. create the `CityModel`
3. apply transform
4. import root resources:
   - appearance, parsed from `&RawValue`
   - geometry templates, parsed from `&RawValue`
5. import vertices
6. import metadata, parsed from `&RawValue`
7. import extensions, parsed from `&RawValue`
8. import extra root attributes
9. import city objects, parsed from `&RawValue`
10. resolve parent/child relations

This order should remain explicit and centralized.

## Geometry resources

Keep a single internal state object:

```rust
struct GeometryResources {
    materials: Vec<MaterialHandle>,
    textures: Vec<TextureHandle>,
    templates: Vec<GeometryTemplateHandle>,
}
```

This is already conceptually correct. The redesign should reuse it instead of duplicating import logic.

## Import Functions

Refactor to one generic family of import helpers:

- `parse_metadata<'de>(raw: &'de RawValue) -> Result<RawMetadataSection<'de>>`
- `build_metadata<'de, SS>(raw: RawMetadataSection<'de>) -> Result<Metadata<SS>>`
- `build_contact<'de, SS>(raw: RawContact<'de>) -> Result<Contact<SS>>`
- `parse_extensions<'de>(raw: &'de RawValue) -> Result<RawExtensionsSection<'de>>`
- `build_extensions<'de, SS>(raw: RawExtensionsSection<'de>) -> Extensions<SS>`
- `parse_appearance<'de>(raw: &'de RawValue) -> Result<RawAppearanceSection<'de>>`
- `import_root_appearance<'de, SS>(...) -> Result<()>`
- `parse_geometry_templates<'de>(raw: &'de RawValue) -> Result<RawGeometryTemplatesSection<'de>>`
- `import_geometry_templates<'de, SS>(...) -> Result<()>`
- `parse_cityobjects<'de>(raw: &'de RawValue) -> Result<HashMap<&'de str, RawCityObject<'de>>>`
- `import_cityobjects<'de, SS>(...) -> Result<()>`
- `import_geometry<'de, SS>(...) -> Result<GeometryHandle>`
- `import_template_geometry<'de, SS>(...) -> Result<GeometryTemplateHandle>`
- `attribute_map<'de, SS>(...) -> Result<Attributes<SS>>`

Every one of these should be generic over `SS: ParseStringStorage<'de>`.

## String handling rule

Every time data enters the `cityjson-rs` model and is representable via `SS::String`, call `SS::store(...)`.

Examples:

- city object identifiers
- extension names, URLs, versions
- material names
- texture image paths
- theme names
- semantic extension names
- metadata string fields
- attribute keys and string values

This gives one obvious rule that is easy to audit.

## Appearance

Implement:

- root field as `Option<&'de RawValue>`
- `RawAppearanceSection<'de>`
- `RawMaterial<'de>`
- `RawTexture<'de>`

Suggested shape:

```rust
#[derive(Deserialize)]
struct RawAppearanceSection<'de> {
    #[serde(default, borrow)]
    materials: Vec<RawMaterial<'de>>,
    #[serde(default, borrow)]
    textures: Vec<RawTexture<'de>>,
    #[serde(rename = "vertices-texture", default)]
    vertices_texture: Vec<[f32; 2]>,
    #[serde(rename = "default-theme-material", default, borrow)]
    default_theme_material: Option<&'de str>,
    #[serde(rename = "default-theme-texture", default, borrow)]
    default_theme_texture: Option<&'de str>,
}
```

Import directly into `Material<SS>` and `Texture<SS>` using `SS::store`.

No reparsing through owned JSON.
No special borrowed path.
No owned-only intermediate structs.

## Geometry Templates

Implement:

- root field as `Option<&'de RawValue>`
- `RawGeometryTemplatesSection<'de>`
- reuse `RawGeometry<'de>` for template entries

Suggested shape:

```rust
#[derive(Deserialize)]
struct RawGeometryTemplatesSection<'de> {
    #[serde(default, borrow)]
    templates: Vec<RawGeometry<'de>>,
    #[serde(rename = "vertices-templates", default)]
    vertices_templates: Vec<[f64; 3]>,
}
```

Template import should be a normal generic helper:

```rust
fn import_geometry_templates<'de, SS>(
    raw: RawGeometryTemplatesSection<'de>,
    model: &mut CityModel<u32, SS>,
    resources: &mut GeometryResources,
) -> Result<()>
where
    SS: ParseStringStorage<'de>;
```

The only special rule is semantic: template geometries cannot themselves be `GeometryInstance`.

That validation should remain explicit.

## Geometry

Replace `RawGeometryOwned` and `RawGeometryBorrowed<'a>` with a single `RawGeometry<'de>`.

For all string-bearing fields, use `&'de str`:

- `lod: Option<&'de str>`
- semantic extension names
- theme names in material/texture mappings

All geometry import helpers should become generic over `SS`.

### GeometryInstance

The borrowed path should no longer reject `GeometryInstance`.

Implement it once in the generic pipeline:

- validate template index
- resolve reference point
- parse transform
- insert instance into the model

The current borrowed rejection is a symptom of architecture drift and should be deleted.

## Root Extra and Attribute Handling

All extra-property handling should be unified through `RawAttribute<'de>`.

This includes:

- root extra properties
- metadata extra properties
- city object attributes
- city object extra properties
- contact address if it remains generic attribute data
- semantic attributes

The implementation should avoid all intermediate `serde_json::Value` conversions.

## Module Layout

Reorganize deserialization code into a structure that mirrors the architecture:

```text
src/de/
  mod.rs
  parse.rs
  root.rs
  sections.rs
  build.rs
  attributes.rs
  geometry.rs
  validation.rs
```

### Responsibilities

- `parse.rs`
  - public entry points
  - `ParseStringStorage`
  - `from_str`

- `root.rs`
  - thin root shell
  - root-level parser boundaries

- `sections.rs`
  - typed borrowed structs for section-local parsing

- `build.rs`
  - root model construction
  - metadata, extensions, city objects

- `attributes.rs`
  - `RawAttribute<'de>`
  - conversion to `AttributeValue<SS>`

- `geometry.rs`
  - geometry import
  - appearance import
  - template import
  - semantic/material/texture mapping import

- `validation.rs`
  - narrow format validations and string parsers
  - header parsing
  - enum parsing helpers

Do not separate files by owned-vs-borrowed mode anywhere.

## Error Model

Keep one error type, but tighten the policy for `UnsupportedFeature`.

### Rule

`UnsupportedFeature` should mean a real product decision, not "this path was never implemented".

After the redesign, the following should not use `UnsupportedFeature`:

- appearance import in borrowed mode
- geometry template import in borrowed mode
- `GeometryInstance` in borrowed mode

It is acceptable to keep `UnsupportedFeature` for genuinely unsupported spec features if the product intentionally does not implement them.

## Existing correctness suite is a hard requirement

The current roundtrip suite in [tests/v2_0.rs](/home/balazs/Development/serde_cityjson/tests/v2_0.rs) must remain in the repository and remain meaningful throughout the rewrite.

This suite is not just "some tests". It is the current behavioral contract for:

- full document roundtrip
- fake-complete fixture deserialization
- geometry instance handling
- appearance
- materials
- textures
- geometry templates
- semantics
- metadata
- extensions
- minimal wrapped fragments for individual CityJSON sections

The shared helpers in [tests/common.rs](/home/balazs/Development/serde_cityjson/tests/common.rs) are also part of that contract because they define how roundtrip correctness is asserted today.

### Policy

1. Do not delete `tests/v2_0.rs`.
2. Do not rewrite it into a completely different style just because the architecture changed.
3. Preserve the current fixture coverage and current test intent.
4. Treat passing `tests/v2_0.rs` as the minimum acceptance gate for every migration phase that touches parsing or serialization.

If the API changes aggressively, adapt the helper layer first so that the existing tests can continue to express the same assertions with minimal churn.

## Migration Steps

## Phase 1: Prepare `cityjson-rs`

1. Make all metadata/contact string fields fully `StringStorage`-generic.
2. Add any missing constructors or setters that accept `SS::String` instead of `String`.
3. Ensure appearance, theme names, geometry semantics, and city object types can all be built from `SS::String`.
4. Add or expose any helper APIs needed to build template geometries and geometry instances generically.

Deliverable:

- `cityjson-rs` supports full generic storage for all string-bearing model fields needed by deserialization.

## Phase 2: Introduce new parser entry point

1. Add `ParseStringStorage<'de>`.
2. Add new `from_str<'de, SS>()`.
3. Keep existing `from_str_owned` and `from_str_borrowed` temporarily as wrappers.
4. Wire tests to exercise both wrappers through the generic parser.

Deliverable:

- new generic parser compiles, even before all old code is removed.

## Phase 3: Introduce staged parser boundaries

1. Enable the `serde_json` `raw_value` feature.
2. Create `src/de/root.rs` for the thin root shell.
3. Create `src/de/sections.rs` for typed borrowed section structs.
4. Replace root-level `OwnedJsonValue` and `BorrowedJsonValue` schema-defined fields with `&RawValue` parser boundaries.
5. Establish the allowed parser-boundary list and document it in code comments.

Deliverable:

- all major CityJSON sections are parsed through explicit `&RawValue` boundaries or directly typed fields, with no ad hoc generic JSON blobs.

## Phase 4: Rebuild attribute handling

1. Add `RawAttribute<'de>`.
2. Implement conversion into `AttributeValue<SS>`.
3. Remove `serde_json_borrow` from attribute conversion.
4. Remove any `Box::leak`-based fallback logic.

Deliverable:

- all attribute and extra-property conversion is typed and storage-generic.

## Phase 5: Rebuild root and section import

1. Implement `build_model<'de, SS>`.
2. Add section parsers for metadata and extensions.
3. Port metadata import to generic functions.
4. Port extensions import to generic functions.
5. Add a section parser for `CityObjects`.
6. Port city object import to generic functions.
7. Port relation resolution to a single generic implementation.

Deliverable:

- root, metadata, extensions, city objects, and relations are imported through one generic path.

## Phase 6: Rebuild appearance and template import

1. Implement root-shell `&RawValue` fields for `appearance` and `geometry-templates`.
2. Implement `RawAppearanceSection<'de>` and `RawGeometryTemplatesSection<'de>`.
3. Parse those sections directly from `&RawValue`.
4. Port the owned appearance import logic to generic form.
5. Port the owned template import logic to generic form.
6. Delete borrowed root-section rejection logic.

Deliverable:

- borrowed and owned modes both support appearance and geometry templates through the same implementation.

## Phase 7: Rebuild geometry import

1. Replace `RawGeometryOwned` and `RawGeometryBorrowed<'de>` with `RawGeometry<'de>`.
2. Port all geometry import helpers to generic form.
3. Implement generic `GeometryInstance` import.
4. Keep only truly intentional unsupported cases.

Deliverable:

- a single geometry importer for both storage modes, with no mode-specific raw geometry tree.

## Phase 8: Adapt the existing correctness harness first

1. Refactor `tests/common.rs` so the existing `tests/v2_0.rs` suite can keep running against the new owned-mode entry point.
2. Keep all existing `tests/v2_0.rs` cases green as the baseline.
3. Do not add borrowed-mode assertions until the owned baseline is restored.

Deliverable:

- the current roundtrip suite remains the active regression harness during the rewrite.

## Phase 9: Extend the existing suite to dual-mode verification

1. Add `roundtrip_value_with<'de, SS>()` in `tests/common.rs`.
2. Keep current assertions as owned-mode baseline.
3. Add borrowed-mode mirrors or generic helpers for the same fixture-backed tests.
4. Add owned-vs-borrowed parity assertions using the same fixtures.

Deliverable:

- the existing correctness suite becomes the shared owned/borrowed parity harness instead of being replaced.

## Phase 10: Remove legacy code

Delete:

- raw owned structs
- raw borrowed structs duplicated from owned structs
- owned-only root section import helpers
- borrowed-only rejection helpers that exist only because the implementation was split
- `serde_json_borrow` dependency
- any dead wrappers or compatibility glue

Deliverable:

- no architectural leftovers from the previous design.

## Phase 11: Decide final public API

Choose one of:

### Option A: Strictly clean API

Expose only:

```rust
pub fn from_str<'de, SS>(input: &'de str) -> Result<CityModel<u32, SS>>
where
    SS: ParseStringStorage<'de>;
```

Pros:

- smallest conceptual surface
- forces users to see the actual abstraction

Cons:

- slightly more verbose for callers

### Option B: Clean internal API with ergonomic wrappers

Expose:

- `from_str<'de, SS>`
- `from_str_owned`
- `from_str_borrowed`

Pros:

- clean internals and familiar call sites

Cons:

- public API suggests there are still two separate parsing implementations even when there are not

Recommendation:

- internally design for Option A
- publicly choose Option B only if user ergonomics is still a hard requirement

## Testing Plan

## Test strategy

Every deserialization fixture test should run in both storage modes.

Introduce a generic test harness:

```rust
fn parse_fixture<'de, SS>(input: &'de str) -> Result<CityModel<u32, SS>>
where
    SS: ParseStringStorage<'de>;
```

Then run the same semantic assertions for:

- `OwnedStringStorage`
- `BorrowedStringStorage<'_>`

### How the existing suite factors into the rewrite

The migration should happen in this order:

1. preserve the current suite as the owned-mode baseline
2. update the internal helper functions so they can call the new generic parser without changing the meaning of the tests
3. get the entire existing suite green again
4. only then add borrowed-mode mirrors or parity extensions

In practice, this means:

- `roundtrip_value` in `tests/common.rs` should first be rewritten to use the new owned wrapper or generic parser
- `assert_eq_roundtrip` and `assert_eq_roundtrip_wrapped` should keep their behavior intact
- all existing test names and fixtures in `tests/v2_0.rs` should continue to run as owned-mode regression tests

Only after that baseline is stable should the suite be extended with:

- `roundtrip_value_borrowed`
- borrowed-vs-owned parity assertions
- fixture matrix execution for both storage modes

### Migration harness recommendation

Refactor the shared test helpers into this shape:

```rust
pub fn roundtrip_value_with<'de, SS>(input: &'de Value) -> Value
where
    SS: ParseStringStorage<'de>;

pub fn roundtrip_value_owned(input: &Value) -> Value {
    roundtrip_value_with::<OwnedStringStorage>(input)
}

pub fn roundtrip_value_borrowed(input: &Value) -> Value {
    roundtrip_value_with::<BorrowedStringStorage<'_>>(input)
}
```

Then:

- keep the current `assert_eq_roundtrip` and `assert_eq_roundtrip_wrapped` wired to owned mode first
- add borrowed variants without changing the original contract
- optionally add a storage-parameterized assertion helper once the new parser is stable

This preserves the value of the existing suite while still letting it grow into a dual-mode correctness harness.

## Required test families

1. root structure
   - `type`
   - `version`
   - transform
   - root extra properties

2. metadata
   - all string fields
   - point of contact
   - extra metadata properties

3. extensions
   - names
   - URL/version strings
   - replacement behavior on duplicate names if relevant

4. city objects
   - identifiers
   - type parsing
   - attributes
   - extra properties
   - parent/child resolution

5. appearance
   - materials
   - textures
   - UV coordinates
   - default theme names

6. geometry templates
   - template vertices
   - template geometries
   - geometry instance references

7. geometry semantics/material/texture mappings
   - all currently supported combinations
   - both standard and extension semantic types

8. attributes
   - scalars
   - arrays
   - nested objects
   - strings and keys in both storage modes

9. parity tests
   - parse owned and borrowed from the same fixture
   - serialize both
   - assert equivalent JSON output

   The existing `tests/v2_0.rs` cases should be reused for this rather than replaced. For each existing fixture-backed test, add either:

   - a second borrowed-mode assertion, or
   - a generic helper that executes the same assertion in both modes

   The fixture list already covers the most important areas and should remain the central source of truth.

10. negative tests
   - malformed root
   - unsupported enum values
   - broken indices
   - invalid parent/child references
   - invalid template references

## Fixture policy

Add one comprehensive "full document" fixture that includes:

- metadata
- extensions
- city objects
- semantics
- appearance
- textures
- geometry templates
- geometry instances
- extra properties

This fixture must be used in both owned and borrowed mode.

The existing `cityjson_fake_complete.city.json` fixture already serves much of this role and should be preserved. The redesign should prefer extending current fixtures over replacing them.

## Performance Validation

After correctness is restored, validate performance.

### Benchmarks

Measure:

- parse time owned old vs new
- parse time borrowed old vs new
- allocation count owned old vs new
- allocation count borrowed old vs new

The new architecture may slightly increase code complexity at section boundaries, but it should reduce unnecessary re-parsing and branch duplication. Borrowed mode should improve on appearance/templates because it becomes truly direct.

## Coding Rules for the Rewrite

1. No function pairs that differ only in owned vs borrowed storage.
2. `RawValue` is allowed only at declared parser boundaries.
3. No schema-defined section may be represented as `serde_json::Value`.
4. No JSON subtree may be serialized and reparsed just to fit another helper.
5. No leaking memory to satisfy borrowed lifetimes.
6. No root-level feature rejection unless it is an intentional product limitation.
7. No mode-specific tests for behavior that should be shared.
8. Prefer small pure conversion helpers over stateful deserializer logic.

## Risks

## Risk 1: Boundary drift

If `RawValue` is used opportunistically instead of by rule, the design will become hard to maintain.

Mitigation:

- document the allowed parser boundaries
- forbid nested `RawValue` unless explicitly declared as a child boundary
- keep section parsing fully typed once inside the boundary

## Risk 2: Lifetimes become noisy

Generic storage plus borrowed section structs will increase visible lifetimes.

Mitigation:

- keep lifetimes at module boundaries
- avoid propagating multiple independent lifetimes
- use one input lifetime `'de` consistently

## Risk 3: `cityjson-rs` API friction

If some model constructors still prefer owned strings, generic import code will get ugly.

Mitigation:

- fix `cityjson-rs` first
- do not add hacks in `serde_cityjson` to compensate for incomplete storage-generic model APIs

## Risk 4: Large one-shot rewrite

A big-bang rewrite can destabilize the crate.

Mitigation:

- land behind the new generic entry point first
- run old and new implementations side-by-side temporarily
- delete old code only after parity tests pass

## Recommended Execution Order

1. fix `cityjson-rs` storage-generic metadata/contact APIs
2. add `ParseStringStorage<'de>` and new generic top-level parser
3. enable `RawValue` and add the thin root shell plus typed section parsers
4. adapt `tests/common.rs` so the existing `tests/v2_0.rs` suite can keep running against the new owned-mode entry point
5. keep the existing `tests/v2_0.rs` suite green as the baseline
6. add typed generic attribute conversion
7. port root, metadata, extensions, city objects
8. port appearance and templates
9. port geometry
10. extend the existing suite with borrowed-mode and owned-vs-borrowed parity coverage
11. delete legacy implementation
12. finalize public API break

## Definition of Done

The redesign is complete when all of the following are true:

- there is exactly one deserialization implementation
- owned and borrowed model construction differ only by `ParseStringStorage<'de>`
- `appearance` and `geometry-templates` deserialize in borrowed mode
- `GeometryInstance` deserializes in borrowed mode
- metadata/contact paths do not allocate in borrowed mode except where the target model intentionally requires ownership
- `RawValue` appears only at declared parser boundaries
- no schema-defined section uses `serde_json::Value`
- no memory leaks or subtree re-parsing remain
- the existing `tests/v2_0.rs` suite still exists and passes
- owned and borrowed parity tests pass on the same fixture suite, ideally by extending the existing helpers rather than replacing them
- all obsolete split-path code is deleted

## Recommendation

The cleanest implementation is:

- one thin borrowed root shell
- `&RawValue` only at declared section boundaries
- typed borrowed parsing within each section
- one generic importer
- one tiny string-storage adapter trait
- optional ergonomic wrappers only at the outermost API

Do not try to preserve the old architecture. The split owned/borrowed raw trees and the use of untyped JSON blobs as a convenience mechanism are the root causes of the current complexity and should be removed rather than repaired.
