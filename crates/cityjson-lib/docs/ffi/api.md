# Binding API

This page shows the common user-facing API across Rust, Python, and C++.

The bindings are intentionally aligned around the same core ideas:

- load a document or feature stream
- inspect summary data
- edit through explicit methods
- serialize back to JSON or feature bytes

The wasm adapter is still work in progress and is not covered here.

## Load And Inspect

=== "Rust"
```rust
use cityjson_lib::json;

let model = json::from_file("amsterdam.city.json")?;
let summary = model.summary();
assert_eq!(summary.cityobject_count, 2);
# Ok::<(), cityjson_lib::Error>(())
```

=== "Python"
```python
from cityjson_lib import CityModel

model = CityModel.parse_document_bytes(open("amsterdam.city.json", "rb").read())
summary = model.summary()
assert summary.cityobject_count == 2
model.close()
```

=== "C++"
```cpp
#include <cityjson_lib/cityjson_lib.hpp>

// assume read_file_bytes(path) from a small local helper
const auto bytes = read_file_bytes("amsterdam.city.json");
auto model = cityjson_lib::Model::parse_document(bytes);
const auto summary = model.summary();
assert(summary.cityobject_count == 2U);
```

## Serialize Back

=== "Rust"
```rust
use cityjson_lib::json;

let model = json::from_file("amsterdam.city.json")?;
let bytes = json::to_vec(&model)?;
let text = json::to_string(&model)?;
# let _ = (bytes, text);
# Ok::<(), cityjson_lib::Error>(())
```

=== "Python"
```python
from cityjson_lib import CityModel, WriteOptions

model = CityModel.parse_document_bytes(open("amsterdam.city.json", "rb").read())
text = model.serialize_document(WriteOptions(pretty=True))
assert '"type":"CityJSON"' in text
model.close()
```

=== "C++"
```cpp
#include <cityjson_lib/cityjson_lib.hpp>

// assume read_file_bytes(path) from a small local helper
const auto bytes = read_file_bytes("amsterdam.city.json");
auto model = cityjson_lib::Model::parse_document(bytes);
const auto text = model.serialize_document(cityjson_lib::WriteOptions{.pretty = true});
assert(text.find("\"type\":\"CityJSON\"") != std::string::npos);
```

## Edit And Clean Up

=== "Rust"
```rust
use cityjson_lib::{json, ops};

let model = json::from_file("amsterdam.city.json")?;
let _cleaned = ops::cleanup(&model)?;
# Ok::<(), cityjson_lib::Error>(())
```

=== "Python"
```python
from cityjson_lib import CityModel

model = CityModel.parse_document_bytes(open("amsterdam.city.json", "rb").read())
model.cleanup()
model.close()
```

=== "C++"
```cpp
#include <cityjson_lib/cityjson_lib.hpp>

// assume read_file_bytes(path) from a small local helper
auto model = cityjson_lib::Model::parse_document(read_file_bytes("amsterdam.city.json"));
model.cleanup();
```

## Notes

- Rust users call the publishable `cityjson_lib` crate directly.
- Python uses the `cityjson-lib` package published to PyPI.
- C++ uses the generated C ABI header plus the RAII wrapper in `ffi/cpp`.
