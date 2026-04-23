# ADR 2 And ADR 3 Borrowed Strings Decision

This note records the decision on whether the next `cityjson-arrow` and
`cityjson-parquet` optimization slice should adopt
`cityjson::v2_0::BorrowedCityModel<'a>` or otherwise move the semantic boundary
to borrowed string storage.

## Decision

Do not adopt `BorrowedCityModel<'a>` as the semantic boundary for the current
optimization program.

For the current architecture:

- `OwnedCityModel` remains the semantic source and sink
- `cityjson-arrow` and `cityjson-parquet` do not gain parallel borrowed-model APIs
- `cjlib` stays centered on one owned semantic wrapper
- borrowed `&str` values are still allowed as short-lived internal references
  inside binders, builders, and table walkers, but they must not become the
  public or cross-module semantic contract

## Why This Is The Decision

### 1. The ADR Boundary Is Already Chosen

ADR 1 and ADR 3 both fix `cityjson::v2_0::OwnedCityModel` as the semantic
boundary.

That means the current optimization question is not "owned or borrowed semantic
model?" but "how do we make the chosen owned semantic boundary cheaper to move
through stream and package transport?"

Changing that boundary now would reopen the core ADR decision instead of
finishing the execution model those ADRs already selected.

### 2. The Current Hot Path Is Dominated Elsewhere

The benchmark follow-up shows that the remaining cost is still concentrated in
whole-model materialization:

- `ModelEncoder::encode` still goes through `encode_parts`
- `ModelDecoder::decode` still goes through `read_model_stream` and
  `decode_parts`
- the live stream reader still uses eager `read_to_end`
- the live stream writer still buffers per-table payloads before writing
- the package reader still decodes all tables and rebuilds full canonical parts
  before reconstruction

Those costs are larger and more structurally important than string storage
ownership alone.

### 3. Borrowed Strings Do Not Remove Arrow Or Parquet Buffer Ownership

On export, Arrow arrays and Parquet payloads still need owned encoded buffers.

A borrowed semantic model could reduce some intermediate `String` cloning, but
it would not remove:

- Arrow string array allocation
- Arrow IPC or file serialization
- package payload writing
- projected JSON text materialization for attribute fallback columns

So borrowed strings are not a direct path to zero-copy export in the current
transport design.

### 4. The Current Import Path Already Reconstructs Owned Values

The import path currently builds owned semantic values while decoding:

- Arrow string columns are converted back into owned `String` values
- projected attributes are reparsed from JSON text into owned attribute values
- several decode helpers already target owned semantic variants

Adding `BorrowedCityModel<'a>` would therefore not be a local substitution.
It would require a separate lifetime-bound import surface, different helper
signatures, and a new ownership story across stream readers, memory maps, and
downstream wrappers.

### 5. Lifetime Coupling Would Spread Across Crate Boundaries

If `cityjson-arrow` or `cityjson-parquet` returned a borrowed semantic model, the model
lifetime would need to be tied to:

- the input byte buffer for live stream reads
- the `mmap` or backing file lifetime for package reads
- any downstream wrapper such as `cjlib::CityModel`

That would complicate the public API, FFI-facing code, and test surface for a
gain the current benchmarks do not identify as primary.

## What Is Still Allowed

This decision is not a ban on borrowing inside the implementation.

The optimization work should still remove unnecessary short-lived string
allocations where it is clean to do so, for example:

- binding `&str` views from Arrow arrays instead of immediately calling
  `.to_string()`
- avoiding duplicate `String` staging before Arrow builders consume the values
- decoding package manifest and table identifiers without cloning more than
  needed

Those are internal micro-optimizations. They do not require a borrowed semantic
model.

## When To Revisit This Decision

Revisit only if later split benchmarks show that string ownership is still a
dominant cost after the current execution-model work lands.

Concrete revisit triggers:

- `convert_decode_parts` remains dominated by string allocation after eager
  stream/package buffering and whole-parts materialization are removed
- a new read-only inspection API is intentionally introduced as a lifetime-bound
  view over Arrow or package storage rather than as an `OwnedCityModel`
- the import path gains stable zero-copy string views from Arrow buffers and
  the measured savings are large enough to justify a new semantic API surface

Until one of those conditions is met, the correct next move is to optimize the
current owned-boundary implementation rather than replacing the boundary.
