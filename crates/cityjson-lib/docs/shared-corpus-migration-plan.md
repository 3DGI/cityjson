# Shared Corpus Migration Plan

This plan turns the current split between `serde_cityjson` fixture data and
`cityjson-benchmarks` case metadata into one shared corpus contract.

The goal is not to make every fixture a benchmark. The goal is to make every
shared case identifiable, reproducible, and consumable by multiple crates.

## Goals

1. make correctness fixtures reusable across `serde_cityjson`, `cityjson_lib`,
   `arrow`, `parquet`, and future format crates
2. keep benchmark case identity and correctness expectations in one canonical
   place
3. separate conformance gating from throughput benchmarking
4. keep generation and acquisition logic outside the corpus definition

## Current Problems

- `serde_cityjson/tests/data` already contains a real shared conformance
  corpus, but it is owned and named as if it were crate-local test data
- `cityjson-benchmarks/catalog/corpus.json` duplicates much of the same case
  taxonomy at the benchmark level
- synthetic cases are described well enough to regenerate, but real-data cases
  are still mostly narratives rather than pinned artifacts
- invariants exist conceptually, but not yet as a machine-readable contract
- downstream crates cannot depend on one stable case identifier and one stable
  expectation set across formats

## Decision

`cityjson-benchmarks` should become the canonical shared corpus repository for
both correctness and benchmarking.

It should own:

- canonical case ids
- case metadata
- correctness invariants
- synthetic profiles
- real-data acquisition metadata
- released corpus artifacts

It should not own:

- parser or serializer implementation logic
- benchmark harness code for downstream crates
- synthetic generator internals
- ad hoc crate-local assertions about shared cases

## Existing Reuse Paths

There are already two useful real-data preparation paths in the current
ecosystem.

### `cjindex`

`cjindex` already has the strongest reusable 3DBAG preparation pipeline.

It currently provides:

- a pinned 3DBAG tile index source
- deterministic tile selection by target city-object count
- one prep pass that materializes `cityjson`, `ndjson`, and `feature-files`
  layouts
- per-output checksums and counts recorded in a manifest
- optional `cjval` validation

This is very close to the real-data provenance contract the shared corpus
needs.

The corpus repo should therefore prefer one of these approaches:

- extract the reusable prep logic into a small shared library or CLI, then let
  both `cjindex` and `cityjson-benchmarks` consume it
- or call the `cjindex` prep pipeline as the initial implementation while the
  corpus contract is still stabilizing

What the corpus repo should own is the acquisition contract and published
artifacts, not a second competing implementation of the same 3DBAG prep flow.

### `serde_cityjson`

`serde_cityjson` already has a narrower benchmark download path for:

- one 3DBAG tile download
- one 3D Basisvoorziening download
- local upgrade of older inputs into `CityJSON 2.0`

That is still useful during migration because it captures the current benchmark
inputs and the historical assumptions behind the existing benchmark tables.

It should not remain the long-term owner of shared real-data acquisition,
because it is:

- crate-local
- benchmark-specific
- pinned to older external versions than the newer `cjindex` prep flow
- limited to a small handpicked download set rather than a corpus contract

The practical role for `serde_cityjson` is therefore:

- source of the current benchmark fixtures and historical labels
- temporary consumer of shared real-data artifacts
- fallback bootstrap path until the corpus repo publishes pinned releases

## Corpus Layers

The shared corpus should be split into three practical layers.

### 1. Conformance Fixtures

Small hand-authored or tightly controlled cases used for correctness gates.

Examples:

- minimal CityJSON document
- transform handling
- geometry semantics variants
- appearance and material coverage
- templates and geometry instances
- CityJSONFeature and JSONL stream boundaries
- extension handling
- malformed and invalid documents

These are not throughput benchmarks.

### 2. Operation Fixtures

Medium-size cases that support correctness and targeted performance work for
real operations.

Examples:

- bounding box traversal
- hierarchy traversal
- semantic-surface lookup
- attribute filtering
- feature extraction
- layout conversion

These can be benchmarked, but they should first be usable as deterministic
correctness fixtures.

### 3. Performance Workloads

Large synthetic or real-data workloads used for throughput, allocation, and
memory measurements.

Examples:

- monolithic 3DBAG CityJSON slices
- JSONL and feature-file layout variants
- dense relation graphs
- deep boundary stress cases
- appearance-heavy write cases

These may have lighter invariants than conformance fixtures, but they still
need stable identity and provenance.

## Proposed Repository Layout

The canonical layout in `cityjson-benchmarks` should look like this:

```text
cityjson-benchmarks/
  catalog/
    cases.json
  cases/
    conformance/
      v2_0/
        cityjson_minimal_complete/
          case.json
          invariants.json
          source.city.json
        geometry_semantics_solid/
          case.json
          invariants.json
          source.city.json
      invalid/
        duplicate_vertices/
          case.json
          invariants.json
          source.city.json
    operations/
      ops_3dbag_base/
        case.json
        invariants.json
        acquisition.json
    workloads/
      stress_vertex_transform/
        case.json
        invariants.json
        profile.json
      io_3dbag_cityjson/
        case.json
        invariants.json
        acquisition.json
  profiles/
    schema/
      cjfake-manifest.schema.json
  artifacts/
    synthetic/
      <corpus-release>/
        ...
    real/
      <corpus-release>/
        ...
  docs/
    ...
```

This replaces the current split between:

- crate-local fixtures in `serde_cityjson/tests/data`
- benchmark metadata in `catalog/corpus.json`
- placeholder invariants documentation

## Case Identity Rules

Every shared case should have one stable `id` and one owner location in the
corpus repository.

Case ids should be:

- lowercase
- underscore-separated
- semantic rather than crate-specific
- stable across artifact regenerations

Examples:

- `cityjson_minimal_complete`
- `geometry_semantics_solid`
- `cityjsonfeature_minimal_complete`
- `spec_complete_omnibus`
- `ops_3dbag_base`
- `stress_vertex_transform`

Version-specific or format-specific variants should be explicit in metadata,
not encoded into unrelated ids.

## Metadata Schema

Each case should have one `case.json` with the minimum common contract:

```json
{
  "id": "stress_vertex_transform",
  "layer": "workload",
  "family": "stress",
  "source_kind": "synthetic-controlled",
  "cityjson_version": "2.0",
  "representations": ["cityjson"],
  "artifact_mode": "generated",
  "artifact_paths": {
    "source": "artifacts/synthetic/2026.04/stress_vertex_transform.city.json"
  },
  "profile": "profiles/cases/stress_vertex_transform.json",
  "primary_cost": "deserialize",
  "secondary_costs": ["memory", "serialize"],
  "geometry_validity": "dummy",
  "scale": "large",
  "operations": ["vertex_import", "transform", "reindex"],
  "tags": ["transform", "vertices", "synthetic", "benchmark"],
  "status": "active"
}
```

Required fields:

- `id`
- `layer`
- `source_kind`
- `cityjson_version`
- `representations`
- `artifact_mode`
- `operations`
- `status`

Common optional fields:

- `family`
- `profile`
- `artifact_paths`
- `primary_cost`
- `secondary_costs`
- `geometry_validity`
- `scale`
- `tags`
- `documentation`
- `supersedes`

## Invariants Schema

Each case should have one `invariants.json` owned by the corpus, not by a
consumer crate.

It should answer two questions:

1. what must always be true after ingest?
2. what transformations are allowed during roundtrip?

Suggested shape:

```json
{
  "parse": {
    "must_succeed": true,
    "expected_error": null
  },
  "counts": {
    "city_objects": 6,
    "vertices": 96,
    "geometry_instances": 4
  },
  "presence": {
    "has_transform": true,
    "has_appearance": false,
    "has_semantics": true,
    "has_templates": false,
    "has_extensions": false
  },
  "roundtrip": {
    "allowed": true,
    "canonicalization": [
      "object_key_order",
      "whitespace",
      "array_compaction"
    ],
    "must_preserve": [
      "cityjson_version",
      "geometry_types",
      "semantic_surface_count",
      "object_ids"
    ]
  },
  "format_expectations": {
    "cityjson": {
      "supported": true
    },
    "jsonl": {
      "supported": false
    },
    "arrow": {
      "supported": true,
      "loss_budget": ["metadata_key_order"]
    },
    "parquet": {
      "supported": true,
      "loss_budget": ["metadata_key_order"]
    }
  }
}
```

This is the missing bridge between correctness and benchmarking.

`serde_cityjson`, `cityjson_lib`, `arrow`, and `parquet` should all read the same
case id and the same invariant set, then apply format-specific assertions only
where unavoidable.

## Invalid Corpus

The shared corpus needs an explicit invalid layer. At the moment the visible
fixtures are mostly positive cases.

Add a dedicated invalid directory with cases such as:

- malformed JSON
- invalid JSONL framing
- missing required fields
- invalid `transform`
- out-of-range or dangling vertex references
- invalid geometry boundary nesting
- broken semantic-surface references
- duplicate object ids
- invalid extension declarations

Invalid cases should use the same `case.json` and `invariants.json` shape, but
with `parse.must_succeed = false` and an `expected_error` classification.

## Real-Data Provenance Contract

Real-data cases are not well defined until they have stable acquisition
metadata.

Every real-data case should include an `acquisition.json` with:

- source dataset name
- upstream source URL
- upstream version or release date
- acquisition script or command reference
- geographic slice selector
- preprocessing steps
- output checksum
- output byte size
- license notes

Without that, a case is only a narrative, not a reproducible corpus member.

For 3DBAG-derived cases, the first implementation should reuse the current
`cjindex` preparation approach rather than rewriting it immediately. The shared
corpus contract should absorb:

- pinned tile-index version
- selected tile list
- total object and feature counts
- per-layout output checksums
- derived layout identities for `cityjson`, `ndjson`, and `feature-files`

`serde_cityjson`'s existing download recipe should be preserved only as a
reference for historical benchmark continuity until those real-data cases are
published from the corpus repo.

## Consumer Contract

Downstream crates should consume the corpus in three different ways.

### `serde_cityjson`

- use conformance fixtures for parse, serialize, and roundtrip tests
- use invalid fixtures for negative tests
- use workloads only for benchmarks

### `cityjson_lib`

- use the same conformance fixtures for facade-level JSON and operation tests
- use operation fixtures for `ops` correctness tests
- use the same workload ids for benchmark integration once harnesses exist

### `arrow` and `parquet`

- use the same conformance and operation fixtures as import/export fixtures
- honor corpus invariants plus explicit format loss budgets
- add format-specific assertions only where the columnar representation cannot
  preserve full JSON shape

## Migration Phases

### Phase 1. Freeze ids and metadata

- promote the existing benchmark case ids to canonical shared ids
- map `serde_cityjson/tests/data/v2_0` fixtures to those ids where possible
- add machine-readable `case.json` and `invariants.json` for the current
  handcrafted fixtures
- keep existing crate-local copies in place during the transition

Exit criteria:

- every currently shared fixture has a stable id
- every synthetic benchmark case has one owner profile
- every consumer can refer to the same case id in test names and reports

### Phase 2. Move correctness ownership

- copy the handcrafted positive fixtures into `cityjson-benchmarks/cases`
- add the first invalid corpus tranche
- define roundtrip and preservation expectations per case
- make `serde_cityjson` consume the shared corpus instead of owning the
  authoritative fixture set

Exit criteria:

- `serde_cityjson` no longer curates the canonical fixture taxonomy
- invariants are corpus-owned rather than crate-owned

### Phase 3. Publish materialized artifacts

- make synthetic cases materializable into released artifacts
- pin real-data acquisitions with checksums and release metadata
- reuse or extract the current `cjindex` 3DBAG prep pipeline for the first
  real-data artifact release
- publish a corpus release index with artifact paths and digests

Exit criteria:

- benchmark consumers can pin a corpus release without running generators
- real-data workloads have stable provenance

### Phase 4. Add multi-format consumers

- wire `cityjson_lib` tests to shared conformance fixtures
- add `arrow` import/export checks against the same ids
- add `parquet` import/export checks against the same ids
- document format-specific loss budgets where exact preservation is impossible

Exit criteria:

- JSON, Arrow, and Parquet consumers share fixture ids and invariants
- format-specific deviations are explicit instead of implicit

### Phase 5. Trim crate-local duplicates

- remove duplicated metadata from `serde_cityjson`
- keep only temporary convenience symlinks or vendored snapshots if necessary
- treat the corpus repo as the only source of truth

Exit criteria:

- one corpus repo owns case definitions
- downstream repos only consume pinned releases or snapshots

## First Concrete Mapping

The first migration should map the current `serde_cityjson` fixture set into
the shared corpus with minimal renaming.

Good first cases:

- `cityjson_minimal_complete`
- `cityjsonfeature_minimal_complete`
- `geometry_complete_solid`
- `geometry_semantics_solid`
- `geometry_semantics_multisurface`
- `geometry_templates`
- `geometry_instance`
- `appearance_minimal_complete`
- `material_complete`
- `texture_complete`
- `extension`
- `metadata_complete`
- `transform`
- `spec_complete_omnibus`

The existing generated stress profiles already map naturally to:

- `stress_geometry_flattening`
- `stress_vertex_transform`
- `stress_attribute_tree`
- `stress_relation_graph`
- `stress_deep_boundary`
- `stress_composite_value`
- `stress_appearance_and_validation`

## Non-Goals

- forcing all benchmarks to become tiny conformance fixtures
- encoding generator internals into the public corpus contract
- requiring every format to preserve exact byte-for-byte JSON output
- hiding lossy format behavior behind vague roundtrip claims

## Immediate Next Steps

1. define `case.json` and `invariants.json` schemas in
   `cityjson-benchmarks`
2. migrate the existing handcrafted `serde_cityjson` fixtures into the new
   case layout
3. add the first invalid corpus tranche
4. publish a first release index for synthetic cases
5. decide whether the real-data prep path will call into `cjindex` directly or
   be extracted from it
6. switch one consumer first: `serde_cityjson`
7. then wire `cityjson_lib`, followed by `arrow` and `parquet`
