# Real-World Coordinates Internally

## Status

Accepted

## Context

`cityjson-rs` used to keep quantized coordinates internally and apply
`transform.scale` and `transform.translate` when real-world values were needed.

That added extra complexity:

- internal code had to care about quantized vs real-world coordinates
- geometry and processing code had to dequantize before doing useful work
- tests and APIs had to carry integer coordinate assumptions
- binary formats such as Arrow and Parquet do not benefit from integer
  quantization anyway, because they already store 64-bit values efficiently

Quantization is mainly useful at serialization boundaries, especially for JSON,
where shorter integer values reduce payload size.

## Decision

Internally, `cityjson-rs` stores vertices only as real-world coordinates
(`f64`).

Quantized coordinates are no longer part of the in-memory model.

If a format needs quantization, that conversion happens only at the boundary:

- on read: quantized input is dequantized into `f64`
- on write: `f64` coordinates may be quantized if the target format expects it
- binary formats that already store `f64` values can pass coordinates through
  directly

## Consequences

Good:

- simpler internal data model
- simpler APIs and fewer coordinate-related types
- geometry code works directly on real-world values
- Arrow/Parquet-style outputs map naturally to the internal representation
- quantization can still be done when needed for JSON export or temporary
  algorithms such as snapping or deduplication

Trade-offs:

- JSON serialization now owns quantization logic
- if exact grid-based behavior is needed for a specific algorithm, that grid has
  to be built explicitly instead of coming from the stored model
