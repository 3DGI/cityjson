# Benchmark Performance Profile

## Status

Accepted

## Context

The first full benchmark run against the shared corpus (`cityjson-corpus`
`artifacts/benchmark-index.json`) measured `cityjson-parquet` package
read/write alongside `cityjson-arrow` stream and `cityjson-json` owned-model
parse/serialize as the baseline. Six synthetic stress cases and three acquired
real-world datasets were included.

A blocking bug was discovered before benchmarks could run: `write_file` panicked
on the `stress_attribute_heavy` corpus case because the underlying
`cityjson-arrow` projection layer raised an error on heterogeneous list
attributes (e.g. `[607, false, 28.47]` — mixed int, bool, float). The fix
landed in `cityjson-arrow` (see its ADR 007): numeric widening rules and a
`ProjectedValueSpec::Json` fallback that encodes incompatible-type values as
JSON strings in a `LargeUtf8` column. `cityjson-parquet` inherits this fix
through its `cityjson-arrow` dependency.

Detailed results are in `benches/results/benchmark_summary.md`.

## Findings

**Read is uniformly faster than cityjson-json.** Parquet and Arrow IPC deliver
near-identical read throughput (within 5%) across all cases — both decode to
the same `OwnedCityModel` through the same `cityjson-arrow` import path. Both
are 2–7× faster than cityjson-json's owned model parser.

**Write splits by workload type.**

- Geometry/boundary/vertex-heavy cases: Parquet is faster than JSON, but
  meaningfully slower than Arrow IPC (Snappy column compression and file
  footer assembly add ~15–30% overhead over the raw Arrow IPC stream).
- Real-world attribute-rich cases (3dbag, basisvoorziening): Parquet is
  2–4× slower than JSON. The schema-inference-before-encode step and the
  per-column `LargeStringArray` allocation dominate; JSON writes in a single
  streaming pass with no schema step.
- `stress_attribute_heavy` write is ~5× slower than JSON because the
  `Json` fallback path (see ADR 007) adds `serde_json::to_string` cost per
  attribute value.

**Parquet vs Arrow IPC:** read is equivalent; write is 15–30% slower. Callers
that need fast write throughput and are not constrained to Parquet should
prefer Arrow IPC stream output.

## Decision

Accept the current performance profile as the baseline for 0.6.x.

No architectural changes are made in response to these numbers. The Parquet
encoding exists to satisfy query-engine compatibility (DuckDB, Spark,
pandas/pyarrow), not to maximise raw write throughput. The read advantage
over cityjson-json is the primary use case and holds uniformly.

## Consequences

- The README benchmark table shows read throughput only; the full breakdown
  is in `benches/results/benchmark_summary.md`.
- Callers writing large attribute-rich city models to Parquet should expect
  2–4× lower throughput than cityjson-json serialization. If write latency
  is critical, pipelining the Parquet encoding step is advisable.
- `cityjson-parquet` inherits `ProjectedValueSpec::Json` from `cityjson-arrow`
  0.6.x. Parquet files written with this version may contain `LargeUtf8`
  columns for heterogeneous attributes that older readers do not expect.
