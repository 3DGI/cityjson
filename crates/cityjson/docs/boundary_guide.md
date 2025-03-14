# CityJSON-rs: Boundary and Geometry Usage Guide

This guide provides practical examples and performance considerations for working with the Boundary and Geometry modules in cityjson-rs.

## Boundary Representations: Flattened vs. Nested

The Boundary module in cityjson-rs provides two ways to represent CityJSON geometry boundaries:

### Flattened Representation (`Boundary`)

The flattened representation (`Boundary<VR>`) is used internally for memory and computational efficiency:

- **Memory Efficiency**: Reduces allocations with densely packed containers
- **Cache Locality**: Contiguous storage improves memory access patterns
- **Computational Efficiency**: Enables SIMD optimizations for vector operations
- **Index-Based Access**: Provides efficient traversal via indices

### Nested Representation (in `nested` module)

The nested representation (defined in the `nested` module) mirrors the CityJSON structure directly:

- **Direct Mapping**: Corresponds to the JSON schema structure
- **Serialization-Friendly**: Simplifies conversion to/from JSON
- **Hierarchical Structure**: Follows CityJSON's nesting pattern
- **Intuitive Access**: Clearer for simple, non-performance-critical operations

## Using the Public API

### Creating Boundaries

The `cityjson-rs` library uses `GeometryBuilder` to create geometries with proper boundaries:

```rust

```

## Performance Considerations

### Memory Efficiency

- The flattened `Boundary` representation typically uses **50-80% less memory** than equivalent nested structures
- Choose appropriate index size for your data:
  - `u16` (≤65,535 vertices): Smallest memory footprint, suitable for small models
  - `u32` (≤4.3 billion vertices): Good balance for most use cases
  - `u64` (virtually unlimited): For extremely large models

### Computational Efficiency

- Working directly with flattened boundaries can be **2-5x faster** for operations like:
  - Traversing geometry hierarchy
  - Computing geometric properties (area, volume)
  - Spatial queries and operations

### Serialization Efficiency

- Converting between flattened and nested representations has overhead
- For best performance, use the companion `serde_cityjson` library which can directly:
  - Parse JSON to flattened representation
  - Serialize flattened representation to JSON

### Builder Performance

- `GeometryBuilder` provides an efficient way to construct complex geometries
- Pre-allocating capacity (e.g., with `with_capacity`) can improve performance for large geometries

## CityJSON Compliance

Cityjson-rs complies with the CityJSON specification:

1. **Complete Schema Support**: The library supports all geometry types defined in the CityJSON specification.

2. **Version Support**: The library supports CityJSON versions 1.0, 1.1, and 2.0.

3. **Extensions**: Supports CityJSON Extensions through extensible types.

4. **Semantic Information**: Properly represents semantic data like material, texture, and appearance information.

5. **Lossless Conversion**: The conversion between flattened and nested representations is lossless, ensuring compliance with the CityJSON specification.

## Best Practices

1. **Use GeometryBuilder**: Always use the `GeometryBuilder` to create geometries rather than manually constructing boundaries

2. **Choose Appropriate Index Size**: Select the smallest index type that can accommodate your data

3. **Minimize Conversions**: Avoid unnecessary conversions between flattened and nested representations

4. **Leverage Public API**: Use public methods rather than accessing internal fields:
   - `check_type()` instead of examining vectors directly
   - `to_nested_*()` methods for conversion to JSON-compatible structures

5. **Validate Boundaries**: Use `is_consistent()` to verify boundary integrity

6. **Error Handling**: All boundary and geometry operations may return `errors::Result<T>`, handle these appropriately
