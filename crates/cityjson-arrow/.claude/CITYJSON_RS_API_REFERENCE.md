# cityjson-rs Public API Reference

**Version**: 1.x (supports CityJSON 1.0, 1.1, 2.0)
**Purpose**: Foundational Rust library for CityJSON data model representation
**Architecture**: Performance-first with flattened geometry representation and global resource pools

---

## Table of Contents

1. [Core Architecture](#core-architecture)
2. [Type System](#type-system)
3. [Module Structure](#module-structure)
4. [Prelude Exports](#prelude-exports)
5. [Error Types](#error-types)
6. [Core Types](#core-types)
7. [Traits](#traits)
8. [Resource System](#resource-system)
9. [Geometry System](#geometry-system)
10. [Attributes System](#attributes-system)
11. [Coordinate System](#coordinate-system)
12. [Boundary System](#boundary-system)
13. [Appearance System](#appearance-system)
14. [Version-Specific APIs](#version-specific-apis)

---

## Core Architecture

### Design Principles

1. **Performance**: Flattened geometry representation with cache-locality optimization
2. **Flexibility**: Support for multiple vertex index types (u16, u32, u64) and owned/borrowed strings
3. **Multiple Versions**: Extensible architecture supporting CityJSON 1.0, 1.1, and 2.0

### Key Patterns

- **Generic Design**: Types are generic over:
  - `VR: VertexRef` - vertex reference type (u16, u32, u64)
  - `RR: ResourceRef` - resource reference type (e.g., ResourceId32)
  - `SS: StringStorage` - string storage strategy (owned or borrowed)
- **Builder Pattern**: `GeometryBuilder` for constructing geometries
- **Resource Pooling**: Centralized resource management with generation-based validation
- **Flattened Structures**: Structure-of-Arrays (SoA) design for efficient memory layout

---

## Type System

### Generic Type Parameters

```rust
// Common generic parameters across the API
VR: VertexRef       // Vertex reference type (u16, u32, u64)
RR: ResourceRef     // Resource reference type (ResourceId32)
SS: StringStorage   // String storage (OwnedStringStorage, BorrowedStringStorage)
C: Coordinate       // Coordinate type (RealWorldCoordinate, QuantizedCoordinate)
```

### Platform Requirements

- **64-bit platforms only** (x86_64, aarch64)
- Minimum Rust version: 1.85.0

---

## Module Structure

```
cityjson-rs/
├── cityjson/          # Version-agnostic core types and traits
│   ├── core/          # Concrete implementations
│   │   ├── appearance.rs
│   │   ├── attributes.rs
│   │   ├── boundary.rs
│   │   ├── coordinate.rs
│   │   ├── extension.rs
│   │   ├── geometry.rs
│   │   ├── metadata.rs
│   │   └── vertex.rs
│   └── traits/        # Core trait definitions
│       ├── coordinate.rs
│       └── semantic.rs
├── resources/         # Resource management
│   ├── pool.rs        # Resource pool implementation
│   ├── storage.rs     # String storage strategies
│   └── mapping.rs     # Resource-to-geometry mappings
├── v1_0/              # CityJSON 1.0 implementation
├── v1_1/              # CityJSON 1.1 implementation
├── v2_0/              # CityJSON 2.0 implementation (default)
└── error.rs           # Error types
```

---

## Prelude Exports

The `prelude` module provides convenient imports:

```rust
use cityjson::prelude::*;
```

### Top-Level Types

- `CityJSON` - Enum for version-specific city models
- `CityJSONVersion` - Version enum (V1_0, V1_1, V2_0)
- `CityModelType` - Type marker (CityJSON, CityJSONFeature)

### Core Types

- `VertexRef`, `VertexIndex`, `VertexIndex16`, `VertexIndex32`, `VertexIndex64`
- `VertexIndexVec`, `VertexIndicesSequence`
- `Boundary`, `Boundary16`, `Boundary32`, `Boundary64`
- `BoundaryType`

### Nested Boundary Types

- `BoundaryNestedMultiPoint*`, `BoundaryNestedMultiLineString*`
- `BoundaryNestedMultiOrCompositeSurface*`, `BoundaryNestedSolid*`
- `BoundaryNestedMultiOrCompositeSolid*`

### Coordinates

- `FlexibleCoordinate`, `QuantizedCoordinate`, `RealWorldCoordinate`, `UVCoordinate`
- `Vertices`, `GeometryVertices16/32/64`, `UVVertices16/32/64`
- `Coordinate` trait

### Geometry

- `GeometryBuilder`, `GeometryType`, `LoD`, `BuilderMode`

### Attributes

- `AttributeValue`, `Attributes`, `OwnedAttributes`, `BorrowedAttributes`

### Resources

- `ResourcePool`, `DefaultResourcePool`, `ResourceRef`, `ResourceId32`
- `StringStorage`, `OwnedStringStorage`, `BorrowedStringStorage`
- `SemanticMap`, `MaterialMap`, `TextureMap`

### Appearance

- `RGB`, `RGBA`, `ImageType`, `TextureType`, `WrapMode`

### Metadata

- `BBox`, `CRS`, `CityModelIdentifier`, `Date`

### Extensions

- `ExtensionCore`, `ExtensionItem`, `ExtensionsCore`

### Error Handling

- `Error`, `Result`

### Traits

- `SemanticTypeTrait`

### Standard Library

- `FromStr`

---

## Error Types

### `Error` Enum

```rust
pub enum Error {
    IncompatibleBoundary(String, String),
    IndexConversion { source_type: String, target_type: String, value: String },
    IndexOverflow { index_type: String, value: String },
    VerticesContainerFull { attempted: usize, maximum: usize },
    InvalidGeometry(String),
    InvalidShell { reason: String, surface_count: usize },
    InvalidRing { reason: String, vertex_count: usize },
    InvalidLineString { reason: String, vertex_count: usize },
    NoActiveElement { element_type: String },
    InvalidReference { element_type: String, index: usize, max_index: usize },
    MissingOuterElement { context: String },
    InvalidGeometryType { expected: String, found: String },
    IncompleteGeometry(String),
    UnsupportedVersion(String, String),
    InvalidCityObjectType(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Usage

All fallible operations return `Result<T>` for proper error handling.

---

## Core Types

### Top-Level Enums

#### `CityJSON<VR, RR, SS>`

Version-tagged enum containing a CityModel:

```rust
pub enum CityJSON<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    V1_0(v1_0::CityModel<VR, RR, SS>),
    V1_1(v1_1::CityModel<VR, RR, SS>),
    V2_0(v2_0::CityModel<VR, RR, SS>),
}
```

#### `CityJSONVersion`

```rust
pub enum CityJSONVersion {
    V1_0,  // Supports 1.0.0 - 1.0.3
    V1_1,  // Supports 1.1.0 - 1.1.3
    V2_0,  // Supports 2.0.0 - 2.0.1 (default)
}
```

**Conversion**: Implements `TryFrom<&str>` and `TryFrom<String>`

#### `CityModelType`

```rust
pub enum CityModelType {
    CityJSON,        // Standard CityJSON object
    CityJSONFeature, // Single feature
}
```

**Conversion**: Implements `TryFrom<&str>` and `TryFrom<String>`

---

## Traits

### `VertexRef`

Core trait for vertex indexing:

```rust
pub trait VertexRef:
    Unsigned + TryInto<usize> + TryFrom<usize> + TryFrom<u32> +
    FromPrimitive + CheckedAdd + Copy + Debug + Default +
    Display + PartialEq + Eq + PartialOrd + Ord + Hash
{
    const MAX: Self;
    const MIN: Self;
}

// Implemented for: u16, u32, u64
```

### `ResourceRef`

Trait for resource identifiers with generation tracking:

```rust
pub trait ResourceRef:
    Copy + Debug + Default + Display +
    PartialEq + Eq + PartialOrd + Ord + Hash
{
    fn new(index: u32, generation: u16) -> Self;
    fn index(&self) -> u32;
    fn generation(&self) -> u16;
}
```

### `StringStorage`

Trait for string storage strategies:

```rust
pub trait StringStorage: Clone + Debug + Default + PartialEq + Eq + Hash {
    type String: AsRef<str> + Eq + PartialEq + PartialOrd + Ord +
                 Hash + Borrow<str> + Clone + Debug + Default + Display;
}

// Implementations:
// - OwnedStringStorage: String = String
// - BorrowedStringStorage<'a>: String = &'a str
```

### `Coordinate`

Marker trait for coordinate types:

```rust
pub trait Coordinate: Default + Clone {}

// Implemented for:
// - FlexibleCoordinate
// - QuantizedCoordinate
// - RealWorldCoordinate
// - UVCoordinate
```

### `SemanticTypeTrait`

Marker trait for semantic type enums:

```rust
pub trait SemanticTypeTrait: Default + std::fmt::Display + Clone {}
```

### `ResourcePool<T, RR>`

Trait for resource pool implementations:

```rust
pub trait ResourcePool<T, RR> {
    type Iter<'a>: Iterator<Item = (RR, &'a T)> where T: 'a, Self: 'a;
    type IterMut<'a>: Iterator<Item = (RR, &'a mut T)> where T: 'a, Self: 'a;

    fn new() -> Self;
    fn with_capacity(capacity: usize) -> Self;
    fn add(&mut self, resource: T) -> RR;
    fn get(&self, id: RR) -> Option<&T>;
    fn get_mut(&mut self, id: RR) -> Option<&mut T>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn remove(&mut self, id: RR) -> Option<T>;
    fn is_valid(&self, id: RR) -> bool;
    fn iter<'a>(&'a self) -> Self::Iter<'a> where T: 'a;
    fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a> where T: 'a;
    fn first(&self) -> Option<(RR, &T)>;
    fn last(&self) -> Option<(RR, &T)>;
    fn find(&self, target: &T) -> Option<RR> where T: PartialEq;
    fn clear(&mut self);
}
```

---

## Resource System

### `ResourceId32`

32-bit resource identifier (index + generation):

```rust
pub struct ResourceId32 {
    index: u32,      // Up to ~4.2 billion slots
    generation: u16, // Up to 65,536 reuses per slot
}

impl ResourceId32 {
    pub fn new(index: u32, generation: u16) -> Self;
    pub fn index(&self) -> u32;
    pub fn generation(&self) -> u16;
    pub fn to_vertex_index<T: VertexRef>(&self) -> Result<VertexIndex<T>>;
}
```

**Generation Overflow**: When generation reaches `u16::MAX`, the slot is retired and not reused.

### `DefaultResourcePool<T, RR>`

Default resource pool implementation:

```rust
pub struct DefaultResourcePool<T, RR: ResourceRef> {
    resources: Vec<Option<T>>,
    generations: Vec<u16>,
    free_list: Vec<u32>,
    _phantom: PhantomData<RR>,
}
```

**Key Features**:
- O(1) add, get, remove operations
- Automatic slot reuse with generation tracking
- Prevents use-after-free with generation validation
- Thread-safe when wrapped in `Arc<Mutex<_>>`

**Usage**:

```rust
let mut pool = DefaultResourcePool::<i32, ResourceId32>::new();
let id = pool.add(42);
assert_eq!(pool.get(id), Some(&42));
pool.remove(id);
assert_eq!(pool.get(id), None); // Invalid after removal
```

### Resource Mappings

#### `SemanticMap<VR, RR>`

Maps semantic information to geometry elements:

```rust
pub struct SemanticMap<VR: VertexRef, RR: ResourceRef> {
    // Internal: SemanticOrMaterialMap structure
}

impl<VR, RR> SemanticMap<VR, RR> {
    pub fn new() -> Self;
    pub fn add_point(&mut self, resource: Option<RR>);
    pub fn add_linestring(&mut self, resource: Option<RR>);
    pub fn add_surface(&mut self, resource: Option<RR>);
    pub fn add_shell(&mut self, offset: VertexIndex<VR>);
    pub fn add_solid(&mut self, offset: VertexIndex<VR>);
    pub fn check_type(&self) -> BoundaryType;
    pub fn is_empty(&self) -> bool;
}
```

#### `MaterialMap<VR, RR>`

Maps material properties to geometry elements (same API as `SemanticMap`).

#### `TextureMap<VR, RR>`

Maps texture coordinates to geometry elements:

```rust
pub struct TextureMap<VR: VertexRef, RR: ResourceRef> {
    // Texture references per ring
    // UV coordinate mappings per vertex
}

impl<VR, RR> TextureMap<VR, RR> {
    pub fn new() -> Self;
    pub fn add_ring(&mut self, texture: Option<RR>, uv_coordinates: Vec<VertexIndex<VR>>);
    pub fn is_empty(&self) -> bool;
}
```

---

## Geometry System

### `GeometryType`

Enumeration of CityJSON geometry types:

```rust
pub enum GeometryType {
    MultiPoint,
    MultiLineString,
    MultiSurface,
    CompositeSurface,
    Solid,
    MultiSolid,
    CompositeSolid,
    GeometryInstance,
}
```

**Conversion**: Implements `FromStr`, `Display`

### `LoD` (Level of Detail)

Represents geometry level of detail:

```rust
pub struct LoD(String);

impl LoD {
    pub fn new(value: String) -> Self;
    // Common constructors for standard LoD values
}
```

**Conversion**: Implements `FromStr`, `Display`

### `BuilderMode`

Controls geometry builder behavior:

```rust
pub enum BuilderMode {
    Regular,  // Build regular geometry
    Template, // Build geometry template for instances
}
```

### `GeometryBuilder<'a, VR, RR, C, Semantic, Material, Texture, Geometry, M, SS>`

Builder for constructing geometries with all associated data:

```rust
pub struct GeometryBuilder<'a, VR, RR, C, Semantic, Material, Texture, Geometry, M, SS>
where
    VR: VertexRef,
    RR: ResourceRef,
    C: Coordinate,
    SS: StringStorage,
{
    // Internal state
}
```

#### Key Methods

**Initialization**:
```rust
pub fn new(model: &'a mut M, type_geometry: GeometryType, builder_mode: BuilderMode) -> Self;
pub fn with_lod(mut self, lod: LoD) -> Self;
```

**Geometry Instance**:
```rust
pub fn with_template(mut self, template_ref: RR) -> Result<Self>;
pub fn with_transformation_matrix(mut self, matrix: [f64; 16]) -> Result<Self>;
pub fn with_reference_point(mut self, point: C) -> Self;
pub fn with_reference_vertex(mut self, vertex: VertexIndex<VR>) -> Self;
```

**Adding Vertices**:
```rust
pub fn add_point(&mut self, point: C) -> usize;
pub fn add_vertex(&mut self, vertex: VertexIndex<VR>) -> usize;
pub fn add_template_point(&mut self, point: RealWorldCoordinate) -> usize;
pub fn add_template_vertex(&mut self, vertex: VertexIndex<VR>) -> usize;
```

**UV Coordinates**:
```rust
pub fn add_uv_coordinate(&mut self, u: f32, v: f32) -> usize;
pub fn map_vertex_to_uv(&mut self, vertex_idx: usize, uv_idx: usize);
```

**Building Geometry Elements**:
```rust
// LineStrings
pub fn add_linestring(&mut self, vertices: &[usize]) -> Result<usize>;

// Rings
pub fn add_ring(&mut self, vertices: &[usize]) -> Result<usize>;

// Surfaces
pub fn start_surface(&mut self) -> usize;
pub fn add_surface_outer_ring(&mut self, ring_idx: usize) -> Result<()>;
pub fn add_surface_inner_ring(&mut self, ring_idx: usize) -> Result<()>;

// Shells
pub fn add_shell(&mut self, surfaces: &[usize]) -> Result<usize>;

// Solids
pub fn start_solid(&mut self) -> usize;
pub fn add_solid_outer_shell(&mut self, shell_idx: usize) -> Result<()>;
pub fn add_solid_inner_shell(&mut self, shell_idx: usize) -> Result<()>;
```

**Adding Semantics** (requires `GeometryModelOps` trait):
```rust
pub fn set_semantic_point(&mut self, index: Option<usize>, semantic: Semantic, deduplicate: bool) -> Result<RR>;
pub fn set_semantic_linestring(&mut self, index: Option<usize>, semantic: Semantic, deduplicate: bool) -> Result<RR>;
pub fn set_semantic_surface(&mut self, index: Option<usize>, semantic: Semantic, deduplicate: bool) -> Result<RR>;
```

**Adding Materials and Textures**:
```rust
pub fn set_material_surface(&mut self, theme: SS::String, index: Option<usize>, material: Material, deduplicate: bool) -> Result<RR>;
pub fn set_texture_ring(&mut self, theme: SS::String, index: Option<usize>, texture: Texture, deduplicate: bool) -> Result<RR>;
```

**Finalizing**:
```rust
pub fn build(self) -> Result<RR>
where
    M: GeometryModelOps<VR, RR, C, Semantic, Material, Texture, Geometry, SS>,
    Geometry: GeometryConstructor<VR, RR, SS>;
```

#### Usage Pattern

```rust
use cityjson::prelude::*;

// Assuming 'model' is a CityModel instance
let geometry_ref = GeometryBuilder::new(&mut model, GeometryType::Solid, BuilderMode::Regular)
    .with_lod(LoD::new("2.2".to_string()))
    // Add vertices
    .add_point(RealWorldCoordinate::new(0.0, 0.0, 0.0));
    // Build rings, surfaces, shells, etc.
    .build()?;
```

---

## Attributes System

### `AttributePool<SS, RR>`

Flattened Structure-of-Arrays (SoA) design for efficient attribute storage:

```rust
pub struct AttributePool<SS: StringStorage, RR: ResourceRef> {
    // Metadata arrays
    keys: Vec<SS::String>,
    types: Vec<AttributeValueType>,
    generations: Vec<u16>,
    is_named: Vec<bool>,

    // Type-specific value arrays (columnar storage)
    bool_values: Vec<Option<bool>>,
    unsigned_values: Vec<Option<u64>>,
    integer_values: Vec<Option<i64>>,
    float_values: Vec<Option<f64>>,
    string_values: Vec<Option<SS::String>>,
    geometry_values: Vec<Option<RR>>,

    // Owner tracking
    owner_types: Vec<AttributeOwnerType>,
    owner_refs: Vec<Option<RR>>,

    // Nested structures (self-referential)
    vector_elements: HashMap<usize, Vec<AttributeId32>>,
    map_elements: HashMap<usize, HashMap<SS::String, AttributeId32>>,

    // Fast lookups and memory management
    key_to_index: HashMap<SS::String, usize>,
    free_list: Vec<usize>,
}
```

**Type Aliases**:
```rust
pub type AttributeId32 = ResourceId32;
pub type OwnedAttributePool = AttributePool<OwnedStringStorage, ResourceId32>;
pub type BorrowedAttributePool<'a> = AttributePool<BorrowedStringStorage<'a>, ResourceId32>;
```

#### Key Methods

**Creation**:
```rust
pub fn new() -> Self;
pub fn with_capacity(capacity: usize) -> Self;
pub fn len(&self) -> usize;
pub fn is_empty(&self) -> bool;
```

**Adding Attributes**:
```rust
pub fn add_null(
    &mut self,
    key: SS::String,
    is_named: bool,
    owner_type: AttributeOwnerType,
    owner_ref: Option<RR>,
) -> AttributeId32;

pub fn add_bool(&mut self, key: SS::String, is_named: bool, value: bool, owner_type: AttributeOwnerType, owner_ref: Option<RR>) -> AttributeId32;
pub fn add_unsigned(&mut self, key: SS::String, is_named: bool, value: u64, owner_type: AttributeOwnerType, owner_ref: Option<RR>) -> AttributeId32;
pub fn add_integer(&mut self, key: SS::String, is_named: bool, value: i64, owner_type: AttributeOwnerType, owner_ref: Option<RR>) -> AttributeId32;
pub fn add_float(&mut self, key: SS::String, is_named: bool, value: f64, owner_type: AttributeOwnerType, owner_ref: Option<RR>) -> AttributeId32;
pub fn add_string(&mut self, key: SS::String, is_named: bool, value: SS::String, owner_type: AttributeOwnerType, owner_ref: Option<RR>) -> AttributeId32;
pub fn add_geometry(&mut self, key: SS::String, is_named: bool, value: RR, owner_type: AttributeOwnerType, owner_ref: Option<RR>) -> AttributeId32;
pub fn add_vector(&mut self, key: SS::String, is_named: bool, elements: Vec<AttributeId32>, owner_type: AttributeOwnerType, owner_ref: Option<RR>) -> AttributeId32;
pub fn add_map(&mut self, key: SS::String, is_named: bool, elements: HashMap<SS::String, AttributeId32>, owner_type: AttributeOwnerType, owner_ref: Option<RR>) -> AttributeId32;
```

**Retrieving Attributes**:
```rust
pub fn get_type(&self, id: AttributeId32) -> Option<AttributeValueType>;
pub fn get_key(&self, id: AttributeId32) -> Option<&SS::String>;
pub fn get_bool(&self, id: AttributeId32) -> Option<bool>;
pub fn get_unsigned(&self, id: AttributeId32) -> Option<u64>;
pub fn get_integer(&self, id: AttributeId32) -> Option<i64>;
pub fn get_float(&self, id: AttributeId32) -> Option<f64>;
pub fn get_string(&self, id: AttributeId32) -> Option<&SS::String>;
pub fn get_geometry(&self, id: AttributeId32) -> Option<RR>;

// Vector operations
pub fn get_vector_elements(&self, id: AttributeId32) -> Option<&Vec<AttributeId32>>;
pub fn get_vector_element(&self, id: AttributeId32, element_idx: usize) -> Option<AttributeId32>;
pub fn get_vector_length(&self, id: AttributeId32) -> Option<usize>;

// Map operations
pub fn get_map_elements(&self, id: AttributeId32) -> Option<&HashMap<SS::String, AttributeId32>>;
pub fn get_map_value(&self, id: AttributeId32, key: &str) -> Option<AttributeId32>;
pub fn get_map_size(&self, id: AttributeId32) -> Option<usize>;
pub fn get_map_keys(&self, id: AttributeId32) -> Option<impl Iterator<Item = &SS::String>>;
pub fn get_map_iter(&self, id: AttributeId32) -> Option<impl Iterator<Item = (&SS::String, AttributeId32)>>;
```

**Management**:
```rust
pub fn is_valid(&self, id: AttributeId32) -> bool;
pub fn remove(&mut self, id: AttributeId32) -> bool;
pub fn remove_by_key(&mut self, key: &str) -> bool;
pub fn get_id_by_key(&self, key: &str) -> Option<AttributeId32>;
pub fn clear(&mut self);
```

### `AttributeValue<SS, RR>`

Enum for building and converting attribute trees:

```rust
pub enum AttributeValue<SS: StringStorage, RR: ResourceRef> {
    Null,
    Bool(bool),
    Unsigned(u64),
    Integer(i64),
    Float(f64),
    String(SS::String),
    Vec(Vec<Box<AttributeValue<SS, RR>>>),
    Map(HashMap<SS::String, Box<AttributeValue<SS, RR>>>),
    Geometry(RR),
}
```

### `AttributeValueType`

Type discriminator for the flattened pool:

```rust
pub enum AttributeValueType {
    Null,
    Bool,
    Unsigned,
    Integer,
    Float,
    String,
    Vec,
    Map,
    Geometry,
}
```

### `Attributes<SS>`

Lightweight container holding references to attributes in the global pool:

```rust
pub struct Attributes<SS: StringStorage> {
    attributes: HashMap<SS::String, AttributeId32>,
}

impl<SS: StringStorage> Attributes<SS> {
    pub fn new() -> Self;
    pub fn insert(&mut self, key: SS::String, id: AttributeId32) -> Option<AttributeId32>;
    pub fn get(&self, key: &str) -> Option<AttributeId32>;
    pub fn remove(&mut self, key: &str) -> Option<AttributeId32>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn contains_key(&self, key: &str) -> bool;
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a SS::String, AttributeId32)> + 'a;
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = &'a SS::String> + 'a;
    pub fn clear(&mut self);
}
```

**Type Aliases**:
```rust
pub type OwnedAttributes = Attributes<OwnedStringStorage>;
pub type BorrowedAttributes<'a> = Attributes<BorrowedStringStorage<'a>>;
```

### `AttributeOwnerType`

Indicates what type of entity owns an attribute:

```rust
pub enum AttributeOwnerType {
    None,       // Deleted attributes
    CityObject, // Owned by a CityObject
    Semantic,   // Owned by a semantic surface
    Metadata,   // Owned by metadata
    CityModel,  // Owned by the CityModel itself
    Element,    // Part of a vector/array or map
}
```

---

## Coordinate System

### `FlexibleCoordinate`

Enum for either quantized or real-world coordinates:

```rust
pub enum FlexibleCoordinate {
    Quantized(QuantizedCoordinate),
    RealWorld(RealWorldCoordinate),
}
```

### `QuantizedCoordinate`

Integer-based coordinates for storage efficiency:

```rust
pub struct QuantizedCoordinate {
    x: i64,
    y: i64,
    z: i64,
}

impl QuantizedCoordinate {
    pub fn new(x: i64, y: i64, z: i64) -> Self;
    pub fn x(&self) -> i64;
    pub fn y(&self) -> i64;
    pub fn z(&self) -> i64;
}
```

### `RealWorldCoordinate`

Floating-point coordinates for precision:

```rust
pub struct RealWorldCoordinate {
    x: f64,
    y: f64,
    z: f64,
}

impl RealWorldCoordinate {
    pub fn new(x: f64, y: f64, z: f64) -> Self;
    pub fn x(&self) -> f64;
    pub fn y(&self) -> f64;
    pub fn z(&self) -> f64;
}
```

### `UVCoordinate`

2D texture coordinates:

```rust
pub struct UVCoordinate {
    u: f32,
    v: f32,
}

impl UVCoordinate {
    pub fn new(u: f32, v: f32) -> Self;
    pub fn u(&self) -> f32;
    pub fn v(&self) -> f32;
}
```

### `Vertices<VR, V>`

Generic container for vertex coordinates:

```rust
pub struct Vertices<VR: VertexRef, V: Coordinate> {
    coordinates: Vec<V>,
    _phantom: PhantomData<VR>,
}

impl<VR, V> Vertices<VR, V> {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
    pub fn reserve(&mut self, additional_capacity: usize) -> Result<()>;
    pub fn len(&self) -> usize;
    pub fn push(&mut self, coordinate: V) -> Result<VertexIndex<VR>>;
    pub fn get(&self, index: VertexIndex<VR>) -> Option<&V>;
    pub fn is_empty(&self) -> bool;
    pub fn as_slice(&self) -> &[V];
    pub fn clear(&mut self);
}
```

**Type Aliases**:
```rust
pub type GeometryVertices16 = Vertices<u16, RealWorldCoordinate>;
pub type GeometryVertices32 = Vertices<u32, RealWorldCoordinate>;
pub type GeometryVertices64 = Vertices<u64, RealWorldCoordinate>;

pub type UVVertices16 = Vertices<u16, UVCoordinate>;
pub type UVVertices32 = Vertices<u32, UVCoordinate>;
pub type UVVertices64 = Vertices<u64, UVCoordinate>;
```

### `VertexIndex<T>`

Typed wrapper for vertex indices:

```rust
pub struct VertexIndex<T: VertexRef>(T);

impl<T: VertexRef> VertexIndex<T> {
    pub fn new(value: T) -> Self;
    pub fn value(&self) -> T;
    pub fn to_usize(&self) -> usize;
    pub fn from_u32(value: u32) -> Option<Self>;
    pub fn is_max(&self) -> bool;
    pub fn is_zero(&self) -> bool;
    pub fn next(&self) -> Option<Self>;
}
```

**Type Aliases**:
```rust
pub type VertexIndex16 = VertexIndex<u16>;
pub type VertexIndex32 = VertexIndex<u32>;
pub type VertexIndex64 = VertexIndex<u64>;
```

**Conversion**:
- `From<u16/u32/u64>` for appropriate target types
- `TryFrom` for narrowing conversions
- `TryFrom<usize>`

### Utility Traits

#### `VertexIndicesSequence<T>`

```rust
pub trait VertexIndicesSequence<T: VertexRef> {
    fn sequence(start: T, count: usize) -> Result<Vec<VertexIndex<T>>>;
}

// Usage:
let indices = VertexIndex16::sequence(0, 5)?;
```

#### `VertexIndexVec<T>`

```rust
pub trait VertexIndexVec<T: VertexRef> {
    fn to_vertex_indices(self) -> Vec<VertexIndex<T>>;
}

// Usage:
let raw_indices = vec![0u16, 1, 2];
let vertex_indices = raw_indices.to_vertex_indices();
```

---

## Boundary System

### `Boundary<VR>`

Flattened representation of CityJSON geometry boundaries:

```rust
pub struct Boundary<VR: VertexRef> {
    pub(crate) vertices: Vec<VertexIndex<VR>>,
    pub(crate) rings: Vec<VertexIndex<VR>>,
    pub(crate) surfaces: Vec<VertexIndex<VR>>,
    pub(crate) shells: Vec<VertexIndex<VR>>,
    pub(crate) solids: Vec<VertexIndex<VR>>,
}
```

**Type Aliases**:
```rust
pub type Boundary16 = Boundary<u16>;
pub type Boundary32 = Boundary<u32>;
pub type Boundary64 = Boundary<u64>;
```

#### Key Methods

```rust
impl<VR: VertexRef> Boundary<VR> {
    pub fn new() -> Self;
    pub fn with_capacity(vertices: usize, rings: usize, surfaces: usize, shells: usize, solids: usize) -> Self;

    // Raw access
    pub fn vertices_raw(&self) -> RawVertexView<'_, VR>;
    pub fn rings_raw(&self) -> RawVertexView<'_, VR>;
    pub fn surfaces_raw(&self) -> RawVertexView<'_, VR>;
    pub fn shells_raw(&self) -> RawVertexView<'_, VR>;
    pub fn solids_raw(&self) -> RawVertexView<'_, VR>;

    // Slice access
    pub fn vertices(&self) -> &[VertexIndex<VR>];
    pub fn rings(&self) -> &[VertexIndex<VR>];
    pub fn surfaces(&self) -> &[VertexIndex<VR>];
    pub fn shells(&self) -> &[VertexIndex<VR>];
    pub fn solids(&self) -> &[VertexIndex<VR>];

    // Setters
    pub fn set_vertices_from_iter<I>(&mut self, iter: I) where I: IntoIterator<Item = VertexIndex<VR>>;
    pub fn set_rings_from_iter<I>(&mut self, iter: I) where I: IntoIterator<Item = VertexIndex<VR>>;
    pub fn set_surfaces_from_iter<I>(&mut self, iter: I) where I: IntoIterator<Item = VertexIndex<VR>>;
    pub fn set_shells_from_iter<I>(&mut self, iter: I) where I: IntoIterator<Item = VertexIndex<VR>>;
    pub fn set_solids_from_iter<I>(&mut self, iter: I) where I: IntoIterator<Item = VertexIndex<VR>>;

    // Conversions to nested representations
    pub fn to_nested_multi_point(&self) -> Result<BoundaryNestedMultiPoint<VR>>;
    pub fn to_nested_multi_linestring(&self) -> Result<BoundaryNestedMultiLineString<VR>>;
    pub fn to_nested_multi_or_composite_surface(&self) -> Result<BoundaryNestedMultiOrCompositeSurface<VR>>;
    pub fn to_nested_solid(&self) -> Result<BoundaryNestedSolid<VR>>;
    pub fn to_nested_multi_or_composite_solid(&self) -> Result<BoundaryNestedMultiOrCompositeSolid<VR>>;

    // Utilities
    pub fn check_type(&self) -> BoundaryType;
    pub fn is_consistent(&self) -> bool;
}
```

### `BoundaryType`

Enum identifying boundary structure:

```rust
pub enum BoundaryType {
    MultiOrCompositeSolid,
    Solid,
    MultiOrCompositeSurface,
    MultiLineString,
    MultiPoint,
    None,
}
```

### Nested Boundary Types

For JSON compatibility, nested representations are provided:

```rust
// Type aliases for common configurations
pub type BoundaryNestedMultiPoint<VR> = Vec<VR>;
pub type BoundaryNestedMultiPoint16 = Vec<u16>;
pub type BoundaryNestedMultiPoint32 = Vec<u32>;
pub type BoundaryNestedMultiPoint64 = Vec<u64>;

pub type BoundaryNestedMultiLineString<VR> = Vec<Vec<VR>>;
pub type BoundaryNestedMultiLineString16 = Vec<Vec<u16>>;
pub type BoundaryNestedMultiLineString32 = Vec<Vec<u32>>;
pub type BoundaryNestedMultiLineString64 = Vec<Vec<u64>>;

pub type BoundaryNestedMultiOrCompositeSurface<VR> = Vec<Vec<Vec<VR>>>;
pub type BoundaryNestedMultiOrCompositeSurface16 = Vec<Vec<Vec<u16>>>;
pub type BoundaryNestedMultiOrCompositeSurface32 = Vec<Vec<Vec<u32>>>;
pub type BoundaryNestedMultiOrCompositeSurface64 = Vec<Vec<Vec<u64>>>;

pub type BoundaryNestedSolid<VR> = Vec<Vec<Vec<Vec<VR>>>>;
pub type BoundaryNestedSolid16 = Vec<Vec<Vec<Vec<u16>>>>;
pub type BoundaryNestedSolid32 = Vec<Vec<Vec<Vec<u32>>>>;
pub type BoundaryNestedSolid64 = Vec<Vec<Vec<Vec<u64>>>>;

pub type BoundaryNestedMultiOrCompositeSolid<VR> = Vec<Vec<Vec<Vec<Vec<VR>>>>>;
pub type BoundaryNestedMultiOrCompositeSolid16 = Vec<Vec<Vec<Vec<Vec<u16>>>>>;
pub type BoundaryNestedMultiOrCompositeSolid32 = Vec<Vec<Vec<Vec<Vec<u32>>>>>;
pub type BoundaryNestedMultiOrCompositeSolid64 = Vec<Vec<Vec<Vec<Vec<u64>>>>>;
```

**Conversion**: Nested types implement `Into<Boundary<VR>>` and `Boundary<VR>` implements conversion methods to nested types.

---

## Appearance System

### Type Aliases

```rust
pub type RGB = [f32; 3];
pub type RGBA = [f32; 4];
```

### Enums

```rust
pub enum ImageType {
    Png,
    Jpg,
}

pub enum WrapMode {
    Wrap,
    Mirror,
    Clamp,
    Border,
    None,
}

pub enum TextureType {
    Unknown,
    Specific,
    Typical,
}
```

### `MaterialCore<SS>`

Material properties:

```rust
pub struct MaterialCore<SS: StringStorage> {
    name: SS::String,
    ambient_intensity: Option<f32>,
    diffuse_color: Option<RGB>,
    emissive_color: Option<RGB>,
    specular_color: Option<RGB>,
    shininess: Option<f32>,
    transparency: Option<f32>,
    is_smooth: Option<bool>,
}

impl<SS> MaterialCore<SS> {
    pub fn new(name: SS::String) -> Self;
    pub fn name(&self) -> &SS::String;
    pub fn set_name(&mut self, name: SS::String);
    pub fn ambient_intensity(&self) -> Option<f32>;
    pub fn set_ambient_intensity(&mut self, value: Option<f32>);
    pub fn diffuse_color(&self) -> Option<&RGB>;
    pub fn set_diffuse_color(&mut self, value: Option<RGB>);
    pub fn emissive_color(&self) -> Option<&RGB>;
    pub fn set_emissive_color(&mut self, value: Option<RGB>);
    pub fn specular_color(&self) -> Option<&RGB>;
    pub fn set_specular_color(&mut self, value: Option<RGB>);
    pub fn shininess(&self) -> Option<f32>;
    pub fn set_shininess(&mut self, value: Option<f32>);
    pub fn transparency(&self) -> Option<f32>;
    pub fn set_transparency(&mut self, value: Option<f32>);
    pub fn is_smooth(&self) -> Option<bool>;
    pub fn set_is_smooth(&mut self, value: Option<bool>);
}
```

### `TextureCore<SS>`

Texture properties:

```rust
pub struct TextureCore<SS: StringStorage> {
    image_type: ImageType,
    image: SS::String,
    wrap_mode: Option<WrapMode>,
    texture_type: Option<TextureType>,
    border_color: Option<RGBA>,
}

impl<SS> TextureCore<SS> {
    pub fn new(image: SS::String, image_type: ImageType) -> Self;
    pub fn image_type(&self) -> &ImageType;
    pub fn set_image_type(&mut self, image_type: ImageType);
    pub fn image(&self) -> &SS::String;
    pub fn set_image(&mut self, image: SS::String);
    pub fn wrap_mode(&self) -> Option<WrapMode>;
    pub fn set_wrap_mode(&mut self, wrap_mode: Option<WrapMode>);
    pub fn texture_type(&self) -> Option<TextureType>;
    pub fn set_texture_type(&mut self, texture_type: Option<TextureType>);
    pub fn border_color(&self) -> Option<RGBA>;
    pub fn set_border_color(&mut self, border_color: Option<RGBA>);
}
```

---

## Metadata Types

### `BBox`

Bounding box representation:

```rust
pub struct BBox {
    values: [f64; 6], // [minx, miny, minz, maxx, maxy, maxz]
}

impl BBox {
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self;
    pub fn as_slice(&self) -> &[f64];
    pub fn min_x(&self) -> f64;
    pub fn min_y(&self) -> f64;
    pub fn min_z(&self) -> f64;
    pub fn max_x(&self) -> f64;
    pub fn max_y(&self) -> f64;
    pub fn max_z(&self) -> f64;
    pub fn width(&self) -> f64;
    pub fn length(&self) -> f64;
    pub fn height(&self) -> f64;
}
```

**Conversion**: `From<[f64; 6]>` and `Into<[f64; 6]>`

### `CityModelIdentifier<SS>`

Dataset identifier:

```rust
pub struct CityModelIdentifier<SS: StringStorage>(SS::String);

impl<SS> CityModelIdentifier<SS> {
    pub fn new(value: SS::String) -> Self;
    pub fn into_inner(self) -> SS::String;
}
```

### `Date<SS>`

RFC 3339 date representation:

```rust
pub struct Date<SS: StringStorage>(SS::String);

impl<SS> Date<SS> {
    pub fn new(value: SS::String) -> Self;
    pub fn into_inner(self) -> SS::String;
}
```

### `CRS<SS>`

Coordinate reference system URL:

```rust
pub struct CRS<SS: StringStorage>(SS::String);

impl<SS> CRS<SS> {
    pub fn new(value: SS::String) -> Self;
    pub fn into_inner(self) -> SS::String;
}
```

---

## Extension System

### `ExtensionCore<SS>`

Single extension definition:

```rust
pub struct ExtensionCore<SS: StringStorage> {
    name: SS::String,
    url: SS::String,
    version: SS::String,
}

impl<SS> ExtensionCore<SS> {
    pub fn new(name: SS::String, url: SS::String, version: SS::String) -> Self;
    pub fn name(&self) -> &SS::String;
    pub fn url(&self) -> &SS::String;
    pub fn version(&self) -> &SS::String;
}
```

### `ExtensionsCore<SS, E>`

Collection of extensions:

```rust
pub struct ExtensionsCore<SS: StringStorage, E> {
    // Internal storage
}

impl<SS, E: ExtensionItem<SS>> ExtensionsCore<SS, E> {
    pub fn new() -> Self;
    pub fn add(&mut self, extension: E) -> &mut Self;
    pub fn remove(&mut self, name: SS::String) -> bool;
    pub fn get(&self, name: &str) -> Option<&E>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

**Iteration**: Implements `IntoIterator` for owned, borrowed, and mutable iteration.

### `ExtensionItem<SS>` Trait

```rust
pub trait ExtensionItem<SS: StringStorage> {
    fn name(&self) -> &SS::String;
}
```

---

## Version-Specific APIs

### Module Structure

Each version module (v1_0, v1_1, v2_0) implements:

- `CityModel<VR, RR, SS>` - Main container for city data
- `CityObject` types - Building, Road, WaterBody, etc.
- `Geometry<VR, RR, SS>` - Version-specific geometry
- `Semantic` types - Version-specific semantic surfaces
- `Material<SS>`, `Texture<SS>` - Appearance types
- `Metadata<SS>` - Version-specific metadata
- `Extension<SS>` - Version-specific extensions

### Common CityModel API Pattern

```rust
// Pattern applies to v1_0, v1_1, and v2_0
pub struct CityModel<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    // Version-specific fields
}

impl<VR, RR, SS> CityModel<VR, RR, SS> {
    pub fn new() -> Self;

    // Vertex management
    pub fn vertices(&self) -> &Vertices<VR, RealWorldCoordinate>;
    pub fn vertices_mut(&mut self) -> &mut Vertices<VR, RealWorldCoordinate>;
    pub fn add_vertex(&mut self, coordinate: RealWorldCoordinate) -> Result<VertexIndex<VR>>;

    // Template vertices
    pub fn template_vertices(&self) -> &Vertices<VR, RealWorldCoordinate>;
    pub fn template_vertices_mut(&mut self) -> &mut Vertices<VR, RealWorldCoordinate>;

    // UV coordinates
    pub fn uv_vertices(&self) -> &Vertices<VR, UVCoordinate>;
    pub fn uv_vertices_mut(&mut self) -> &mut Vertices<VR, UVCoordinate>;
    pub fn add_uv_coordinate(&mut self, coordinate: UVCoordinate) -> Result<VertexIndex<VR>>;

    // CityObject management
    pub fn city_objects(&self) -> &HashMap<SS::String, CityObject<VR, RR, SS>>;
    pub fn city_objects_mut(&mut self) -> &mut HashMap<SS::String, CityObject<VR, RR, SS>>;
    pub fn add_city_object(&mut self, id: SS::String, object: CityObject<VR, RR, SS>);
    pub fn get_city_object(&self, id: &str) -> Option<&CityObject<VR, RR, SS>>;
    pub fn get_city_object_mut(&mut self, id: &str) -> Option<&mut CityObject<VR, RR, SS>>;

    // Resource pools
    pub fn semantics(&self) -> &DefaultResourcePool<Semantic, RR>;
    pub fn semantics_mut(&mut self) -> &mut DefaultResourcePool<Semantic, RR>;
    pub fn add_semantic(&mut self, semantic: Semantic) -> RR;
    pub fn get_or_insert_semantic(&mut self, semantic: Semantic) -> RR;

    pub fn materials(&self) -> &DefaultResourcePool<Material<SS>, RR>;
    pub fn materials_mut(&mut self) -> &mut DefaultResourcePool<Material<SS>, RR>;
    pub fn add_material(&mut self, material: Material<SS>) -> RR;
    pub fn get_or_insert_material(&mut self, material: Material<SS>) -> RR;

    pub fn textures(&self) -> &DefaultResourcePool<Texture<SS>, RR>;
    pub fn textures_mut(&mut self) -> &mut DefaultResourcePool<Texture<SS>, RR>;
    pub fn add_texture(&mut self, texture: Texture<SS>) -> RR;
    pub fn get_or_insert_texture(&mut self, texture: Texture<SS>) -> RR;

    pub fn geometries(&self) -> &DefaultResourcePool<Geometry<VR, RR, SS>, RR>;
    pub fn geometries_mut(&mut self) -> &mut DefaultResourcePool<Geometry<VR, RR, SS>, RR>;
    pub fn add_geometry(&mut self, geometry: Geometry<VR, RR, SS>) -> RR;
    pub fn add_template_geometry(&mut self, geometry: Geometry<VR, RR, SS>) -> RR;

    // Metadata and extensions
    pub fn metadata(&self) -> Option<&Metadata<SS>>;
    pub fn metadata_mut(&mut self) -> Option<&mut Metadata<SS>>;
    pub fn set_metadata(&mut self, metadata: Metadata<SS>);

    pub fn extensions(&self) -> &ExtensionsCore<SS, Extension<SS>>;
    pub fn extensions_mut(&mut self) -> &mut ExtensionsCore<SS, Extension<SS>>;
}
```

### Type Aliases for Common Configurations

Each version module typically provides:

```rust
// Example from v1_1 module
pub type CityModel32 = CityModel<u32, ResourceId32, OwnedStringStorage>;
pub type CityObject32 = CityObject<u32, ResourceId32, OwnedStringStorage>;
pub type Geometry32 = Geometry<u32, ResourceId32, OwnedStringStorage>;
// ... etc
```

---

## Usage Patterns

### Creating a CityModel

```rust
use cityjson::prelude::*;
use cityjson::v1_1::*;

// Create a new CityModel with u32 indices and owned strings
let mut model = CityModel32::new();
```

### Adding Vertices

```rust
let v0 = model.add_vertex(RealWorldCoordinate::new(0.0, 0.0, 0.0))?;
let v1 = model.add_vertex(RealWorldCoordinate::new(10.0, 0.0, 0.0))?;
let v2 = model.add_vertex(RealWorldCoordinate::new(10.0, 10.0, 0.0))?;
```

### Building a Geometry

```rust
use cityjson::prelude::*;

let geometry_ref = GeometryBuilder::new(&mut model, GeometryType::Solid, BuilderMode::Regular)
    .with_lod(LoD::new("2.2".to_string()))
    // Add vertices to boundary
    .add_point(RealWorldCoordinate::new(0.0, 0.0, 0.0));
    // ... add more points, rings, surfaces, shells
    .build()?;
```

### Working with Attributes

```rust
use cityjson::prelude::*;
use cityjson::cityjson::core::attributes::*;

let mut pool = OwnedAttributePool::new();

// Add attributes
let height_id = pool.add_float(
    "height".to_string(),
    true,
    25.5,
    AttributeOwnerType::CityObject,
    None,
);

// Create attribute container for a CityObject
let mut attrs = OwnedAttributes::new();
attrs.insert("height".to_string(), height_id);

// Retrieve
if let Some(height) = pool.get_float(attrs.get("height").unwrap()) {
    println!("Height: {}", height);
}
```

### Working with Resource Pools

```rust
use cityjson::prelude::*;

// Add semantic to pool
let wall_semantic = /* create semantic */;
let wall_ref = model.add_semantic(wall_semantic);

// Retrieve later
if let Some(semantic) = model.semantics().get(wall_ref) {
    // Use semantic
}
```

---

## Key Design Decisions

### Flattened vs Nested Boundaries

- **Flattened** (`Boundary<VR>`): Used internally for performance
  - Single contiguous arrays with offset indices
  - Better cache locality
  - Efficient processing

- **Nested** (`BoundaryNested*`): Used for JSON serialization
  - Matches CityJSON spec structure
  - Easy to serialize/deserialize
  - Conversion methods provided

### Attribute Storage

The `AttributePool` uses a Structure-of-Arrays design:
- Maps directly to Parquet columnar storage
- Avoids Rust enum unions for better serialization
- Each type stored in separate array
- Self-referential for nested structures (Vec, Map)

### Resource Management

Generation-based validation prevents use-after-free:
- Each resource slot has a generation counter
- IDs combine index + generation
- Reusing a slot increments generation
- Old IDs become invalid automatically
- Slots at max generation (u16::MAX) are retired

### Memory Efficiency

- Choice of vertex index type (u16/u32/u64) controls maximum vertices
- String storage choice (owned vs borrowed) affects lifetime management
- Flattened structures reduce allocation overhead
- Resource pools enable efficient reuse

---

## Common Pitfalls

1. **Index Type Mismatches**: Ensure consistent use of `VR` parameter across related types
2. **Generation Validation**: Always check if resource IDs are valid before use
3. **Capacity Limits**: `Vertices` containers have hard limits based on index type
4. **Builder State**: GeometryBuilder methods must be called in correct order (e.g., start_surface before add_surface_outer_ring)
5. **String Lifetimes**: BorrowedStringStorage requires careful lifetime management

---

## Performance Considerations

1. **Pre-allocate**: Use `with_capacity` constructors when size is known
2. **Batch Operations**: Add multiple items at once rather than one-by-one
3. **Index Type Selection**: Use smallest viable index type (u16 < u32 < u64)
4. **Resource Deduplication**: Use `get_or_insert_*` methods to avoid duplicates
5. **Flattened Boundaries**: Use flattened boundaries for internal processing, convert to nested only for I/O

---

## Thread Safety

- Most types are `Send` but not `Sync`
- For concurrent access, wrap in `Arc<Mutex<_>>` or `Arc<RwLock<_>>`
- Resource pools support concurrent access when properly synchronized
- Attribute pools are not thread-safe without external synchronization

---

## Further Reading

- CityJSON Specification: https://www.cityjson.org/specs/
- Repository: https://github.com/cityjson/cityjson-rs
- Examples: See `examples/` directory in repository
- Tests: Comprehensive test coverage in `tests/` directory

---

**Document Version**: 1.0
**Generated**: 2025-11-13
**Crate Version**: Based on cityjson-rs master branch