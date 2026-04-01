# cityarrow

`cityarrow` is the Arrow IPC transport crate for `cityjson-rs`.

It converts `OwnedCityModel` values into the canonical `CityModelArrowParts`
package shape and reads them back without changing the semantic model.

## What It Provides

- conversion between `OwnedCityModel` and transport parts
- package write/read support for Arrow IPC file packages
- schema definitions for the canonical tables and manifest
- the shared package contract used by `cityparquet`

## Related Documents

- [Arrow IPC package layout specification](cityjson-arrow-ipc-spec.md)
- [Shared package schema](package-schema.md)
- [Transport design](design.md)

## Public Surface

The crate exposes:

- `to_parts` and `from_parts`
- `write_package_ipc_dir` and `read_package_ipc_dir`
- the canonical schema and manifest types
