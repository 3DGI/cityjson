# cityjson-lib

Python bindings for the [`cityjson-lib`](https://github.com/3DGI/cityjson)
Rust crate — a higher-level read/write facade for
[CityJSON](https://www.cityjson.org) 2.0.

The distribution name is `cityjson-lib`; the import name is `cityjson_lib`.
Under the hood the package ships a prebuilt native library (built from the
Rust C-ABI core) and a thin `ctypes` wrapper — no Rust toolchain or compiler
is required on the user side.

## Install

```bash
pip install cityjson-lib
```

Prebuilt wheels are published for:

- Linux x86_64 (manylinux)
- macOS x86_64 and arm64
- Windows AMD64

Python 3.11, 3.12, and 3.13 are supported.

## Quick start

```python
from cityjson_lib import CityModel, probe_bytes

payload = open("model.city.json", "rb").read()

probe = probe_bytes(payload)
print(probe.root_kind, probe.version)

model = CityModel.parse_document_bytes(payload)
summary = model.summary()
print(summary.model_type, summary.cityobject_count)
```

For an end-to-end authoring example that exercises the full public Python
API, see [`examples/fake_complete.py`](examples/fake_complete.py).

## Links

- Rust workspace and docs: <https://github.com/3DGI/cityjson>
- CityJSON specification: <https://www.cityjson.org>
- Issues: <https://github.com/3DGI/cityjson/issues>

## License

Dual-licensed under MIT or Apache-2.0, at your option.
