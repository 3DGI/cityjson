# Status Report

## Current State

`cityjson-parquet` is the persistent package layer for `cityjson-rs`. It
produces and reads seekable single-file packages whose table payloads use Arrow
IPC framing and whose manifest is located via a fixed-size binary footer.

- `PackageWriter` encodes any `OwnedCityModel` to a `.cityjson-parquet` file
- `PackageReader` decodes the file back to an `OwnedCityModel` or a manifest
- `read_package_manifest` reads only the footer and manifest JSON without
  loading any table payload
- `spatial::SpatialIndex` constructs a Hilbert-curve ordered spatial index from
  a decoded `CityModelArrowParts`

## What Changed In This Slice

- added repo infrastructure: README.md, CLAUDE.md, STATUS.md, CHANGELOG.md,
  justfile, .gitignore, pyproject.toml, properdocs.yml
- added docs/ with index, API overview, package spec, schema, and design docs
- fixed `Cargo.toml` bug: `readme` now points to this repo's own `README.md`
  instead of `../cityjson-arrow/README.md`

## Verification Snapshot

The tree is expected to pass:

- `just fmt`
- `just lint`
- `just check`
- `just test`   (requires `../cityjson-arrow` as a sibling checkout)
- `just rustdoc`

## Remaining Limits

- no streaming write path; full model materialisation happens before the first
  byte is written
- `SpatialIndex::query` is a linear scan; Hilbert ordering is not exploited for
  interval pruning
- `cityjson-parquet` depends on doc-hidden `cityjson-arrow` internals; a clean
  public API boundary between the two crates does not yet exist
- tests require both repos checked out as siblings; there is no standalone test
  corpus
