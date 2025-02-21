//! The cityjson-rs library defines the types and methods for representing the complete CityJSON data model in Rust.
//! *cityjson-rs* is meant to be a core dependency in Rust-based CityJSON software, so that the dependent applications can extend the types with their specific functionality.
//! Therefore, *citjson-rs* is designed with performance, flexibility, and ease-of-use in mind.
//! The three criteria are implemented in the following features:
//!
//! - The Geometry representation is flattened into densely packed containers to minimize allocations, improve cache-locality, and enable SIMD operations. This is very different to the nested arrays defined by the CityJSON schema. However, the implementation details are hidden from the API.
//! - Vertex indices, and consequently boundaries, semantics, and appearances can be specialized with either `u16`, `u32` or `u64` types to enable various use cases and memory optimizations.
//! - Supports both borrowed and owned values.
//! - Getter and setter methods are implemented for each CityJSON object and their members to provide a stable API and hide implementation details.
//! - The API is thoroughly documented, including usage examples.
//! - Supports CityJSON Extensions.
//! - Supports multiple CityJSON versions, such as v1.0, v1.1, v2.0, and it is extensible for future versions.

pub mod cityjson;
pub mod errors;
pub mod resources;
pub mod v1_0;
pub mod v1_1;
pub mod v2_0;
