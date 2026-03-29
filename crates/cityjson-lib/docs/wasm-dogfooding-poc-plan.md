# Wasm Dogfooding Proof Of Concept

## Goal

Build a simple but full wasm application that exercises:

- parse
- model storage
- traversal
- geometry extraction
- serialization

The real question is not "can we compile something to wasm", but whether the
remaining generic parameters in `cityjson-rs` are worth keeping because they
materially help wasm or other FFI targets.

## Current Shape Of The Stack

`cjlib` already wraps `cityjson::v2_0::OwnedCityModel` directly:

- [`src/model.rs`](../src/model.rs)

The public `v2_0` model in `cityjson-rs` already fixes `ResourceId32`:

- [`../cityjson-rs/src/v2_0/citymodel.rs`](../../cityjson-rs/src/v2_0/citymodel.rs)
- [`../cityjson-rs/src/resources/handles.rs`](../../cityjson-rs/src/resources/handles.rs)

That means the public generics that still matter are:

- vertex index width: `VR: VertexRef`
- string storage: `SS: StringStorage`

## Recommendation

I would build the wasm proof so that the foreign boundary is narrow and stable,
and only the internal model variant changes between builds.

Expected conclusion before measuring:

- `ResourceId` parameterization can be removed safely.
- `StringStorage` parameterization is probably not worth keeping for wasm or
  FFI-facing models.
- `VertexIndex` parameterization is the only one that looks worth a real wasm
  experiment.

The important design rule is:

- the wasm boundary should expose simple byte buffers and plain result structs
- the generic model should stay entirely behind that boundary
- the proof should compare multiple internal model variants behind the exact
  same host API

## Why

### Resource IDs

This is already effectively fixed in the public `v2_0` API through
`ResourceId32` and typed handles. Wasm consumers should not need configurable
resource-id internals.

### String Storage

Borrowed string storage only switches parse-time behavior in `serde_cityjson`:

- [`../../serde_cityjson/src/de/parse.rs`](../../serde_cityjson/src/de/parse.rs)

That means:

- it is mainly a deserializer concern, not a stable model-boundary concern
- it introduces lifetime constraints that are awkward across wasm/FFI
- it does not avoid the host-to-wasm memory copy
- borrowed mode rejects escaped strings that require owned allocation during
  parsing

That makes it a weak fit for a long-lived wasm model.

### Vertex Indices

Vertex index width directly affects storage in the flat boundary buffers:

- [`../../cityjson-rs/src/backend/default/boundary.rs`](../../cityjson-rs/src/backend/default/boundary.rs)
- [`../../cityjson-rs/src/backend/default/vertex.rs`](../../cityjson-rs/src/backend/default/vertex.rs)

This is the one parameter that can plausibly buy real wasm memory savings.

## Proof Of Concept Shape

Create a tiny wasm adapter crate as a `cdylib`, then use a minimal HTML and JS
host page.

Expose only a narrow wasm API:

```rust
analyze_cityjson(bytes: &[u8]) -> JsValue
extract_first_mesh(bytes: &[u8]) -> MeshBuffers
roundtrip(bytes: &[u8]) -> Vec<u8>
```

The host page should treat wasm as a pure compute engine:

- JS owns file input and browser events
- wasm owns parse, model creation, traversal, extraction, and serialization
- JS only receives summary values and render-ready buffers

The app flow should be:

1. Load or drop a `.city.json` file.
2. Parse it inside wasm.
3. Show counts and timings.
4. Extract the first geometry and render a simple wireframe in a `<canvas>`.
5. Serialize back to CityJSON and show output size.

That is enough to exercise the real stack without turning the experiment into a
frontend project.

## Concrete Deliverable

The proof should feel like a tiny single-page "CityJSON inspector":

- drag and drop a file
- see parse success or failure
- see a small table of model counts
- see the first geometry rendered as a wireframe
- click a roundtrip button and get output bytes and timing

That is enough to answer the architecture question because it exercises the
same end-to-end path that a real foreign consumer would use.

## Proposed Repo Shape

I would keep the proof inside this repository so that it becomes part of the
dogfooding record.

Suggested layout:

```text
wasm/
  cjlib-wasm/
    Cargo.toml
    src/lib.rs
    src/api.rs
    src/convert.rs
    src/metrics.rs
    static/
      index.html
      app.js
      style.css
```

Purpose of each piece:

- `src/lib.rs`
  Entry points exported to JS.
- `src/api.rs`
  Stable wasm-facing request and response structs.
- `src/convert.rs`
  Lower a `CityModel` into render-ready buffers and summary values.
- `src/metrics.rs`
  Small timing and memory helpers.
- `static/index.html`
  Minimal UI shell.
- `static/app.js`
  Browser-side glue for file loading, calling wasm, and drawing.
- `static/style.css`
  Light presentation only. No framework needed.

## Wasm API Contract

Avoid exporting rich Rust model objects to JS. That would hide the real
question under binding complexity.

Prefer a very small contract like this:

```rust
pub struct AnalyzeResult {
    pub root_kind: String,
    pub cityobject_count: u32,
    pub geometry_count: u32,
    pub vertex_count: u32,
    pub semantic_count: u32,
    pub material_count: u32,
    pub texture_count: u32,
    pub parse_millis: f64,
    pub model_bytes_estimate: u64,
}

pub struct MeshBuffers {
    pub positions: Vec<f32>,
    pub indices: Vec<u32>,
    pub object_id: String,
    pub geometry_index: u32,
}
```

Suggested exported functions:

```rust
analyze_cityjson(bytes: &[u8]) -> AnalyzeResult
extract_first_mesh(bytes: &[u8]) -> MeshBuffers
roundtrip(bytes: &[u8]) -> RoundtripResult
```

Where `RoundtripResult` includes:

- output byte length
- serialize time
- optional equality or semantic-equivalence flag if you choose to check it

This keeps the boundary realistic for future C ABI and wasm targets:

- input is bytes
- output is summaries or flat buffers
- no exposed generic Rust object graphs

## Internal Adapter Strategy

The adapter crate should hide the actual model type behind one internal alias:

```rust
type Model = ...;
```

Then the exported functions all call generic internal helpers:

```rust
fn parse_model(bytes: &[u8]) -> Result<Model>;
fn summarize(model: &Model) -> AnalyzeResult;
fn first_mesh(model: &Model) -> Result<MeshBuffers>;
fn serialize(model: &Model) -> Result<Vec<u8>>;
```

That makes the comparison fair because the host behavior stays constant while
the storage strategy changes.

## Build Variants

Build the same adapter in three variants.

### 1. `owned-u32`

```rust
type Model = cityjson::v2_0::OwnedCityModel;
```

This is the baseline and matches `cjlib` today.

### 2. `owned-u16`

```rust
type Model = cityjson::v2_0::CityModel<u16, OwnedStringStorage>;
```

This tests whether smaller vertex indices materially help wasm memory and
payload size on suitably small datasets.

### 3. `borrowed-u32`

Use borrowed parsing only for a one-shot benchmark:

- parse
- analyze
- drop

Do not try to keep this model alive across calls or expose it as a persistent
wasm object.

### Optional 4. `owned-u64`

I would not start with this, but it can be useful as a sanity check if you want
to confirm that index width really moves memory in the expected direction.

## Feature And Build Strategy

Use cargo features to choose the model variant at compile time:

```toml
[features]
default = ["owned-u32"]
owned-u32 = []
owned-u16 = []
borrowed-u32 = []
```

Then one small internal module selects the active type alias.

The first build target should be:

- `wasm32-unknown-unknown`

And the first build tool should be whichever is least invasive for the repo.
The exact bundler is not important. A static page plus generated wasm artifacts
is enough.

## Frontend Flow

The browser UI only needs four visible zones:

### 1. Input

- file picker
- drag and drop area
- sample-file selector if you want canned test cases

### 2. Parse Summary

Show:

- model kind
- parse time
- counts for cityobjects, geometries, vertices, semantics, materials, textures
- current wasm memory pages before and after parse if available

### 3. Geometry Preview

Render only one geometry:

- first city object with renderable geometry
- first supported geometry in that object
- triangulate nothing if you want to stay minimal; wireframe is enough

The point is not graphics fidelity. The point is proving that the wasm
boundary can extract useful render-ready geometry from the typed model.

### 4. Roundtrip

Show:

- serialize time
- output byte size
- whether re-parsing the output succeeds

That last check matters because it proves the full read-write-read cycle.

## Geometry Extraction Scope

Do not try to support every CityJSON geometry in the first pass.

I would support only:

- `MultiSurface`
- `CompositeSurface`
- `Solid`

For unsupported cases:

- return a clear "not previewable" status
- still allow parse and roundtrip

That keeps the proof aligned with the architecture question instead of getting
stuck in rendering completeness.

## Dataset Matrix

Use a small fixed dataset set and keep it explicit in the note or in a manifest.

Minimum useful matrix:

1. `tiny-minimal`
   A very small valid CityJSON file already in repo test data.

2. `medium-realistic`
   A moderately sized real file with enough geometry and attributes to exercise
   the storage layout meaningfully.

3. `escaped-strings`
   A valid file containing escaped strings. This is specifically to probe the
   failure mode of borrowed string storage.

4. `u16-overflow`
   A file with more than `65_535` vertices. This is specifically to probe the
   failure mode and ergonomics of `u16` vertex storage.

5. `feature-root`
   Optional, but useful if you want to test whether the wasm proof should admit
   `CityJSONFeature` inputs as well as full documents.

The point of the matrix is to expose qualitative behavior, not just throughput.

## Metrics To Record

Record the same small table for every variant and every dataset:

- parse success or failure
- failure category
- parse wall time
- serialize success or failure
- serialize wall time
- input bytes
- output bytes
- wasm memory pages before parse
- wasm memory pages after parse
- wasm memory pages after extraction
- vertex count
- extracted position buffer length
- extracted index buffer length

If the browser memory readings are noisy, also record a rough model-size
estimate inside wasm from the lengths of the main backing vectors. It does not
need to be perfect. It only needs to track directionally.

## Decision Criteria

This proof should finish with an explicit decision table.

### Keep `ResourceId` generic only if

- wasm or FFI genuinely needs multiple resource-id layouts at the public
  boundary

I do not expect that to be true.

### Keep `StringStorage` generic only if

- borrowed mode delivers a material parse or memory win
- and that win survives the actual host-to-wasm copy model
- and its failure mode on escaped strings is acceptable
- and it does not poison the foreign boundary with lifetime-driven complexity

I do not expect all of those to hold.

### Keep `VertexRef` generic only if

- `u16` delivers a clear memory win on realistic wasm-sized files
- the overflow boundary is easy to explain and fail loudly
- the additional generic complexity stays mostly internal

This is the only parameter with a realistic chance of passing.

## Phased Execution Plan

### Phase 1. Portability Check

- install `wasm32-unknown-unknown`
- verify `cityjson-rs`, `serde_cityjson`, and `cjlib` can build for wasm
- gate or avoid obviously non-wasm file APIs where needed

Success condition:

- the stack builds with a minimal wasm adapter crate

### Phase 2. Minimal Functional Demo

- implement `analyze_cityjson`
- implement `roundtrip`
- build the tiny host page

Success condition:

- a browser can parse a file and show summary values

### Phase 3. Geometry Proof

- implement `extract_first_mesh`
- render a simple wireframe preview

Success condition:

- at least one representative geometry can be drawn from the typed model

### Phase 4. Variant Comparison

- build `owned-u32`
- build `owned-u16`
- build `borrowed-u32`
- run the dataset matrix

Success condition:

- there is enough evidence to simplify or keep each remaining generic

## Risks And Non-Goals

Non-goals:

- a polished viewer
- complete CityJSON rendering coverage
- long-lived mutable model handles exposed to JS
- a stable public wasm API

Risks:

- wasm target portability issues surface before the model comparison starts
- borrowed mode may be too constrained to be worth benchmarking beyond a
  one-shot parse path
- `u16` may complicate code more than the saved memory justifies

## Expected Outcome

The most likely useful end state is:

- `cjlib` continues to expose only owned models
- `cityjson-rs` fixes resource ids publicly and probably string storage too
- vertex-index configurability remains only if the wasm data shows a meaningful
  benefit

If the proof does not show a clear `u16` win, the cleanest outcome is to
simplify the public model all the way down to the current owned `u32` shape.

## What To Measure

Measure exactly these things:

- parse time
- wasm linear-memory growth
- size of returned geometry/index buffers
- failure behavior when the model exceeds `u16` vertex capacity

Use the same dataset set for every variant:

- tiny valid file
- medium realistic file
- file with escaped strings
- file with more than `65_535` vertices

## Interpreting The Result

### If `borrowed-u32` wins only slightly

Remove `StringStorage` from the main public model surface, or demote it to a
parser-internal or niche API.

That is the most likely outcome.

### If `owned-u16` gives a real wasm win

Keep vertex index configurability, but keep it internal or advanced-user
oriented. The default public story should still be `u32`.

### If `owned-u16` does not materially help

Collapse the model to `u32` vertex indices as well.

## Suggested Implementation Order

1. Add a tiny wasm adapter crate that depends on `cjlib`.
2. Feature-gate or avoid any file-oriented APIs in the wasm path.
3. Implement `analyze_cityjson`, `extract_first_mesh`, and `roundtrip`.
4. Add the three model variants behind cargo features.
5. Run the same inputs through all variants and record memory and timing.
6. Decide whether to keep only `VertexRef`, or simplify all remaining
   parameterization away.

## Current Blocker

I attempted a target check from `cjlib` with:

```bash
cargo check --target wasm32-unknown-unknown
```

That currently fails because the target is not installed in this environment:

```text
can't find crate for `core`
the `wasm32-unknown-unknown` target may not be installed
```

So the next concrete step is to install that target, then verify whether the
current `cjlib -> cityjson-rs -> serde_cityjson` stack compiles cleanly before
building the adapter crate.
