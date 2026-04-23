# Plan: Fully Configurable, Deterministic, Validated cjfake

## Context

cjfake generates fake CityJSON data for testing. The current implementation has:
- A flat `CJFakeConfig` with ~30 fields, but builder methods ignore their `Option<XBuilder>` params
- Seed support at the top level, but sub-builders (`MaterialBuilder`, `TextureBuilder`) use `rand::rng()` internally, breaking determinism
- No serialization (`build_string()`/`build_vec()` are `todo!()`)
- No cjval validation integration (can't validate without serialization)

Goals: (1) precise configurability, (2) deterministic output with single shared RNG, (3) valid CityJSON output verified by cjval.

---

## Phase 1: Fix Determinism

**Problem**: `MaterialBuilder`, `TextureBuilder` call `rand::rng()` instead of using the builder's `SmallRng`.

### Files to modify:
- `src/material.rs` — Add `rng: &mut R` parameter to `MaterialBuilder::new()` and all property methods (`.name()`, `.diffuse_color()`, etc.). Remove all `rand::rng()` calls.
- `src/texture.rs` — Same treatment: `TextureBuilder::new(rng)`, remove `rand::rng()`.
- `src/attribute.rs` — Already accepts `rng` in `AttributesFaker::generate()` and `AttributesBuilder::with_random_attributes()`. No changes needed.
- `src/metadata.rs` — Already accepts `&mut SmallRng`. No changes needed.

### Approach:
Change builder signatures to accept `&mut SmallRng` (or generic `&mut R: Rng`). The `CityModelBuilder` already owns a `SmallRng` and passes `&mut self.rng` to metadata — do the same for material and texture generation in `citymodel.rs`.

### Verification:
- Existing `seed` test in `tests/api.rs` should produce byte-identical output across runs
- Add a new test that builds twice with same seed and asserts `==` on serialized output (after Phase 2)

---

## Phase 2: Serialization + cjval Validation

### Files to modify:
- `Cargo.toml` — Add `serde_cityjson = { path = "../serde_cityjson", optional = true }` and feature gate `serialize = ["dep:serde_cityjson"]`, default enabled
- `src/citymodel.rs` — Implement `build_string()` and `build_vec()` behind `#[cfg(feature = "serialize")]`
- `src/main.rs` — Call `build_string()` and print to stdout

### Implementation:
```rust
#[cfg(feature = "serialize")]
pub fn build_string(self) -> serde_cityjson::errors::Result<String> {
    let model = self.build();
    serde_cityjson::v2_0::to_string(&model)
}

#[cfg(feature = "serialize")]
pub fn build_vec(self) -> serde_cityjson::errors::Result<Vec<u8>> {
    self.build_string().map(|s| s.into_bytes())
}
```

### Validation tests:
- `tests/validation.rs` — New file. Uses existing `common_lib::validate()` to run cjval on serialized output
- Test: default config, each geometry type individually, hierarchy, templates
- Update `tests/fuzz.rs` — Uncomment the cjval TODO, serialize and validate in the proptest body

### Verification:
- `cargo test` passes with cjval validation
- Fix any cjval failures that surface (likely texture UV count mismatches)

---

## Phase 3: Config Restructuring

### Files to modify:
- `src/cli.rs` — Restructure `CJFakeConfig` into nested sub-configs

### New structure:
```
CJFakeConfig
  seed: Option<u64>
  cityobjects: CityObjectConfig
    allowed_types: Option<Vec<CityObjectType>>
    min/max_cityobjects
    hierarchy: bool
    min/max_children (NEW — currently hardcoded as nr/2)
  geometry: GeometryConfig
    allowed_types: Option<Vec<GeometryType>>
    min/max_members_* (existing MultiPoint, MultiLineString, MultiSurface, Solid, MultiSolid)
    min/max_members_compositesurface (NEW)
    min/max_members_compositesolid (NEW)
    min/max_cityobject_geometries
    allowed_lods: Option<Vec<LoD>> (NEW)
  vertices: VertexConfig
    min/max_coordinate
    min/max_vertices
  materials: MaterialConfig
    enabled: bool (NEW, default true)
    min/max_materials
    nr_themes
    generate_ambient_intensity: Option<bool> (NEW — None=random, Some(true)=always, Some(false)=never)
    generate_diffuse_color: Option<bool> (NEW)
    generate_emissive_color: Option<bool> (NEW)
    generate_specular_color: Option<bool> (NEW)
    generate_shininess: Option<bool> (NEW)
    generate_transparency: Option<bool> (NEW)
  textures: TextureConfig
    enabled: bool (NEW, default true)
    min/max_textures
    nr_themes
    max_vertices_texture
    allow_none: bool
    allowed_image_types: Option<Vec<ImageType>> (NEW)
  templates: TemplateConfig
    enabled: bool (replaces use_templates)
    min/max_templates
  metadata: MetadataConfig
    enabled: bool (NEW, default true)
    geographical_extent: bool (NEW, default true)
    identifier: bool (NEW, default true)
    reference_date: bool (NEW, default true)
    reference_system: bool (NEW, default true)
    title: bool (NEW, default true)
    point_of_contact: bool (NEW, default true)
  attributes: AttributeConfig
    enabled: bool (NEW, default true)
    min/max_attributes (NEW — currently hardcoded 3..=8)
    max_depth: u8 (NEW — currently hardcoded 2)
    random_keys: bool (NEW, default true)
    random_values: bool (NEW, default true)
  semantics: SemanticConfig
    enabled: bool (NEW, default true)
    allowed_types: Option<Vec<SemanticType>> (NEW)
```

CLI stays flat via `#[clap(flatten)]`. All sub-configs implement `Default` matching current behavior.

### Update all consumers:
- `src/citymodel.rs` — Change `self.config.min_materials` to `self.config.materials.min_materials`, etc.
- `src/lib.rs` — `LoDFaker` reads `allowed_lods`, `SemanticTypeFaker` reads `semantics` config
- `src/vertex.rs` — Reads from `config.vertices`
- `tests/api.rs`, `tests/fuzz.rs` — Update config construction

---

## Phase 4: Full Configurability

### 4a. Honor `Option<XBuilder>` parameters
- `src/citymodel.rs` — When `Some(builder)` is passed to `.metadata()`, `.materials()`, `.textures()`, `.attributes()`, use the user-provided builder instead of internal generation
- When `None` is passed, use the sub-config to control generation

### 4b. `enabled` flags
- `materials(None)` checks `config.materials.enabled` — if false, skip entirely (no materials, no themes)
- Same for textures, metadata, attributes, semantics

### 4c. Fine-grained material properties
- In `citymodel.rs` materials generation: replace `rng.random_bool(0.5)` with config lookup:
  - `None` → `rng.random_bool(0.5)` (current behavior)
  - `Some(true)` → always generate
  - `Some(false)` → never generate

### 4d. LoD restriction
- `src/lib.rs` `LoDFaker` — Accept `Option<&[LoD]>`. When `Some`, pick from that list. When `None`, pick from all 20.

### 4e. Semantic type restriction
- `src/lib.rs` `SemanticTypeFaker` — Check `config.semantics.enabled`. When `allowed_types` is `Some`, intersect with valid types for the CityObjectType.

### 4f. CompositeXxx member counts
- `gen_composite_surface` and `gen_composite_solid` — Read from `config.geometry.min/max_members_compositesurface/compositesolid` instead of hardcoded `1..=3`

### 4g. Multiple geometries per CityObject
- The `min/max_members_cityobject_geometries` config exists but only one geometry is generated. Wrap geometry generation in a loop using `get_nr_items()`.

### Reuse existing code:
- `get_nr_items()` in `src/lib.rs:238` for all range-to-count conversions
- `get_cityobject_subtype()` in `src/lib.rs:313` for hierarchy
- `pick_geometry_type()` in `src/citymodel.rs:321` for geometry selection
- `generate_geometry()` in `src/citymodel.rs:363` for dispatching to generators
- `make_semantic_handle()`, `make_ring()`, `make_surface()`, `make_surfaces()` — all existing helpers

---

## Phase 5: Enhanced Testing

### Determinism test (`tests/api.rs` or `tests/validation.rs`):
```rust
#[test]
fn deterministic_output() {
    let config = CJFakeConfig { seed: Some(42), ..Default::default() };
    let json1 = CityModelBuilder::new(config.clone(), Some(42))...build_string().unwrap();
    let json2 = CityModelBuilder::new(config, Some(42))...build_string().unwrap();
    assert_eq!(json1, json2);
}
```

### Per-type cjval validation (`tests/validation.rs`):
- One test per geometry type (7 tests)
- One test with hierarchy enabled
- One test with templates enabled
- One test with all features enabled (materials, textures, semantics, hierarchy, templates)

### Proptest + cjval (`tests/fuzz.rs`):
- Serialize output and call `common_lib::validate()`
- Expand proptest config strategies to cover new config fields

---

## Verification

After all phases:
1. `cargo test` — all existing + new tests pass
2. `cargo run -- --help` — shows all CLI flags organized by domain
3. `cargo run` — prints valid CityJSON to stdout
4. `cargo run | cjval -` — validates with cjval (or pipe to file and validate)
5. `cargo run -- --seed 42` produces identical output on repeated runs
6. `cargo run -- --no-materials --no-textures` (or `--materials-enabled false`) produces valid CityJSON without appearance
