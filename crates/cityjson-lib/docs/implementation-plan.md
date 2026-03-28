# Implementation Plan

This plan covers the next two concrete cleanup steps for the `cjlib` rewrite.

## Step 1: Align The Facade Boundary

### Goal

Make `cjlib` behave like a thin facade instead of a blurred proxy for
`cityjson-rs`.

### Required Changes

1. Remove implicit boundary leakage from `CityModel`.

- keep `as_inner`, `as_inner_mut`, `into_inner`, `AsRef`, and `AsMut`
- remove `Deref` and `DerefMut`
- avoid re-exporting extra `cityjson` items at the crate root when
  `cjlib::cityjson::...` is the intended advanced path

2. Keep unfinished workflow modules obviously unfinished.

- do not return cloned models, zero measurements, or empty reports
- replace placeholder behavior with explicit `todo!()` markers
- keep only one illustrative function in `ops` until there is a real semantic
  backend to delegate to

### Success Criteria

- `CityModel` is explicit at the type boundary
- the crate root is smaller and less misleading
- unfinished workflow areas fail loudly instead of pretending to work

## Step 2: Clean Up The JSON Boundary

### Goal

Keep `cjlib::json` focused on explicit JSON document and feature handling
without presenting stream aggregation as a stable API.

### Required Changes

1. Keep the document path simple.

- `CityModel::from_slice` and `CityModel::from_file` stay as the ergonomic
  single-document path
- `json::from_file` stays document-oriented

2. Keep streams explicit.

- `json::read_feature_stream` and `json::write_feature_stream` remain the
  stream APIs
- remove public aggregation helpers such as `json::merge_feature_stream`
- remove compatibility aliases such as `CityModel::from_stream` and
  `json::from_stream`

3. Make unsupported paths explicit.

- do not silently aggregate `.jsonl` files in `json::from_file`
- return a structured error that points callers to `json::read_feature_stream`
- leave legacy version import branches as explicit `todo!()` until real import
  support exists

### Success Criteria

- JSON documents and JSON feature streams have separate, explicit entry points
- there is no public API that implies hidden semantic aggregation
- unsupported or unfinished branches are obvious in code and tests

## Testing Policy During The Rewrite

Tests should distinguish three states clearly:

- implemented behavior: tests pass
- intentionally unimplemented behavior: tests fail because they hit `todo!()`
- unsupported behavior: tests pass by asserting a structured error

That keeps the suite honest while the facade surface is still being reduced.
