# ADR 008: Adopt The Owned-Model JSON Boundary Surface

## Status

Accepted

## Context

The draft vNext plan for the CityJSON stack narrows repository ownership:

- `cityjson-rs` owns the in-memory model and lower-level relational contracts
- `cityjson-json` owns JSON document and `CityJSONSeq` parsing and writing

Before this change, `cityjson-json` still exposed:

- borrowed-model parsing as a primary public surface
- a builder-style JSON writer (`as_json(...).to_vec()`, `.to_string()`, `.to_writer()`)
- a `CityJSONSeq` writer that required a separate base root model instead of
  deriving the stream header from the feature models being written

That surface kept the crate tied to an older API shape and obscured which
operations were cheap versus explicitly serializing or materializing data.

## Decision

`cityjson-json` now exposes an explicit owned-model API:

- `read_model`
- `read_feature`
- `read_feature_with_base`
- `read_feature_stream`
- `write_model`
- `to_vec`
- `write_feature_stream`

The new API is option-struct driven:

- `ReadOptions`
- `WriteOptions`
- `CityJsonSeqWriteOptions`

`CityJSONSeq` writing is now feature-first. `write_feature_stream` accepts owned
feature models, validates that their shared root state is compatible, derives
the header from that shared state, and then emits the header plus feature
items.

The old borrowed parse entry point and builder-style JSON writer were removed
from the primary public surface.

## Consequences

Positive:

- the crate boundary matches the intended repository role more closely
- expensive JSON work is explicit in API names
- the stream writer no longer requires callers to manufacture and keep a
  separate header model purely for serialization

Negative:

- this is an intentionally breaking API change
- downstream code using borrowed parsing or the builder-style writer must move
  to the explicit read/write functions

## Remaining Gap Versus The Cross-Repo Plan

The plan proposes `ReadOptions.symbol_storage` using
`cityjson_types::symbols::SymbolStorageOptions`. That field is not implemented here
because the `cityjson-rs` revision currently used by this repository does not
yet expose that symbol-storage API.

When `cityjson-rs` lands the symbol-storage surface, `cityjson-json` should add
that option directly rather than introducing a local compatibility layer.
