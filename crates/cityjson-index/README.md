# cjindex

`cjindex` provides random-access retrieval of CityJSON features by identifier or bounding box, backed by a persistent SQLite index.

It works with three on-disk storage layouts behind a single query interface: CityJSONSeq (`NDJSON`), regular CityJSON, and individual CityJSONFeature files. The caller always gets back a `cjlib::CityModel` containing exactly one CityObject with its metadata, transform, and vertices.

## Motivation

CityJSON datasets at national scale are typically split into tiles and stored as flat files. Retrieving a single feature by ID requires scanning entire files. Spatial queries require loading and filtering whole tiles.

`cjindex` solves this by maintaining a sidecar SQLite index that maps feature IDs to byte offsets within source files and stores per-feature bounding boxes in an R*Tree. Queries resolve to a direct `pread` with no scanning and no loading of unnecessary data.

The library does not modify, convert, or copy source files. They remain the source of truth in whatever layout they already use.

## Usage

```rust
use cjindex::{BBox, CityIndex, StorageLayout};
use std::path::Path;

let index = CityIndex::open(
    StorageLayout::Ndjson {
        paths: vec!["tiles/0566.city.jsonl".into(), "tiles/0599.city.jsonl".into()],
    },
    Path::new("index.sqlite"),
)?;

let model = index.get("NL.IMBAG.Pand.0566100000032571")?;
let models = index.query(&BBox {
    min_x: 84710.0,
    max_x: 84757.0,
    min_y: 446846.0,
    max_y: 446944.0,
})?;

for result in index.query_iter(&BBox {
    min_x: 84710.0,
    max_x: 84757.0,
    min_y: 446846.0,
    max_y: 446944.0,
})? {
    let model = result?;
    let _ = model;
}

index.reindex()?;
```

## Storage Layouts

`cjindex` supports three ways of storing CityJSON data on disk. Each has different trade-offs. The index and query interface is identical regardless of layout.

### NDJSON / CityJSONSeq

```text
tiles/
├── 0566.city.jsonl
├── 0599.city.jsonl
└── 0637.city.jsonl
```

Each `.city.jsonl` file follows the CityJSONSeq specification. The first line is the metadata object, including CRS and transform. Every subsequent line is a self-contained CityJSONFeature with one CityObject and its own local vertices.

**Indexing.** The indexer reads each file line by line, recording the byte offset and length of every feature line. It parses each line just enough to extract the feature ID and compute a bounding box from its vertices and the file's transform.

**Reading.** A lookup seeks directly to the byte offset and reads exactly the feature's bytes. The line deserializes into a `CityModel`. Metadata from the file's first line is attached from the index cache.

**Trade-offs.** This is the simplest and fastest layout. Each feature is self-contained, so reads are a single `pread` plus `serde_json::from_slice`. The downside is that vertices are duplicated across features that share them, but at tile granularity that overhead is usually acceptable.

### CityJSON

```text
tiles/
├── 0566.city.json
├── 0599.city.json
└── 0637.city.json
```

Regular CityJSON files with a shared `"vertices"` array and a `"CityObjects"` dictionary.

**Indexing.** The indexer parses each file to extract metadata and the byte range of the `"vertices"` array. It then walks the `"CityObjects"` dictionary, recording each entry's byte offset and length. Bounding boxes are computed by resolving each object's boundary vertex indices into the shared array and applying the transform.

**Reading.** A lookup reads two byte ranges from the same file: the CityObject's bytes and the shared vertices array. The shared vertices are cached, keyed by file path, so repeated lookups within the same tile do not re-read them. The reader collects only the vertex indices referenced by the object's geometry, builds a local vertices array, remaps the boundary indices, and assembles a `CityModel` with one CityObject.

**Trade-offs.** Source files are smaller because there is no vertex duplication, and the layout is compatible with tools that do not support CityJSONSeq. The read path is more complex due to vertex remapping, and the indexer needs a position-aware JSON parser to locate entries within the `"CityObjects"` dictionary. Batch queries within the same tile benefit from the vertices cache.

### Feature Files

```text
features/
├── metadata.city.json
├── 0566/
│   ├── metadata.city.json
│   ├── NL.IMBAG.Pand.0566100000032571.city.jsonl
│   └── NL.IMBAG.Pand.0566100000032572.city.jsonl
└── 0599/
    ├── NL.IMBAG.Pand.0599100000023396.city.jsonl
    └── NL.IMBAG.Pand.0599100000023397.city.jsonl
```

One CityJSONFeature per file in a directory hierarchy. Metadata is stored in separate CityJSON files with no CityObjects, just CRS, transform, and any other top-level properties. Metadata files are discovered via a glob pattern and resolved by nearest ancestor: a feature uses the metadata file closest to it in the directory tree.

**Indexing.** The indexer walks the directory tree using `ignore::WalkBuilder`, which respects `.gitignore` files. It collects metadata files matching the metadata glob, then feature files matching the feature glob. Each feature file's metadata is resolved by finding the nearest ancestor metadata file in the directory hierarchy. Since each file is one feature, the byte offset is always 0 and the length is the file size.

**Reading.** A lookup reads the entire file and deserializes it. Metadata is attached from the index cache.

**Trade-offs.** The filesystem acts as a natural index: features are individually addressable by path, easy to serve over HTTP, and easy to update incrementally. The downside is filesystem overhead at scale, especially inode pressure, wasted block space, and slow directory traversal.

## Metadata Handling

Each storage layout stores CityJSON metadata differently:

- **NDJSON**: the first line of each file is the metadata object.
- **CityJSON**: metadata is part of the top-level JSON object.
- **Feature files**: metadata lives in separate `.city.json` files discovered by glob pattern. Each feature resolves to the nearest ancestor metadata file.

During indexing, metadata is parsed and stored as JSON in the `sources` table. At query time, metadata is deserialized from this cache and attached to the returned `CityModel`. The caller always gets a complete `CityModel` with metadata, regardless of layout.

```rust
let all_meta = index.metadata()?;
```

## Index Structure

The index is a single SQLite file with the following tables:

- `sources` - one row per source file, or per metadata file for the feature-files layout. Stores the file path and a cached copy of the parsed metadata as JSON. For the CityJSON layout, also stores the byte range of the shared vertices array.
- `features` - one row per CityObject. Maps feature ID to its source file and byte range.
- `feature_bbox` - an R*Tree virtual table for spatial queries. Stores 2D bounding boxes in CRS coordinates, with the transform already applied during indexing.
- `bbox_map` - joins R*Tree rowids back to feature IDs, because R*Tree rows only support numeric columns.

The index can be deleted and rebuilt at any time with `reindex()`. It is not a cache; it is derived entirely from the source files.

## Rebuilding the Index

```rust
index.reindex()?;
```

`reindex()` drops all existing index data and rescans every source file. It is the only way to update the index; there is no incremental update mechanism. For very large datasets, a full reindex is dominated by JSON parsing and SQLite batch inserts.

## Dependencies

- `cjlib` - local CityJSON types and serialization
- `rusqlite` - SQLite bindings, with bundled SQLite for R*Tree support
- `serde_json` - JSON parsing
- `globset` - glob pattern matching
- `ignore` - `.gitignore`-aware directory walking
- `lru` - LRU cache for shared vertices in the CityJSON backend
- `memmap2` - memory-mapped file reads

## Current Scope

- `CityIndex` is the top-level API.
- `StorageLayout` describes the supported source layouts.
- The backing index is planned to use SQLite plus an R*Tree for bounding-box lookup.
- The storage backends are split by source layout: NDJSON, CityJSON, and feature-file trees.

## Dependency

`cjindex` is intended to extend the local `cjlib` crate:

```toml
cjlib = { path = "/home/balazs/Development/cjlib" }
```

## Status

This repository is still a scaffold, not a finished implementation. The public types and fixture prep are in place, but the index and storage backends still need to be implemented.
