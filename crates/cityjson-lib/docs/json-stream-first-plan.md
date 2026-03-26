# JSON Stream-First Plan

## Goal

Make the JSON boundary in `cjlib` reasonably complete for real use without
committing to full semantic aggregation yet.

The immediate target is:

- full-document `CityJSON` read/write
- single `CityJSONFeature` read/write
- `CityJSONFeature` stream read/write
- thin `cjlib` delegation into `serde_cityjson`

The immediate non-goal is:

- strict semantic aggregation of a `CityJSON` + `CityJSONFeature` stream back
  into one larger model

## Decision

Remove the aggregation helper from the public API for now.

That means:

- do not treat `merge_feature_stream` as part of the stable JSON surface
- do not let `CityModel::from_stream` imply semantic aggregation as a central
  workflow
- prefer explicit stream iteration and stream writing APIs

Aggregation can return later once it is implemented at the right semantic
layer, most likely in `cityjson-rs`.

## Why

The remaining hard part in aggregation is not JSON syntax.
It is validated model merge and remapping:

- vertices
- appearance pools
- geometry templates
- geometry instances
- root-state compatibility

That logic is more naturally owned by `cityjson-rs` than by `serde_cityjson`
or `cjlib`.

But `cjlib` does not need that solved yet in order to become useful for real
datasets.
It mainly needs a complete-enough JSON boundary for:

- reading real files
- iterating real feature streams
- writing results back out
- driving downstream tests and experiments

## Target Public Shape

### `serde_cityjson`

Should expose:

- document parse
- feature parse
- feature-stream read
- document serialization
- feature serialization
- feature-stream write

Preferred shape:

```rust
pub fn from_str_owned(input: &str) -> Result<OwnedCityModel>;
pub fn from_feature_str_owned(input: &str) -> Result<OwnedCityModel>;

pub fn read_feature_stream<R: std::io::BufRead>(
    reader: R,
) -> Result<impl Iterator<Item = Result<OwnedCityModel>>>;

pub fn to_string(model: &OwnedCityModel) -> Result<String>;
pub fn to_string_feature(model: &OwnedCityModel) -> Result<String>;

pub fn write_feature_stream<I, W>(writer: W, models: I) -> Result<()>
where
    I: IntoIterator<Item = OwnedCityModel>,
    W: std::io::Write;
```

Iterator-based reading is the default.
If collecting is useful, it should be layered on top:

```rust
let models = read_feature_stream(reader)?.collect::<Result<Vec<_>>>()?;
```

### `cjlib::json`

Should remain a thin facade over `serde_cityjson` for:

- probing
- document parse
- feature parse
- feature-stream read
- document serialization
- feature serialization
- feature-stream write

The facade should not recreate JSON parsing or model merge logic.

## API Changes

Short term:

- remove `merge_feature_stream` from the intended public API
- remove it from examples and docs as a recommended path
- stop treating `CityModel::from_stream` as a semantic aggregation alias

If a compatibility `from_stream` survives temporarily, it should be clearly
documented as transitional and not the center of the design.

Preferred steady-state direction:

- `CityModel::from_slice`
- `CityModel::from_file`
- `json::from_feature_slice`
- `json::read_feature_stream`
- `json::to_string`
- `json::to_feature_string`
- `json::write_feature_stream`

## Implementation Order

1. Adjust the docs to make the stream-first direction explicit.

2. In `serde_cityjson`, keep the current feature parser and iterator path, and
   add feature-stream writing support.

3. In `cjlib::json`, add thin write-side stream helpers that delegate to
   `serde_cityjson`.

4. Remove aggregation from the documented API surface in:

- `docs/json-api.md`
- `docs/guide.md`
- `docs/public-api.md`
- examples that currently imply stream aggregation is the primary path

5. Reduce tests to the workflows we actually want to support now:

- document read/write
- single-feature read/write
- feature-stream iteration
- feature-stream roundtrip writing

6. Leave semantic aggregation as deferred work for a later `cityjson-rs`
   centered design.

## Testing Priority

Add or keep tests for:

- reading a real `.city.json`
- reading a real `.city.jsonl` stream as self-contained models
- writing one feature model back as `CityJSONFeature`
- writing multiple models as a JSONL feature stream
- stream read -> inspect/transform -> stream write roundtrip

Do not make `cjlib` correctness depend on stream aggregation yet.

## Deferred Work

Explicitly defer:

- semantic aggregation of feature streams
- root-level pool merge and remapping
- appearance/template merge semantics
- a stable merge API in `cjlib`

When that work resumes, the first question should be:

How much of the merge/remap logic belongs in `cityjson-rs` as validated
submodel operations?

## Done For This Phase

This phase is complete when:

- `cjlib` can read and write ordinary `CityJSON`
- `cjlib` can read and write `CityJSONFeature`
- `cjlib` can iterate feature streams from real files
- `cjlib` can write feature streams back out
- the public docs no longer present aggregation as a required JSON capability
