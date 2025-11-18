# Implementation Plan: Global Attribute Pool

## Overview

Add a global `AttributePool` as a member of `CityModelCore`, alongside existing resource pools (semantics, materials, textures, geometries). This enables centralized attribute storage while maintaining compile-time safety through Rust's disjoint field borrowing.

## Implementation Steps

### Step 1: Add Attribute Pool Field to CityModelCore

**File:** `/home/user/cityjson-rs/src/backend/default/citymodel.rs`

**Location:** Inside the `CityModelCore` struct (around line 33-82)

**Changes:**
```rust
pub struct CityModelCore<
    C: Coordinate,
    VR: VertexRef,
    RR: ResourceRef,
    SS: StringStorage,
    Semantic,
    Material,
    Texture,
    Geometry,
    Metadata,
    Transform,
    Extensions,
    CityObjects,
> {
    // ... existing fields ...

    /// Pool of texture objects
    textures: DefaultResourcePool<Texture, RR>,
    /// Pool of vertex textures (UV coordinates)
    vertices_texture: Vertices<VR, UVCoordinate>,

    // ADD THIS NEW FIELD:
    /// Global attribute pool for all attributes in the model
    attributes: AttributePool<SS, RR>,

    /// Default theme material reference
    default_theme_material: Option<RR>,
    // ... rest of fields
}
```

**Import Required:**
Add at top of file if not already present:
```rust
use crate::cityjson::core::attributes::AttributePool;
```

---

### Step 2: Update Constructor Methods

**File:** `/home/user/cityjson-rs/src/backend/default/citymodel.rs`

**Location:** `new()` method (around line 116)

**Changes:**
```rust
pub fn new(type_citymodel: CityModelType, version: Option<CityJSONVersion>) -> Self {
    Self {
        type_citymodel,
        version,
        extensions: None,
        extra: None,
        metadata: None,
        cityobjects: CityObjects::default(),
        transform: None,
        vertices: Vertices::new(),
        geometries: DefaultResourcePool::new_pool(),
        template_vertices: Vertices::new(),
        template_geometries: DefaultResourcePool::new_pool(),
        semantics: DefaultResourcePool::new_pool(),
        materials: DefaultResourcePool::new_pool(),
        textures: DefaultResourcePool::new_pool(),
        vertices_texture: Vertices::new(),
        attributes: AttributePool::new(),  // ADD THIS LINE
        default_theme_material: None,
        default_theme_texture: None,
    }
}
```

**Location:** `with_capacity()` method (around line 140)

**Changes:**
```rust
#[allow(clippy::too_many_arguments)]
pub fn with_capacity(
    type_citymodel: CityModelType,
    version: Option<CityJSONVersion>,
    cityobjects_capacity: usize,
    vertex_capacity: usize,
    semantic_capacity: usize,
    material_capacity: usize,
    texture_capacity: usize,
    geometry_capacity: usize,
    create_cityobjects: impl FnOnce(usize) -> CityObjects,
) -> Self {
    Self {
        type_citymodel,
        version,
        extensions: None,
        extra: None,
        metadata: None,
        cityobjects: create_cityobjects(cityobjects_capacity),
        transform: None,
        vertices: Vertices::with_capacity(vertex_capacity),
        geometries: DefaultResourcePool::with_capacity(geometry_capacity),
        template_vertices: Vertices::new(),
        template_geometries: DefaultResourcePool::new(),
        semantics: DefaultResourcePool::with_capacity(semantic_capacity),
        materials: DefaultResourcePool::with_capacity(material_capacity),
        textures: DefaultResourcePool::with_capacity(texture_capacity),
        vertices_texture: Vertices::new(),
        attributes: AttributePool::new(),  // ADD THIS LINE (or with_capacity if desired)
        default_theme_material: None,
        default_theme_texture: None,
    }
}
```

---

### Step 3: Add Accessor Methods

**File:** `/home/user/cityjson-rs/src/backend/default/citymodel.rs`

**Location:** Add new section after existing pool methods (after line 603, before type_citymodel() method)

**Add:**
```rust
// ==================== ATTRIBUTES ====================

/// Get a reference to the attribute pool
pub fn attributes(&self) -> &AttributePool<SS, RR> {
    &self.attributes
}

/// Get a mutable reference to the attribute pool
pub fn attributes_mut(&mut self) -> &mut AttributePool<SS, RR> {
    &mut self.attributes
}

/// Get the number of attributes in the model
pub fn attribute_count(&self) -> usize {
    self.attributes.len()
}

/// Check if there are any attributes
pub fn has_attributes(&self) -> bool {
    !self.attributes.is_empty()
}

/// Clear all attributes from the pool
pub fn clear_attributes(&mut self) {
    self.attributes.clear();
}
```

---

### Step 4: Update Documentation

**File:** `/home/user/cityjson-rs/CLAUDE.md`

**Location:** In the "Flattened Attributes API" section (around line 57-100)

**Update the "Creating and using attributes" section:**

**Creating and using attributes:**

```rust
use cityjson::cityjson::core::attributes::{AttributeOwnerType, OwnedAttributePool};

// The attribute pool is now a member of CityModel
let mut city_model = CityModel::new(CityModelType::CityJSON, Some(CityJSONVersion::V1_1));

// Add attributes to the global pool
let height_id = city_model.attributes_mut().add_float(
    "measuredHeight".to_string(),
    true,  // is_named
    22.3,
    AttributeOwnerType::CityObject,
    None,
);

// Link attribute to CityObject
let mut city_object = CityObject::new("building-1".to_string(), CityObjectType::Building);
city_object.attributes_mut().insert("measuredHeight".to_string(), height_id);
city_model.add_cityobject(city_object);

// Retrieve values from the pool
if let Some(height) = city_model.attributes().get_float(height_id) {
    println!("Height: {}", height);
}
```


**Update the "Nested attributes" section** to show the pattern with CityModel:

**Nested attributes (Maps and Vectors):**

```rust
// Create nested structure: address with location geometry
let mut address_map = HashMap::new();

let country_id = city_model.attributes_mut().add_string(
    "Country".to_string(), true, "Canada".to_string(),
    AttributeOwnerType::Element, None,
);
address_map.insert("Country".to_string(), country_id);

// Add geometry to address using GeometryBuilder
let location_geometry_ref = geometry_builder.build()?;
let location_id = city_model.attributes_mut().add_geometry(
    "location".to_string(), true, location_geometry_ref,
    AttributeOwnerType::Element, None,
);
address_map.insert("location".to_string(), location_id);

// Create the address map attribute
let address_id = city_model.attributes_mut().add_map(
    "address".to_string(), false, address_map,
    AttributeOwnerType::Element, None,
);

// Wrap in vector (CityJSON allows multiple addresses)
let addresses_vec_id = city_model.attributes_mut().add_vector(
    "address".to_string(), true, vec![address_id],
    AttributeOwnerType::CityObject, None,
);

// Link to CityObject
city_object.attributes_mut().insert("address".to_string(), addresses_vec_id);
```


**Add a new section on "Usage with GeometryBuilder":**

### Using Attributes During Geometry Construction

When building geometries with `GeometryBuilder`, you have access to the global attribute pool through the model reference:

```rust
use cityjson::prelude::*;
use cityjson::v1_1::*;

let mut city_model = CityModel::new(CityModelType::CityJSON, Some(CityJSONVersion::V1_1));
let mut geometry_builder = GeometryBuilder::new(&mut city_model, GeometryType::Solid);

// Add semantic attributes during geometry construction
let material_id = geometry_builder.model.attributes_mut().add_string(
    "material".to_string(),
    true,
    "concrete".to_string(),
    AttributeOwnerType::Semantic,
    None,
);

let year_id = geometry_builder.model.attributes_mut().add_integer(
    "year_constructed".to_string(),
    true,
    1985,
    AttributeOwnerType::Semantic,
    None,
);

// Create semantic and attach attributes
let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
roof_semantic.attributes_mut().insert("material".to_string(), material_id);
roof_semantic.attributes_mut().insert("year_constructed".to_string(), year_id);

// Use semantic in geometry
geometry_builder.set_semantic_surface(0, roof_semantic, false)?;
```


---

### Step 5: Add Usage Pattern Documentation

**File:** `/home/user/cityjson-rs/CLAUDE.md`

**Location:** Add new section after "Using Attributes During Geometry Construction"

**Add:**

### Important Usage Pattern: Ordering Matters

Because Rust enforces exclusive mutable borrows, follow this pattern when working with attributes:

**✅ CORRECT - Add to pool first, then use references:**
```rust
// 1. Add attribute to pool (borrows city_model.attributes)
let height_id = city_model.attributes_mut().add_float(
    "height".to_string(), true, 42.0,
    AttributeOwnerType::CityObject, None,
);
// Borrow ends here

// 2. Get object and use attribute ID (new borrow of city_model.cityobjects)
let city_object = city_model.get_cityobject_mut(co_ref).unwrap();
city_object.attributes_mut().insert("height".to_string(), height_id);
```

**❌ INCORRECT - Borrowing conflicts:**
```rust
// This won't compile:
let city_object = city_model.get_cityobject_mut(co_ref).unwrap(); // borrows city_model
let height_id = city_model.attributes_mut().add_float(...); // ERROR: already borrowed!
city_object.attributes_mut().insert("height".to_string(), height_id);
```

**For Multiple Attributes:**
```rust
// Add all attributes first
let height_id = city_model.attributes_mut().add_float(...);
let width_id = city_model.attributes_mut().add_float(...);
let name_id = city_model.attributes_mut().add_string(...);

// Then attach to objects
let city_object = city_model.get_cityobject_mut(co_ref).unwrap();
let attrs = city_object.attributes_mut();
attrs.insert("height".to_string(), height_id);
attrs.insert("width".to_string(), width_id);
attrs.insert("name".to_string(), name_id);
```


---

### Step 6: Update Tests

**File:** Create `/home/user/cityjson-rs/tests/global_attribute_pool.rs`

**Content:**
```rust
//! Tests for global attribute pool integration

use cityjson::prelude::*;
use cityjson::v1_1::*;
use cityjson::cityjson::core::attributes::{AttributeOwnerType, AttributePool};

#[test]
fn test_attribute_pool_in_city_model() {
    let city_model = OwnedCityModel::new(CityModelType::CityJSON, Some(CityJSONVersion::V1_1));

    assert_eq!(city_model.attribute_count(), 0);
    assert!(!city_model.has_attributes());
}

#[test]
fn test_add_cityobject_attributes() {
    let mut city_model = OwnedCityModel::new(CityModelType::CityJSON, Some(CityJSONVersion::V1_1));

    // Add attributes to pool
    let height_id = city_model.attributes_mut().add_float(
        "height".to_string(),
        true,
        42.5,
        AttributeOwnerType::CityObject,
        None,
    );

    let name_id = city_model.attributes_mut().add_string(
        "name".to_string(),
        true,
        "Building A".to_string(),
        AttributeOwnerType::CityObject,
        None,
    );

    // Create city object and link attributes
    let mut city_object = CityObject::new("building-1".to_string(), CityObjectType::Building);
    city_object.attributes_mut().insert("height".to_string(), height_id);
    city_object.attributes_mut().insert("name".to_string(), name_id);

    let co_ref = city_model.add_cityobject(city_object);

    // Verify
    assert_eq!(city_model.attribute_count(), 2);

    let retrieved_co = city_model.get_cityobject(co_ref).unwrap();
    let attrs = retrieved_co.attributes().unwrap();

    assert_eq!(attrs.get("height"), Some(height_id));
    assert_eq!(attrs.get("name"), Some(name_id));

    // Verify values in pool
    assert_eq!(city_model.attributes().get_float(height_id), Some(42.5));
    assert_eq!(city_model.attributes().get_string(name_id), Some(&"Building A".to_string()));
}

#[test]
fn test_semantic_attributes_with_geometry_builder() {
    let mut city_model = OwnedCityModel::new(CityModelType::CityJSON, Some(CityJSONVersion::V1_1));

    // Add vertices
    let v0 = city_model.add_vertex(QuantizedCoordinate::new(0, 0, 0)).unwrap();
    let v1 = city_model.add_vertex(QuantizedCoordinate::new(1, 0, 0)).unwrap();
    let v2 = city_model.add_vertex(QuantizedCoordinate::new(1, 1, 0)).unwrap();
    let v3 = city_model.add_vertex(QuantizedCoordinate::new(0, 1, 0)).unwrap();

    // Start geometry builder
    let mut builder = GeometryBuilder::new(
        &mut city_model,
        GeometryType::MultiSurface,
        BuilderMode::Regular,
    );

    // Add semantic attribute through builder's model reference
    let material_id = builder.model.attributes_mut().add_string(
        "material".to_string(),
        true,
        "concrete".to_string(),
        AttributeOwnerType::Semantic,
        None,
    );

    // Create semantic with attribute
    let mut roof = Semantic::new(SemanticType::RoofSurface);
    roof.attributes_mut().insert("material".to_string(), material_id);

    // Add to builder
    builder.add_vertex(v0).unwrap();
    builder.add_vertex(v1).unwrap();
    builder.add_vertex(v2).unwrap();
    builder.add_vertex(v3).unwrap();

    let ring_idx = builder.add_ring(&[0, 1, 2, 3]).unwrap();
    builder.start_surface().unwrap();
    builder.add_surface_outer_ring(ring_idx).unwrap();
    let surface_idx = builder.end_surface().unwrap();

    builder.set_semantic_surface(surface_idx, roof, false).unwrap();

    // Build geometry
    let geometry = builder.build().unwrap();

    // Verify semantic has attribute reference
    let semantics = geometry.semantics().unwrap();
    let surfaces = semantics.surfaces();
    assert_eq!(surfaces.len(), 1);

    let semantic = &surfaces[0];
    assert!(semantic.attributes().is_some());
    assert_eq!(semantic.attributes().unwrap().get("material"), Some(material_id));
}

#[test]
fn test_nested_attributes() {
    let mut city_model = OwnedCityModel::new(CityModelType::CityJSON, Some(CityJSONVersion::V1_1));

    // Create nested map structure
    let street_id = city_model.attributes_mut().add_string(
        "street".to_string(),
        true,
        "Main St".to_string(),
        AttributeOwnerType::Element,
        None,
    );

    let number_id = city_model.attributes_mut().add_integer(
        "number".to_string(),
        true,
        123,
        AttributeOwnerType::Element,
        None,
    );

    let mut address_map = std::collections::HashMap::new();
    address_map.insert("street".to_string(), street_id);
    address_map.insert("number".to_string(), number_id);

    let address_id = city_model.attributes_mut().add_map(
        "address".to_string(),
        true,
        address_map,
        AttributeOwnerType::CityObject,
        None,
    );

    // Verify nested structure
    let street_val_id = city_model.attributes().get_map_value(address_id, "street").unwrap();
    assert_eq!(
        city_model.attributes().get_string(street_val_id),
        Some(&"Main St".to_string())
    );
}

#[test]
fn test_clear_attributes() {
    let mut city_model = OwnedCityModel::new(CityModelType::CityJSON, Some(CityJSONVersion::V1_1));

    city_model.attributes_mut().add_float(
        "test".to_string(), true, 1.0,
        AttributeOwnerType::CityObject, None,
    );

    assert_eq!(city_model.attribute_count(), 1);

    city_model.clear_attributes();

    assert_eq!(city_model.attribute_count(), 0);
    assert!(!city_model.has_attributes());
}
```

---

### Step 7: Add Example Code

**File:** Create `/home/user/cityjson-rs/examples/global_attribute_pool.rs`

**Content:**
```rust
//! Example demonstrating the global attribute pool pattern

use cityjson::prelude::*;
use cityjson::v1_1::*;
use cityjson::cityjson::core::attributes::AttributeOwnerType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new city model with global attribute pool
    let mut city_model = OwnedCityModel::new(
        CityModelType::CityJSON,
        Some(CityJSONVersion::V1_1),
    );

    println!("=== Creating Building with Attributes ===");

    // Add attributes to the global pool first
    let height_id = city_model.attributes_mut().add_float(
        "measuredHeight".to_string(),
        true,
        25.5,
        AttributeOwnerType::CityObject,
        None,
    );

    let name_id = city_model.attributes_mut().add_string(
        "buildingName".to_string(),
        true,
        "City Hall".to_string(),
        AttributeOwnerType::CityObject,
        None,
    );

    let year_id = city_model.attributes_mut().add_integer(
        "yearOfConstruction".to_string(),
        true,
        1985,
        AttributeOwnerType::CityObject,
        None,
    );

    // Create city object and attach attributes
    let mut building = CityObject::new(
        "building-001".to_string(),
        CityObjectType::Building,
    );

    let attrs = building.attributes_mut();
    attrs.insert("measuredHeight".to_string(), height_id);
    attrs.insert("buildingName".to_string(), name_id);
    attrs.insert("yearOfConstruction".to_string(), year_id);

    let building_ref = city_model.add_cityobject(building);

    println!("Created building with {} attributes",
             city_model.attribute_count());

    // Retrieve and display
    let building = city_model.get_cityobject(building_ref).unwrap();
    if let Some(attrs) = building.attributes() {
        for (key, attr_id) in attrs.iter() {
            print!("  {}: ", key.as_ref());

            if let Some(f) = city_model.attributes().get_float(attr_id) {
                println!("{}", f);
            } else if let Some(s) = city_model.attributes().get_string(attr_id) {
                println!("{}", s.as_ref());
            } else if let Some(i) = city_model.attributes().get_integer(attr_id) {
                println!("{}", i);
            }
        }
    }

    println!("\n=== Creating Geometry with Semantic Attributes ===");

    // Add vertices
    let v0 = city_model.add_vertex(QuantizedCoordinate::new(0, 0, 0))?;
    let v1 = city_model.add_vertex(QuantizedCoordinate::new(10, 0, 0))?;
    let v2 = city_model.add_vertex(QuantizedCoordinate::new(10, 10, 0))?;
    let v3 = city_model.add_vertex(QuantizedCoordinate::new(0, 10, 0))?;

    // Create geometry with semantic attributes
    let mut geom_builder = GeometryBuilder::new(
        &mut city_model,
        GeometryType::MultiSurface,
        BuilderMode::Regular,
    );

    // Add semantic attributes through builder
    let material_id = geom_builder.model.attributes_mut().add_string(
        "roofMaterial".to_string(),
        true,
        "tile".to_string(),
        AttributeOwnerType::Semantic,
        None,
    );

    let color_id = geom_builder.model.attributes_mut().add_string(
        "roofColor".to_string(),
        true,
        "red".to_string(),
        AttributeOwnerType::Semantic,
        None,
    );

    // Create semantic
    let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
    roof_semantic.attributes_mut().insert("roofMaterial".to_string(), material_id);
    roof_semantic.attributes_mut().insert("roofColor".to_string(), color_id);

    // Build geometry
    geom_builder.add_vertex(v0)?;
    geom_builder.add_vertex(v1)?;
    geom_builder.add_vertex(v2)?;
    geom_builder.add_vertex(v3)?;

    let ring_idx = geom_builder.add_ring(&[0, 1, 2, 3])?;
    geom_builder.start_surface()?;
    geom_builder.add_surface_outer_ring(ring_idx)?;
    let surface_idx = geom_builder.end_surface()?;

    geom_builder.set_semantic_surface(surface_idx, roof_semantic, false)?;

    let geometry = geom_builder.build()?;

    println!("Created geometry with semantic attributes");
    println!("Total attributes in model: {}", city_model.attribute_count());

    Ok(())
}
```

---

### Step 8: Run Tests and Verify

**Commands to execute:**
```bash
# Format code
cargo fmt

# Run linter
just lint

# Run all tests
cargo test

# Run specific test
cargo test global_attribute_pool

# Run example
cargo run --example global_attribute_pool

# Build documentation
just doc
```

---

## Success Criteria

- ✅ `AttributePool` is a member of `CityModelCore`
- ✅ All constructors initialize the attribute pool
- ✅ Accessor methods (`attributes()`, `attributes_mut()`) work correctly
- ✅ GeometryBuilder can access attribute pool through `model.attributes_mut()`
- ✅ Tests pass showing CityObject, Semantic, and nested attributes work
- ✅ Documentation updated with usage patterns
- ✅ Example code demonstrates the pattern
- ✅ No RefCell or interior mutability needed
- ✅ Compile-time borrow checking enforced

---

## Notes for Implementation

1. **Rust's Disjoint Borrowing**: The key insight is that Rust allows simultaneous mutable borrows of different struct fields, so `model.semantics`, `model.materials`, and `model.attributes` can all be accessed within the same scope.

2. **Ordering Pattern**: Users must add to pools before borrowing objects from pools. This is a simple mental model and enforced by the compiler.

3. **GeometryBuilder Access**: The builder already has `&mut CityModel`, so accessing `self.model.attributes_mut()` is natural and consistent with how it accesses other pools.

4. **No Breaking Changes**: Existing code continues to work. The attribute pool is simply available for those who want to use it.

5. **Future Extension**: If needed, convenience wrapper methods can be added to CityModel later without changing the core pattern.
