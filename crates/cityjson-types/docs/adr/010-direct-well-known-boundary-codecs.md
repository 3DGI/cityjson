# 010: Direct well-known boundary codecs

Date: 2026-07-21

Status: Accepted

## Context

CityJSON boundaries store references into a separate vertex pool in one flat buffer.
Implicit-end offset buffers describe topology: every offset starts a child whose end
is the next offset, or the child buffer's length for the last child. Thus `rings`
partitions vertex references, `surfaces` partitions rings, `shells` partitions
surfaces, and `solids` partitions shells.

Interoperability belongs in this foundation crate, where a third-party geometry
model or codec would add a workspace-wide dependency, require intermediate geometry
allocations, and impose another model's topology and dialect decisions. The
flattened representation already contains everything required for direct codecs.

## Decision

WKB/EWKB and WKT/EWKT convert directly to and from `Boundary` plus `Vertices`.
Every implicit-end range becomes one nested child: ring ranges become coordinate
sequences, surface ranges become polygon ring lists, shell ranges select surfaces,
and solid ranges select shells. WKB encodes these ranges as counts and binary child
records; WKT encodes them as parenthesis levels and comma-separated text. Routing
WKT through WKB would allocate and immediately decode an unnecessary binary buffer
and couple text formatting to irrelevant binary framing.

Both codecs apply the same topology rules:

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
WKB is little-endian XYZ and ISO WKT requires explicit `Z`. EWKB/EWKT require an
explicit multi-point, multi-line-string, multi-polygon, polyhedral-surface, or TIN
selection because surface-backed CityJSON has several valid representations. EWKT
uses PostGIS's `SRID=<number>;` prefix and XYZ-without-`Z` spelling. SRIDs are
top-level only, preventing conflicting nested metadata.

Only finite XYZ is accepted. XY would invent height; M and ZM have no lossless
destination in the three-coordinate vertex model. Empty and singular geometries,
mixed or nested SRIDs, ambiguous dimension/type variants, malformed or trailing
input, invalid rings, and non-triangular TIN members are rejected rather than
guessed at or converted lossily.

## Consequences

The codecs are dependency-free and share topology behavior. WKB preserves finite
`f64` bits. WKT emits shortest round-trippable finite decimals: values round-trip,
but original decimal spelling is lost, and other implementations need not preserve
identical bits through decimal conversion.

Flattening loses solid and shell boundaries. Polygon order survives, but parsing
cannot infer those groups; the requested target boundary supplies new grouping.
Future dimensions, `EMPTY`, alternative byte orders, or lossless solid topology
need a separate explicit decision.
