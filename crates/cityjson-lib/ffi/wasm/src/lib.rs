#![allow(clippy::all, clippy::pedantic)]

//! wasm adapter over the shared `cjlib-ffi-core` substrate.
//!
//! The public browser-facing exports will stay task-oriented and narrower than
//! the shared core. This crate is scaffolded so the adapter can grow without
//! reshaping the root `cjlib` crate.

pub use cjlib_ffi_core as core;

/// Marker for the future wasm-facing task API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmStatus {
    Placeholder,
}
