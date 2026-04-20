# cityjson-index

Index CityJSON datasets with a persistent SQLite sidecar. `cityjson-index` is the Rust crate; `cjindex` is the CLI.

## Problem

CityJSON datasets are often large, awkward to scan repeatedly, and split across storage layouts that make ad hoc testing expensive.

This crate gives you a consistent indexing layer so you can:

- inspect dataset layout and freshness
- reindex changed data
- fetch features by identifier
- query features by bounding box
- read regular CityJSON, CityJSONSeq / NDJSON, and feature-file datasets through the same API

## What It Does

`cityjson-index` aims to be a small, predictable indexing layer for CityJSON data:

- builds or refreshes a `.cityjson-index.sqlite` sidecar
- tracks indexed source and feature metadata
- reconstructs CityJSON feature payloads on read
- exposes a CLI for dataset inspection, querying, and retrieval

## Install

### Library

```toml
[dependencies]
cityjson-index = "0.3.1"
```

### CLI

```bash
cargo install cityjson-index --bin cjindex
```

Or run it from a checkout:

```bash
cargo run --bin cjindex -- --help
```

## Usage

### As a Library

The main entry points are:

- `cityjson_index::CityIndex`
- `cityjson_index::resolve_dataset`
- `cityjson_index::StorageLayout`

Example:

```rust
use std::path::Path;

use cityjson_index::{CityIndex, resolve_dataset};

let resolved = resolve_dataset(Path::new("/data/3dbag"), None)?;
let index = CityIndex::open(resolved.storage_layout(), &resolved.index_path)?;
let status = index.status()?;
assert!(status.exists);
# Ok::<(), cityjson_lib::Error>(())
```

### As a CLI

The CLI is dataset-oriented:

```bash
cjindex inspect /data/3dbag
cjindex index /data/3dbag
cjindex reindex /data/3dbag
cjindex validate /data/3dbag
cjindex get /data/3dbag --id NL.IMBAG.Pand.0503100000012869-0
cjindex query /data/3dbag --min-x 4.4 --max-x 4.5 --min-y 51.8 --max-y 51.9
cjindex metadata /data/3dbag
```

Useful patterns:

- `inspect` reports detected layout, freshness, and coverage
- `validate` exits non-zero when the index is missing, stale, or out of sync
- `get` and `query` emit a line-oriented CityJSON stream

Explicit low-level mode is still available when you want to specify the layout directly:

```bash
cjindex get \
  --layout feature-files \
  --root /data/feature-files \
  --index /tmp/cjindex.sqlite \
  --id NL.IMBAG.Pand.0503100000012869-0
```

## Storage Layouts

### CityJSONSeq / NDJSON

Each `.city.jsonl` file begins with metadata, followed by one feature per line. The index stores byte offsets for each feature line.

### CityJSON

Regular CityJSON files share a vertices array and a `CityObjects` dictionary. The index stores the feature package ranges and reconstructs the requested model on read.

### Feature Files

Each feature lives in its own file. Metadata is discovered through ancestor `.json` files and cached in the SQLite index.

## Development

This repository includes helper binaries and benchmarks under `src/bin/` and `benches/`.
They are gated behind the `dev-binaries` feature so the published CLI stays focused on `cjindex`.
The release-facing FFI and Python packaging live under `ffi/`.

Useful local commands:

```bash
just check
just lint
just test
just ffi
just ci
just prep-test-data
```

`just prep-test-data` and `just bench-release` require the `dev-binaries` feature.

## Use of AI in this project

This crate was written with AI assistance and human guidance.
Development used an iterative process of testing, benchmarking, and optimization controlled and verified by me.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contributing

Contributions are welcome.
Please keep changes focused, add tests when behavior changes, and run `just ci` before opening a pull request.
