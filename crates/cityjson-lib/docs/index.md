# Welcome to cjlib

A software library for working with semantic 3D city models, based on the [CityJSON](https://cityjson.org) data model.

## Goal setting

1. A software library with a public API for reading and creating city models from/to CityJSON.
2. The API gives direct, language-idiomatic access to the CityJSON objects and allows to create them.
3. The API hides as much as possible from the complexity of CityJSON, but still allows to access each detail.
4. The Rust implementation is the "source of truth" and other languages are implemented through FFI-s on it. The single source of truth implementation reduces code duplication, thus allows us to maintain the library in several languages with little effort.
5. Other languages: C++, Python, WASM. The API for each language feels natural and idiomatic to use.

Detailed functionality:

- [ ] Store the complete data that can be represented by the CityJSON specs, including,
    - [ ] Geometry templates,
    - [ ] CityJSONFeatures,
    - [ ] Extensions.
- [ ] Allow the creation of FFI-s in
    - [ ] C++,
    - [ ] Python,
    - [ ] WASM.
- [ ] Create new CityModels from scratch.
- [ ] Modify an existing CityModel.
    - [ ] Modify root property values (eg. `version`).
    - [ ] Modify/add/remove `Metadata`.
    - [ ] Modify CityObjects in an existing CityModel.
        - [ ] Modify the geometry.
            - [ ] Modify the boundary
                - [ ] Change the coordinates of a vertex.
                - [ ] Add a vertex.
                - [ ] Remove a vertex.
            - [ ] Modify the `texture/material`.
                - [ ] Change the texture/material of a surface.
                - [ ] Add a new texture/material of a surface.
                - [ ] Remove the texture/material of a surface.
            - [ ] Modify the `semantics`.
                - [ ] Change the semantics value of a surface.
                - [ ] Add new semantics to a surface.
                - [ ] Remove the semantics from a surface.
        - [ ] Modify the attributes.
            - [ ] Change the attribute value.
            - [ ] Add a new attribute.
            - [ ] Remove an attribute.
        - [ ] Add a new CityObject.
        - [ ] Add the CityObjects from CityJSONFeature(s).
        - [ ] Add another CityModel.
        - [ ] Drop a CityObject.
    - [ ] Operators, such as `+` (and maybe boolean) on CityModels.
- [ ] Drop a CityModel.

## Scope

- [ ] Maps the complete core CityJSON objects to their equivalent language-specific structure.
- [ ] Provides structures for the CityJSON geometric primitives. This means de/referencing the geometries when reading/writing to CityJSON files.
- [ ] Implements getters and setters for CityModel and each object in CityJSON.
- Does not provide operations on the CityObjects and their geometries (eg. intersect, volume, compare, validate etc.).

## Extensions

Extension handling is currently not part of cjlib.
But a reference implementation of the Noise Extension needs to be written.
This implementation builds on cjlib and extends its structures with those from the Extension.

However, cjlib *should* be able to handle Extensions, hopefully automagically, since the ADE-implementation topic is one of the arguments that we use when comparing CityGML and CityJSON.

## Source of truth in Rust

One way to achieve this is providing a C interface for FFI and then having the other languages interact with the C interface through their normal means.
However, this means to create C bindings to cjlib from each of the languages.

Alternatively, the bindings can be created directly from the Rust library, instead of going through C manually.
There are good Rust libraries that already do this.

So then the bindings for each language is stored in a separate crate.
The core implementation is written in Rust, in the crate **cjlib**.
The bindings use **cjlib** and wrap the required structures around it.

- The C++ crate is either [cxx](https://cxx.rs/index.html) or [cbindgen]() in [**cjlib_cpp**](https://github.com/balazsdukai/cjlib_cpp)
- The python crate is with [PyO3](https://github.com/PyO3/pyo3) in [**cjlib_py**](https://github.com/balazsdukai/cjlib_py)
- The WASM crate is [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) in [**cjlib_wasm**](https://github.com/balazsdukai/cjlib_wasm)

Currently, the bindings-libraries are separated into repositories as listed above, but they are going to be integrated into this repository.

## Architecture and implementation

The idea is to map the CityJSON data model to rust data structures almost one-to-one.
Except for how the geometry boundaries are represented.
While CityJSON uses arrays of indices, cjlib intends to use arrays of coordinates (akin to Simple Features).
This is to reduce the complexity of the code for creating city models.

Nevertheless, there are three alternative architectures in consideration.
Read [the document that outlines them](https://github.com/balazsdukai/cjlib/blob/master/experiments/direct-json-vs-api.md) for more details.

## Other libraries

### Inspiration

GeoJSON rust

## Contributing

For full documentation visit [mkdocs.org](https://www.mkdocs.org).

### Commands

* `mkdocs new [dir-name]` - Create a new project.
* `mkdocs serve` - Start the live-reloading docs server.
* `mkdocs build` - Build the documentation site.
* `mkdocs -h` - Print help message and exit.

### Project layout

    mkdocs.yml    # The configuration file.
    docs/
        index.md  # The documentation homepage.
        ...       # Other markdown pages, images and other files.
