# Python FFI Plan

This page describes the Python binding direction over the shared low-level FFI
core.

The initial consumer is `cjio`-style functionality, but the target is a
general Python library over the Rust model.

## Goal

Expose a Python API that is:

- object-oriented and inspectable
- efficient enough to keep heavy data movement in Rust-owned storage
- generic enough to support model editing rather than just a bag of special
  helper methods

The Python layer should reuse the same low-level core as C++, not fall back to
document-shaped dict/list manipulation.

## Public Python Shape

The public Python surface should center on classes and live views such as:

- `CityModel`
- `CityObject`
- `Geometry`
- collection and iterator views
- bulk geometry and boundary accessors

On top of that, Python can offer convenience algorithms that compose the shared
core primitives into higher-level workflows.

## What The Shared Core Must Provide For Python

Python needs low-level access to:

- model handles and ownership
- collection access for cityobjects, geometries, templates, materials, and
  textures
- bulk vertex and boundary buffers
- bulk remap, import, append, and extract operations
- cleanup and validation-sensitive mutation paths
- explicit transform and quantization controls
- document, feature, and feature-stream parse and serialize entry points
- stable error and version concepts

Those same primitives should remain available to C++, even if the C++ layer
wraps them with different ergonomics.

## What Stays Python-specific

The following belong in the Python wrapper layer:

- classes, properties, and iterators
- mapping and sequence protocol integration
- Python exception types
- reporting, summaries, and convenience algorithms
- host-side file and path helpers

Those are not part of the shared low-level contract.

## Non-goals

This plan should not:

- force Python to mirror the C++ value-builder surface
- expose raw low-level handles as the normal public API
- move generic reporting or convenience logic into the shared core
- treat dict/list document surgery as the default editing model

## Deliverables

1. Bind the shared low-level core into a generic Python model/view layer.
2. Expose the bulk operations needed for extraction, remapping, import, and
   cleanup.
3. Rebuild `cjio`-style workflows on top of that generic layer instead of on
   ad hoc document-shape helpers.
