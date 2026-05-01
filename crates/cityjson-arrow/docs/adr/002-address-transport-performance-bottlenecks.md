# Address Transport Performance Bottlenecks Before Judging The Shared Model Boundary

## Status

Accepted

## Context

`cityjson-arrow` and `cityjson-parquet` exist to move `cityjson-rs` data through Arrow-
shaped transport while keeping `cityjson_types::v2_0::OwnedCityModel` as the shared
semantic boundary.

That design deliberately aims to stay close to a columnar representation. In
principle, it should not lose badly to a text decoder that starts from nested
JSON and still has to build the same shared model.

End-to-end benchmarks from the `cjlib` benchmark campaign on real 3DBAG data
show that the current implementation does lose:

- base tile read: `serde_cityjson` `33.48 ms`, `cityjson-arrow` `37.46 ms`,
  `cityjson-parquet` `40.78 ms`
- base tile write: `serde_cityjson` `9.19 ms`, `cityjson-arrow` `56.81 ms`,
  `cityjson-parquet` `66.86 ms`
- 4x cluster read: `serde_cityjson` `132.74 ms`, `cityjson-arrow` `152.79 ms`,
  `cityjson-parquet` `160.57 ms`
- 4x cluster write: `serde_cityjson` `41.92 ms`, `cityjson-arrow` `235.26 ms`,
  `cityjson-parquet` `271.72 ms`

An implementation review found that these results are not just a property of
the benchmark definition. The current transport implementation is leaving clear
performance on the table.

The main issues observed were:

- `from_parts` reconstructs the model through eager `Vec<Row>` materialization,
  grouping, cloning, and sorting rather than binding Arrow columns once and
  walking them directly
- hot read helpers repeatedly resolve columns by name through
  `RecordBatch::column_by_name` inside row loops
- some tables are reread after already being decoded once, for example when
  attaching geometries back to cityobjects
- per-geometry reconstruction clones and resorts grouped semantic and mapping
  rows instead of consuming them in a layout-aware way
- package readers eagerly collect all batches and concatenate them into fresh
  `RecordBatch` values before conversion
- package IO is directory-oriented and split over many canonical tables, so the
  format cost includes many file operations before model reconstruction starts

One concrete write-path issue was already identified and fixed: `to_parts`
previously recomputed geometry and template geometry export rows multiple times
across geometry, semantic, and appearance export passes. It now computes those
row sets once and reuses them.

That fix did not by itself erase the benchmark gap, which indicates that the
problem is broader than a single duplicated traversal.

## Decision

`cityjson-arrow` will treat the current benchmark shortfall as an implementation
problem in transport conversion and package IO, not as evidence that the shared
`OwnedCityModel` boundary or the columnar transport decomposition is
conceptually wrong.

The project will optimize and evaluate performance in three layers:

1. package IO only
2. `to_parts` and `from_parts` conversion only
3. end-to-end `format <-> OwnedCityModel`

The end-to-end benchmark remains the primary product metric, because the shared
model boundary is intentional. However, split benchmarks are required to
localize cost before architectural conclusions are drawn.

Implementation work should prioritize:

- binding required Arrow columns once per table instead of repeated name-based
  lookup
- reducing or eliminating intermediate `Vec<Row>` materialization on the read
  path
- avoiding repeated sorts, clones, and rereads of already decoded tables
- keeping export derivation single-pass wherever possible
- reconsidering package read behavior that eagerly concatenates all batches
- measuring the impact of package table granularity and file count separately
  from conversion cost

## Consequences

Good:

- benchmark interpretation stays honest without prematurely blaming the shared
  data model design
- optimization work has a concrete target list in the current implementation
- future benchmark results can distinguish format IO cost from reconstruction
  cost
- `serde_cityjson` remains a useful external performance bar for the shared
  model boundary

Trade-offs:

- the transport layer will likely need more specialized, less uniformly
  row-oriented code to become competitive
- some conversion code will become more complex because it will be organized
  around bound columns and reconstruction order instead of convenient temporary
  row structs
- package-format experiments may show that the current directory-of-tables
  layout carries fixed costs that are acceptable for interchange but poor for
  latency-sensitive end-to-end use

## Post-Acceptance Note: 2026-04-02

The first post-refactor `cjlib` run materially improved the native paths, so
the current evidence still supports this ADR's core reading.

- `cityjson-arrow` read time improved by about `22%` on both pinned 3DBAG cases
- `cityjson-parquet` read time improved by about `26%` to `29%`
- `cityjson-arrow` write improved by about `16%`
- `cityjson-parquet` write improved by about `27%` to `30%`

The same run also showed that the remaining write gap is still large and that
read allocation totals did not meaningfully change, which means the benchmark
story is still dominated by unsplit conversion and materialization cost.

That follow-up is recorded in
[ADR 2 and ADR 3 benchmark follow-up](../adr-002-003-benchmark-follow-up.md).

Follow-up work:

- add split benchmarks for `read_package_*`, `write_package_*`, `from_parts`,
  and `to_parts`
- profile `from_parts` after column binding to confirm where the remaining read
  cost lands
- revisit package read batching and concatenation strategy
