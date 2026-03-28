# cjindex

Random-access retrieval of CityJSON features by identifier or bounding box, backed by a persistent SQLite index.

Works with three on-disk storage layouts — CityJSONSeq (NDJSON), regular CityJSON, and individual CityJSONFeature files — behind a single query interface. The caller always gets back a `cityjson_rs::CityModel` containing exactly one CityObject with its metadata, transform, and vertices.

## Motivation

CityJSON datasets at national scale (10M+ buildings) are typically split into tiles and stored as flat files. Retrieving a single feature by ID requires scanning entire files. Spatial queries require loading and filtering whole tiles.

cjindex solves this by maintaining a sidecar SQLite index that maps feature IDs to byte offsets within source files and stores per-feature bounding boxes in an R\*Tree. Queries resolve to a direct `pread` — no scanning, no loading unnecessary data.

The library does not modify, convert, or copy source files. They remain the source of truth in whatever layout they already use.

## Usage

```rust
use cjindex::{CityIndex, StorageLayout};
use std::path::Path;

// Open or create an index over NDJSON files.
let index = CityIndex::open(
    StorageLayout::ndjson(&["tiles/0566.city.jsonl", "tiles/0599.city.jsonl"]),
    Path::new("index.sqlite"),
)?;

// Retrieve a single feature by ID.
// Returns a CityModel with one CityObject, metadata, and vertices.
let model = index.get("NL.IMBAG.Pand.0566100000032571")?;

// Spatial query — all features intersecting a bounding box.
let models = index.query(&BBox::new(84710.0, 446846.0, 84757.0, 446944.0))?;

// Lazy spatial query for large result sets.
for result in index.query_iter(&BBox::new(84710.0, 446846.0, 84757.0, 446944.0))? {
    let model = result?;
    // ...
}

// Rebuild the index from scratch (e.g. after source files changed).
index.reindex()?;
```

## Storage layouts

cjindex supports three ways of storing CityJSON data on disk. Each has different trade-offs. The index and query interface is identical regardless of layout.

### NDJSON (CityJSONSeq)

```
tiles/
├── 0566.city.jsonl
├── 0599.city.jsonl
└── 0637.city.jsonl
```

Each `.city.jsonl` file follows the CityJSONSeq specification. The first line is the metadata object (CRS, transform). Every subsequent line is a self-contained CityJSONFeature — a valid CityJSON document with one CityObject and its own local vertices.

```rust
let index = CityIndex::open(
    StorageLayout::ndjson(&["tiles/0566.city.jsonl", "tiles/0599.city.jsonl"]),
    Path::new("index.sqlite"),
)?;
```

**Indexing.** The indexer reads each file line by line, recording the byte offset and length of every feature line. It parses each line just enough to extract the feature ID and compute a bounding box from its vertices and the file's transform.

**Reading.** A lookup seeks directly to the byte offset and reads exactly the feature's bytes. The line deserializes into a `CityModel`. Metadata (CRS, transform) from the file's first line is attached from the index cache.

**Trade-offs.** This is the simplest and fastest layout. Each feature is self-contained, so reads are a single `pread` + `serde_json::from_slice`. The only downside is that vertices are duplicated across features that share them — but at tile granularity this overhead is negligible.

### CityJSON

```
tiles/
├── 0566.city.json
├── 0599.city.json
└── 0637.city.json
```

Regular CityJSON files with a shared `"vertices"` array and a `"CityObjects"` dictionary.

```rust
let index = CityIndex::open(
    StorageLayout::cityjson(&["tiles/0566.city.json", "tiles/0599.city.json"]),
    Path::new("index.sqlite"),
)?;
```

**Indexing.** The indexer parses each file to extract metadata and the byte range of the `"vertices"` array. It then walks the `"CityObjects"` dictionary, recording each entry's byte offset and length. Bounding boxes are computed by resolving each object's boundary vertex indices into the shared array and applying the transform.

**Reading.** A lookup reads two byte ranges from the same file: the CityObject's bytes and the shared vertices array. The shared vertices are cached (LRU, keyed by file path) so that repeated lookups within the same tile don't re-read them. The reader collects only the vertex indices referenced by the object's geometry, builds a local vertices array, remaps the boundary indices, and assembles a `CityModel` with one CityObject.

**Trade-offs.** Source files are smaller (no vertex duplication) and compatible with tools that don't support CityJSONSeq. The read path is more complex due to vertex remapping, and the indexer needs a position-aware JSON parser to locate entries within the `"CityObjects"` dictionary. Batch queries within the same tile benefit from the vertices cache.

### Feature files

```
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

One CityJSONFeature per file in a directory hierarchy. Metadata is stored in separate CityJSON files (no CityObjects, just CRS, transform, and any other top-level properties). Metadata files are discovered via a glob pattern and resolved by nearest ancestor — a feature uses the metadata file closest to it in the directory tree.

```rust
let index = CityIndex::open(
    StorageLayout::feature_files(
        "features/",
        "**/metadata.city.json",
        "**/*.city.jsonl",
    ),
    Path::new("index.sqlite"),
)?;
```

**Indexing.** The indexer walks the directory tree (using `ignore::WalkBuilder`, which respects `.gitignore` files). It collects all metadata files matching the metadata glob, then all feature files matching the feature glob. Each feature file's metadata is resolved by finding the nearest ancestor metadata file in the directory hierarchy. Since each file is one feature, the byte offset is always 0 and the length is the file size.

**Reading.** A lookup reads the entire file and deserializes it. Metadata is attached from the index cache.

**Trade-offs.** The filesystem acts as a natural index — features are individually addressable by path, trivially servable over HTTP, and easy to update incrementally (just write a file). The downsides are filesystem overhead at scale (10M small files means inode pressure, wasted block space, slow directory traversal) and the need for a separate metadata discovery mechanism.

## Metadata handling

Each storage layout stores CityJSON metadata (CRS, transform, extensions, extra properties) differently:

- **NDJSON**: the first line of each file is the metadata object.
- **CityJSON**: metadata is part of the top-level JSON object.
- **Feature files**: metadata lives in separate `.city.json` files discovered by glob pattern. Each feature resolves to the nearest ancestor metadata file.

During indexing, metadata is parsed and stored as JSON in the `sources` table. At query time, metadata is deserialized from this cache and attached to the returned `CityModel`. The caller always gets a complete `CityModel` with metadata, regardless of layout.

```rust
// Access metadata for all indexed sources.
let all_meta = index.metadata()?;
```

## Index structure

The index is a single SQLite file with the following tables:

- **`sources`** — one row per source file (or per metadata file for the feature files layout). Stores the file path and a cached copy of the parsed metadata as JSON. For the CityJSON layout, also stores the byte range of the shared vertices array.
- **`features`** — one row per CityObject. Maps feature ID to its source file and byte range (offset + length).
- **`feature_bbox`** — an R\*Tree virtual table for spatial queries. Stores 2D bounding boxes in CRS coordinates (transform already applied during indexing).
- **`bbox_map`** — joins R\*Tree rowids back to feature IDs (R\*Tree rows only support numeric columns).

The index can be deleted and rebuilt at any time with `reindex()`. It is not a cache — it is derived entirely from the source files.

## Rebuilding the index

```rust
// Rebuild after source files have changed.
index.reindex()?;
```

`reindex()` drops all existing index data and rescans every source file. It is the only way to update the index — there is no incremental update mechanism. For 10M features, a full reindex takes on the order of minutes (dominated by JSON parsing and SQLite batch inserts at ~1M rows/sec).

## Dependencies

- [`cityjson-rs`](https://github.com/3DGI/cityjson-rs) — CityJSON types and (de)serialization
- [`rusqlite`](https://crates.io/crates/rusqlite) — SQLite bindings (with bundled SQLite for R\*Tree support)
- [`serde_json`](https://crates.io/crates/serde_json) — JSON parsing
- [`globset`](https://crates.io/crates/globset) — glob pattern matching
- [`ignore`](https://crates.io/crates/ignore) — `.gitignore`-aware directory walking
- [`lru`](https://crates.io/crates/lru) — LRU cache for shared vertices (CityJSON backend)
- [`memmap2`](https://crates.io/crates/memmap2) — memory-mapped file reads
