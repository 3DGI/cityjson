# cityarrow

`cityarrow` is the live Arrow IPC transport layer for `cityjson-rs`.
`cityparquet` is the sibling crate for the persistent single-file package
boundary.

The semantic API stays centered on `cityjson::v2_0::OwnedCityModel`.
Canonical Arrow tables remain internal and are shared between the live stream
and package implementations.

## Public Surface

- `cityarrow::ModelEncoder` and `cityarrow::ModelDecoder`
- `cityparquet::PackageWriter` and `cityparquet::PackageReader`
- shared schema and manifest types from `cityarrow::schema`

## Current Architecture

- live transport uses a small JSON prelude followed by ordered Arrow IPC table
  frames
- persistent transport uses one seekable package file with table payloads,
  manifest-at-end metadata, and a footer index
- both readers drive the same incremental decoder over ordered canonical table
  batches
- `cityarrow::internal` keeps doc-hidden conversion and transport hooks for
  sibling crates and split benchmarks

## Current Status

- package schema id: `cityarrow.package.v3alpha2`
- the public `to_parts` / `from_parts` surface is gone
- live stream read no longer uses eager `read_to_end`
- live stream and package writes no longer buffer every serialized table payload
  before writing
- package manifest reads are footer-first instead of full-file scans

## Verification

The repository currently keeps:

- integration roundtrip tests for the live stream and package APIs
- shared-corpus roundtrip tests for curated transport-safe conformance fixtures
- a split benchmark target in `benches/split.rs` for conversion-only,
  transport-only, and end-to-end measurements
- `just lint`
- `just test`

The shared-corpus test and bench helpers look for the sibling
`cityjson-benchmarks` checkout by default. Override the defaults with:

- `CITYARROW_SHARED_CORPUS_ROOT`
- `CITYARROW_CORRECTNESS_INDEX`
- `CITYARROW_BENCHMARK_INDEX`

## Repository Map

- `src/convert/mod.rs`: canonical export/import and incremental decoder
- `src/stream.rs`: live stream framing
- `src/transport.rs`: canonical table ids and transport helpers
- `src/schema.rs`: shared schema and manifest definitions
- `cityparquet/src/package/mod.rs`: persistent package implementation
- `tests/`: roundtrip tests
- `docs/`: ADRs, design notes, and format docs
