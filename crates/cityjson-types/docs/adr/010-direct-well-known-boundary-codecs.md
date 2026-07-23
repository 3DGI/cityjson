# 010: Direct WKT/WKB boundary encoding and decoding

Date: 2026-07-21

Status: Accepted

## Context

CityJSON boundaries store references into a separate vertex pool in one flat buffer.
Implicit-end offset buffers describe topology: every offset starts a child whose end
is the next offset, or the child buffer's length for the last child. Thus `rings`
partitions vertex references, `surfaces` partitions rings, `shells` partitions
surfaces, and `solids` partitions shells.

Interoperability belongs in this foundation crate, where a third-party geometry
model or geometry conversion library would add a workspace-wide dependency, require
intermediate geometry allocations, and impose another model's topology and dialect decisions. The
flattened representation already contains everything required for direct WKT/WKB
encoding and decoding.

## Decision

WKB/EWKB and WKT/EWKT convert directly to and from `Boundary` plus `Vertices`.
Every implicit-end range becomes one nested child: ring ranges become coordinate
sequences, surface ranges become polygon ring lists, shell ranges select surfaces,
and solid ranges select shells. WKB encodes these ranges as counts and binary child
records; WKT encodes them as parenthesis levels and comma-separated text. Routing
WKT through WKB would allocate and immediately decode an unnecessary binary buffer
and couple text formatting to irrelevant binary framing.

Both format implementations apply the same topology rules:

- points follow the vertex-reference buffer and lines follow ring ranges;
- surfaces become polygons, retaining their ordered rings;
- solids flatten to polygons in stored `solids -> shells -> surfaces` order when
  the selected interchange type has no solid/shell grouping;
- all stored ordering and every repeated vertex-reference occurrence are retained;
- polygon rings serialize closed and parse into CityJSON's open-ring form;
- parsing rebuilds the requested target boundary where the interchange type cannot
  express the original shape, with shared structural, reference, coordinate, ring,
  and trailing-input validation semantics.

ISO WKB/WKT support standards-based interchange; PostGIS EWKB/EWKT support the
extended surface types and SRID convention used by database/GIS integrations. ISO
WKB is little-endian XYZ and ISO WKT requires explicit `Z`. EWKB/EWKT require the
caller to select multi-point, multi-line-string, multi-polygon, polyhedral-surface,
or TIN framing explicitly. Point and line boundaries have one compatible selection.
A surface-backed boundary, however, can be represented as a generic `MultiPolygon`
or as a `PolyhedralSurface`; it can also be a TIN when every surface is one
triangular ring. Solid boundaries use those same surface encodings after shell
grouping is flattened. CityJSON topology does not record which of these PostGIS
semantic labels the caller intends. EWKT uses PostGIS's `SRID=<number>;` prefix and
XYZ-without-`Z` spelling. SRIDs are top-level only, preventing conflicting nested
metadata.

Only finite XYZ is accepted. XY would invent height; M and ZM have no lossless
destination in the three-coordinate vertex model. Empty and singular geometries,
mixed or nested SRIDs, ambiguous dimension/type variants, malformed or trailing
input, invalid rings, and non-triangular TIN members are rejected rather than
guessed at or converted lossily.

## Consequences

The WKT/WKB implementations are dependency-free and share topology behavior. WKB preserves finite
`f64` bits. WKT emits shortest round-trippable finite decimals: values round-trip,
but original decimal spelling is lost, and other implementations need not preserve
identical bits through decimal conversion.

### GEOS interoperability

`Boundary` can encode XYZ geometry as either little-endian ISO WKB or PostGIS
EWKB. GEOS's WKB reader automatically detects and reads both dialects. GEOS can
also write either dialect: since GEOS 3.10, its C API selects the output with
`GEOSWKBWriter_setFlavor`, using `GEOS_WKB_ISO` for ISO WKB and
`GEOS_WKB_EXTENDED` for EWKB.

The safe Rust [`geos`](https://docs.rs/geos/11.1.1/geos/) wrapper does not expose
that flavor selection. Its `WKBWriter` returns WKB bytes and exposes output
dimension, byte order, and SRID controls, but XYZ output uses GEOS's default
extended flavor. Consequently, the high-level Rust API can emit XYZ EWKB but
cannot request XYZ ISO WKB. This is not a limitation of GEOS or its low-level
Rust C bindings: `geos-sys` exposes the flavor functions for GEOS 3.10 and
newer.

The Rust `WKBWriter` predates GEOS's flavor-selection API, and the Rust project
states that it wraps only a subset of GEOS. No documented intentional rejection
of ISO WKB output was found, so this is treated as a missing high-level wrapper
rather than a format or architecture constraint. Until the wrapper exposes WKB
flavor, a safe Rust GEOS test can verify that GEOS decodes CityJSON-generated
ISO WKB, but it cannot perform a same-dialect XYZ ISO WKB round trip without
using `geos-sys` directly.

Flattening loses solid and shell boundaries. Polygon order survives, but parsing
cannot infer those groups; the requested target boundary supplies new grouping.
Future dimensions, `EMPTY`, alternative byte orders, or lossless solid topology
need a separate explicit decision.
