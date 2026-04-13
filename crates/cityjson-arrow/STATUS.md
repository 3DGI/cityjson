# Status Report

## Current State

`cityjson-arrow` and `cityjson-parquet` now share one internal canonical table contract
while exposing only owned semantic APIs.

- live Arrow transport is `ModelEncoder` / `ModelDecoder`
- persistent package transport is `PackageWriter` / `PackageReader`
- the package schema id is `cityjson-arrow.package.v2alpha2`
- canonical tables are internal and doc-hidden

## What Changed In The Current Slice

- `ModelEncoder` no longer routes through a public-style parts aggregate before
  stream serialization
- `ModelDecoder` no longer rebuilds full canonical parts before semantic import
- the live stream path no longer uses eager `read_to_end`
- the live stream writer no longer buffers every serialized table payload before
  writing
- package writes are direct-to-file with manifest entries collected from file
  offsets
- package manifest reads are footer-first
- package reads now feed ordered canonical batches directly into the incremental
  decoder
- split benchmarks now exist for conversion-only, transport-only, and
  end-to-end paths

## Verification Snapshot

The current tree is expected to pass:

- `just lint`
- `just test`

The benchmark surface now includes:

- `convert_encode_parts`
- `convert_decode_parts`
- `stream_write_model`
- `stream_read_model`
- `stream_write_parts`
- `stream_read_parts`
- `package_write_model`
- `package_read_model`
- `package_write_parts`
- `package_read_parts`
- `package_read_manifest`

## Remaining Limits

- conversion still materializes canonical rows and record batches in memory
- geometry and appearance reconstruction still uses grouped sidecar staging by
  canonical ids
- the contract is still alpha and intentionally not frozen as a stable archival
  format
