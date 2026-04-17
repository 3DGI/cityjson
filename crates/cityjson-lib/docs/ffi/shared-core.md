# Shared Low-level FFI Core

This document describes the low-level contract shared by the Python, C++, and
future wasm bindings.

## Core Rule

- share one low-level ownership and error model
- allow each binding to shape its own high-level API

## What The Shared Core Covers

The current ABI covers:

- opaque model handles and lifecycle
- probe, parse, and serialize entry points
- summary and metadata access
- cityobject and geometry inspection
- copied vertex, UV, and boundary extraction
- targeted mutation
- cleanup, append, and extract workflows

## What Stays Target-Specific

- C++ RAII wrappers and STL-facing types
- Python classes, dataclasses, and convenience helpers
- wasm export shape and JS-facing packaging

## Design Constraints

- Rust remains the only place that owns semantic invariants
- bulk operations are preferred over chatty crossings
- explicit boundary functions stay explicit
- transport-specific internals do not become public semantic types
