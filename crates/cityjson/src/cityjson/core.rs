//! Core CityJSON types and implementations.
//!
//! This module re-exports the active backend implementation.
//! The backend is selected via feature flags in Cargo.toml.
//!
//! When multiple backends are enabled, `backend-default` takes priority.
//! For benchmarking, access backends directly via `crate::backend::default` or `crate::backend::nested`.

// Re-export the active backend as the core implementation
// Default backend takes priority when multiple backends are enabled
#[cfg(feature = "backend-default")]
pub use crate::backend::default::*;

// Only use nested backend if default is not enabled
#[cfg(all(feature = "backend-nested", not(feature = "backend-default")))]
pub use crate::backend::nested::*;
