//! Backend implementations for CityJSON data structures.
//!
//! This module provides different backend implementations for the CityJSON data model.
//! Each backend provides the same public API through the core module but with different
//! internal representations optimized for different use cases.
//!
//! Available backends:
//! - `default`: The default flattened representation optimized for performance (enabled by default)
//! - `nested`: Alternative nested representation (work in progress)

#[cfg(feature = "backend-default")]
pub mod default;

#[cfg(feature = "backend-nested")]
pub mod nested;
