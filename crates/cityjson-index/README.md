# cjindex

`cjindex` provides a persistent indexing layer on top of `cjlib`.

The goal is to store feature locations, metadata, and bounding boxes in a queryable index so
CityJSON data can be opened by identifier or spatial extent without scanning the whole source
every time.

This repository is scaffolded from the design sketch in `DESIGN.rs`.

## Current scope

- `CityIndex` is the top-level API.
- `StorageLayout` describes the supported source layouts.
- The backing index is planned to use SQLite plus an R-tree for bounding-box lookup.
- The storage backends are split by source layout: NDJSON, CityJSON, and feature-file trees.

## Dependency

`cjindex` is intended to extend the local `cjlib` crate:

```toml
cjlib = { path = "/home/balazs/Development/cjlib" }
```

## Status

This is a scaffold, not a finished implementation. The public types are in place, but the index
and storage backends still need to be implemented.

