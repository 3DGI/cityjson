# cityarrow Live Arrow IPC Stream Specification

This document describes the current live transport format used by
`cityarrow::ModelEncoder` and `cityarrow::ModelDecoder`.

## Version

- stream magic: `CITYARROW_STREAM_V3\0`
- package schema carried in the prelude header:
  `cityarrow.package.v2alpha2`

## Layout

```text
STREAM_MAGIC
prelude_len: u64 little-endian
prelude_json
frame_0
frame_1
...
0xFF
```

## Prelude

`prelude_json` is UTF-8 JSON with:

- `header: CityArrowHeader`
- `projection: ProjectionLayout`

The prelude is intentionally small. It carries only the semantic header and the
projected column layout needed to validate later frames.

## Frames

Each frame encodes one canonical table batch:

```text
table_tag: u8
rows: u64 little-endian
arrow_ipc_stream_payload
```

- `table_tag` identifies the canonical table
- `rows` is the declared row count for validation
- the payload is one Arrow IPC stream written with Arrow `StreamWriter`

The payload is self-delimiting, so the writer can stream frames directly
without knowing payload lengths up front.

## End Marker

The live stream ends with a single tag byte `255`.

## Reader Rules

- the reader must validate `STREAM_MAGIC`
- the reader must decode the JSON prelude first
- frames must appear in canonical order
- duplicate table tags are invalid
- required tables may not be skipped
- each decoded payload schema must match the canonical schema for that table
- each decoded batch row count must match the declared `rows` value
