# Cut V3 Schema For Arrow-Native Projection And Batch-Native Conversion

## Status

Accepted

## Context

[ADR 2](002-address-transport-performance-bottlenecks.md),
[ADR 3](003-separate-live-arrow-ipc-from-persistent-package-io.md), and
[ADR 4](004-reduce-conversion-cost-with-ordinal-canonical-relations.md)
stabilized the transport boundary and removed several avoidable costs, but the
latest downstream split benchmarks still show that conversion dominates the hot
path.

For the 2026-04-02 split diagnostics pinned in ADR 4:

- tile `convert_encode_parts`: about `37.46 ms`
- tile `convert_decode_parts`: about `22.37 ms`
- tile `stream_write_parts`: about `0.46 ms`
- tile `stream_read_parts`: about `0.45 ms`
- cluster `convert_encode_parts`: about `172.56 ms`
- cluster `convert_decode_parts`: about `94.08 ms`
- cluster `stream_write_parts`: about `5.49 ms`
- cluster `stream_read_parts`: about `2.10 ms`

That result means the remaining problem is not Arrow IPC framing or package
manifest I/O. The problem is the current `v2alpha2` canonical schema and
conversion contract.

The current model has three structural flaws:

1. generic projected attributes are not Arrow-native
   - discovered attribute fields are flattened into `LargeUtf8` columns with a
     `_json` suffix
   - export converts `OwnedAttributeValue -> serde_json::Value -> String`
   - import converts `String -> serde_json::Value -> OwnedAttributeValue`
2. the schema encourages row-first staging instead of batch-first construction
   - export builds whole `Vec<Row>` structures before building arrays
   - import reconstructs `Vec<Row>` and grouped hash maps from arrays before
     rebuilding `OwnedCityModel`
3. the current projection layout is flatter and noisier than the CityJSON data
   it represents
   - dynamic attribute namespaces become many prefixed columns instead of one
     typed nested value
   - nested maps and lists are hidden inside JSON strings instead of surfacing
     as Arrow lists and structs

Because the package schema is still alpha and the project now wants a clean
design rather than compatibility scaffolding, the next step should be a hard
schema break, not another incremental cleanup on top of `v2alpha2`.

## Decision

`cityarrow` and `cityparquet` will cut a new canonical transport schema:
`cityarrow.package.v3alpha1`.

`v3alpha1` keeps the public semantic boundary from ADR 3:

`OwnedCityModel -> canonical tables -> live stream/package -> OwnedCityModel`

`v3alpha1` also keeps the canonical table set and ordinal relation choices from
ADR 4 unless a later ADR explicitly changes them.

The breaking change is focused on the schema and execution model for projected
payloads and conversion.

### 1. Hard Break, No Migration

The implementation rules are:

1. the package schema id becomes `cityarrow.package.v3alpha1`
2. the mainline encoder writes only `v3alpha1`
3. the mainline decoder reads only `v3alpha1`
4. no compatibility reader, migration helper, or JSON fallback lane will be
   kept for `v2alpha2`

This is a deliberate clean break.

### 2. Recursive Typed Projection Layout

The flat `ProjectedValueType` plus flat `Vec<ProjectedFieldSpec>` layout is
replaced by a recursive projection grammar that mirrors Arrow types directly.

The conceptual shape is:

```rust
struct ProjectionLayout {
    root_extra: Option<ProjectedStructSpec>,
    metadata_extra: Option<ProjectedStructSpec>,
    cityobject_attributes: Option<ProjectedStructSpec>,
    cityobject_extra: Option<ProjectedStructSpec>,
    geometry_extra: Option<ProjectedStructSpec>,
    semantic_attributes: Option<ProjectedStructSpec>,
}

struct ProjectedStructSpec {
    fields: Vec<ProjectedFieldSpec>,
}

struct ProjectedFieldSpec {
    name: String,
    nullable: bool,
    value: ProjectedValueSpec,
}

enum ProjectedValueSpec {
    Null,
    Boolean,
    UInt64,
    Int64,
    Float64,
    Utf8,
    GeometryRef,
    List {
        item_nullable: bool,
        item: Box<ProjectedValueSpec>,
    },
    Struct(ProjectedStructSpec),
}
```

The supported value vocabulary is intentionally smaller than arbitrary JSON:

- `Null`
- `Boolean`
- `UInt64`
- `Int64`
- `Float64`
- `Utf8`
- `GeometryRef`
- `List<T>`
- `Struct{...}`

Notably absent:

- no JSON string encoding
- no opaque binary fallback
- no Arrow union types
- no "mixed scalar" coercion rules

If a dataset needs a shape outside this grammar, encoding fails with a schema
error instead of silently tunneling it through JSON.

### 3. Dynamic Attribute Namespaces Become Struct Columns

Dynamic attribute payloads are no longer expanded into many top-level
`*_json` columns.

Instead, each dynamic namespace becomes one explicit struct-typed table column:

- metadata table: `root_extra`, `metadata_extra`
- cityobjects table: `attributes`, `extra`
- geometries table: `extra`
- semantics table: `attributes`

The nested fields inside those struct columns preserve the actual CityJSON key
names. The `_json` suffix convention, prefix-decoding rules, and encoded flat
projection column names are removed from `v3alpha1`.

This makes the schema closer to the source model and gives the encoder and
decoder one recursive value shape to handle instead of many flat JSON-carrying
columns.

### 4. Projection Inference Is Strict And Structural

Projection discovery will infer one structural type per attribute path.

The rules are:

1. `null` contributes nullability but does not choose the type on its own
2. all non-null values for the same attribute path must agree on scalar kind
3. all non-null values for the same attribute path must agree on container
   shape
4. object keys are unioned into struct fields recursively
5. list item shapes are unified recursively
6. `GeometryRef` is a first-class logical type and is represented in Arrow as
   `UInt64`
7. incompatible shapes are rejected during export

Examples of incompatible shapes:

- `UInt64` in one row and `Utf8` in another
- `Float64` in one row and `Struct` in another
- `List<UInt64>` in one row and `List<Struct>` in another
- `GeometryRef` in one row and numeric scalar in another

The encoder will not normalize or widen these differences. The dataset must be
structurally coherent to use `v3alpha1`.

### 5. Encode Directly Into Arrow Builders

`encode_parts` is redefined as a batch-native conversion pipeline.

The implementation rules are:

1. each canonical table owns a dedicated encoder that wraps Arrow builders
2. export appends values directly into those builders while traversing the
   `OwnedCityModel`
3. `emit_tables` becomes orchestration only
4. generic projected payloads are appended recursively into struct/list/scalar
   builders
5. whole-model `Vec<Row>` staging is not allowed in the hot path

This applies first to cityobject export and then to the remaining canonical
tables.

Small temporary vectors that are intrinsic to one logical value are allowed.
Whole-table or whole-model row staging is not.

### 6. Decode Directly From RecordBatch Views

`decode_parts` is redefined as a batch-native import pipeline.

The implementation rules are:

1. canonical table dispatch works on bound Arrow arrays and retained
   `RecordBatch` references
2. grouped relations are indexed as `id -> Range<usize>` or equivalent ordered
   views, not rebuilt as `HashMap<u64, Vec<Row>>`
3. unique relations are indexed by row position or `id -> row_index`
4. projected payloads are reconstructed recursively from Arrow arrays without
   `serde_json`
5. row reconstruction helpers such as `read_*_rows` are removed from the hot
   path

This keeps the decoder aligned with the columnar transport rather than
immediately collapsing back into row-shaped intermediate data.

### 7. Material And Texture Payloads Stay Explicitly Typed

The `v3alpha1` redesign is about generic projected attributes and the
conversion contract. Material and texture payloads are already expressed with
typed canonical fields, so this ADR does not fold them into a new dynamic
projection mechanism.

They may be revisited later, but they are not part of this break.

## Example

Given a cityobject with attributes conceptually shaped like:

```json
{
  "name": "building-1",
  "metrics": {
    "height": 12.5,
    "levels": 3
  },
  "related_geometries": ["<geometry-ref>", "<geometry-ref>"]
}
```

the `cityobjects` table in `v3alpha1` conceptually carries:

- `cityobject_id: LargeUtf8`
- `cityobject_ix: UInt64`
- `object_type: Utf8`
- `geographical_extent: FixedSizeList<Float64, 6>?`
- `attributes: Struct<`
  `name: Utf8,`
  `metrics: Struct<height: Float64, levels: UInt64>,`
  `related_geometries: List<GeometryRef>`
  `>?`
- `extra: Struct<...>?`

No JSON strings are involved in that path.

## Consequences

Good:

- the transport schema matches CityJSON attribute structure much more closely
- the generic attribute path becomes actually columnar
- the conversion code can be expressed in Arrow-native builders and array views
- `encode_parts` and `decode_parts` can drop the most obvious allocation and
  clone-heavy staging patterns
- schema inspection becomes more meaningful because nested lists and structs are
  visible in Arrow instead of hidden inside strings

Trade-offs:

- this is a full schema break and old packages/streams are intentionally not
  supported
- structurally heterogeneous free-form attributes now fail fast instead of
  sneaking through as JSON
- projection inference becomes more explicit and more opinionated
- nested Arrow columns are less flat for ad hoc manual inspection than the old
  many-column expansion

## Non-Goals

This ADR does not:

- change the live stream versus persistent package split from ADR 3
- change the canonical table set or ordinal relation rules from ADR 4
- introduce compatibility readers for old schema versions
- preserve arbitrary heterogeneous JSON values by tunneling them through a
  string column

## Follow-On Work

The implementation sequence after this ADR is:

1. add `v3alpha1` schema types and remove `v2alpha2` from the mainline path
2. replace flat projection discovery with recursive structural inference
3. rewrite cityobject export/import around struct-column builders and views
4. apply the same direct builder/view model to geometry, boundary, semantic,
   and appearance tables
5. delete JSON projection helpers and row-materialization helpers from the hot
   path

## Results Snapshot: 2026-04-07

The downstream `cjlib` runs after the initial `v3` cut and later conversion
remediation now give this ADR a concrete benchmark reading.

Compared with the earlier downstream run `cityarrow optimize decode,encode`
from `2026-04-04T20:42:15Z`, the `cityarrow optimize v0.5.0` run at
`2026-04-07T22:36:17Z` moved the end-to-end numbers to:

| Path | Previous | Current | Delta |
| --- | --- | --- | --- |
| `cityarrow` tile read | `25.72 ms` | `15.72 ms` | `-38.9%` |
| `cityarrow` cluster read | `101.83 ms` | `66.09 ms` | `-35.1%` |
| `cityarrow` tile write | `47.05 ms` | `35.85 ms` | `-23.8%` |
| `cityarrow` cluster write | `166.59 ms` | `123.87 ms` | `-25.6%` |
| `cityparquet` tile read | `25.42 ms` | `15.97 ms` | `-37.2%` |
| `cityparquet` cluster read | `100.63 ms` | `65.40 ms` | `-35.0%` |
| `cityparquet` tile write | `53.95 ms` | `44.72 ms` | `-17.1%` |
| `cityparquet` cluster write | `168.54 ms` | `125.23 ms` | `-25.7%` |

Against the same-run `serde_json::Value` baseline, the native read paths now
win decisively on both pinned fixtures:

| Case | Baseline read | `cityarrow` read | `cityparquet` read |
| --- | --- | --- | --- |
| tile | `25.04 ms` | `15.72 ms` | `15.97 ms` |
| cluster | `104.51 ms` | `66.09 ms` | `65.40 ms` |

Write improved materially, but it is still not competitive with the JSON
baseline:

| Case | Baseline write | `cityarrow` write | `cityparquet` write |
| --- | --- | --- | --- |
| tile | `7.89 ms` | `35.85 ms` | `44.72 ms` |
| cluster | `36.71 ms` | `123.87 ms` | `125.23 ms` |

The allocation reading is now strong evidence that the batch-native conversion
work is paying off, not just the stream or package containers:

- read total allocated bytes dropped by about `47%` to `49%` versus the
  earlier `decode,encode` run
- read peak heap also dropped by about `7%` to `9%`
- in the `v0.5.0` run, read total allocated bytes are about `46%` to `47%`
  below `cjlib::json` and about `11%` to `12%` below `serde_json::Value`
  while peak heap stays only about `7%` to `11%` above `cjlib::json` and about
  `37%` below `serde_json::Value`

This newer `v3` line also now beats the last pre-`v3` high-water mark from ADR
4, `cityarrow v2alpha2 conversion cleanup`, by another:

- about `29%` to `30%` on native read time for both `cityarrow` and
  `cityparquet`
- about `33%` to `36%` on `cityarrow` write time
- about `14%` on `cityparquet` tile write and about `33%` on `cityparquet`
  cluster write

The practical reading is now:

- the `v3` schema and conversion redesign did eventually pay back the temporary
  regression from the first schema-cut implementation pass
- read performance is no longer merely close to the JSON baseline; it is now a
  clear win on real end-to-end `OwnedCityModel` workloads
- the latest gains are matched by a large reduction in read allocation churn,
  which fits the ADR's goal of removing JSON projection and row-staging costs
- write performance is improved but still dominated by export conversion work,
  so the next optimization target remains write-side conversion rather than
  transport framing

## Implementation Notes: 2026-04-08

The current `cityarrow`/`cityparquet` patch series tightens the write path
without changing the ADR's main conclusion.

The main code changes are:

- geometry and template boundary tables now append borrowed slices directly
  into flat batch buffers instead of staging `Vec<Vec<_>>` rows and flattening
  them later
- ring texture `uv_indices` use the same batch-native list buffer path
- boundary payloads borrow `cityjson-rs` columnar views directly via
  `Boundary::to_columnar()` instead of cloning boundary vectors first
- cityobject projected attributes skip empty attribute maps before structural
  projection

The local split benchmark snapshot after these changes is:

- `encode_parts`: about `2.27 ms -> 1.77 ms`
- `stream_write_model`: about `2.61 ms -> 2.23 ms`
- `stream_write_parts`: about `139 us -> 117 us`
- `package_write_model`: about `3.17 ms -> 2.43 ms`

The read-side timings stayed in the same general range, which is consistent
with the change being a write-path conversion optimization rather than a
schema or transport rewrite.

The remaining write bottleneck is still the projected attribute conversion
layer, not `cityparquet` transport and not `cityjson-rs` geometry storage.
