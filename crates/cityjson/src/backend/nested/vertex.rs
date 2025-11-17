//! Vertex types for the nested backend.
//!
//! The nested backend uses the same vertex storage as the default backend
//! for cache locality. Vertex pools are backend-agnostic.

pub use crate::backend::default::vertex::*;
