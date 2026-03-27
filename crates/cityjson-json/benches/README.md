# Benchmarks

This directory contains the Criterion suites for the current `serde_cityjson`
crate.

## Suites

- `bench-read` measures `from_str_owned`, `from_str_borrowed` where valid, and
  `serde_json::Value`
- `bench-write` measures `to_string`, `to_string_validated`, and
  `serde_json::to_string`

## Data

- real-world regression inputs live in `tests/data/downloaded/`
- synthetic benchmark profiles live in `tests/data/generated/`
- `cjfake` generates the synthetic inputs deterministically at benchmark time

## Workflow

1. `just download`
2. `just bench-read`
3. `just bench-write`
4. `just bench-report`

The reporting script writes plots and a Markdown summary into
`benches/results/`.
