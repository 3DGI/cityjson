# Performance Analysis

This note captures the current performance assessment of `serde_cityjson` and `cityjson-rs` based on the latest benchmark setup and the current implementation.

## Summary

The current read results are mixed. Geometry-heavy cases are often competitive or faster than the `serde_json::Value` baseline, but attribute-heavy reading is clearly weaker.

The current write results are much worse than the baseline across the board. That is not primarily a benchmark bug anymore. The main issue is the current write architecture.

## Highest ROI

The main write problem is architectural, not a small bug.

- `to_string` and `to_string_validated` still do `serde_json::to_string(&as_json(model))` in `src/v2_0.rs`.
- `SerializableCityModel::serialize` first builds a full `serde_json::Value` tree via `src/v2_0.rs` and `src/ser/citymodel.rs`.
- The split write benchmark shows `as_json_to_value` is already close to `to_string`, so most of the cost is DOM construction, not final string encoding.

This means the highest-value change is to replace the current write path with a true streaming serializer and add APIs like `to_writer` and `to_vec`.

While doing that, the serializer should also use a shared write context. Right now it rebuilds global lookup maps repeatedly:

- cityobject IDs are cloned in `src/ser/citymodel.rs`
- material dense indices are rebuilt per geometry in `src/ser/mappings.rs`
- texture dense indices are rebuilt per geometry in `src/ser/mappings.rs`

Those should be precomputed once per write.

Geometry serialization is also paying an extra conversion cost. `src/ser/geometry.rs` uses `to_nested_*` plus `serde_json::to_value`, which allocates another intermediate representation. If `cityjson-rs` can expose boundary iterators or raw nested views, those should be serialized directly.

## Read Path

The attribute-heavy weakness looks real and structural.

- `AttributeValue` and `Attributes` are recursive `HashMap` and `Vec` containers in `src/de/attributes.rs` and `../cityjson-rs/src/backend/default/attributes.rs`.
- `serde_json::Value` is much closer to the source JSON shape, so it has an easier time on deep attribute trees.

The lowest-risk improvement here is to switch hot maps from `std::HashMap` to `ahash::AHashMap`. The project already depends on `ahash`, but the hot read and write paths still use the standard hash map in many places.

There is also avoidable work in semantic attribute import. `src/de/geometry.rs` currently filters semantic attributes into one temporary `HashMap` and then clones them into another before converting them. That should become a direct parse path that skips reserved keys without the extra map-plus-clone pass.

Longer term, if attribute-heavy performance becomes a top priority, the bigger design question is whether `cityjson-rs` should keep `Attributes` as a hash-map-backed structure at all. A more contiguous representation would likely improve both parse and write performance, but that is a larger API and storage design tradeoff.

## Lower Priority

`validate_default_themes` is not the current problem. The benchmark split shows `to_string_validated` is basically tied with `to_string`, so optimizing theme validation is unlikely to move the needle.

Capacity planning is worth doing, but it is secondary. `cityjson-rs` already has `CityModelCapacities` and `reserve_import`, while the parser still starts from `CityModel::new`. Preallocating with measured capacities should help reduce reallocations, but it is unlikely to matter as much as removing the DOM-building write path.

## Suggested Order

1. Implement a streaming write serializer.
2. Add a shared write context for IDs, materials, textures, and template indices.
3. Remove intermediate boundary conversions during serialization.
4. Switch hot attribute-heavy paths to `AHashMap`.
5. Remove the semantic attribute filter-and-clone pass.

## Benchmark Interpretation

The latest summary supports this prioritization.

- Read performance is mixed rather than uniformly bad.
- Attribute-heavy read is the clearest weak spot.
- Write performance is dominated by the typed-model-to-JSON-tree conversion layer.
- The comparison bug in the old write benchmark was real, but it was not the main source of the current write slowdown.
