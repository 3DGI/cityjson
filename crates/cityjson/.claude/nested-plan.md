# Nested Backend Implementation Plan

**Goal**: Implement the nested backend API to support the same benchmarks as the default backend, enabling performance comparisons between the two representations.

**Status**: Type definitions complete, API methods needed

**Date**: 2025-11-17

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Backend Architecture Overview](#backend-architecture-overview)
3. [Key Design Differences](#key-design-differences)
4. [Implementation Phases](#implementation-phases)
5. [Phase 1: Foundation](#phase-1-foundation---core-infrastructure)
6. [Phase 2: CityModel Container](#phase-2-citymodel-container)
7. [Phase 3: Geometry Construction](#phase-3-geometry-construction)
8. [Phase 4: Appearance System](#phase-4-appearance-system)
9. [Phase 5: Supporting Types](#phase-5-supporting-types)
10. [Phase 6: Integration and Testing](#phase-6-integration-and-testing)
11. [Phase 7: Feature Flags and Benchmarks](#phase-7-feature-flags-and-benchmarks)
12. [Implementation Priorities](#implementation-priorities)
13. [Success Criteria](#success-criteria)

---

## Executive Summary

The nested backend currently has **complete type definitions** but lacks **method implementations** and **API integration**. To enable benchmarking against the default backend, we need to implement the same API surface while maintaining the nested representation.

**Key Insight**: Much of the v2_0 implementation can be reused or adapted since it's largely backend-agnostic. The main differences are in:

1. **Boundary representation** - Nested structures vs flattened arrays
2. **Semantics/Materials/Textures values** - Inline enums vs resource pool references
3. **Attributes** - Inline `AttributeValue` enum vs flattened `AttributePool` (nested already has correct inline implementation!)
4. **Resource references** - Simple `usize` indices vs `ResourceId32` composite IDs

**Reuse Potential**: ~60% of code can be reused or directly imported from v2_0 and default backend.

**Estimated Implementation Time**: 25-35 hours

---

## Backend Architecture Overview

### Default Backend (`backend/default/`)
- **Boundary**: Flattened representation with separate arrays for vertices, rings, surfaces, shells, solids
- **Resources**: Global resource pools (semantics, materials, textures, geometries) with `ResourceId32` references
- **Attributes**: Flattened Structure of Arrays (`AttributePool`) for columnar storage
- **Performance**: Optimized for cache locality and iteration
- **Memory**: Lower overhead for large models

### Nested Backend (`backend/nested/`)
- **Boundary**: Nested representation matching JSON structure (`Vec<Vec<Vec<...>>>`)
- **Resources**: Inline storage (no global pools), simple `usize` indices
- **Attributes**: Inline `AttributeValue` enum (already implemented correctly!)
- **Performance**: More memory allocations, simpler traversal
- **Memory**: Higher overhead but matches JSON 1:1

### Backend Selection
Configured via feature flags in `Cargo.toml`:
- `backend-default` - Default flattened representation (default)
- `backend-nested` - Alternative nested representation
- `backend-both` - Enable both for comparison

The active backend is re-exported via `cityjson::core`.

---

## Key Design Differences

### 1. Resource Management

| Aspect | Default Backend | Nested Backend |
|--------|----------------|----------------|
| Semantics | Global `SemanticPool<RR>` | Inline in `Geometry.semantics.surfaces: Vec<Semantic<SS>>` |
| Materials | Global `MaterialPool<RR>` | In `Appearance.materials: Vec<Material<SS>>` |
| Textures | Global `TexturePool<RR>` | In `Appearance.textures: Vec<Texture<SS>>` |
| Geometries | Global `GeometryPool<RR>` | Inline in `CityObject.geometry: Vec<Geometry<SS>>` |
| References | `ResourceId32` (pool + index) | `usize` (array index) |

### 2. Boundary Representation

| Geometry Type | Default (Flattened) | Nested |
|---------------|---------------------|--------|
| MultiPoint | `Boundary { vertices: Vec<VR> }` | `Vec<VertexIndex32>` |
| MultiLineString | `Boundary { vertices, rings }` | `Vec<Vec<VertexIndex32>>` |
| Solid | `Boundary { vertices, rings, surfaces, shells }` | `Vec<Vec<Vec<Vec<VertexIndex32>>>>` |

### 3. Semantic Values

| Geometry Type | Default | Nested |
|---------------|---------|--------|
| MultiSurface | `SemanticMap<VR, RR>` with boundary indices | `SemanticValues::PointOrLineStringOrSurface(Vec<Option<usize>>)` |
| Solid | Same map structure | `SemanticValues::Solid(Vec<Vec<Option<usize>>>)` |
| MultiSolid | Same map structure | `SemanticValues::MultiSolid(Vec<Vec<Vec<Option<usize>>>>)` |

### 4. Material and Texture Values

Similar nested enum structure as semantics:
- `MaterialValues::Solid(Vec<Vec<Option<usize>>>)` - per-surface material indices
- `TextureValues::Solid(Vec<Vec<Option<usize>>>)` - per-ring texture indices

Stored in theme-keyed HashMaps:
```rust
materials: Option<HashMap<String, MaterialValues>>
textures: Option<HashMap<String, TextureValues>>
```

---

## Implementation Phases

### Overview

| Phase | Priority | Reuse % | Complexity | Est. Time |
|-------|----------|---------|------------|-----------|
| 1. Foundation | Critical | 95% | Low | 1-2h |
| 2. CityModel | Critical | 40% | High | 6-8h |
| 3. Geometry | Critical | 50% | High | 10-14h |
| 4. Appearance | High | 85% | Low | 3-4h |
| 5. Supporting | Medium | 100% | Low | 1-2h |
| 6. Integration | High | 70% | Medium | 3-4h |
| 7. Benchmarks | High | 80% | Low | 2-3h |

---

## Phase 1: Foundation - Core Infrastructure

**Priority**: Critical (required for any functionality)
**Estimated Time**: 1-2 hours

### 1.1 Vertex Management (`vertex.rs`)

**Status**: TODO (stub file)
**Reuse**: 95% - Direct re-export

**Implementation**:
```rust
// src/backend/nested/vertex.rs
pub use crate::backend::default::vertex::*;
```

**Rationale**: Vertex storage is identical - both backends use vertex pools for cache locality. The `Vertices<VR, C>` container is already generic and backend-agnostic.

### 1.2 Coordinate Types (`coordinate.rs`)

**Status**: TODO (stub file)
**Reuse**: 100% - Direct re-export

**Implementation**:
```rust
// src/backend/nested/coordinate.rs
pub use crate::cityjson::core::coordinate::*;
```

**Types included**:
- `QuantizedCoordinate` - Integer [x, y, z]
- `RealWorldCoordinate` - Float [x, y, z]
- `UVCoordinate` - Texture coordinates [u, v]
- `Coordinate` trait

### 1.3 Transform (`transform.rs`)

**Status**: TODO (stub file)
**Reuse**: 100% - Direct re-export from v2_0

**Implementation**:
```rust
// src/backend/nested/transform.rs
pub use crate::v2_0::transform::*;
```

**Types included**:
- `Transform` struct with `scale` and `translate`
- All getter/setter methods

---

## Phase 2: CityModel Container

**Priority**: Critical (required for builder benchmark)
**Estimated Time**: 6-8 hours

### 2.1 CityModel Core Structure (`citymodel.rs`)

**Status**: Type definition exists, needs full API
**Reuse**: 40% (structure exists, need custom implementations)

#### Existing Type (✓ Complete)
```rust
pub struct CityModel<SS: StringStorage> {
    id: Option<SS::String>,
    type_cm: CityModelType,
    version: Option<CityJSONVersion>,
    transform: Option<Transform>,
    cityobjects: CityObjects<SS>,
    metadata: Option<Metadata<SS>>,
    appearance: Option<Appearance<SS>>,
    geometry_templates: Option<GeometryTemplates<SS>>,
    extra: Option<Attributes<SS>>,
    extensions: Option<Extensions<SS>>,
    vertices: Vertices<u32, QuantizedCoordinate>,
}
```

#### A. Constructor Methods (NEW)

**Required API**:
```rust
impl<SS: StringStorage> CityModel<SS> {
    pub fn new(type_citymodel: CityModelType) -> Self;

    pub fn with_capacity(
        type_citymodel: CityModelType,
        cityobjects_capacity: usize,
        vertex_capacity: usize,
        material_capacity: usize,
        texture_capacity: usize,
    ) -> Self;
}
```

**Implementation Notes**:
- Initialize all `Option` fields as `None`
- Pre-allocate `cityobjects` HashMap with capacity
- Pre-allocate vertex storage
- Pre-allocate appearance vectors (materials/textures)

#### B. Vertex Management (ADAPT from default)

**Required API**:
```rust
// Regular vertices (quantized coordinates)
pub fn add_vertex(&mut self, coordinate: QuantizedCoordinate) -> Result<VertexIndex<u32>>;
pub fn get_vertex(&self, index: VertexIndex<u32>) -> Option<&QuantizedCoordinate>;
pub fn vertices(&self) -> &Vertices<u32, QuantizedCoordinate>;
pub fn vertices_mut(&mut self) -> &mut Vertices<u32, QuantizedCoordinate>;
pub fn clear_vertices(&mut self);

// UV texture coordinates
pub fn add_uv_coordinate(&mut self, uvcoordinate: UVCoordinate) -> Result<VertexIndex<u32>>;
pub fn get_uv_coordinate(&self, index: VertexIndex<u32>) -> Option<&UVCoordinate>;
pub fn vertices_texture(&self) -> &Vertices<u32, UVCoordinate>;
pub fn vertices_texture_mut(&mut self) -> &mut Vertices<u32, UVCoordinate>;

// Template vertices (real-world coordinates)
pub fn add_template_vertex(&mut self, coordinate: RealWorldCoordinate) -> Result<VertexIndex<u32>>;
pub fn get_template_vertex(&self, index: VertexIndex<u32>) -> Option<&RealWorldCoordinate>;
pub fn template_vertices(&self) -> &Vertices<u32, RealWorldCoordinate>;
pub fn template_vertices_mut(&mut self) -> &mut Vertices<u32, RealWorldCoordinate>;
pub fn clear_template_vertices(&mut self);
```

**Implementation Notes**:
- Delegate to `Vertices<VR, C>::add()` method
- No resource pool complexity - direct storage
- Add fields for UV and template vertices if not present:
  ```rust
  vertices_texture: Vertices<u32, UVCoordinate>,
  vertices_template: Vertices<u32, RealWorldCoordinate>,
  ```

#### C. Materials Management (NEW - nested-specific)

**Required API**:
```rust
pub fn add_material(&mut self, material: Material<SS>) -> usize;
pub fn get_material(&self, idx: usize) -> Option<&Material<SS>>;
pub fn get_material_mut(&mut self, idx: usize) -> Option<&mut Material<SS>>;
pub fn find_material(&self, material: &Material<SS>) -> Option<usize>;
pub fn material_count(&self) -> usize;
pub fn iter_materials(&self) -> impl Iterator<Item = (usize, &Material<SS>)>;
pub fn iter_materials_mut(&mut self) -> impl Iterator<Item = (usize, &mut Material<SS>)>;
pub fn default_theme_material(&self) -> Option<&SS::String>;
pub fn set_default_theme_material(&mut self, theme: Option<SS::String>);
```

**Implementation Strategy**:
- Materials stored in `self.appearance.materials: Vec<Material<SS>>`
- Return index (position in vector) instead of `ResourceId32`
- `find_material()`: Linear search with `PartialEq` (for deduplication)
- Auto-initialize `appearance` if `None` when first material added

**Key Difference from Default**:
- No resource pool - simple vector storage
- References are `usize` indices, not `ResourceId32`

#### D. Textures Management (NEW - nested-specific)

**Required API**: Same pattern as materials
```rust
pub fn add_texture(&mut self, texture: Texture<SS>) -> usize;
pub fn get_texture(&self, idx: usize) -> Option<&Texture<SS>>;
pub fn get_texture_mut(&mut self, idx: usize) -> Option<&mut Texture<SS>>;
pub fn find_texture(&self, texture: &Texture<SS>) -> Option<usize>;
pub fn texture_count(&self) -> usize;
pub fn iter_textures(&self) -> impl Iterator<Item = (usize, &Texture<SS>)>;
pub fn default_theme_texture(&self) -> Option<&SS::String>;
pub fn set_default_theme_texture(&mut self, theme: Option<SS::String>);
```

**Implementation**: Same as materials

#### E. Semantics Management (SPECIAL - nested-specific)

**Challenge**: Semantics are NOT stored globally in nested backend. They live inside `Geometry.semantics.surfaces: Vec<Semantic<SS>>`.

**Options**:

**Option 1**: No global semantic methods (minimal API)
- GeometryBuilder directly adds semantics to geometry
- No deduplication at model level

**Option 2**: Helper methods for search (recommended)
```rust
// Search all geometries in all city objects for a matching semantic
pub fn find_semantic_in_model(&self, semantic: &Semantic<SS>) -> Option<(String, usize, usize)>;
// Returns: (cityobject_id, geometry_idx, semantic_idx)
```

**Recommendation**: Start with Option 1 for MVP, add Option 2 if deduplication needed.

#### F. Geometries Management (NEW - nested-specific)

**Challenge**: Geometries stored inline in `CityObject.geometry: Vec<Geometry<SS>>`, not in a global pool.

**Required API**:
```rust
// Helper to add geometry to a specific city object
pub fn add_geometry_to_cityobject(
    &mut self,
    cityobject_id: &str,
    geometry: Geometry<SS>,
) -> Result<usize>;

pub fn get_geometry_from_cityobject(
    &self,
    cityobject_id: &str,
    geometry_idx: usize,
) -> Option<&Geometry<SS>>;

// Template geometries (stored globally)
pub fn add_template_geometry(&mut self, geometry: Geometry<SS>) -> usize;
pub fn get_template_geometry(&self, idx: usize) -> Option<&Geometry<SS>>;
pub fn get_template_geometry_mut(&mut self, idx: usize) -> Option<&mut Geometry<SS>>;
pub fn template_geometry_count(&self) -> usize;
```

**Implementation Notes**:
- Regular geometries: delegate to `cityobjects.get_mut(id)?.geometry_mut().push(geometry)`
- Template geometries: stored in `geometry_templates.templates: Vec<Geometry<SS>>`
- Return `usize` index

#### G. CityObjects Management (ADAPT structure)

**Required API**:
```rust
pub fn cityobjects(&self) -> &HashMap<String, CityObject<SS>>;
pub fn cityobjects_mut(&mut self) -> &mut HashMap<String, CityObject<SS>>;
pub fn add_cityobject(&mut self, id: String, cityobject: CityObject<SS>);
pub fn get_cityobject(&self, id: &str) -> Option<&CityObject<SS>>;
pub fn get_cityobject_mut(&mut self, id: &str) -> Option<&mut CityObject<SS>>;
pub fn clear_cityobjects(&mut self);
```

**Note**: The nested backend uses `HashMap<String, CityObject<SS>>` directly, not a custom `CityObjects` container with resource IDs.

#### H. Metadata, Extensions, Extra (REUSE)

**Required API**:
```rust
// Metadata
pub fn metadata(&self) -> Option<&Metadata<SS>>;
pub fn metadata_mut(&mut self) -> &mut Metadata<SS>; // Auto-initialize

// Extensions
pub fn extensions(&self) -> Option<&Extensions<SS>>;
pub fn extensions_mut(&mut self) -> &mut Extensions<SS>; // Auto-initialize

// Extra attributes
pub fn extra(&self) -> Option<&Attributes<SS>>;
pub fn extra_mut(&mut self) -> &mut Attributes<SS>; // Auto-initialize

// Transform
pub fn transform(&self) -> Option<&Transform>;
pub fn transform_mut(&mut self) -> &mut Transform; // Auto-initialize

// Model metadata
pub fn type_citymodel(&self) -> &CityModelType;
pub fn version(&self) -> Option<CityJSONVersion>; // Return V2_0
pub fn id(&self) -> Option<&SS::String>;
pub fn set_id(&mut self, id: Option<SS::String>);
```

**Implementation**: Standard Option getter/mutator pattern

---

### 2.2 CityObject Management (`cityobject.rs`)

**Status**: Type definition exists, needs methods
**Reuse**: 70% from v2_0

#### Existing Type (✓ Complete)
```rust
pub struct CityObject<SS: StringStorage> {
    type_co: CityObjectType<SS>,
    geometry: Option<Vec<Geometry<SS>>>,
    attributes: Option<Attributes<SS>>,
    geographical_extent: Option<BBox>,
    children: Option<Vec<String>>,
    parents: Option<Vec<String>>,
    extra: Option<Attributes<SS>>,
}
```

#### Required API

```rust
impl<SS: StringStorage> CityObject<SS> {
    // Constructor
    pub fn new(type_co: CityObjectType<SS>) -> Self;

    // Getters
    pub fn type_cityobject(&self) -> &CityObjectType<SS>;
    pub fn geometry(&self) -> Option<&Vec<Geometry<SS>>>;
    pub fn attributes(&self) -> Option<&Attributes<SS>>;
    pub fn geographical_extent(&self) -> Option<&BBox>;
    pub fn children(&self) -> Option<&Vec<String>>;
    pub fn parents(&self) -> Option<&Vec<String>>;
    pub fn extra(&self) -> Option<&Attributes<SS>>;

    // Mutators (auto-initialize Options)
    pub fn geometry_mut(&mut self) -> &mut Vec<Geometry<SS>>;
    pub fn attributes_mut(&mut self) -> &mut Attributes<SS>;
    pub fn children_mut(&mut self) -> &mut Vec<String>;
    pub fn parents_mut(&mut self) -> &mut Vec<String>;
    pub fn extra_mut(&mut self) -> &mut Attributes<SS>;
    pub fn set_geographical_extent(&mut self, bbox: Option<BBox>);
}
```

**Implementation Notes**:
- Mutators should auto-initialize `None` → `Some(default_value)`
- Can copy logic from `v2_0/cityobject.rs` almost directly
- Only difference: `Vec<Geometry<SS>>` instead of `Vec<RR>`

#### CityObjectType (REUSE)

**Implementation**:
```rust
pub use crate::v2_0::cityobject::CityObjectType;
```

CityObjectType is already generic over `SS: StringStorage` and backend-agnostic.

---

## Phase 3: Geometry Construction

**Priority**: Critical (required for builder benchmark)
**Estimated Time**: 10-14 hours

### 3.1 Geometry Type (`geometry.rs`)

**Status**: Type definition exists, needs methods
**Reuse**: Structure exists, add ~15 methods

#### Existing Type (✓ Complete)
```rust
pub struct Geometry<SS: StringStorage> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary>,
    semantics: Option<Semantics<SS>>,
    materials: Option<HashMap<String, MaterialValues>>,
    textures: Option<HashMap<String, TextureValues>>,
    instance_template: Option<usize>,
    instance_reference_point: Option<RealWorldCoordinate>,
    instance_transformation_matrix: Option<[f64; 16]>,
}
```

#### Required API

```rust
impl<SS: StringStorage> Geometry<SS> {
    // Constructor
    pub fn new(
        type_geometry: GeometryType,
        lod: Option<LoD>,
        boundaries: Option<Boundary>,
        semantics: Option<Semantics<SS>>,
        materials: Option<HashMap<String, MaterialValues>>,
        textures: Option<HashMap<String, TextureValues>>,
        instance_template: Option<usize>,
        instance_reference_point: Option<RealWorldCoordinate>,
        instance_transformation_matrix: Option<[f64; 16]>,
    ) -> Self;

    // Getters
    pub fn type_geometry(&self) -> &GeometryType;
    pub fn lod(&self) -> Option<&LoD>;
    pub fn boundaries(&self) -> Option<&Boundary>;
    pub fn semantics(&self) -> Option<&Semantics<SS>>;
    pub fn materials(&self) -> Option<&HashMap<String, MaterialValues>>;
    pub fn textures(&self) -> Option<&HashMap<String, TextureValues>>;
    pub fn instance_template(&self) -> Option<usize>;
    pub fn instance_reference_point(&self) -> Option<&RealWorldCoordinate>;
    pub fn instance_transformation_matrix(&self) -> Option<&[f64; 16]>;
}
```

**Implementation**: Straightforward field accessors

---

### 3.2 GeometryBuilder (CRITICAL - Most Complex)

**Status**: Does not exist
**Reuse**: ~60% logic from `backend/default/geometry.rs`
**Complexity**: HIGH

This is the most critical and complex component for benchmarks.

#### Design Approach

**Strategy**: Adapt the default backend's GeometryBuilder, modifying only the `build()` method to construct nested structures instead of flattened.

**Reusable Components** (from default):
- Builder state tracking (vertices, rings, surfaces, shells, solids)
- Vertex accumulation logic
- Ring/surface/shell construction logic
- Semantic/material/texture tracking
- UV coordinate management
- Validation logic

**Need to Modify**:
- Boundary construction (nested vs flattened)
- Semantic values construction (nested enum)
- Material/texture values construction (nested enum)

#### Required API

```rust
pub struct GeometryBuilder<'a, SS: StringStorage> {
    model: &'a mut CityModel<SS>,
    type_geometry: GeometryType,
    builder_mode: BuilderMode,
    lod: Option<LoD>,
    // ... internal state (copy from default backend)
}

impl<'a, SS: StringStorage> GeometryBuilder<'a, SS> {
    // Constructor
    pub fn new(
        model: &'a mut CityModel<SS>,
        type_geometry: GeometryType,
        builder_mode: BuilderMode,
    ) -> Self;

    // Configuration
    pub fn with_lod(self, lod: LoD) -> Self;
    pub fn with_template(self, template_idx: usize) -> Result<Self>;
    pub fn with_transformation_matrix(self, matrix: [f64; 16]) -> Self;

    // Vertex operations
    pub fn add_vertex(&mut self, index: VertexIndex<u32>) -> Result<&mut Self>;
    pub fn add_point(&mut self, coordinate: QuantizedCoordinate) -> Result<&mut Self>;
    pub fn add_template_vertex(&mut self, index: VertexIndex<u32>) -> Result<&mut Self>;
    pub fn add_template_point(&mut self, coordinate: RealWorldCoordinate) -> Result<&mut Self>;

    // Ring operations
    pub fn add_ring(&mut self, vertex_indices: &[usize]) -> Result<usize>;
    pub fn start_ring(&mut self) -> Result<&mut Self>;
    pub fn end_ring(&mut self) -> Result<usize>;

    // Surface operations
    pub fn start_surface(&mut self) -> Result<&mut Self>;
    pub fn add_surface_outer_ring(&mut self, ring_idx: usize) -> Result<&mut Self>;
    pub fn add_surface_inner_ring(&mut self, ring_idx: usize) -> Result<&mut Self>;
    pub fn end_surface(&mut self) -> Result<usize>;

    // Shell operations
    pub fn start_shell(&mut self) -> Result<&mut Self>;
    pub fn add_shell_surface(&mut self, surface_idx: usize) -> Result<&mut Self>;
    pub fn end_shell(&mut self) -> Result<usize>;

    // Solid operations
    pub fn start_solid(&mut self) -> Result<&mut Self>;
    pub fn add_solid_outer_shell(&mut self, shell_idx: usize) -> Result<&mut Self>;
    pub fn add_solid_inner_shell(&mut self, shell_idx: usize) -> Result<&mut Self>;
    pub fn end_solid(&mut self) -> Result<usize>;

    // Semantics
    pub fn set_semantic_point(&mut self, point_idx: usize, semantic: Semantic<SS>) -> Result<&mut Self>;
    pub fn set_semantic_linestring(&mut self, linestring_idx: usize, semantic: Semantic<SS>) -> Result<&mut Self>;
    pub fn set_semantic_surface(&mut self, surface_idx: usize, semantic: Semantic<SS>, is_roof: bool) -> Result<&mut Self>;

    // Materials
    pub fn set_material_surface(&mut self, theme: String, surface_idx: usize, material_idx: usize) -> Result<&mut Self>;

    // Textures
    pub fn set_texture_ring(&mut self, theme: String, ring_idx: usize, texture_idx: usize) -> Result<&mut Self>;
    pub fn add_uv_to_vertex(&mut self, vertex_idx: usize, uv: UVCoordinate) -> Result<&mut Self>;

    // Build
    pub fn build(self) -> Result<Geometry<SS>>;
}
```

#### Implementation Strategy

**Step 1**: Copy builder state from default backend
```rust
pub struct GeometryBuilder<'a, SS: StringStorage> {
    model: &'a mut CityModel<SS>,
    type_geometry: GeometryType,
    builder_mode: BuilderMode,
    lod: Option<LoD>,
    template_geometry: Option<usize>,
    transformation_matrix: Option<[f64; 16]>,

    // Vertex tracking (copy from default)
    vertices: Vec<VertexOrPoint>,
    template_vertices: Vec<TemplateVertexOrPoint>,

    // Boundary construction (copy from default)
    rings: Vec<Vec<usize>>,           // indices into vertices
    surfaces: Vec<SurfaceInProgress>, // surfaces with their rings
    shells: Vec<Vec<usize>>,          // shells with their surfaces
    solids: Vec<SolidInProgress>,     // solids with their shells

    // Active element tracking (copy from default)
    active_surface: Option<usize>,
    active_solid: Option<usize>,

    // Semantic storage (copy from default)
    point_semantics: HashMap<usize, Semantic<SS>>,
    linestring_semantics: HashMap<usize, Semantic<SS>>,
    surface_semantics: HashMap<usize, Semantic<SS>>,

    // Material storage (copy from default)
    surface_materials: Vec<(String, Vec<(usize, usize)>)>, // theme -> [(surface_idx, material_idx)]

    // Texture storage (copy from default)
    ring_textures: Vec<(String, Vec<(usize, usize)>)>, // theme -> [(ring_idx, texture_idx)]

    // UV coordinates (copy from default)
    uv_coordinates: Vec<UVCoordinate>,
    vertex_uv_mapping: HashMap<usize, usize>,
}
```

**Step 2**: Copy all methods from default backend EXCEPT `build()`

Most methods can be copied verbatim:
- `new()`, `with_lod()`, `with_template()`, etc.
- `add_vertex()`, `add_ring()`, `start_surface()`, etc.
- `set_semantic_*()`, `set_material_*()`, `set_texture_*()`, etc.

**Step 3**: Implement custom `build()` method

This is where nested-specific logic goes:

```rust
pub fn build(self) -> Result<Geometry<SS>> {
    // 1. Build nested boundaries
    let boundaries = match self.type_geometry {
        GeometryType::MultiPoint => Some(Boundary::MultiPoint(
            self.vertices.into_iter()
                .map(|v| self.resolve_vertex(v))
                .collect::<Result<Vec<_>>>()?
        )),

        GeometryType::MultiLineString => Some(Boundary::MultiLineString(
            self.rings.into_iter()
                .map(|ring| {
                    ring.into_iter()
                        .map(|idx| self.resolve_vertex_by_idx(idx))
                        .collect::<Result<Vec<_>>>()
                })
                .collect::<Result<Vec<_>>>()?
        )),

        GeometryType::Solid => {
            // Build nested structure: Vec<Vec<Vec<Vec<VertexIndex>>>>
            let mut solid_boundaries = Vec::new();
            for shell_surfaces in &self.shells {
                let mut shell = Vec::new();
                for &surface_idx in shell_surfaces {
                    let surface = &self.surfaces[surface_idx];
                    let mut surface_rings = Vec::new();

                    // Outer ring
                    if let Some(outer_idx) = surface.outer_ring {
                        surface_rings.push(self.build_ring(outer_idx)?);
                    }

                    // Inner rings
                    for &inner_idx in &surface.inner_rings {
                        surface_rings.push(self.build_ring(inner_idx)?);
                    }

                    shell.push(surface_rings);
                }
                solid_boundaries.push(shell);
            }
            Some(Boundary::Solid(solid_boundaries))
        },

        // Similar for other types...
    };

    // 2. Build nested semantics
    let semantics = if !self.surface_semantics.is_empty() {
        Some(self.build_nested_semantics()?)
    } else {
        None
    };

    // 3. Build nested materials
    let materials = if !self.surface_materials.is_empty() {
        Some(self.build_nested_materials()?)
    } else {
        None
    };

    // 4. Build nested textures
    let textures = if !self.ring_textures.is_empty() {
        Some(self.build_nested_textures()?)
    } else {
        None
    };

    // 5. Construct geometry
    Ok(Geometry::new(
        self.type_geometry,
        self.lod,
        boundaries,
        semantics,
        materials,
        textures,
        self.template_geometry,
        self.transformation_matrix.map(|_| /* compute reference point */),
        self.transformation_matrix,
    ))
}
```

**Step 4**: Helper methods for nested structure construction

```rust
impl<'a, SS: StringStorage> GeometryBuilder<'a, SS> {
    fn build_nested_semantics(&self) -> Result<Semantics<SS>> {
        // Collect all unique semantics
        let mut surfaces = Vec::new();
        let mut semantic_index_map = HashMap::new();

        for (_, semantic) in &self.surface_semantics {
            if !semantic_index_map.contains_key(semantic) {
                let idx = surfaces.len();
                surfaces.push(semantic.clone());
                semantic_index_map.insert(semantic, idx);
            }
        }

        // Build SemanticValues based on geometry type
        let values = match self.type_geometry {
            GeometryType::Solid => {
                let mut solid_values = Vec::new();
                for shell_surfaces in &self.shells {
                    let mut shell_values = Vec::new();
                    for &surface_idx in shell_surfaces {
                        let semantic_idx = self.surface_semantics.get(&surface_idx)
                            .and_then(|s| semantic_index_map.get(s))
                            .copied();
                        shell_values.push(semantic_idx);
                    }
                    solid_values.push(shell_values);
                }
                SemanticValues::Solid(solid_values)
            },
            // Similar for other types...
        };

        Ok(Semantics { surfaces, values })
    }

    fn build_nested_materials(&self) -> Result<HashMap<String, MaterialValues>> {
        let mut result = HashMap::new();

        for (theme, mappings) in &self.surface_materials {
            let values = match self.type_geometry {
                GeometryType::Solid => {
                    let mut solid_values = Vec::new();
                    for shell_surfaces in &self.shells {
                        let mut shell_values = Vec::new();
                        for &surface_idx in shell_surfaces {
                            let material_idx = mappings.iter()
                                .find(|(idx, _)| *idx == surface_idx)
                                .map(|(_, mat_idx)| *mat_idx);
                            shell_values.push(material_idx);
                        }
                        solid_values.push(shell_values);
                    }
                    MaterialValues::Solid(solid_values)
                },
                // Similar for other types...
            };
            result.insert(theme.clone(), values);
        }

        Ok(result)
    }

    fn build_nested_textures(&self) -> Result<HashMap<String, TextureValues>> {
        // Similar to materials but operates on rings
        // ...
    }
}
```

**Testing Strategy**:
1. Start with simple MultiPoint geometry
2. Add MultiLineString
3. Add Solid (most complex)
4. Add semantics
5. Add materials and textures

---

### 3.3 Semantics (`semantics.rs`)

**Status**: Types exist (private), needs to be made public and add methods
**Reuse**: 80% from v2_0

#### Current Status
- Module is private: `mod semantics;`
- Types are complete and correct

#### Required Changes

**Step 1**: Make module public
```rust
// In src/backend/nested.rs
pub mod semantics; // Change from `mod semantics`
```

**Step 2**: Add methods to `Semantic<SS>` (copy from v2_0)

```rust
impl<SS: StringStorage> Semantic<SS> {
    pub fn new(type_semantic: SemanticType<SS>) -> Self;

    pub fn type_semantic(&self) -> &SemanticType<SS>;

    pub fn children(&self) -> Option<&Vec<usize>>;
    pub fn children_mut(&mut self) -> &mut Vec<usize>; // Auto-initialize
    pub fn has_children(&self) -> bool;

    pub fn parent(&self) -> Option<usize>;
    pub fn set_parent(&mut self, idx: usize);
    pub fn has_parent(&self) -> bool;

    pub fn attributes(&self) -> Option<&Attributes<SS>>;
    pub fn attributes_mut(&mut self) -> &mut Attributes<SS>; // Auto-initialize
}
```

**Step 3**: Methods for `Semantics<SS>` container (NEW)

```rust
impl<SS: StringStorage> Semantics<SS> {
    pub fn new(surfaces: Vec<Semantic<SS>>, values: SemanticValues) -> Self;

    pub fn surfaces(&self) -> &Vec<Semantic<SS>>;
    pub fn surfaces_mut(&mut self) -> &mut Vec<Semantic<SS>>;

    pub fn values(&self) -> &SemanticValues;
}
```

#### SemanticType (REUSE)

```rust
pub use crate::v2_0::geometry::semantic::SemanticType;
```

---

### 3.4 Boundary Utilities (`boundary.rs`)

**Status**: Types exist, need utility methods
**Reuse**: 30% (types complete, add conversions)

#### Required Additions

**Validation methods**:
```rust
impl Boundary {
    pub fn validate(&self) -> Result<()>;
    pub fn check_type(&self) -> BoundaryType;
}
```

**Conversion utilities** (for testing/debugging):
```rust
// Convert nested to flattened (already partially exists in default backend)
impl From<Boundary> for crate::backend::default::boundary::Boundary<u32> {
    fn from(nested: Boundary) -> Self {
        // Implementation...
    }
}

// Convert flattened to nested (NEW)
impl From<crate::backend::default::boundary::Boundary<u32>> for Boundary {
    fn from(flattened: crate::backend::default::boundary::Boundary<u32>) -> Self {
        // Implementation...
    }
}
```

---

## Phase 4: Appearance System

**Priority**: High (required for builder benchmark materials/textures)
**Estimated Time**: 3-4 hours

### 4.1 Material (`appearance.rs`)

**Status**: Type exists, needs methods
**Reuse**: 100% from v2_0

#### Implementation

Copy all methods from `v2_0/appearance/material.rs`:

```rust
impl<SS: StringStorage> Material<SS> {
    pub fn new(name: SS::String) -> Self;

    // Getters
    pub fn name(&self) -> &SS::String;
    pub fn ambient_intensity(&self) -> Option<f32>;
    pub fn diffuse_color(&self) -> Option<&RGB>;
    pub fn emissive_color(&self) -> Option<&RGB>;
    pub fn specular_color(&self) -> Option<&RGB>;
    pub fn shininess(&self) -> Option<f32>;
    pub fn transparency(&self) -> Option<f32>;
    pub fn is_smooth(&self) -> Option<bool>;

    // Setters
    pub fn set_name(&mut self, name: SS::String);
    pub fn set_ambient_intensity(&mut self, value: Option<f32>);
    pub fn set_diffuse_color(&mut self, color: Option<RGB>);
    pub fn set_emissive_color(&mut self, color: Option<RGB>);
    pub fn set_specular_color(&mut self, color: Option<RGB>);
    pub fn set_shininess(&mut self, value: Option<f32>);
    pub fn set_transparency(&mut self, value: Option<f32>);
    pub fn set_is_smooth(&mut self, value: Option<bool>);
}
```

**Implementation**: Straightforward field getters/setters

---

### 4.2 Texture (`appearance.rs`)

**Status**: Type exists, needs methods
**Reuse**: 100% from v2_0

#### Implementation

Copy all methods from `v2_0/appearance/texture.rs`:

```rust
impl<SS: StringStorage> Texture<SS> {
    pub fn new(image: SS::String, image_type: ImageType) -> Self;

    // Getters
    pub fn image(&self) -> &SS::String;
    pub fn image_type(&self) -> &ImageType;
    pub fn wrap_mode(&self) -> Option<WrapMode>;
    pub fn texture_type(&self) -> Option<TextureType>;
    pub fn border_color(&self) -> Option<RGBA>;

    // Setters
    pub fn set_image(&mut self, image: SS::String);
    pub fn set_image_type(&mut self, image_type: ImageType);
    pub fn set_wrap_mode(&mut self, mode: Option<WrapMode>);
    pub fn set_texture_type(&mut self, texture_type: Option<TextureType>);
    pub fn set_border_color(&mut self, color: Option<RGBA>);
}
```

---

### 4.3 Appearance Container (`appearance.rs`)

**Status**: Type exists, needs methods
**Reuse**: 80%

#### Required API

```rust
impl<SS: StringStorage> Appearance<SS> {
    pub fn new() -> Self;

    // Materials (note: indices managed by CityModel, not Appearance)
    pub fn materials(&self) -> Option<&Vec<Material<SS>>>;
    pub fn materials_mut(&mut self) -> &mut Vec<Material<SS>>; // Auto-initialize

    // Textures
    pub fn textures(&self) -> Option<&Vec<Texture<SS>>>;
    pub fn textures_mut(&mut self) -> &mut Vec<Texture<SS>>; // Auto-initialize

    // Texture vertices
    pub fn vertices_texture(&self) -> Option<&VerticesTexture>;
    pub fn vertices_texture_mut(&mut self) -> &mut VerticesTexture; // Auto-initialize

    // Default themes
    pub fn default_theme_material(&self) -> Option<&SS::String>;
    pub fn set_default_theme_material(&mut self, theme: Option<SS::String>);

    pub fn default_theme_texture(&self) -> Option<&SS::String>;
    pub fn set_default_theme_texture(&mut self, theme: Option<SS::String>);
}
```

**Implementation**: Standard Option field access with auto-initialization

---

## Phase 5: Supporting Types

**Priority**: Medium (needed for completeness)
**Estimated Time**: 1-2 hours

### 5.1 Metadata (`metadata.rs`)

**Status**: TODO (stub file)
**Reuse**: 100% from v2_0

**Implementation**:
```rust
pub use crate::v2_0::metadata::*;
```

**Types included**:
- `Metadata<SS>` - Dataset metadata
- `Contact` - Point of contact
- `ContactRole`, `ContactType` - Enums
- `CRS` - Coordinate reference system
- `Date` - Reference date
- All getter/setter methods

---

### 5.2 Extension (`extension.rs`)

**Status**: TODO (stub file)
**Reuse**: 100% from v2_0

**Implementation**:
```rust
pub use crate::v2_0::extension::*;
```

**Types included**:
- `Extension` - Extension definition
- `Extensions` - Extension container
- All methods

---

### 5.3 Geometry Structs (`geometry_struct.rs`)

**Status**: TODO (stub file)
**Action**: Investigate what default backend has here

**Likely contents**:
- `GeometryType` enum (MultiPoint, MultiLineString, Solid, etc.)
- `LoD` enum (Level of Detail)
- `BBox` struct (Bounding box)
- Related types

**Implementation approach**:
1. Check `backend/default/geometry_struct.rs`
2. If types are backend-agnostic, re-export from there or from `cityjson::core`
3. If nested-specific, implement analogous types

---

## Phase 6: Integration and Testing

**Priority**: High (required for benchmarks)
**Estimated Time**: 3-4 hours

### 6.1 Update Module Exports (`src/backend/nested.rs`)

**Changes**:
```rust
pub mod appearance;
pub mod attributes;
pub mod boundary;
pub mod citymodel;
pub mod cityobject;
pub mod coordinate;
pub mod extension;
pub mod geometry;
pub mod geometry_struct;
pub mod metadata;
pub mod semantics;  // ← Change from `mod` to `pub mod`
pub mod transform;
pub mod vertex;

// Re-export key types for convenience
pub use appearance::{Appearance, Material, Texture};
pub use attributes::{Attributes, AttributeValue, OwnedAttributes, BorrowedAttributes};
pub use boundary::Boundary;
pub use citymodel::CityModel;
pub use cityobject::{CityObject, CityObjectType};
pub use coordinate::*;
pub use extension::{Extension, Extensions};
pub use geometry::{Geometry, GeometryBuilder};
pub use metadata::Metadata;
pub use semantics::{Semantic, SemanticType, Semantics};
pub use transform::Transform;
pub use vertex::Vertices;
```

---

### 6.2 Create Type Aliases for Common Usage

**Location**: Add to `src/backend/nested/mod.rs` or create `src/backend/nested/aliases.rs`

```rust
use crate::resources::storage::{OwnedStringStorage, BorrowedStringStorage};

// Owned versions (most common)
pub type OwnedCityModel = CityModel<OwnedStringStorage>;
pub type OwnedCityObject = CityObject<OwnedStringStorage>;
pub type OwnedGeometry = Geometry<OwnedStringStorage>;
pub type OwnedSemantic = Semantic<OwnedStringStorage>;
pub type OwnedMaterial = Material<OwnedStringStorage>;
pub type OwnedTexture = Texture<OwnedStringStorage>;

// Borrowed versions
pub type BorrowedCityModel<'a> = CityModel<BorrowedStringStorage<'a>>;
pub type BorrowedCityObject<'a> = CityObject<BorrowedStringStorage<'a>>;
pub type BorrowedGeometry<'a> = Geometry<BorrowedStringStorage<'a>>;
```

---

### 6.3 Unit Tests

Create comprehensive unit tests for each module:

**Example: `src/backend/nested/tests/citymodel_tests.rs`**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_citymodel_new() {
        let model = OwnedCityModel::new(CityModelType::CityJSON);
        assert_eq!(model.type_citymodel(), &CityModelType::CityJSON);
    }

    #[test]
    fn test_add_vertex() {
        let mut model = OwnedCityModel::new(CityModelType::CityJSON);
        let coord = QuantizedCoordinate::new(100, 200, 300);
        let idx = model.add_vertex(coord).unwrap();
        assert_eq!(model.get_vertex(idx), Some(&coord));
    }

    #[test]
    fn test_add_material() {
        let mut model = OwnedCityModel::new(CityModelType::CityJSON);
        let material = Material::new("concrete".to_string());
        let idx = model.add_material(material.clone());
        assert_eq!(model.get_material(idx), Some(&material));
    }

    // More tests...
}
```

**Test Coverage Goals**:
- CityModel: vertex management, materials, textures
- CityObject: construction, attributes
- Geometry: construction, getters
- GeometryBuilder: simple geometries, complex geometries, semantics, materials, textures
- Semantics: construction, relationships
- Boundary: validation, conversions

---

## Phase 7: Feature Flags and Benchmarks

**Priority**: High (goal of this project)
**Estimated Time**: 2-3 hours

### 7.1 Verify Feature Flags (`Cargo.toml`)

**Ensure correct configuration**:
```toml
[features]
default = ["backend-default"]
backend-default = []
backend-nested = []
backend-both = ["backend-default", "backend-nested"]
```

**Verify in code** (`src/cityjson/core.rs`):
```rust
// Default backend takes priority when both are enabled
#[cfg(feature = "backend-default")]
pub use crate::backend::default::*;

// Only use nested backend if default is not enabled
#[cfg(all(feature = "backend-nested", not(feature = "backend-default")))]
pub use crate::backend::nested::*;
```

---

### 7.2 Modify Benchmarks for Dual-Backend Testing

#### Option A: Feature-Gated Benchmark Groups (Recommended)

**File**: `benches/builder.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

#[cfg(feature = "backend-default")]
mod default_benches {
    use super::*;
    use cityjson::backend::default::*;
    use cityjson::cityjson::core::attributes::*;

    pub fn bench_build_with_geometries_default(c: &mut Criterion) {
        c.bench_function("default/build_10k_with_geometries", |b| {
            b.iter(|| {
                // Existing benchmark code
            });
        });
    }

    pub fn bench_build_without_geometries_default(c: &mut Criterion) {
        c.bench_function("default/build_10k_without_geometries", |b| {
            b.iter(|| {
                // Existing benchmark code
            });
        });
    }
}

#[cfg(feature = "backend-nested")]
mod nested_benches {
    use super::*;
    use cityjson::backend::nested::*;

    pub fn bench_build_with_geometries_nested(c: &mut Criterion) {
        c.bench_function("nested/build_10k_with_geometries", |b| {
            b.iter(|| {
                // Same benchmark code, different types
            });
        });
    }

    pub fn bench_build_without_geometries_nested(c: &mut Criterion) {
        c.bench_function("nested/build_10k_without_geometries", |b| {
            b.iter(|| {
                // Same benchmark code, different types
            });
        });
    }
}

criterion_group!(
    benches,
    #[cfg(feature = "backend-default")]
    default_benches::bench_build_with_geometries_default,
    #[cfg(feature = "backend-default")]
    default_benches::bench_build_without_geometries_default,
    #[cfg(feature = "backend-nested")]
    nested_benches::bench_build_with_geometries_nested,
    #[cfg(feature = "backend-nested")]
    nested_benches::bench_build_without_geometries_nested,
);

criterion_main!(benches);
```

#### Option B: Parameterized Benchmarks (More Complex)

```rust
fn bench_builder_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder");

    #[cfg(feature = "backend-default")]
    {
        use cityjson::backend::default::*;
        group.bench_function(BenchmarkId::new("default", "10k_with_geo"), |b| {
            // benchmark code
        });
    }

    #[cfg(feature = "backend-nested")]
    {
        use cityjson::backend::nested::*;
        group.bench_function(BenchmarkId::new("nested", "10k_with_geo"), |b| {
            // same benchmark code
        });
    }

    group.finish();
}
```

---

### 7.3 Benchmark Execution Commands

**Run default backend only**:
```bash
cargo bench --features backend-default
```

**Run nested backend only**:
```bash
cargo bench --features backend-nested
```

**Run both backends** (requires `backend-both` feature):
```bash
cargo bench --features backend-both
```

**Generate baseline for comparison**:
```bash
# Run default and save baseline
cargo bench --features backend-default -- --save-baseline default

# Run nested and save baseline
cargo bench --features backend-nested -- --save-baseline nested

# Compare
cargo bench --features backend-default -- --baseline nested
```

**View results**:
```bash
# HTML reports generated in target/criterion/
open target/criterion/report/index.html
```

---

### 7.4 Expected Benchmark Results

**builder.rs**:
- `build_10k_without_geometries`: Should be similar (mostly CityObject creation)
- `build_10k_with_geometries`: Nested likely slower (more allocations)

**memory.rs**:
- Nested backend expected to use more memory (nested structures)

**processor.rs**:
- Query operations may be slower or faster depending on access patterns

---

## Implementation Priorities

### Must-Have for MVP (Minimum Viable Benchmarks)

**Priority 1 (Critical)**:
1. ✅ Vertex management - REUSE from default
2. ✅ Coordinate types - REUSE from core
3. CityModel basic structure and vertex methods
4. CityObject construction
5. **GeometryBuilder** - Most critical, most complex
6. Geometry getters
7. Basic semantics support

**Priority 2 (High)**:
8. Material/Texture types and methods
9. CityModel material/texture management
10. Appearance container
11. Semantics full API

**Priority 3 (Medium)**:
12. Metadata - REUSE
13. Extensions - REUSE
14. Transform - REUSE
15. Boundary utilities

**Priority 4 (Nice-to-Have)**:
16. Full validation
17. Conversion utilities
18. Advanced features

---

## Success Criteria

### Minimum Viable Implementation (MVP)

✅ **Basic Functionality**:
- [ ] Can instantiate `CityModel::new()`
- [ ] Can add vertices with `add_vertex()`
- [ ] Can construct CityObject
- [ ] Can build simple geometry (MultiPoint)
- [ ] Can build complex geometry (Solid) with GeometryBuilder

✅ **Builder Benchmark Support**:
- [ ] GeometryBuilder API complete
- [ ] Can add semantics to surfaces
- [ ] Can add materials to surfaces
- [ ] Can add textures to rings
- [ ] `builder.rs` benchmark compiles and runs

### Full Implementation

✅ **Complete API**:
- [ ] All v2_0 API methods have nested equivalents

✅ **Benchmark Capability**:
- [ ] Can run benchmarks on either backend via feature flags
- [ ] Can run benchmarks on both backends simultaneously
- [ ] HTML reports show side-by-side comparison
- [ ] Performance differences are measurable and explainable

---

## Development Workflow

### Recommended Implementation Order

**Week 1: Foundation**
- Day 1-2: Phase 1 (re-exports) + CityModel structure
- Day 3-4: CityObject + basic CityModel methods
- Day 5: Start GeometryBuilder

**Week 2: Core Functionality**
- Day 1-3: Complete GeometryBuilder (hardest part)
- Day 4: Semantics API
- Day 5: Test with simple geometries

**Week 3: Appearance and Integration**
- Day 1-2: Material/Texture API + Appearance
- Day 3: Supporting types (metadata, extensions)
- Day 4-5: Benchmark integration and testing

### Testing Strategy

Write the most basic, simple unit tests, maximum one per type.
Unit tests can be omitted if the logic is tested by one of the benchmarks.

---

## Estimated Lines of Code

| Component | Reuse % | New LOC | Complexity |
|-----------|---------|---------|------------|
| vertex.rs | 95% | ~10 | Trivial |
| coordinate.rs | 100% | ~5 | Trivial |
| transform.rs | 100% | ~5 | Trivial |
| metadata.rs | 100% | ~5 | Trivial |
| extension.rs | 100% | ~5 | Trivial |
| appearance.rs (Material/Texture) | 90% | ~100 | Low |
| appearance.rs (Container) | 70% | ~150 | Low |
| cityobject.rs | 70% | ~200 | Medium |
| semantics.rs | 80% | ~100 | Low |
| geometry.rs | 50% | ~300 | Medium |
| **GeometryBuilder** | **40%** | **~800** | **High** |
| citymodel.rs | 30% | ~600 | High |
| geometry_struct.rs | TBD | ~100 | Low |
| boundary.rs | 50% | ~200 | Medium |
| benchmarks | 80% | ~150 | Low |
| **Total** | **~60%** | **~2,735** | **Med-High** |

---

## Risk Mitigation

### High-Risk Areas

**1. GeometryBuilder Complexity**
- **Risk**: Most complex component, ~800 LOC, easy to introduce bugs
- **Mitigation**:
  - Extensive unit tests for each geometry type
  - Compare output with default backend
  - Start with simple geometries, progress to complex

**2. Nested Structure Construction**
- **Risk**: Complex nested Vec structures error-prone
- **Mitigation**:
  - Type aliases for readability
  - Helper methods for construction
  - Visual inspection tools

**3. Semantic/Material/Texture Values**
- **Risk**: Enum variants must match boundary structure depth
- **Mitigation**:
  - Validation methods
  - Runtime checks during construction
  - Comprehensive tests

### Medium-Risk Areas

**4. Performance Degradation**
- **Risk**: Nested backend significantly slower than default
- **Expected**: Some performance loss acceptable (it's a baseline)
- **Mitigation**: Profile and optimize hot paths

**5. API Compatibility**
- **Risk**: Nested API diverges from default, benchmarks don't work
- **Mitigation**: Mirror v2_0 API closely

---

## Notes and Considerations

### Design Decisions

1. **No Global Resource Pools**: Nested backend uses inline storage and simple indices. This is intentional to match JSON structure.

2. **Attributes Already Correct**: The nested backend already has the correct inline `AttributeValue` enum. No changes needed.

3. **String Storage Generic**: Both backends support `OwnedStringStorage` and `BorrowedStringStorage<'a>`.

4. **GeometryBuilder is Critical**: Without GeometryBuilder, benchmarks cannot run. This is the highest priority component.

5. **Reuse Maximization**: ~60% of code can be reused. Focus new work on nested-specific logic (boundary construction, semantic values, material values).

### Open Questions

1. **Should semantics be deduplicated?** : No

2. **Should we support geometry templates fully?**: Yes, implement for completeness

3. **What goes in geometry_struct.rs?** : Not used, not needed

---

## Conclusion

This plan provides a systematic approach to implementing the nested backend API. By prioritizing reusable components and focusing development effort on nested-specific logic (GeometryBuilder, boundary construction), we can achieve the goal of enabling benchmark comparisons between backends.

**Key Success Factors**:
1. Maximize code reuse (~60%)
2. Focus on GeometryBuilder (highest complexity)
3. Test incrementally
4. Mirror v2_0 API for compatibility

**Timeline**: 25-35 hours of focused development

**Next Steps**: Begin with Phase 1 (re-exports) and progress through phases systematically.
