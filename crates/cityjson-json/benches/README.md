# Benchmarks

This directory contains the Criterion suites for the current `serde_cityjson`
crate.

## Suites

- `bench-read` measures `from_str_owned`, `from_str_borrowed` where valid, and
  `serde_json::Value`
- `bench-write` measures `to_string`, `to_string_validated`, and
  `serde_json::to_string`

## Data

- real-world performance inputs are consumed from the shared
  `../cityjson-benchmarks/artifacts/benchmark-index.json`
- the shared corpus repo publishes the generated workload outputs and the
  acquired real-world CityJSON workload artifacts
- no local bootstrap download is needed for the benchmark harness

## Workflow

1. `just bench-read`
2. `just bench-write`
3. `just bench-report`

The reporting script writes plots and a Markdown summary into
`benches/results/`.
