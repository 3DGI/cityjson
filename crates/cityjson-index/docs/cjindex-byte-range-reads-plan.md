# Plan for Proper Indexed Byte-Range Reads in `NDJSON` and `CityJSON`

## Goal

Make the `NDJSON` and regular `CityJSON` backends actually exploit the indexed
offset and length data during reads, instead of rereading whole source files for
each hit.

The target end state is:

- `NDJSON` reads exactly the indexed feature byte range for `get` and query hits
- `CityJSON` reads only the indexed `CityObject` range and indexed shared
  `vertices` range, not the entire tile file
- repeated reads from the same source file can reuse open handles or equivalent
  lightweight state safely

This is performance work, but it must preserve correctness first.

## Current Problem

Today, both backends pay whole-file read cost on every hit:

- `NDJSON` does `fs::read(&loc.source_path)` and then slices the indexed span
- `CityJSON` does `fs::read(&loc.source_path)`, slices one `CityObject`, loads
  `vertices`, and reconstructs a one-object feature payload

That means the index currently helps find the relevant offsets, but the read
path still pays for full-file I/O.

For `CityJSON`, this is compounded by extra per-hit work:

- parse one object fragment
- collect referenced vertices
- build localized vertices
- remap boundary indices
- rebuild a one-object feature package

So there are two problems:

1. too much I/O
2. too much per-hit reconstruction

This plan addresses the first problem directly and structures the second so it
can be optimized next.

## Design Principles

### Honor the index literally

If the index records:

- `offset`
- `length`
- `vertices_offset`
- `vertices_length`

then the read path should operate from those spans, not from the whole source
document.

### Prefer standard file reads before larger machinery

The first implementation should use explicit positioned reads from open files.

Recommended primitives:

- `File::open`
- `FileExt::read_exact_at` on Unix

This is simpler and lower-risk than introducing `mmap` immediately.

### Separate I/O optimization from semantic reconstruction

Do not mix:

- "read the right bytes"
- "rebuild the right semantic model"

The first optimization target is byte acquisition. After that, we can optimize
semantic reconstruction independently.

## NDJSON Plan

### Desired end state

For a single feature read:

1. open the `.jsonl` source file
2. read exactly `offset..offset+length`
3. combine that feature slice with cached or serialized metadata
4. call `cjlib::json::from_feature_slice_with_base(...)`

No whole-file read should remain in the hot path.

### Phase 1: Add a small positioned-read helper

Add a helper in [src/lib.rs](/home/balazs/Development/cjindex/src/lib.rs)
along the lines of:

- `read_exact_range(path: &Path, offset: u64, length: u64) -> Result<Vec<u8>>`

Requirements:

- validate `length` fits in memory allocation limits
- fail clearly on short reads
- keep the helper reusable by both backends

### Phase 2: Refactor `NdjsonBackend::read_one`

Replace the current whole-file strategy with:

- positioned read of the feature span
- no slicing from a whole-file buffer

Keep metadata handling unchanged in the first pass, unless profiling shows
metadata serialization is material.

### Phase 3: Optional open-file reuse

If the positioned-read-only version still leaves measurable overhead:

- add a small cache of open file handles keyed by `PathBuf`

Requirements:

- safe shared access
- bounded cache size
- no correctness dependency on cache hits

This should be an optimization layer, not part of the correctness story.

## CityJSON Plan

### Desired end state

For a single object read:

1. read exactly the indexed `CityObject` span
2. read exactly the indexed shared `vertices` span
3. obtain the base root metadata without rereading the whole tile
4. rebuild the one-object feature payload from those pieces
5. call the typed `feature + base` helper in `cjlib`

No full `.city.json` read should remain in the hot path.

### Phase 1: Make the base metadata source explicit

The current read path depends on having full document bytes because
`cjlib::json::from_feature_parts_with_base(...)` still takes base-document
bytes.

That means a proper CityJSON byte-range read likely needs one of these:

1. store serialized base metadata in the SQLite index
2. store base metadata in an adjacent sidecar cache built at scan/index time
3. extend the helper boundary again so it can accept typed base metadata rather
   than raw base document bytes

Recommended first step:

- store serialized base metadata in the index or source scan result, because it
  is already computed during scanning

Without this step, CityJSON cannot stop rereading the whole file cleanly.

### Phase 2: Add positioned range reads for object and vertices

After base metadata is available without rereading the file:

- read the `CityObject` fragment by indexed span
- read the shared `vertices` fragment by indexed span

The existing extraction helpers should continue to work, because they already
accept byte slices for those fragments.

### Phase 3: Preserve and tighten the vertices cache

The current shared-vertices cache should remain, but its input should change:

- cache parsed vertices keyed by source path
- populate the cache from the positioned `vertices` read
- do not require full base-document bytes to fill the cache

This preserves the one useful reuse mechanism CityJSON already has.

### Phase 4: Remove full-file dependency from `CityJsonBackend::read_one`

Once base metadata is separately available:

- delete the `fs::read(&loc.source_path)` hot-path read
- assemble the `FeatureParts` input from:
  - object fragment bytes
  - shared vertices bytes
  - stored base metadata bytes

That is the main correctness milestone for CityJSON I/O.

## Index and Schema Implications

### NDJSON

No schema change is strictly required for NDJSON if metadata remains available
through the current cached `Meta`.

### CityJSON

CityJSON likely requires a schema or persistence change, because the current
read path depends on whole-document base bytes.

Options:

1. add a `base_metadata_json` blob per source record in SQLite
2. add a source-cache file per indexed CityJSON tile
3. rederive and serialize base metadata at startup and cache it in memory

Recommended option:

- store `base_metadata_json` per source in SQLite

Why:

- it is durable
- it matches the indexed-source model
- it avoids a second filesystem dependency
- it makes the read path self-contained

## Verification Plan

### Correctness

1. all existing release-mode tests must pass
2. CityJSON and NDJSON integration tests must continue to produce identical
   semantic results
3. byte-range reads must fail loudly on truncated or inconsistent spans
4. query and query_iter results must remain unchanged

### Performance

After correctness is restored:

1. rerun the full release benchmark suite
2. compare:
   - NDJSON `get`
   - NDJSON `query`
   - CityJSON `get`
   - CityJSON `query`
3. update the ADR and benchmark results doc with the new implementation notes

## Risks

### CityJSON helper boundary

The largest design risk is that CityJSON still depends on base-document bytes at
the `cjlib` boundary. If that is not addressed cleanly, the read path will keep
dragging whole-file I/O back in.

### Platform-specific positioned reads

`FileExt::read_exact_at` is straightforward on Unix, but any abstraction should
be written carefully if cross-platform behavior matters later.

### Micro-optimizing the wrong layer

If byte-range reads land but CityJSON remains dominated by object-localization
and boundary-remap cost, the next bottleneck will simply become clearer. That
is acceptable. We need to remove the current I/O distortion first.

## Acceptance Criteria

- `NdjsonBackend::read_one` no longer reads the entire source file
- `CityJsonBackend::read_one` no longer reads the entire source file
- CityJSON base metadata is available without rereading the whole tile
- existing integration tests remain green in release mode
- full release benchmarks can quantify the effect of the change

## Follow-up

After byte-range reads land:

- optimize CityJSON semantic reconstruction
- consider open-file caching if positioned reads alone are not enough
- consider whether NDJSON should also cache source file handles or mapped
  buffers for dense tile-local workloads
