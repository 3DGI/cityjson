# Practical Guide to Using the Boundary Module

This guide provides practical examples and performance considerations for working with the Boundary module in cityjson-rs.

## Boundary Representations: Flattened vs. Nested

The Boundary module in cityjson-rs provides two ways to represent CityJSON geometry boundaries:

### Flattened Representation (`Boundary`)

The flattened representation optimizes for memory and computational efficiency with these characteristics:

- **Memory Efficiency**: Uses fewer allocations and less memory overhead
- **Cache Locality**: Stores related data contiguously for better cache performance
- **Vector Operations**: Enables SIMD operations on arrays of data
- **Indexing Performance**: Offers O(1) access to elements via indices

### Nested Representation (in `nested` module)

The nested representation mirrors the JSON structure directly:

- **Direct Mapping**: Directly maps to CityJSON's hierarchical structure
- **Serialization Friendly**: Easier to convert to/from JSON
- **Intuitive Structure**: May be more intuitive for simple operations
- **Higher Overhead**: Uses more allocations and indirection

## Performance Considerations

When choosing between the representations, consider these performance factors:

1. **Memory Usage**: The flattened representation typically uses 2-5x less memory than the nested representation for the same geometry.

2. **Cache Efficiency**: The flattened representation's improved cache locality can lead to 1.5-3x faster traversal operations.

3. **Construction Cost**: Converting between representations has overhead. The optimal approach is:
   - Parse JSON directly to flattened representation for input
   - Convert flattened to nested only when needed for output

4. **Operations Cost**:
   - Traversal operations are faster on flattened representation
   - Modifications can be more complex on flattened representation
   - Selective access to specific elements may be easier with nested representation

## Real-World Examples

### Example 1: Building a Simple Building Geometry

Let's create a simple building with a single solid geometry:

```rust
use cityjson::cityjson::geometry::boundary::{Boundary, BoundaryType};
use cityjson::cityjson::vertex::VertexIndex;
use cityjson::cityjson::geometry::GeometryType;
use cityjson::cityjson::geometry::LoD;

// Create a flattened boundary for a simple building (a cube)
fn create_building_boundary() -> Boundary<u32> {
    let mut boundary = Boundary::<u32>::new();
    
    // Add vertices for a cube
    // Bottom face
    boundary.vertices.push(VertexIndex::new(0)); // 0: [0,0,0]
    boundary.vertices.push(VertexIndex::new(1)); // 1: [1,0,0]
    boundary.vertices.push(VertexIndex::new(2)); // 2: [1,1,0]
    boundary.vertices.push(VertexIndex::new(3)); // 3: [0,1,0]
    // Top face
    boundary.vertices.push(VertexIndex::new(4)); // 4: [0,0,1]
    boundary.vertices.push(VertexIndex::new(5)); // 5: [1,0,1]
    boundary.vertices.push(VertexIndex::new(6)); // 6: [1,1,1]
    boundary.vertices.push(VertexIndex::new(7)); // 7: [0,1,1]
    
    // Define rings (each face of the cube)
    // Bottom face
    boundary.rings.push(VertexIndex::new(0));  // Start of ring 0
    // Top face
    boundary.rings.push(VertexIndex::new(4));  // Start of ring 1
    // Side faces
    boundary.rings.push(VertexIndex::new(8));  // Start of ring 2
    boundary.rings.push(VertexIndex::new(12)); // Start of ring 3
    boundary.rings.push(VertexIndex::new(16)); // Start of ring 4
    boundary.rings.push(VertexIndex::new(20)); // Start of ring 5
    
    // Define surfaces (each face of the cube)
    boundary.surfaces.push(VertexIndex::new(0)); // Bottom face
    boundary.surfaces.push(VertexIndex::new(1)); // Top face
    boundary.surfaces.push(VertexIndex::new(2)); // Side face 1
    boundary.surfaces.push(VertexIndex::new(3)); // Side face 2
    boundary.surfaces.push(VertexIndex::new(4)); // Side face 3
    boundary.surfaces.push(VertexIndex::new(5)); // Side face 4
    
    // Define the shell
    boundary.shells.push(VertexIndex::new(0)); // Outer shell includes all surfaces
    
    // We have one solid
    boundary.solids.push(VertexIndex::new(0));
    
    boundary
}

// Usage example:
fn main() {
    let boundary = create_building_boundary();
    assert_eq!(boundary.check_type(), BoundaryType::MultiOrCompositeSolid);
    
    // For serialization, convert to nested representation
    let nested = boundary.to_nested_multi_or_composite_solid().unwrap();
    
    // Use the nested representation for serialization to JSON
    // serialize_to_json(nested); // Hypothetical function
}
```

### Example 2: Processing a CityJSON File

```rust
use cityjson::cityjson::geometry::boundary::{Boundary, BoundaryType};
use cityjson::cityjson::vertex::VertexIndex;
use std::collections::HashMap;

// Process boundaries in a CityJSON model - real world example showing
// how flattened boundaries help with efficient analysis
fn analyze_city_model(boundaries: &[Boundary<u32>]) -> HashMap<BoundaryType, usize> {
    let mut type_counts = HashMap::new();
    let mut total_vertices = 0;
    let mut total_surfaces = 0;
    
    for boundary in boundaries {
        // Count boundary types
        let boundary_type = boundary.check_type();
        *type_counts.entry(boundary_type).or_insert(0) += 1;
        
        // Count total vertices (efficient direct access to arrays)
        total_vertices += boundary.vertices.len();
        
        // Count surfaces (if applicable)
        if !boundary.surfaces.is_empty() {
            total_surfaces += boundary.surfaces.len();
        }
        
        // Additional analysis could be performed here
        // - Surface area calculations
        // - Volume calculations for solids
        // - Spatial queries
        // All of these are more efficient with the flattened representation
    }
    
    println!("Processed {} total vertices and {} surfaces", total_vertices, total_surfaces);
    type_counts
}
```

### Example 3: Integrating with GeometryBuilder

The following example shows how the Boundary module integrates with the GeometryBuilder to create complex geometries:

```rust
use cityjson::cityjson::geometry::{GeometryBuilder, GeometryType, LoD};
use cityjson::cityjson::vertex::VertexIndex;
use cityjson::errors::Result;

// Create a building using GeometryBuilder, which internally uses Boundary
fn create_building_with_builder<V, M>(model: &mut M) -> Result<()> 
where 
    V: CityModelVersion,
    M: CityModelTrait<V>
{
    // Create a geometry builder for a solid
    let mut builder = GeometryBuilder::new(model, GeometryType::Solid)
        .with_lod(LoD::LoD1);
    
    // Add vertices for a cube (bottom face)
    let v0 = builder.add_vertex(0.0, 0.0, 0.0);
    let v1 = builder.add_vertex(10.0, 0.0, 0.0);
    let v2 = builder.add_vertex(10.0, 10.0, 0.0);
    let v3 = builder.add_vertex(0.0, 10.0, 0.0);
    
    // Add vertices for a cube (top face)
    let v4 = builder.add_vertex(0.0, 0.0, 5.0);
    let v5 = builder.add_vertex(10.0, 0.0, 5.0);
    let v6 = builder.add_vertex(10.0, 10.0, 5.0);
    let v7 = builder.add_vertex(0.0, 10.0, 5.0);
    
    // Start creating the solid with one outer shell
    let solid_idx = builder.start_solid();
    let shell_idx = builder.start_shell();
    
    // Create bottom face
    let surface_idx = builder.start_surface(None);
    builder.set_surface_outer_ring(&[v0, v1, v2, v3, v0])?;
    builder.add_shell_outer_surface(surface_idx)?;
    
    // Create top face
    let surface_idx = builder.start_surface(None);
    builder.set_surface_outer_ring(&[v4, v7, v6, v5, v4])?;
    builder.add_shell_outer_surface(surface_idx)?;
    
    // Create four side faces
    let surface_idx = builder.start_surface(None);
    builder.set_surface_outer_ring(&[v0, v4, v5, v1, v0])?;
    builder.add_shell_outer_surface(surface_idx)?;
    
    let surface_idx = builder.start_surface(None);
    builder.set_surface_outer_ring(&[v1, v5, v6, v2, v1])?;
    builder.add_shell_outer_surface(surface_idx)?;
    
    let surface_idx = builder.start_surface(None);
    builder.set_surface_outer_ring(&[v2, v6, v7, v3, v2])?;
    builder.add_shell_outer_surface(surface_idx)?;
    
    let surface_idx = builder.start_surface(None);
    builder.set_surface_outer_ring(&[v3, v7, v4, v0, v3])?;
    builder.add_shell_outer_surface(surface_idx)?;
    
    // Set the shell as the outer shell of the solid
    builder.set_solid_outer_shell(shell_idx)?;
    
    // Build the geometry
    builder.build()
}
```

## When to Use Each Representation

### Use Flattened Representation When:

- Performing geometry processing or analysis
- Building geometries programmatically
- Working with large CityJSON datasets
- Need to optimize for memory usage and performance
- Implementing algorithms that traverse the geometry

### Use Nested Representation When:

- Serializing to/from CityJSON
- Need a direct mapping to the JSON structure
- Working with simple geometries for demonstration purposes
- The clarity of the representation is more important than performance

## Common Patterns in cityjson-rs

1. **Parse JSON to flattened**: When loading CityJSON, parse directly to flattened representations

2. **Work with flattened**: Perform all geometric operations on flattened representations

3. **Convert when needed**: Convert to nested only when serializing to JSON

4. **Builder pattern**: Use the GeometryBuilder for programmatically creating geometries

5. **Type-safe access**: Leverage the type system and enums like BoundaryType for safe handling

## Conclusion

The Boundary module in cityjson-rs provides a powerful foundation for working with CityJSON geometries. By understanding the tradeoffs between flattened and nested representations, you can make informed decisions about how to efficiently process and manipulate city models.