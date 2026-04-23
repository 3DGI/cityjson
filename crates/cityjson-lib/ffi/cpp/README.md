# C++ Binding Layout

This directory holds the public C++ wrapper over the shared low-level
`cityjson_lib` FFI core.

Current layout:

- `include/`: public headers
- `examples/`: standalone authoring examples
- `tests/`: C++ smoke coverage

The write-side wrapper is now centered on typed authoring objects instead of
boundary-only helpers:

- `Value` for recursive attributes and extra members
- `Contact` for metadata point-of-contact authoring
- `CityObjectDraft`, `RingDraft`, `SurfaceDraft`, `ShellDraft`, and `GeometryDraft`
- typed resource ids for cityobjects, geometries, templates, semantics, materials, and textures
- `Model` methods for metadata, extensions, appearance resources, hierarchy links, and serialization

The main end-to-end reference is [examples/fake_complete.cpp](examples/fake_complete.cpp),
which builds the full fake-complete CityJSON fixture through the C++ API alone.

The shared C ABI header is generated into `../core/include/cityjson_lib/cityjson_lib.h` via
`just ffi build header`. The C++ wrapper treats that header as the canonical
low-level contract rather than duplicating declarations.

The wrapper is installable through CMake and exposes a generated package config
that installs the headers and links to the shared Rust FFI library.

The wrapper follows the same MIT-or-Apache-2.0 licensing as the Rust crates.
