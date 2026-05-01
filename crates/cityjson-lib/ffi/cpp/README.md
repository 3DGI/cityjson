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
- `ModelSelection` for CityObject and geometry selection, relationship expansion, set operations, and extraction

The main end-to-end reference is [examples/fake_complete.cpp](examples/fake_complete.cpp),
which builds the full fake-complete CityJSON fixture through the C++ API alone.

## Selection and merge workflows

`Model::subset_cityobjects(...)` is the simple whole-CityObject extraction
helper. Use `ModelSelection` when a workflow needs to expand through parent or
child relations, select individual geometries, combine selections, or test for
an empty selection before extracting.

```cpp
#include <array>
#include <string_view>

#include <cityjson_lib/cityjson_lib.hpp>

auto model = cityjson_lib::Model::parse_document(bytes);

const auto selection = cityjson_lib::ModelSelection::select_cityobjects_by_id(
    model,
    std::array{std::string_view{"building-part-1"}});
const auto with_relatives = selection.include_relatives(model);
const auto extracted = model.extract_selection(with_relatives);

const auto first_geometry =
    cityjson_lib::ModelSelection::select_geometries_by_cityobject_id_and_index(
        model,
        std::array{cityjson_lib::GeometrySelectionSpec{"building-1", 0U}});
const auto second_geometry =
    cityjson_lib::ModelSelection::select_geometries_by_cityobject_id_and_index(
        model,
        std::array{cityjson_lib::GeometrySelectionSpec{"building-1", 1U}});

const auto combined = first_geometry.union_with(second_geometry);
const auto overlap = first_geometry.intersection_with(second_geometry);
const bool empty = overlap.is_empty();

const auto geometry_extract = model.extract_selection(combined);
const std::array<const cityjson_lib::Model* const, 2> models{
    &extracted,
    &geometry_extract,
};
const auto merged = cityjson_lib::Model::merge_models(models);
```

`Model` and `ModelSelection` are move-only RAII wrappers over the shared C ABI
handles. Methods throw `cityjson_lib::StatusError` when the C core reports an
error.

The shared C ABI header is generated into `../core/include/cityjson_lib/cityjson_lib.h` via
`just ffi build header`. The C++ wrapper treats that header as the canonical
low-level contract rather than duplicating declarations.

The wrapper is installable through CMake and exposes a generated package config
that installs the headers and links to the shared Rust FFI library.

The wrapper follows the same MIT-or-Apache-2.0 licensing as the Rust crates.
