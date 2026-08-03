# ADR-004: Shared Benchmark Results and Presentation

## Status

Proposed.

## Date

2026-08-03

## Context

The workspace has several mature benchmark harnesses, but no common way to
store, compare, and present their results.

`cityjson-types` has the most complete presentation workflow. Its performance
scripts collect Criterion, dhat, and Cachegrind output into a long-form CSV and
provide terminal views for run history, named-baseline comparisons,
commit-to-commit comparisons, and metric time series. The analyzer nevertheless
assumes one fixed twelve-column schema, treats `backend` as its only variable
dimension, and infers whether a metric is better when it increases or decreases
from the metric name and unit.

`cityjson-lib` has a related but independently evolved set of collectors,
analyzers, plots, and Markdown reports. Other format crates have additional
comparison scripts. Copying the `cityjson-types` scripts into another crate
would create another fork of the same presentation concerns.

`cityjson-index` has a different measurement shape. Its benchmark runner emits
one wide JSON record per measured operation, including dataset, layout, worker
count, operation, variant, elapsed time, memory observations, source sizes, and
CityJSON-specific counts. The Tyler-pipeline simulation introduced in
`5d1f8de` uses these dimensions to reproduce the production OOM at 24 workers.
The current compact text output and raw JSON do not make worker scaling,
baseline comparisons, or historical changes easy to inspect.

Moving the existing analyzer unchanged would not solve the index use case:
worker count, layout, dataset, warmth, and variant would either be lost or
encoded into ad hoc benchmark names. It would also preserve heuristic metric
direction and provide no representation for a benchmark process that is killed
before it can emit its final JSON report. An OOM investigation needs incomplete
runs to remain visible rather than disappearing from the history.

The shared concern is result normalization and presentation, not benchmark
execution. Criterion microbenchmarks and the Tyler pipeline have different
setup, lifetime, and profiling requirements and should not be forced through a
single workspace runner.

## Decision

Create a workspace-level benchmark results and presentation system under
`tools/benchmarks/`.

The system will provide:

- a versioned canonical result model;
- adapters for supported benchmark output formats;
- persistent per-crate histories;
- generic filtering, comparison, time-series, and matrix views;
- terminal, Markdown, and CSV presentation formats.

Benchmark workload preparation and execution remain owned by each crate. The
shared system does not introduce a Rust workspace crate and does not change any
public Rust API.

### Canonical event stream

Use JSON Lines as the canonical persistent format. Each line is one complete
event and includes a `schema_version`. A run is represented by three event
types:

1. `run_start` records the run identity and environment before measurement
   begins;
2. `measurement` records one numeric observation as soon as it is available;
3. `run_end` records the final outcome.

The event stream is append-only. A supervising process owns the stream so that
measurements written before a crash or kill remain valid even when the
benchmark process cannot emit a final report.

A `run_start` contains at least:

- `run_id`, unique within the history;
- `suite`, such as `cityjson-types` or `cityjson-index`;
- UTC timestamp and Git commit;
- user-provided description and benchmark mode;
- suite-specific benchmark version;
- Rust compiler version;
- an explicit environment identifier for the machine on which the benchmark
  ran.

A `measurement` contains at least:

- `run_id`;
- a stable benchmark ID;
- an object of arbitrary dimensions;
- metric name, numeric value, and base unit;
- objective: `minimize`, `maximize`, or `informational`;
- optional sample count.

The identity of a comparable observation is:

```text
(suite, benchmark, canonical dimensions, metric, unit)
```

Dimension keys are sorted when constructing this identity. Dimensions remain
structured data and are not concatenated into benchmark names.

A `run_end` contains `run_id`, completion timestamp, and one of these outcomes:

- `completed`;
- `failed`, with an optional error;
- `killed`, with an optional exit code or signal.

The presentation layer must not label a killed process as OOM without reliable
evidence from the supervising environment. If an external profiler or resource
controller provides peak-memory or OOM evidence, its adapter records that data
explicitly.

Store measurements in base units only. For example, elapsed time is stored once
in nanoseconds and memory once in bytes. Renderers derive milliseconds,
seconds, MiB, and percentages without adding duplicate observations to the
history.

### Adapters and ownership

The shared tooling owns format-level adapters for data sources that are not
specific to a crate:

- Criterion estimates;
- dhat heap profiles;
- Cachegrind summaries;
- the structured `cityjson-index` benchmark report;
- the legacy twelve-column benchmark-history CSV.

Crate-level campaign scripts select workloads, set environment variables, run
profilers, and invoke these adapters. They also define stable benchmark IDs,
dimensions, metric objectives, and benchmark-version changes for their suite.

The `cityjson-index` adapter maps fields as follows:

- dataset, layout, worker count, operation, variant, warmth, source position,
  batch size, and reader count become dimensions when present;
- elapsed time and memory fields become `minimize` measurements;
- source sizes and CityJSON counts become `informational` measurements;
- absent optional values remain absent and are never converted to zero.

Raw Criterion directories, profiler output, and index JSON reports remain
optional diagnostic artifacts. The canonical event stream is the source used
for cross-run presentation.

### Presentation interface

Expose one shared command-line entry point with these views:

- `list` discovers runs, benchmark IDs, dimensions, values, metrics, and units;
- `snapshot` presents one selected run;
- `compare` compares runs selected by description, timestamp, or commit;
- `series` presents one metric across time;
- `matrix` pivots one or two dimensions, for example worker count against
  elapsed time and peak RSS.

All views accept repeatable dimension filters of the form `key=value`. A
comparison only includes observations with identical comparison identities.
Added, removed, and missing observations are reported separately.

Metric impact comes from the explicit objective. `minimize` metrics improve
when they decrease, `maximize` metrics improve when they increase, and
`informational` metrics receive no regression status. Percentage thresholds
affect status and highlighting but never hide raw values.

Terminal output is the interactive default. Markdown output is suitable for
checked-in benchmark reports, and CSV output supports further analysis. ANSI
color is disabled automatically for non-terminal output and can be overridden
explicitly.

Each crate exposes short `just` recipes that supply its history path and suite
defaults. The shared implementation must not assume that it is launched from a
particular crate directory.

### History ownership and compatibility

Histories remain per crate. A single workspace history would mix unrelated
campaigns, benchmark versions, and retention policies while making ordinary
crate changes modify a central large file.

Existing `cityjson-types` history remains readable through the legacy CSV
adapter. Initial adoption must not rewrite or delete historical results. New
runs may use the canonical JSONL history while comparisons can read both
sources.

`cityjson-lib` and other crates can adopt the common model later. Their current
plotting or report-specific behavior may remain as thin extensions until an
equivalent shared view exists. Adoption is not required for the initial
`cityjson-types` and `cityjson-index` extraction.

## Implementation Plan

### 1. Establish and test the common model

- Define the schema version, event validation, canonical dimension ordering,
  run selection, and comparison identity.
- Implement readers for canonical JSONL and legacy benchmark CSV.
- Add fixture-based tests for valid, invalid, partial, failed, and killed runs.
- Document when benchmark IDs or benchmark versions must change.

### 2. Extract presentation from `cityjson-types`

- Move the generic table, comparison, series, filtering, coloring, and
  sparkline behavior into the workspace tool.
- Replace metric-name direction heuristics with adapter-supplied objectives.
- Point `cityjson-types` recipes at the shared command while preserving current
  user-facing workflows and legacy-history access.
- Keep its Criterion, dhat, Cachegrind, Massif, and Memcheck campaign choices in
  the crate.

### 3. Add `cityjson-index` normalization and views

- Add a supervising campaign wrapper that writes `run_start` before invoking
  `bench-index`, appends measurements incrementally, and always attempts to
  append `run_end` after normal exit or observable failure.
- Normalize the existing benchmark report without removing its human-readable
  or JSON output modes.
- Add crate recipes for recording a described run and presenting its history.
- Provide a worker-scaling matrix that makes the 1, 4, and 24 worker Tyler
  measurements, memory growth, and incomplete outcomes directly comparable.

### 4. Validate parity and document adoption

- Compare shared-tool output against representative existing
  `cityjson-types` reports.
- Generate a Markdown Tyler-pipeline report from recorded index results.
- Update benchmark guides to distinguish raw artifacts, canonical history, and
  rendered reports.
- Keep the adapters and presenter covered by deterministic tests that do not
  require the external CityJSON corpus or execute expensive benchmarks.

### 5. Consider later workspace migrations separately

After the first two suites are stable, evaluate replacing the independently
evolved `cityjson-lib` analyzer and common parser copies. Plotting extensions
and other format-crate comparison scripts should migrate only when the shared
model can preserve their existing information and workflows.

## Acceptance Criteria

- Existing `cityjson-types` history can be listed and compared without
  migration.
- Representative `cityjson-types` fixtures produce equivalent snapshots,
  baseline deltas, and time series under the shared presenter.
- A `cityjson-index` history can filter and pivot dataset, layout, operation,
  and worker count without encoding those values into benchmark IDs.
- A Tyler-pipeline report presents elapsed time and memory for 1, 4, and 24
  workers and visibly distinguishes completed, failed, and killed runs.
- Measurements emitted before an interrupted run remain readable.
- Metric direction is explicit; informational counts are never reported as
  performance improvements or regressions.
- Terminal, Markdown, and CSV renderings are deterministic for fixed input.
- The shared tools have no mandatory third-party Python dependency and support
  the workspace's Python 3.11 through 3.13 range.
- Crate benchmark runners and public Rust APIs remain independent of the
  presentation implementation.
- `just ci` passes after adoption; changes touching `cityjson-index` also pass
  its required Python test suite.

## Consequences

### Positive

- benchmark presentation behavior has one workspace owner;
- index performance work gains repeatable worker-scaling and baseline views;
- arbitrary dimensions can be filtered and compared without schema changes;
- explicit metric objectives remove fragile naming heuristics;
- partial and killed runs remain part of the experimental record;
- future crates can reuse the system without adopting a common runner.

### Negative

- the canonical event model and adapters add an abstraction between raw
  profiler output and reports;
- histories may temporarily contain both legacy CSV and canonical JSONL;
- a supervisor is required to reliably record abnormal benchmark termination;
- generic matrix and filtering behavior is more complex than the current fixed
  `backend` grouping;
- existing crate-specific plots may remain separate until migrated.

## Rejected Alternatives

### Copy the `cityjson-types` scripts into `cityjson-index`

This duplicates a large analyzer and still cannot represent index dimensions
or interrupted runs without index-specific modifications.

### Move the current analyzer unchanged to the workspace root

The current fixed CSV schema, single `backend` dimension, and inferred metric
direction would become workspace-wide constraints rather than solving the
underlying mismatch.

### Build one workspace benchmark runner

Criterion microbenchmarks, format throughput campaigns, and the Tyler pipeline
have materially different setup and profiling needs. A common runner would
couple unrelated workloads and make crate-level iteration harder.

### Store all benchmark history in one workspace file

This creates unnecessary contention and mixes unrelated suites and retention
policies. Shared tooling does not require shared data ownership.

### Use a fixed wide CSV schema

Wide rows work for one known suite but require schema changes whenever another
dimension or metric is added. They also encourage duplicated derived units and
cannot naturally represent streaming partial results.

### Introduce a Rust crate for presentation

The existing tooling, renderers, and profiler parsers are Python and shell
programs. A Rust crate would add build and release surface without improving
the benchmarked libraries or the interchange boundary.
