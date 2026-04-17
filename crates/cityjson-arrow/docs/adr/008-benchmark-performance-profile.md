# Benchmark Performance Profile

## Status

Accepted

## Context

The first full benchmark run against the shared corpus (`cityjson-corpus`
`artifacts/benchmark-index.json`) covered six synthetic stress cases and three
acquired real-world datasets. Implementations compared:

- `cityjson-arrow` stream read/write (`read_stream` / `write_stream`)
- `cityjson-parquet` package read/write (`read_file` / `write_file`)
- `cityjson-json` owned model parse/serialize (baseline)

All numbers are Criterion mean throughput on the same machine. The corpus index
also contained real-world 3dbag and basisvoorziening files.

Detailed results are in `benches/results/benchmark_summary.md`. The findings
below describe the structural pattern, not individual numbers.

## Findings

**Read is uniformly faster.** Arrow IPC and Parquet are 2–7× faster than
cityjson-json on every case. The binary columnar layout avoids JSON tokenizing
and the per-character allocation it causes. Parquet and Arrow IPC deliver
near-identical read performance (within 5%), since both decode to the same
in-memory `OwnedCityModel` via the same import path.

**Write splits by workload type.**

- Geometry/boundary/vertex-heavy cases: Arrow IPC is 2–5× faster than JSON,
  because large numeric arrays (vertex coordinates, boundary indices) pack
  contiguously into Arrow buffers without per-element formatting.
- Real-world attribute-rich cases (3dbag, basisvoorziening): Arrow IPC is
  2–3× slower than JSON. These files have many heterogeneous string and
  numeric attribute columns. The projection discovery phase iterates all
  city objects before encoding begins, and each string column requires
  per-value allocation into `LargeStringArray`. cityjson-json writes a
  single-pass streaming serialization with no schema inference step.
- `stress_attribute_heavy` is the worst write outlier (~5× slower than JSON)
  because it combines many attribute columns with the `Json` fallback
  serialization path (heterogeneous list — see ADR 007).

**Parquet adds modest write overhead over Arrow IPC** (~15–30%) due to Snappy
column compression and file footer assembly. On read, the Parquet decoder is
within noise of Arrow IPC.

## Decision

Accept the current performance profile as the baseline for 0.6.x.

No architectural changes are made in response to these numbers. The read
advantage is the primary use case for the Arrow/Parquet layer (query-engine
integration, columnar analytics), and it holds across all workloads.

The write disadvantage on attribute-heavy real-world data is a known structural
cost of schema-inference-before-encode. Possible future mitigations
(streaming attribute inference, lazy column allocation) are tracked separately
and are not in scope for this release.

## Consequences

- The README benchmark table shows read throughput only, because that is the
  primary transport use case. The full read and write breakdown is in
  `benches/results/benchmark_summary.md`.
- Write performance for attribute-heavy real-world data (3dbag, basisvoorziening)
  is expected to lag cityjson-json by 2–3×. Callers that need fast write
  throughput on such data should evaluate whether the Parquet/Arrow encoding
  step can be deferred or pipelined.
- The `stress_attribute_heavy` write result is not representative of typical
  city model data; it stresses the `Json` fallback path that exists only for
  CityJSON files with intentionally heterogeneous attribute types.
