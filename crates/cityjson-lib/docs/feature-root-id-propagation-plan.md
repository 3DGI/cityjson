# Feature Root `id` Propagation Plan

## Goal

Make root `id` a typed part of the semantic `CityModel` for `CityJSONFeature`, instead of letting it fall through `extra`.

Target semantics:

- `CityJSONFeature`: root `id` is first-class model state and identifies the main feature `CityObject`.
- `CityJSON`: root `id` is not typed model state and remains an ordinary extra root property.
- Strict `CityJSONSeq` compatibility checks must ignore feature root `id` as shared-root state.
- Roundtrip through `cityjson-rs`, `serde_cityjson`, `cityarrow`, and `cityparquet` must preserve this distinction.

## Design Choice

Use the backend core for storage, but keep the public API typed in terms of handles.

- In `cityjson-rs` backend core, add `id: Option<RR>` to `CityModelCore<..., RR, ...>`.
- In the public `v2_0::CityModel`, expose `Option<CityObjectHandle>`.
- Do not expose `ResourceId32` publicly just to support this feature.

Reason:

- `CityModelCore` already uses `RR` for internal identity-bearing references.
- The semantic meaning of feature root `id` is "a handle to one city object in this model", not an arbitrary string.
- This keeps parse/build and transport code aligned with the existing handle-based graph model.

## Phase 1: `cityjson-rs`

### 1. Extend `CityModelCore`

Files:

- `/home/balazs/Development/cityjson-rs/src/backend/default/citymodel.rs`
- `/home/balazs/Development/cityjson-rs/src/v2_0/citymodel.rs`

Work:

- Add `id: Option<RR>` field to `CityModelCore`.
- Initialize it in `new()` and `with_capacities()`.
- Add internal accessors:
  - `id(&self) -> Option<RR>`
  - `set_id(&mut self, id: Option<RR>)`
- Expose public `v2_0::CityModel` accessors:
  - `id(&self) -> Option<CityObjectHandle>`
  - `set_id(&mut self, id: Option<CityObjectHandle>)`

### 2. Preserve invariants in public API

Files:

- `/home/balazs/Development/cityjson-rs/src/v2_0/citymodel.rs`

Work:

- Decide and document invariant:
  - recommended: `id` is only semantically valid for `CityModelType::CityJSONFeature`
- Do not hard-fail in the setter unless the crate already enforces this style elsewhere.
- Update `fmt::Display` and any debug-oriented output to show `id`.

### 3. Audit compile fallout and helpers

Likely touchpoints:

- any clone/build/constructor helpers
- tests and fixtures assuming `extra()` is the only root extension point
- any future raw accessors or projections that should expose model root `id`

Expected result:

- `CityModel` can carry a typed optional feature root `id` without abusing `extra`.

## Phase 2: `serde_cityjson`

### 4. Parse root `id` based on root `type`

Files:

- `/home/balazs/Development/serde_cityjson/src/de/root.rs`
- `/home/balazs/Development/serde_cityjson/src/de/build.rs`
- `/home/balazs/Development/serde_cityjson/src/v2_0.rs`

Work:

- Change root parsing so `id` is not blindly collected into `PreparedRoot.extra`.
- Capture raw root `id` separately in `PreparedRoot`.
- In build logic:
  - for `CityJSONFeature`, resolve root `id` to the corresponding `CityObjectHandle` after city object import, then store it in `model.set_id(...)`
  - for `CityJSON`, leave root `id` in `extra`

Important sequencing:

- `CityObjectHandle` resolution requires imported city objects, so feature-id application must happen after `import_cityobjects`.
- If the root `id` does not match a city object key in a `CityJSONFeature`, return a hard parse error.

### 5. Serialize root `id` based on model type

Files:

- `/home/balazs/Development/serde_cityjson/src/ser/citymodel.rs`
- `/home/balazs/Development/serde_cityjson/src/v2_0.rs`

Work:

- Add serializer support for root `id` as a first-class field.
- Emit root `id` only when:
  - `options.type_name == CityJSONFeature`
  - `model.id()` is present
- Resolve the handle back to the canonical `CityObject` identifier string using the existing write context / handle-to-id map.
- Do not emit typed `id` for `CityJSON`.
- Keep `extra` serialization for actual extra root properties only.

### 6. Fix strict `CityJSONSeq` shared-root semantics

Files:

- `/home/balazs/Development/serde_cityjson/src/v2_0.rs`

Work:

- Ensure `shared_root_signature()` excludes feature root `id`.
- Ensure `write_cityjsonseq_*` still emits feature root `id` in each `CityJSONFeature` item.
- Ensure `read_feature_stream()` and `from_feature_*_with_base()` materialize self-contained feature models with typed `id`, not `extra["id"]`.
- Ensure `merge_feature_stream()` accepts feature root `id` as feature-specific state rather than conflicting shared-root state.

### 7. Keep feature assembly paths aligned

Files:

- `/home/balazs/Development/serde_cityjson/src/v2_0.rs`

Work:

- Review:
  - `MaterializedFeaturePartsDocument`
  - `materialize_feature_document()`
  - `build_feature_base_root()`
  - `ensure_compatible_feature_root()`
- Rule:
  - `id` remains present in serialized `CityJSONFeature`
  - `id` is not part of the base/shared root signature
  - `id` is validated as feature-local state

## Phase 3: `cityarrow` and `cityparquet`

### 8. Decide transport support explicitly

Files:

- `/home/balazs/Development/cityarrow/src/convert/mod.rs`
- `/home/balazs/Development/cityarrow/src/schema.rs`
- `/home/balazs/Development/cityarrow/tests/adr3_roundtrip.rs`
- `/home/balazs/Development/cityarrow/cityparquet/src/package/mod.rs`

Current state:

- transport persists `citymodel_kind`
- transport persists `root_extra`
- transport does not appear to persist typed feature root `id`

Decision:

- `cityarrow` / `cityparquet` are relevant if they claim semantic roundtrip of `OwnedCityModel`, which they currently do.
- Therefore feature root `id` must be preserved explicitly.

Recommended minimal design:

- Add nullable metadata/header field for feature root id string, for example `feature_root_id`.
- Encode it only for `CityJSONFeature`.
- Decode it by resolving the string to the matching `CityObjectHandle` after city objects are rebuilt.

Why string, not handle:

- Arrow/Parquet transport already persists durable object identity by `cityobject_id` strings, not raw handles.
- Handles are process-local and reconstruction-dependent.

### 9. Apply the transport changes

Work:

- Extend metadata row structs with optional `feature_root_id`.
- Extend schema generation with a nullable UTF-8 column.
- Populate it from `model.id()` during encode.
- Restore it after `CityObjects` import during decode.
- Keep `root_extra` free of typed feature root `id`.

### 10. If transport support is intentionally out of scope, fail explicitly

Fallback only if the above is deferred:

- Reject `CityJSONFeature` models with `id()` during `cityarrow` encode with a precise unsupported error.

This is second-best. Silent loss is not acceptable.

## Phase 4: Tests

Keep tests narrow and semantic. Avoid large fixture churn.

### 11. `cityjson-rs` unit tests

Files:

- `/home/balazs/Development/cityjson-rs/tests/...`
- or colocated unit tests in `src/v2_0/citymodel.rs`

Add:

- create a `CityJSONFeature` model with one object, set `id`, assert `model.id()` returns that handle
- create a `CityJSON` model and assert ordinary root extra still works independently

### 12. `serde_cityjson` unit tests

Files:

- `/home/balazs/Development/serde_cityjson/tests/v2_0.rs`

Add minimal cases:

- parsing `CityJSONFeature` with root `id`:
  - `model.id()` is set
  - `model.extra()` does not contain `"id"`
- parsing `CityJSON` with root `id`:
  - `model.id()` is `None`
  - `model.extra()` does contain `"id"`
- serializing `CityJSONFeature`:
  - emits root `"id"`
  - does not duplicate `"id"` via `extra`
- strict writer:
  - base `CityJSON` + feature with typed `id` succeeds
- strict merge:
  - `merge_feature_stream()` accepts feature root `id`
- invalid feature:
  - root `id` referencing a missing city object is rejected

### 13. `cityarrow` / `cityparquet` tests

Files:

- `/home/balazs/Development/cityarrow/tests/adr3_roundtrip.rs`
- targeted unit tests in `cityarrow/src/convert/mod.rs` if needed

Add:

- roundtrip a `CityJSONFeature` model with root `id`
- assert decoded model keeps `type_citymodel() == CityJSONFeature`
- assert decoded `model.id()` resolves to the intended object
- assert normalized JSON includes root `"id"`

## Acceptance Criteria

The work is complete when all of the following are true:

- `CityJSONFeature.id` is stored as typed model state in `cityjson-rs`
- `CityJSON.id` still behaves as ordinary root extra
- `serde_cityjson` parse and write paths preserve the distinction
- strict `CityJSONSeq` validation no longer rejects feature streams because of typed feature `id`
- `cityarrow` / `cityparquet` either preserve the field or reject unsupported cases explicitly
- targeted regression tests pin the semantics

## Suggested Implementation Order

1. `cityjson-rs` core field and public accessors
2. `serde_cityjson` parser changes
3. `serde_cityjson` serializer and strict-stream fixes
4. `serde_cityjson` tests
5. `cityarrow` / `cityparquet` transport support
6. transport roundtrip tests

This order keeps the semantic model stable before touching boundary formats and transport.

## Concrete Patch Plan

This section translates the semantic matrix into exact fixture paths, `case.json`
payloads, and test entry points.

### Repo: `cityjson-benchmarks`

#### 1. Valid `CityJSONFeature` with resolvable root `id`

Add directory:

- `/home/balazs/Development/cityjson-benchmarks/cases/conformance/v2_0/cityjsonfeature_root_id_resolves`

Add files:

- `/home/balazs/Development/cityjson-benchmarks/cases/conformance/v2_0/cityjsonfeature_root_id_resolves/cityjsonfeature_root_id_resolves.city.jsonl`
- `/home/balazs/Development/cityjson-benchmarks/cases/conformance/v2_0/cityjsonfeature_root_id_resolves/case.json`

`case.json` payload:

```json
{
  "artifact_mode": "checked-in",
  "artifact_paths": {
    "source": "cases/conformance/v2_0/cityjsonfeature_root_id_resolves/cityjsonfeature_root_id_resolves.city.jsonl"
  },
  "assertions": [
    "feature_root_id_resolves",
    "feature_boundaries_preserved"
  ],
  "cityjson_version": "2.0",
  "description": "Hand-written CityJSONFeature fixture where root id resolves to a real CityObject in the same feature.",
  "family": "spec_atom",
  "geometry_validity": "dummy",
  "id": "cityjsonfeature_root_id_resolves",
  "layer": "conformance",
  "operations": [
    "parse",
    "stream_iteration",
    "serialize"
  ],
  "primary_cost": "deserialize",
  "representation": "cityjsonfeature",
  "scale": "tiny",
  "secondary_costs": [
    "serialize"
  ],
  "source_kind": "synthetic-controlled",
  "version": 1
}
```

#### 2. Invalid `CityJSONFeature` with unresolved root `id`

Add directory:

- `/home/balazs/Development/cityjson-benchmarks/cases/invalid/invalid_cityjsonfeature_root_id_unresolved`

Add files:

- `/home/balazs/Development/cityjson-benchmarks/cases/invalid/invalid_cityjsonfeature_root_id_unresolved/invalid_cityjsonfeature_root_id_unresolved.city.jsonl`
- `/home/balazs/Development/cityjson-benchmarks/cases/invalid/invalid_cityjsonfeature_root_id_unresolved/case.json`
- `/home/balazs/Development/cityjson-benchmarks/cases/invalid/invalid_cityjsonfeature_root_id_unresolved/README.md`

`case.json` payload:

```json
{
  "artifact_mode": "checked-in",
  "artifact_paths": {
    "source": "cases/invalid/invalid_cityjsonfeature_root_id_unresolved/invalid_cityjsonfeature_root_id_unresolved.city.jsonl"
  },
  "assertions": [
    "feature_root_id_unresolved"
  ],
  "cityjson_version": "2.0",
  "description": "Hand-written invalid CityJSONFeature fixture whose root id does not resolve to any CityObject in the same feature.",
  "family": "invalid",
  "geometry_validity": "dummy",
  "id": "invalid_cityjsonfeature_root_id_unresolved",
  "layer": "invalid",
  "operations": [
    "parse",
    "validate"
  ],
  "primary_cost": "validate",
  "representation": "cityjsonfeature",
  "scale": "tiny",
  "secondary_costs": [
    "deserialize"
  ],
  "source_kind": "synthetic-controlled",
  "version": 1
}
```

`README.md` should state one sentence precisely:

- `CityJSONFeature.id` must resolve to a `CityObject` key in the same feature item; this case uses a dangling root id and must be rejected.

#### 3. Strict `CityJSONSeq` operation case proving feature `id` is not shared-root state

Add directory:

- `/home/balazs/Development/cityjson-benchmarks/cases/operations/ops_cityjsonseq_feature_root_id_not_shared`

Add files:

- `/home/balazs/Development/cityjson-benchmarks/cases/operations/ops_cityjsonseq_feature_root_id_not_shared/ops_cityjsonseq_feature_root_id_not_shared.city.jsonl`
- `/home/balazs/Development/cityjson-benchmarks/cases/operations/ops_cityjsonseq_feature_root_id_not_shared/case.json`
- `/home/balazs/Development/cityjson-benchmarks/cases/operations/ops_cityjsonseq_feature_root_id_not_shared/README.md`

`case.json` payload:

```json
{
  "artifact_mode": "checked-in",
  "artifact_paths": {
    "source": "cases/operations/ops_cityjsonseq_feature_root_id_not_shared/ops_cityjsonseq_feature_root_id_not_shared.city.jsonl"
  },
  "assertions": [
    "feature_root_id_not_shared_state",
    "feature_boundaries_preserved"
  ],
  "cityjson_version": "2.0",
  "description": "Hand-written strict CityJSONSeq stream with a base CityJSON header and multiple CityJSONFeature items that differ only by feature root id.",
  "family": "operation_kernel",
  "geometry_validity": "dummy",
  "id": "ops_cityjsonseq_feature_root_id_not_shared",
  "layer": "operation",
  "operations": [
    "parse",
    "stream_iteration",
    "serialize"
  ],
  "primary_cost": "serialize",
  "representation": "jsonl",
  "scale": "tiny",
  "secondary_costs": [
    "deserialize"
  ],
  "source_kind": "synthetic-controlled",
  "version": 1
}
```

`README.md` should state two invariants:

- feature root `id` is preserved per feature item
- feature root `id` does not participate in strict shared-root compatibility checks

#### 4. `CityJSON.id` remains ordinary root extra

Add directory:

- `/home/balazs/Development/cityjson-benchmarks/cases/conformance/v2_0/cityjson_root_id_extra_property`

Add files:

- `/home/balazs/Development/cityjson-benchmarks/cases/conformance/v2_0/cityjson_root_id_extra_property/cityjson_root_id_extra_property.city.json`
- `/home/balazs/Development/cityjson-benchmarks/cases/conformance/v2_0/cityjson_root_id_extra_property/case.json`

`case.json` payload:

```json
{
  "artifact_mode": "checked-in",
  "artifact_paths": {
    "source": "cases/conformance/v2_0/cityjson_root_id_extra_property/cityjson_root_id_extra_property.city.json"
  },
  "assertions": [
    "root_extra_preserved"
  ],
  "cityjson_version": "2.0",
  "description": "Hand-written CityJSON fixture where root id is preserved as an ordinary extra root property.",
  "family": "spec_atom",
  "geometry_validity": "dummy",
  "id": "cityjson_root_id_extra_property",
  "layer": "conformance",
  "operations": [
    "parse",
    "serialize"
  ],
  "primary_cost": "deserialize",
  "representation": "cityjson",
  "scale": "tiny",
  "secondary_costs": [
    "serialize"
  ],
  "source_kind": "synthetic-controlled",
  "version": 1
}
```

### Repo: `serde_cityjson`

Primary file:

- `/home/balazs/Development/serde_cityjson/tests/v2_0.rs`

#### Corpus-backed conformance tests

Extend the existing ad hoc feature fixture coverage with two explicit tests:

- `fn cityjsonfeature_root_id_resolves()`
- `fn cityjson_root_id_extra_property()`

These should follow the current direct style already used by:

- `fn cityjsonfeature_minimal_complete()`

and simply load the corresponding benchmark source via `conformance_case_input(...)`
and call `assert_eq_roundtrip(...)`.

Add borrowed parser parity twins in the same file:

- `fn cityjsonfeature_root_id_resolves_borrowed()`
- `fn cityjson_root_id_extra_property_borrowed()`

#### Explicit semantic regression tests

Add these unit tests in `/home/balazs/Development/serde_cityjson/tests/v2_0.rs`:

- `fn cityjsonfeature_root_id_is_typed_not_extra()`
- `fn cityjsonfeature_root_id_must_resolve_to_a_cityobject()`
- `fn strict_cityjsonseq_writer_accepts_feature_root_id_as_feature_local_state()`
- `fn cityjson_root_id_remains_extra_property_not_typed()`

Test intent:

- `cityjsonfeature_root_id_is_typed_not_extra`
  - parse the new valid feature fixture with `from_feature_str_owned(...)`
  - assert `model.type_citymodel() == CityModelType::CityJSONFeature`
  - assert `model.id().is_some()`
  - assert `model.extra().get("id").is_none()`
  - serialize and assert root `id` is present once
- `cityjsonfeature_root_id_must_resolve_to_a_cityobject`
  - load the invalid fixture
  - assert `from_feature_str_owned(...)` returns an error containing a stable fragment such as `feature root id`
- `strict_cityjsonseq_writer_accepts_feature_root_id_as_feature_local_state`
  - construct base `CityJSON` plus two feature models with different typed ids
  - call `write_cityjsonseq_with_transform_refs(...)`
  - assert success and emitted feature items preserve their distinct root ids
- `cityjson_root_id_remains_extra_property_not_typed`
  - parse the new `CityJSON` fixture with `from_str_owned(...)`
  - assert `model.type_citymodel() == CityModelType::CityJSON`
  - assert `model.id().is_none()`
  - assert `model.extra()["id"]` is present

#### Invalid corpus table

Extend the invalid fixture list in `/home/balazs/Development/serde_cityjson/tests/v2_0.rs`
to include:

- `invalid_cityjsonfeature_root_id_unresolved`

Keep this in the same table-driven invalid block as the current invalid fixtures, so the
new case is exercised by the generic invalid-corpus pass as well as the explicit semantic
test above.

### Repo: `cityarrow`

Add a new test file instead of growing `adr3_roundtrip.rs` further:

- `/home/balazs/Development/cityarrow/tests/feature_root_id_roundtrip.rs`

Add test functions:

- `fn arrow_roundtrip_preserves_cityjsonfeature_root_id()`
- `fn arrow_roundtrip_preserves_cityjson_root_id_as_extra()`

Test intent:

- `arrow_roundtrip_preserves_cityjsonfeature_root_id`
  - build an `OwnedCityModel` with `CityModelType::CityJSONFeature`
  - add `CityObjects`
  - set typed feature root `id`
  - roundtrip through `ModelEncoder.encode(...)` and `ModelDecoder.decode(...)`
  - assert decoded `model.id()` resolves to the expected object
  - assert decoded `model.extra().get("id").is_none()`
- `arrow_roundtrip_preserves_cityjson_root_id_as_extra`
  - build a `CityJSON` model
  - insert root `extra["id"]`
  - roundtrip through Arrow
  - assert typed `model.id()` is `None`
  - assert root extra `id` survives

Do not add the unresolved-id negative case here unless the Arrow transport ends up
accepting raw dangling ids before semantic reconstruction. If decode resolves ids
immediately, the parser layer remains the authoritative rejection point.

### Repo: `cityparquet`

Create a dedicated test directory and file:

- `/home/balazs/Development/cityarrow/cityparquet/tests/package_feature_root_id_roundtrip.rs`

Add test functions:

- `fn package_roundtrip_preserves_cityjsonfeature_root_id()`
- `fn package_roundtrip_preserves_cityjson_root_id_as_extra()`

Test intent:

- `package_roundtrip_preserves_cityjsonfeature_root_id`
  - build a `CityJSONFeature` model with typed root id
  - write through `PackageWriter.write_file(...)`
  - read back with `PackageReader.read_file(...)`
  - assert decoded typed id survives and is not duplicated into root extra
- `package_roundtrip_preserves_cityjson_root_id_as_extra`
  - build a `CityJSON` model with root extra `id`
  - package roundtrip it
  - assert typed `id()` is `None`
  - assert root extra `id` survives

As with `cityarrow`, only add a dangling-id negative decode test if the package format
materially allows unresolved raw ids to exist after transport schema changes.

### Fixture Body Shape

Keep the fixtures minimal and hand-authored.

#### `cityjsonfeature_root_id_resolves.city.jsonl`

One-line `CityJSONFeature`:

- root `type: "CityJSONFeature"`
- root `id: "building-1"`
- `CityObjects` contains `building-1`
- one minimal geometry or even no geometry if current parser accepts object-only fixtures
- empty `vertices` allowed if the object has no geometry

#### `invalid_cityjsonfeature_root_id_unresolved.city.jsonl`

One-line `CityJSONFeature`:

- root `type: "CityJSONFeature"`
- root `id: "missing-main-object"`
- `CityObjects` contains some other object id, not `missing-main-object`

#### `ops_cityjsonseq_feature_root_id_not_shared.city.jsonl`

Three-line strict stream:

1. base `CityJSON` header with `type`, `version`, and minimal shared root state
2. first `CityJSONFeature` with root `id: "building-1"`
3. second `CityJSONFeature` with root `id: "building-2"`

The two feature items should be otherwise compatible with the same shared root.

#### `cityjson_root_id_extra_property.city.json`

One `CityJSON` document:

- root `type: "CityJSON"`
- root `id: "document-external-id"`
- at least one real `CityObject`

### Recommended Implementation Sequence

1. Add the four benchmark fixtures and validate their metadata against
   `/home/balazs/Development/cityjson-benchmarks/schemas/case.schema.json`
2. Implement the `cityjson-rs` typed `id` field
3. Update `serde_cityjson` parse/write semantics and land the four tests in
   `/home/balazs/Development/serde_cityjson/tests/v2_0.rs`
4. Add `cityarrow` transport preservation tests
5. Add `cityparquet` package preservation tests
