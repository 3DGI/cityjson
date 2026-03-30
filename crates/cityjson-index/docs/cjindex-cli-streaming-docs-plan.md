# cjindex CLI, Streaming, and Docs Execution Plan

## Goals

Implement and validate three deliverables on trunk:

1. A usable `cjindex` CLI that can `index`, `reindex`, `get`, and `query` against all supported storage layouts.
2. A full documentation cleanup that removes obsolete scaffold-era claims, documents the current implementation and benchmark data, and explains the benchmark workloads.
3. A fully streaming `query_iter()` path that does not first materialize all matching feature locations into memory.

The work should be executed in parallel where the write sets allow it, then merged back to trunk with tests, formatting, linting, docs, and a final commit.

## Constraints

- Preserve the current semantic contract that reads return full `cjlib::CityModel` values.
- Keep CLI output schemas consistent across stdout and file output.
- Do not regress current benchmark realism: read workloads must continue to cover the full prepared corpus rather than repeatedly hitting a single hot tile.
- Maintain code correctness while merging worktree results back to trunk.

## Required CLI Behavior

### Commands

- `index`
- `reindex`
- `get`
- `query`
- `metadata` if it remains in the CLI surface

### Output behavior

- `get` and `query` must support stdout output in `CityJSONSeq`-style line-oriented JSON:
  - first line: a `CityJSON` metadata/base document with no `CityObjects`
  - subsequent lines: `CityJSONFeature` records
- `get` and `query` must also support writing the same line-oriented schema to a file
- The file output format is newline-delimited JSON with the same schema as stdout

### Streaming behavior

- `query_iter()` must stream matching features without first collecting all matching feature locations
- CLI `query` should consume that streaming path instead of eagerly materializing every result

## Parallelization Plan

### Worktree A: CLI and streaming implementation

Scope:

- `src/main.rs`
- `src/lib.rs`
- tests for CLI and iterator behavior
- `justfile` updates needed to support verification commands

Tasks:

- replace scaffolded CLI subcommands with working implementations
- add output selection and writing helpers
- add true streaming bbox lookup support in the index layer
- update `query_iter()` to use the streaming lookup path
- ensure `get` and `query` emit the required line-oriented schema
- add tests for CLI output shape and iterator streaming semantics where practical

### Worktree B: docs cleanup and benchmark documentation

Scope:

- `README.md`
- benchmark result docs
- ADR updates
- any obsolete plan/result docs that now need cross-links or corrections

Tasks:

- remove scaffold-era and obsolete storage-layout claims
- document the real current behavior for NDJSON, CityJSON, and feature-files
- add CLI usage examples
- document benchmark data provenance and workload construction
- fold in the current benchmark results and caveats
- describe the meaning of hot-cache vs cold-cache results where relevant

### Trunk integration pass

Tasks:

- merge both worktrees back onto trunk
- resolve conflicts, especially where docs reference code or CLI flags
- run formatting, linting, tests, and documentation verification
- run any benchmark commands needed to support updated docs if the docs changes require refreshed numbers
- commit all changes once trunk is green

## Implementation Outline

### 1. CLI

- keep the existing storage-layout argument model unless a small ergonomic improvement is clearly needed
- add explicit output destination flags for stdout vs file
- centralize metadata/base-document retrieval and line-oriented emission
- ensure `get` emits a metadata line even for a single hit
- ensure missing IDs return a clear non-zero error instead of an empty successful stream
- ensure `query` streams in bbox order with one metadata line followed by feature lines

### 2. Full streaming query iterator

- add a SQLite-backed bbox row iterator in the index layer
- avoid returning `Vec<FeatureLocation>` from the iterator path
- keep `query()` available for callers that want eager materialization, but implement it by collecting from the streaming iterator or a shared streaming primitive
- preserve deterministic ordering

### 3. Tests

- extend existing layout tests to cover the new CLI-visible behavior where possible
- add targeted tests for:
  - CLI `get` output shape
  - CLI `query` output shape
  - stdout/file parity
  - `query_iter()` returning the expected features without eager full-result buffering assumptions
- add a verification path for the new `just doc` target if one does not already exist

### 4. Docs

- rewrite the README around the current implementation state
- document the corrected CityJSON root-plus-descendants semantics
- explain benchmark dataset preparation and the 191-tile corpus shape
- include the latest benchmark numbers and caveats
- update ADRs/results docs where the CLI and streaming behavior materially changes project conclusions

## Verification

The final trunk pass must run:

- `just fmt`
- `just lint`
- `just test`
- `just doc`

And, if required to support the doc updates:

- the current benchmark commands used by the existing benchmark result docs

## Merge and Commit

- merge worktree branches back to trunk with non-destructive conflict resolution
- re-run the full verification set on trunk
- commit everything in one final integration commit summarizing CLI implementation, streaming query iteration, and docs cleanup
