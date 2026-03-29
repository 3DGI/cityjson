# C++ FFI Plan

This page describes the C++ binding direction over the shared low-level FFI
core.

The initial pressure comes from Roofer, but the target is a general C++ layer,
not a Roofer-specific side channel.

## Goal

Expose an idiomatic C++ wrapper that can:

- read CityJSON data into Rust-owned models
- build and edit models from C++ values
- write documents and feature outputs with explicit transform and quantization
  control

The C++ layer should reuse the shared low-level core rather than inventing its
own foreign semantics.

## Public C++ Shape

The public C++ surface should feel like a normal C++ library:

- RAII ownership for model wrappers
- standard container-friendly input and output types
- explicit reader, builder, and writer entry points
- value-oriented helper types where they improve ergonomics

That public layer can be richer than the shared core as long as it compiles
down to the same underlying handle and bulk-operation model.

## What The Shared Core Must Provide For C++

The C++ wrapper needs low-level coverage for:

- model creation and destruction
- metadata and extension access
- cityobject creation and lookup
- bulk vertex and boundary insertion
- geometry, template, material, texture, and semantics access
- submodel extraction and import
- document, feature, and feature-stream write paths
- explicit transform and quantization write options
- stable status and error reporting

Those primitives should be shared with Python even if the Python wrapper
projects them differently.

## What Stays C++-specific

The following belong in the C++ wrapper layer, not in the shared core:

- RAII classes and move semantics
- STL-oriented builders and lightweight view types
- target-local convenience adapters for Roofer or other C++ consumers

The shared core should remain lower-level than the public C++ API.

## Non-goals

This plan should not:

- make Roofer-specific semantics part of the shared core
- expose raw DTO-style C structs as the primary C++ API
- require Python or wasm to adopt the C++ builder surface
- treat the C++ public wrapper as the definition of the shared FFI contract

## Deliverables

1. Cover the shared low-level core primitives needed by C++ model build,
   inspection, and writing.
2. Add a first-class C++ wrapper over that core.
3. Move Roofer onto the wrapper instead of maintaining a separate CityJSON
   writer path.
