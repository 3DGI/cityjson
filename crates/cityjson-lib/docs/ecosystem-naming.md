# CityJSON Ecosystem Naming Map

This repository belongs to a broader CityJSON family of crates and tools.
The naming goal is simple:

- make the CityJSON relationship obvious immediately
- show which project owns which layer of the stack
- keep repo names consistent enough that the ecosystem reads as one system

## Naming Rule

Use `cityjson-` as the shared prefix for public repository names.

That gives the ecosystem one visible brand while still leaving room for role
suffixes such as `-rs`, `-json`, `-arrow`, `-index`, and `-benchmarks`.

## Rename Map

The map below treats the current names as the source of truth and the
`cityjson-*` names as the recommended public family.

| Current name | Proposed name | Role |
| --- | --- | --- |
| `cityjson-rs` | `cityjson-rs` | Semantic CityJSON model family and invariants |
| `serde_cityjson` | `cityjson-json` | JSON and JSONL parsing, probing, and serialization |
| `cjlib` | `cityjson-lib` | Central Rust-facing library, explicit format modules, and shared FFI core |
| `cityarrow` / `cityparquet` | `cityjson-arrow` / `cityjson-parquet` | Columnar transport and storage layers |
| `cjfake` | `cityjson-fake` | Synthetic data generation and fixture shaping |
| `cjindex` | `cityjson-index` | Corpus shaping, reshaping, and indexing |
| `cjlib-benchmarks` | `cityjson-benchmarks` | Benchmark drivers, workloads, and corpus harnesses |

## Why These Names

The proposed names separate concerns without hiding the relationship to the
shared CityJSON ecosystem:

- `cityjson-rs` stays the semantic core.
- `cityjson-json` makes the wire-format boundary explicit.
- `cityjson-lib` signals that the repo is the main Rust library for the
  ecosystem without pretending to be the semantic model itself.
- `cityjson-arrow` and `cityjson-parquet` identify format-specific boundary
  layers instead of generic storage helpers.
- `cityjson-fake`, `cityjson-index`, and `cityjson-benchmarks` read as
  supporting infrastructure around the same family.

## Transitional Note

The rename map is about public identity, not an immediate mechanical rewrite.
Crate names, package names, and filesystem paths can keep their current forms
while the repository and documentation branding converge on `cityjson-*`.
