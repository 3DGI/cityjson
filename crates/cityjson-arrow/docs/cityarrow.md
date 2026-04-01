# cityarrow

`cityarrow` is the live Arrow IPC transport crate for `cityjson-rs`.

It converts `OwnedCityModel` values into canonical Arrow transport tables and
streams them between processes without changing the semantic model.

## What It Provides

- `ModelEncoder` and `ModelDecoder` for live Arrow IPC stream transport
- schema definitions for the canonical tables and manifest
- the shared package contract used by `cityparquet`

## Related Documents

- [Arrow IPC stream and table layout specification](cityjson-arrow-ipc-spec.md)
- [Shared package schema](package-schema.md)
- [Transport design](design.md)

## Public Surface

The crate exposes:

- `ModelEncoder` and `ModelDecoder`
- the canonical schema and manifest types
