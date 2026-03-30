# Plan: Reproducible 3DBAG Benchmark Corpus Preparation

## Goal

Replace the current benchmark corpus with a reproducible dataset derived from
3DBAG CityJSON downloads and keep the prepared data suitable for all three
storage backends:

- feature-files
- regular CityJSON
- NDJSON / CityJSONSeq

The target corpus size is approximately 270,000 `CityObject`s total.

## Source of Truth

Use the pinned 3DBAG tile index:

- `https://data.3dbag.nl/v20250903/tile_index.fgb`

Do not use a moving `latest` endpoint in the prep pipeline. The script should
record the exact tile index URL and a content hash of the downloaded index in a
manifest file so the input set can be reproduced later.

The tile index exposes the CityJSON download URL per tile. The prep script will
read the `cj_download` field from each selected record and download that file
directly.

## Language Choice

Implement the prep tool in Rust.

Reasoning:

- the repository is already Rust-first
- the prep logic can reuse the existing JSON and file-handling patterns
- the script can stay self-contained and deterministic
- the tool can call out to `cjseq` and `cjval` for format conversion and
  validation without introducing a separate Python dependency stack

The Rust binary should live in `src/bin/` and should be wired into `justfile`
with a dedicated recipe for release-mode execution.

## Selection Rule

Select tiles deterministically from the tile index until the cumulative object
count is close to 270,000.

The selection algorithm should:

1. read all tiles from the fixed tile index
2. sort them by a stable key, such as `tile_id`
3. accumulate the tile-level `CityObject` count from the index metadata
4. stop once the total is within a small acceptance window around the target
   count

Recommended acceptance window:

- target: `270,000`
- acceptable range: `265,000` to `275,000`

This avoids depending on the total corpus size of the release while keeping the
benchmark dataset reasonably stable.

The selected tile list must be written to a manifest so the exact subset can be
reused without re-running the selection step.

## Prep Pipeline

The prep tool should build the corpus in a staging directory and only move the
final outputs into place after all validation succeeds.

### 1. Download and lock the tile list

- download the tile index once
- compute and store its checksum
- extract the selected tile metadata into a manifest
- store per-tile metadata in the manifest:
  - `tile_id`
  - download URL
  - bbox
  - source object count
  - source file size if available

### 2. Download the source CityJSON tiles

- download each selected `cj_download` URL
- save the raw files under a staging directory
- preserve the tile identifiers in the file names so the mapping stays obvious
- retry transient network failures, but fail hard on checksum or parse errors

### 3. Validate the raw CityJSON

- run `cjval` on every downloaded CityJSON file
- reject files that fail validation
- record validation success in the manifest

### 4. Normalize and convert to backend layouts

The pipeline should produce three output trees from the validated source tiles.

#### CityJSON

- keep the validated downloaded CityJSON tiles as the `cityjson` backend input
- if a normalization pass is needed, perform it deterministically and record the
  exact command in the manifest

#### NDJSON / CityJSONSeq

- use `cjseq` to convert each validated CityJSON tile into a line-oriented
  CityJSONSeq file
- preserve record order deterministically
- validate the resulting sequence files with `cjval`

#### feature-files

- split each selected CityJSON tile into individual feature files
- preserve the source metadata in the expected ancestor metadata tree
- ensure every emitted feature file is validated
- if `cjseq` can be used to help split or normalize the source tile first, do
  that; otherwise use the Rust prep tool for the split step and still validate
  the results with `cjval`

### 5. Build the final directory tree

The final tree should mirror the current benchmark expectations:

- `feature-files/`
- `cityjson/`
- `ndjson/`

The script should write the three trees atomically, or at least avoid partial
publication if a later validation step fails.

## Reproducibility Contract

The prep process must be reproducible from the repo plus the pinned tile index.

To make that true, the script should emit:

- the tile index URL
- the tile index checksum
- the selected tile IDs and their counts
- the exact total object count
- the download URL for every tile
- per-file checksums for the generated artifacts
- the commands used for conversion and validation

The manifest should be sufficient to reproduce the same corpus without rerunning
the tile-selection logic.

## Validation Strategy

The script should fail unless all of the following are true:

- every selected source tile downloads successfully
- every source tile passes `cjval`
- every converted NDJSON file passes `cjval`
- the feature-files tree is internally consistent
- the final counts match the manifest
- the total object count stays within the target window

After the data prep succeeds, run the benchmark and test gates against the
prepared corpus to confirm the tree is usable by `cjindex`.

## Deliverables

Implement the following artifacts:

- a Rust prep binary under `src/bin/`
- a `just` recipe for the release-mode prep command
- a manifest file describing the selected tiles and output checksums
- updated benchmark-data documentation in `README.md` or `docs/` describing how
  to regenerate the corpus

## Open Questions / Risks

- The tile index may not expose the object-count field under the same name on
  every release. The prep tool should detect the field once and fail with a
  clear error if it is missing.
- The exact conversion capabilities of `cjseq` may not cover every splitting
  step directly. If so, the Rust prep tool should own the split step and use
  `cjseq` for the CityJSON <-> CityJSONSeq conversion and validation pieces.
- Tile counts will change across 3DBAG releases. The manifest is the source of
  truth for a given prepared corpus; the script should not silently drift to a
  different dataset.
