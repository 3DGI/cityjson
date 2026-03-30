#![allow(clippy::all, clippy::pedantic)]

//! Shared low-level FFI core for non-Rust bindings.
//!
//! This crate will own the common low-level substrate used by the C++, Python,
//! and wasm layers. The public contract is intentionally narrow for now: the
//! project structure exists, but the exported ABI is still to be implemented.

pub use cjlib;

/// Marker for the future shared FFI surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiCoreStatus {
    Placeholder,
}
