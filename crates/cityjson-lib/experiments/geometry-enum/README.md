Tests the deserialization speed into a Geometry Enum comparted to a Geometry Struct.

Deserializing into an enum is 2.86x slower than into a struct. That is 1.56s for structs vs. 4.47s for enums, 4.08s for
serde_json::Value.

```rust
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum GeometryEnum {
    MultiSurface {
        boundaries: MultiSurfaceBoundary
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub struct GeometryStruct {
    #[serde(rename = "type")]
    type_geom: String,
    boundaries: MultiSurfaceBoundary
}
```

Prepare the data with `python extract_boundaries.py`

Run the benchmarks with `cargo bench`
