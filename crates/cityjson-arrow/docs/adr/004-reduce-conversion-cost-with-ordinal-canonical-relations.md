# Reduce Conversion Cost With Ordinal Canonical Relations

## Status

Accepted

## Context

[ADR 2](002-address-transport-performance-bottlenecks.md) and
[ADR 3](003-separate-live-arrow-ipc-from-persistent-package-io.md) established
two things:

- the remaining benchmark gap should be treated as an implementation problem
- the public transport boundary should stay centered on
  `OwnedCityModel -> live stream/package -> OwnedCityModel`

The first ADR 3 execution-model refactor delivered the new public surfaces and
removed the old directory-oriented package contract, but the downstream split
benchmarks showed that conversion still dominated the hot path.

For the pinned 3DBAG tile case, the diagnostic split run showed:

- `convert_encode_parts`: about `42.1 ms`
- `convert_decode_parts`: about `28.3 ms`
- `stream_read_parts`: about `0.60 ms`
- `package_read_parts`: about `0.71 ms`

That reading made the next optimization target clear: reduce conversion cost
before spending more effort on transport framing.

An implementation review identified four avoidable costs in the canonical
schema and conversion code:

- non-metadata canonical tables repeated `citymodel_id` even though a stream or
  package contains exactly one model
- geometry and cityobject-child relations repeated string ids that were then
  cloned, hashed, and compared again during import
- decoder staging keyed geometry attachment by `String` instead of ordered
  cityobject position
- semantic and template-semantic reconstruction rebuilt canonical order
  through clone-and-sort work even though canonical ordinals already define it

## Decision

`cityarrow` and `cityparquet` will make the internal canonical transport
contract more ordinal and less string-keyed.

The implementation rules are:

1. keep the public semantic boundary unchanged:
   `ModelEncoder` / `ModelDecoder` and `PackageWriter` / `PackageReader`
2. remove `citymodel_id` from all non-metadata canonical tables
3. replace internal cityobject relations with ordinal references where the
   relation is local to one model:
   - `geometries.cityobject_ix`
   - `geometry_instances.cityobject_ix`
   - `cityobject_children.parent_cityobject_ix`
   - `cityobject_children.child_cityobject_ix`
4. stage import attachments in vector-backed structures keyed by ordinal index
   rather than `HashMap<String, ...>`
5. consume semantic and template-semantic rows in canonical ordinal order
   without clone-and-sort rebuilds
6. cut the package schema to `cityarrow.package.v2alpha2`

This is an internal transport-schema change, not a public semantic API change.
No compatibility reader will be kept for the previous alpha package schema.

## Consequences

Good:

- stream and package payloads become smaller because repeated `citymodel_id`
  columns and repeated string relations disappear
- write-side conversion does less string cloning and row construction
- read-side conversion does less hashing, string lookup, and grouped staging
- the transport contract is cleaner: semantic ids stay where they are required
  for reconstruction and external identity, while local attachment edges use
  compact ordinals

Trade-offs:

- the package schema is breaking again and remains alpha-only
- canonical tables are less self-describing when inspected manually because
  some relations are ordinal instead of string-based
- the importer becomes more order-sensitive and depends more heavily on
  canonical ordering invariants
- this slice does not remove the remaining `encode_parts` cost center

## Results Snapshot: 2026-04-02

The first run after this slice was the downstream `cjlib` campaign
`cityarrow v2alpha2 conversion cleanup`.

Compared with the previous campaign `cityarrow refactor 9f3d51e`:

| Path | Previous | Current | Delta |
| --- | --- | --- | --- |
| `cityarrow` tile read | `28.59 ms` | `22.56 ms` | `-21.1%` |
| `cityarrow` cluster read | `115.30 ms` | `93.46 ms` | `-18.9%` |
| `cityarrow` tile write | `59.59 ms` | `56.35 ms` | `-5.4%` |
| `cityarrow` cluster write | `211.06 ms` | `185.85 ms` | `-11.9%` |
| `cityparquet` tile read | `28.42 ms` | `22.68 ms` | `-20.2%` |
| `cityparquet` cluster read | `114.32 ms` | `92.20 ms` | `-19.3%` |
| `cityparquet` tile write | `58.13 ms` | `52.16 ms` | `-10.3%` |
| `cityparquet` cluster write | `209.50 ms` | `186.51 ms` | `-10.9%` |

Against the same-run `serde_json::Value` baseline, the native read paths now
beat the shared-model JSON path on both pinned fixtures:

| Case | Baseline read | `cityarrow` read | `cityparquet` read |
| --- | --- | --- | --- |
| tile | `25.06 ms` | `22.56 ms` | `22.68 ms` |
| cluster | `103.77 ms` | `93.46 ms` | `92.20 ms` |

Write improved, but it remains far behind the baseline:

| Case | Baseline write | `cityarrow` write | `cityparquet` write |
| --- | --- | --- | --- |
| tile | `7.63 ms` | `56.35 ms` | `52.16 ms` |
| cluster | `35.83 ms` | `185.85 ms` | `186.51 ms` |

The split diagnostics explain the remaining gap.

For the tile fixture:

- `convert_encode_parts`: about `37.46 ms`
- `convert_decode_parts`: about `22.37 ms`
- `stream_write_parts`: about `0.46 ms`
- `stream_read_parts`: about `0.45 ms`
- `package_write_parts`: about `2.84 ms`
- `package_read_parts`: about `0.53 ms`

For the 4x cluster fixture:

- `convert_encode_parts`: about `172.56 ms`
- `convert_decode_parts`: about `94.08 ms`
- `stream_write_parts`: about `5.49 ms`
- `stream_read_parts`: about `2.10 ms`
- `package_write_parts`: about `9.82 ms`
- `package_read_parts`: about `2.32 ms`

The practical reading is:

- this slice succeeded on read
- this slice improved write, but not nearly enough
- the next optimization target is still export conversion, especially
  `encode_parts`, not stream framing or package manifest I/O
