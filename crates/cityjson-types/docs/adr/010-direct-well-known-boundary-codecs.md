# 010: Direct well-known boundary codecs

Date: 2026-07-21

Status: Accepted

## Context

CityJSON boundaries use flat vertex references and offsets. WKB/WKT interoperability
is needed without third-party codec dependencies.

## Decision

WKB/EWKB and WKT/EWKT convert directly to and from `Boundary` plus `Vertices`.
WKT does not pass through WKB, avoiding an intermediate allocation and binary
framing unrelated to text. Both codecs traverse the same CityJSON topology.

ISO WKB is little-endian XYZ and ISO WKT uses explicit `Z`. EWKB/EWKT provide
explicit type selection for multi-point, multi-line-string, multi-polygon,
polyhedral surface, and TIN; SRID is top-level only. EWKT follows PostGIS
`SRID=<number>;` and XYZ-without-`Z` convention.

Surface-backed boundaries serialize as polygons. Solids are flattened where the
target has no equivalent shell grouping. Rings serialize closed and parse back
as CityJSON open rings; TIN requires one triangular ring. Parsers accept finite
XYZ, reject empty/singular/M/ZM/trailing input, and preserve occurrences.

## Consequences

The codecs are dependency-free and preserve format-visible ordering. WKB keeps
finite f64 bits; WKT uses shortest finite decimal formatting and cannot preserve
every bit-level distinction. Neither interchange format restores flattened solid
grouping. Future dimensions, EMPTY, or solid topology need an explicit decision.
