# cityjson-index

`cityjson-index` is a Rust crate for indexing CityJSON datasets with a persistent SQLite sidecar. It also ships `cjindex`, a command-line tool for inspecting, indexing, and querying supported datasets.

## Install

### Library

```toml
[dependencies]
cityjson-index = "0.3.0"
```

### CLI

```bash
cargo install cityjson-index --bin cjindex
```

Or run it from a checkout:

```bash
cargo run --bin cjindex -- --help
```

## What It Does

- Detects the supported storage layout under a dataset directory
- Builds or refreshes a `.cityjson-index.sqlite` sidecar
- Retrieves features by identifier
- Streams features that intersect a bounding box
- Prints dataset metadata and index health information

The supported layouts are:

- CityJSONSeq / NDJSON
- regular CityJSON
- individual CityJSONFeature files

## CLI

The default workflow is dataset-oriented:

```bash
cjindex inspect /data/3dbag
cjindex index /data/3dbag
cjindex reindex /data/3dbag
cjindex validate /data/3dbag
cjindex get /data/3dbag --id NL.IMBAG.Pand.0503100000012869-0
cjindex query /data/3dbag --min-x 4.4 --max-x 4.5 --min-y 51.8 --max-y 51.9
cjindex metadata /data/3dbag
```

`inspect` reports the detected layout, counts, freshness, and coverage.
`validate` performs the same checks but exits non-zero when the index is missing, stale, or out of sync.

`get` and `query` emit a line-oriented CityJSON stream:

- the first record is a `CityJSON` document containing metadata and an empty `CityObjects` object
- each following record is a `CityJSONFeature`

The explicit low-level mode is still available when you want to specify the layout directly:

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

Useful local commands:

```bash
just check
just lint
just test
just ci
just prep-test-data
```

`just prep-test-data` and `just bench-release` require the `dev-binaries` feature.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
