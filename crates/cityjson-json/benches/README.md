# Benchmarks

This directory contains the Criterion suites for the current `serde_cityjson`
crate.

## Suites

- `bench-read` measures `from_str_owned`, `from_str_borrowed` where valid, and
  `serde_json::Value`
- `bench-write` measures `to_string`, `to_string_validated`, and
  `serde_json::to_string`

## Data

- real-world regression inputs are consumed from the shared
  `../cityjson-benchmarks/artifacts/benchmark-index.json`
- the shared corpus repo publishes the generated benchmark outputs and the
  raw acquired 3DBAG file paths
- the local 3D Basisvoorziening bootstrap data still lives in
  `tests/data/downloaded/`

## Workflow

1. `just download`
2. `just bench-read`
3. `just bench-write`
4. `just bench-report`

The reporting script writes plots and a Markdown summary into
`benches/results/`.
