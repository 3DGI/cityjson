# Manifest-Driven Read and Write Bench Suite

## Status

Accepted

## Related Commits

- `97b64db` initial split into read/write Criterion benches
- `d768f46` manifest-driven profile loading and suite cleanup

## Context

The previous benchmark setup mixed several concerns in one place:

- read and write benchmarks lived in one Criterion harness
- synthetic cases were encoded as Rust constructor functions
- the benchmark catalog was duplicated across code, manifest notes, and README text
- report generation assumed a fixed benchmark layout and hardcoded labels

That made the suite difficult to evolve. A change to a synthetic case required
editing Rust, JSON documentation, and reporting code separately. It also made it
easy for the catalog to drift from the actual benchmark inputs.

The new benchmark goals were:

- keep the real-world regression datasets
- keep synthetic cases deterministic and reproducible
- make the manifest the source of truth for what the suite contains
- split read and write measurement paths so each suite prepares only what it uses
- keep reporting deterministic enough to regenerate README tables and plots

## Decision

The benchmark suite was restructured around a manifest-driven catalog plus two
separate Criterion entrypoints:

1. `benches/read.rs` measures deserialization with:
   - `cityjson-json::from_str_owned`
   - `cityjson-json::from_str_borrowed` where valid
   - `serde_json::Value` as the baseline
2. `benches/write.rs` measures serialization with:
   - `cityjson-json::to_string`
   - `cityjson-json::to_string_validated`
   - `serde_json::to_string` as the baseline

The shared benchmark index in
`../cityjson-benchmarks/artifacts/benchmark-index.json` is now the catalog of
benchmark workload cases. It declares:

- case id
- description
- the path to the benchmark input file
- whether the case came from a generated or acquired source

The benchmark harness reads those files directly. Synthetic cases are already
materialized by the shared corpus repo, so `cityjson-json` no longer needs a
local `cjfake` generation step.

The shared benchmark module prepares data outside the timed closure and writes
suite metadata into `benches/results/suite_metadata_*.json`. The reporting
script consumes those metadata files together with Criterion estimates to
generate:

- `benches/results/speed_relative_read.png`
- `benches/results/speed_relative_write.png`
- `benches/results/benchmark_summary.md`

## Consequences

Good:

- the manifest is the single source of truth for benchmark membership
- read and write suites now prepare only the data they need
- synthetic cases are deterministic and separated from the Rust harness logic
  - the reporting layer can be regenerated from the recorded Criterion output
  - README benchmark tables can be refreshed mechanically instead of edited by hand

Trade-offs:

- the harness now includes a small shared-index loader
- the benchmark setup is more explicit than the previous hardcoded constructors

Rejected alternatives:

- keeping all benchmark cases hardcoded in Rust
- keeping a local benchmark corpus mirror in `tests/data/generated/`

## Notes

The manifest catalog is intentionally about benchmark structure, not benchmark
results. The measured timings and throughput numbers remain separate artifacts
under `benches/results/`.
