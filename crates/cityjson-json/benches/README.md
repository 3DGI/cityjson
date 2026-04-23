# Benchmarks

This directory contains the Criterion suites for the current `cityjson-json`
crate.

## Suites

- `bench-read` measures `read_model` and `serde_json::Value`
- `bench-write` measures `to_vec`, `to_vec_validated`, and
  `serde_json::to_string`

## Data

- real-world performance inputs are consumed from the benchmark index selected
  by `CITYJSON_JSON_BENCHMARK_INDEX`, or from
  `$CITYJSON_SHARED_CORPUS_ROOT/artifacts/benchmark-index.json`
- the shared corpus repo publishes the generated workload outputs and the
  acquired real-world CityJSON workload artifacts
- no local bootstrap download is needed for the benchmark harness
- ad hoc local inputs can be turned into a temporary benchmark index with
  `just bench-local /path/to/cityjson-or-directory`

## Workflow

1. `just bench-read`
2. `just bench-write`
3. `just bench-report`

For the shared benchmark suite, use `just bench` to refresh the plots, markdown
summary, and the benchmark snippet in the main README.

For local benchmarking against your own input, run one of:

1. `just bench-local /path/to/cityjson-or-directory`
2. `just bench-local-read /path/to/cityjson-or-directory`
3. `just bench-local-write /path/to/cityjson-or-directory`

The reporting script writes plots and a Markdown summary into
`benches/results/`. Only the shared-suite workflow updates the main `README.md`
benchmark snapshot.
