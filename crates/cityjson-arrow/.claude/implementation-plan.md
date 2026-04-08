# Cityarrow Write Optimization Plan

Date: 2026-04-08

This file supersedes `.claude/todo.md` and
`.claude/optimization-remediation-plan.md`.

## Goal

Fix the remaining write inefficiencies in `cityarrow` after `v0.5.1`.

The plan is deliberately narrower than the earlier schema-cut plans:

- keep the focus on the `cityarrow` crate
- optimize the encode path before considering larger decode or transport work
- spend effort where the current evidence says the time is going

## Current Reading

The downstream `cjlib` results and ADR 005 now make three things clear:

1. write is still much slower than the same-run JSON baseline
2. that gap is not primarily allocator churn
3. that gap is not primarily package I/O or `cityparquet`

The native write paths already allocate much less than the JSON baselines and
`cityarrow` and `cityparquet` are nearly identical on allocation and cache
metrics. So the next work should target CPU-heavy conversion logic in
`src/convert/mod.rs`, not framing, not Arrow transport, and not another schema
cut.

## What Not To Chase First

Do not spend the next round on these unless new measurements contradict the
current data:

- `cityparquet` package writing
- `RecordBatch::clone()` cleanup in the transport layer
- another boundary schema change
- speculative decode-side refactors
- geometry boundary storage in `cityjson-rs`

Those are not where the current write deficit is coming from.

## Primary Hotspots In `cityarrow`

### 1. Projection discovery still walks and rebuilds attribute trees

Current functions:

- `discover_projection_layout`
- `discover_attribute_projection`
- `merge_attribute_map_into_spec`
- `infer_projected_value_spec`

The remaining issue is not only the old top-level attribute clone, which is
already gone. Nested `AttributeValue::Map` values are still converted into a
fresh `OwnedAttributes` inside `infer_projected_value_spec` just so they can be
fed back through `merge_attribute_map_into_spec`.

That keeps the projection discovery prepass allocation-heavy on attribute-rich
models.

### 2. Projected attribute export still allocates per-field scratch vectors

Current functions:

- `cityobjects_batch_from_model`
- `metadata_batch`
- `semantics_batch_from_model`
- `projected_struct_array_from_attributes`
- `projected_value_array`

For each projected struct field, the exporter builds a new
`Vec<Option<&OwnedAttributeValue>>` across all rows. Nested structs recurse and
repeat the pattern. That means the projected attribute path is still doing a
large amount of temporary row-to-column reshaping even after the obvious clone
removal.

### 3. Material and texture payload export still clones column vectors

Current functions:

- `materials_batch_from_model`
- `textures_batch_from_model`
- `list_f64_array`

Both payload builders first collect full column vectors, then clone those same
vectors again while matching the projection fields. This is avoidable and is
likely one of the larger remaining write-side costs outside attribute
projection.

### 4. Ring texture export still allocates one `Vec<u64>` per textured ring

Current functions:

- `append_geometry_ring_texture_rows`
- `append_template_geometry_ring_texture_rows`
- `GeometryRingTextureTableBuffer`
- `TemplateGeometryRingTextureTableBuffer`

Each textured ring currently materializes a fresh `Vec<u64>` of UV ids before
appending it to the list buffer. That keeps per-ring allocation churn in a hot
geometry-adjacent path that should be append-only.

### 5. Surface and ring layout analysis is recomputed per attachment family

Current functions:

- `ring_layouts`
- `template_ring_layouts`
- `append_geometry_semantic_rows`
- `append_geometry_material_rows`
- `append_geometry_ring_texture_rows`

The exporter derives topology-derived counts and layouts multiple times while
walking the same boundary payload. That is not the first-order cost, but once
the bigger temporary allocations are removed it becomes worth collapsing into a
single per-geometry analysis pass.

## Workstreams

### Workstream 1: Add Cityarrow-Local Write Benchmarks

Do not start the next refactor round without a write benchmark in this repo
that stresses attribute-rich models and appearance-heavy models.

### Scope

- `benches/split.rs`
- test or fixture helpers as needed

### Tasks

1. Add at least one encode-heavy fixture with many projected attributes.
2. Add at least one encode-heavy fixture with materials and textures.
3. Keep the benchmark surface split so conversion can still be distinguished
   from package I/O.
4. Capture the current `encode_parts`, `stream_write_model`, and
   `package_write_model` timings before changing the exporter again.

### Exit Criteria

- benchmark fixtures expose attribute-heavy and appearance-heavy write costs
- the repo can detect whether a change helps conversion specifically or only
  moves transport overhead around

### Workstream 2: Remove Nested Attribute Spec Discovery Clones

This is the first actual code target.

### Scope

- `src/convert/mod.rs`

### Tasks

1. Replace the `AttributeValue::Map` branch in `infer_projected_value_spec`
   with a borrowed map walker instead of building a temporary
   `OwnedAttributes`.
2. Split `merge_attribute_map_into_spec` into two borrowed entry points:
   - one for `OwnedAttributes`
   - one for nested `HashMap<String, OwnedAttributeValue>`-style map values
3. Keep sort and nullability behavior identical so this remains a
   non-schema-changing cleanup.
4. Add targeted tests for nested projected attribute inference so the borrowed
   path cannot silently drift from the current behavior.

### Exit Criteria

- no nested attribute map is cloned purely for projection inference
- nested map inference stays behaviorally identical on existing fixtures

### Workstream 3: Replace Per-Field Scratch Vectors In Projected Export

This is the main write-path target.

### Scope

- `src/convert/mod.rs`

### Design Direction

Keep the current Arrow schema and projection semantics, but stop rebuilding
full `Vec<Option<&OwnedAttributeValue>>` scratch columns for every struct field.

The likely end state is a small set of reusable projected column encoders that:

- bind the child Arrow fields once
- append values directly into typed builders or flat buffers
- recurse on nested structs without first materializing a whole row slice per
  child field

### Tasks

1. Introduce typed projected value appenders for the scalar cases:
   - bool
   - u64
   - i64
   - f64
   - utf8
   - geometry ref
2. Add a struct encoder that walks rows once and dispatches each child value
   directly to its child encoder.
3. Keep list handling explicit. If a fully builder-native list encoder is too
   invasive for the first pass, land scalar and struct improvements first and
   measure again.
4. Convert these call sites first:
   - `cityobjects_batch_from_model`
   - `metadata_batch`
   - `semantics_batch_from_model`
5. Remove `field_from_schema` from hot projected paths and use
   `SchemaFieldLookup` consistently.

### Exit Criteria

- projected struct export no longer allocates one row-scratch vector per field
- nested projected attributes are encoded through reusable appenders
- schema field lookup on encode stays cached throughout the hot path

### Workstream 4: Remove Appearance Payload Clones

This is the next most obvious pure `cityarrow` write win.

### Scope

- `src/convert/mod.rs`

### Tasks

1. Rewrite `materials_batch_from_model` to build only the columns that the
   active material projection actually needs.
2. Rewrite `textures_batch_from_model` the same way.
3. Replace the current `Vec<Option<Vec<f64>>>` staging for color payloads with
   direct flat-buffer builders where practical.
4. Avoid cloning full `name`, `ambient_intensity`, `image_type`, `wrap_mode`,
   and similar vectors per projected field match arm.

### Exit Criteria

- appearance payload batch construction does not clone fully built columns
- optional projection fields are skipped without first constructing unused
  payload vectors

### Workstream 5: Make Ring Texture Export Append-Only

### Scope

- `src/convert/mod.rs`

### Tasks

1. Extend `U64ListBatchBuffer` with an iterator-based append API so callers can
   stream values into the flat buffer without allocating a temporary `Vec<u64>`.
2. Rewrite `append_geometry_ring_texture_rows` to validate and append UV ids in
   one pass.
3. Rewrite `append_template_geometry_ring_texture_rows` the same way.
4. Preserve the current error behavior for missing UV indices inside textured
   rings.

### Exit Criteria

- no per-ring `Vec<u64>` allocation remains on the texture export path
- ring texture validation and append happen in the same pass

### Workstream 6: Reuse Boundary Layout Analysis Per Geometry

This is lower priority than Workstreams 2 to 5, but it should become worth
doing once the larger temporary allocations are gone.

### Scope

- `src/convert/mod.rs`

### Tasks

1. Introduce one analyzed boundary layout object per exported geometry.
2. Share that layout across:
   - semantic export
   - material export
   - ring texture export
3. Remove duplicated `ring_layouts` and `template_ring_layouts` logic where the
   only difference is the calling context.

### Exit Criteria

- boundary-derived counts and ring spans are computed once per geometry
- semantics, materials, and textures consume the same analyzed layout

## Sequencing

Land the work in this order:

1. Workstream 1: better write benchmarks
2. Workstream 2: nested attribute inference clone removal
3. Workstream 3: projected attribute exporter rewrite
4. Workstream 4: appearance payload clone removal
5. Workstream 5: append-only ring texture export
6. Workstream 6: shared boundary layout analysis

That order keeps the highest-confidence wins first and avoids hiding the
effects of the projected attribute rewrite behind unrelated cleanup.

## Validation

Every workstream should be checked with:

- `just fmt`
- `just lint`
- `just test`
- `cargo bench --bench split -- --noplot`

For the larger steps, also compare against the downstream `cjlib` write bench
surface before concluding that a refactor is worth keeping.

The acceptance bar for the next round is:

- measurable write improvement in `cityarrow` local benches
- improvement carries through to `cjlib` tile and cluster write workloads
- no regression in native read speed large enough to erase the current ADR 005
  gains

## Obvious `cityjson-rs` Follow-Ups

This plan should not block on upstream work, but there are two plausible
follow-ups if the `cityarrow`-only changes are not enough.

### 1. Borrowed nested attribute traversal helpers

If `cityarrow` ends up needing its own internal borrowed walkers for nested
attribute maps in multiple places, that API probably belongs in `cityjson-rs`
instead.

The useful shape would be a borrow-only traversal helper for nested
`AttributeValue::Map` values so serializers do not need local bridge code for
walking attribute trees.

### 2. Dense texture-map slice helpers

The current UV export path has to validate that textured rings do not contain
missing UV vertices while converting `VertexIndex<u32>` values to Arrow ids.

If `cityjson-rs` can expose a helper that yields already validated dense ring
UV slices for textured rings, `cityarrow` can simplify that path further. This
is optional, not a blocker.

## Done Criteria

This optimization round is complete when:

- projected attribute inference and export no longer depend on large temporary
  row-shaped allocations
- appearance payload export stops cloning column vectors
- ring texture export is append-only
- local and downstream write benches improve materially
- the remaining write bottleneck, if any, is narrow enough to justify a more
  structural exporter rewrite with fresh evidence
