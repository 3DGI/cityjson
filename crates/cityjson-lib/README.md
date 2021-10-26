# cjlib

## Goal setting

1. A software library with a public API for reading and creating city models from/to CityJSON.
2. The API gives direct, language-idiomatic access to the CityJSON objects and allows to create them.
3. The API hides as much as possible from the complexity of CityJSON, but still allows to access each detail.
4. The Rust implementation is the "source of truth" and other languages are implemented through FFI-s on it. The single source of truth implementation reduces code duplication, thus allows us to maintain the library in several languages with little effort.
5. Other languages: C++, Python, WASM. The API for each language feels natural and idiomatic to use.

## Scope

+ Maps the complete core CityJSON objects to their equivalent language-specifi structure.
+ Provides structures for the CityJSON geometric primitives. This means de/referencing the geometries when reading/writing to CityJSON files.
+ Implements getters and setters for CityModel and each object in CityJSON.
+ Does not provide operations on the CityObjects and their geometries (eg. intersect, volume, compare, validate etc.).
+ Does not handle extensions.

## Extensions

Extension handling is not part of cjlib. 
But a reference implementation of the Noise Extension needs to be written.
This implementation builds on cjlib and extends its structures with those from the Extension.

## Source of truth in Rust

One way to achieve this is providing a C interface for FFI and then having the other languages interact with the C interface through their normal means.
However, this means to create C bindings to cjlib from each of the languages.

Alternatively, the bindings can be created directly from the Rust library, instead of going through C manually. 
There are good Rust libraries that already do this.

So then the bindings for each language is stored in a separate crate.
The core implementation is written in Rust, in the crate **cjlib**.
The bindings use **cjlib** and wrap the required structures around it.

+ The C++ crate is either [cxx](https://cxx.rs/index.html) or [cbindgen]() in **cjlib_cpp**
+ The python crate is with [PyO3](https://github.com/PyO3/pyo3) in **cjlib_py**
+ The WASM crate is [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) in **cjlib_wasm**

## C++

### [cxx](https://cxx.rs/index.html)

However, cxx doesn't allow to get the Rust types from another crate or package, which makes it impossible to set up separate crates for the language bindings, as described above.

*"For now, types used as extern Rust types are required to be defined by the same crate that contains the bridge using them. This restriction may be lifted in the future."* [ref](https://cxx.rs/extern-rust.html#opaque-rust-types)