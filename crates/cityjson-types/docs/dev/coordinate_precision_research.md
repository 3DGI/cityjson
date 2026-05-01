# How GIS libraries handle the quantized-vs-world coordinate tension

- Done with Claude Opus 4.6 Research mode

**The dominant pattern across established GIS libraries is to dequantize to float64 at the I/O boundary and accept precision loss — but the libraries closest to CityJSON's architecture (LAS point clouds, MVT, TopoJSON) increasingly store source integers and compute world coordinates on demand.** No production library uses a runtime storage enum, and no CityJSON implementation today achieves lossless roundtrip. The evidence validates a variant of your Option 1 — quantized integers as canonical storage with a computed world-coordinate view — with the strongest precedent coming from laspy's dual-access API and vector-tile-js's lazy dequantization. Here is the detailed evidence.

---

## JTS's PrecisionModel is metadata, not storage

JTS (Java Topology Suite) has the most explicit precision architecture in the GIS ecosystem, and its design is widely misunderstood. The `PrecisionModel` class offers three modes — `FLOATING` (full float64), `FLOATING_SINGLE` (float32 semantics), and `FIXED` (grid-snapping at a configurable scale factor). But **PrecisionModel never changes how coordinates are stored in memory**. Even in `FIXED` mode, coordinates are stored as rounded `double` values. Even in `FLOATING_SINGLE` mode, the backing storage is `double`. The JTS Javadoc explicitly confirms: "Coordinates are represented internally as Java double-precision values."

PrecisionModel is attached to geometries indirectly through `GeometryFactory`. Each `Geometry` holds a reference to its creating factory, and `geometry.getPrecisionModel()` delegates to `factory.getPrecisionModel()`. This makes PrecisionModel a per-factory, shared-across-geometries annotation — not a per-coordinate storage decision. The factory constructor methods "do not change the input coordinates in any way. In particular, they are not rounded to the supplied PrecisionModel." Rounding only occurs during **constructive operations** (overlay, buffer) where `makePrecise()` is explicitly called.

The actual storage abstraction is `CoordinateSequence` — a separate interface that controls how coordinates are physically stored. JTS ships three implementations: `CoordinateArraySequence` (array of Coordinate objects), `PackedCoordinateSequence.Double` (flat `double[]` array), and `PackedCoordinateSequence.Float` (flat `float[]` array — the only non-double storage). Crucially, `CoordinateSequence` and `PrecisionModel` are **orthogonal concerns** — you can mix any storage implementation with any precision mode. Custom `CoordinateSequence` implementations are the intended extension point for alternative storage formats.

JTS originally had a concept of separate internal/external coordinate representations (possibly integer-scaled), but this was abandoned. The deprecated `toInternal()`/`toExternal()` methods carry the Javadoc note: "no longer needed, since internal representation is same as external representation." **Martin Davis's design rationale was about computational robustness, not storage efficiency.** David Skea recommended the explicit precision model during JTS's initial development (2000–2001) to handle the fundamental floating-point robustness problem in computational geometry — ensuring snap-rounding during overlay operations.

GEOS (the C/C++ port) simplifies this further. Its C API exposes precision as a single `double gridSize` parameter via `GEOSGeom_setPrecision()`, which returns a **new geometry** with rounded coordinates. Shapely 2.x wraps this as `shapely.set_precision(geometry, grid_size)`, also returning a new immutable geometry. None of these libraries ever store integer coordinates internally.

**Mapping to your options:** JTS's architecture validates the principle of separating storage format from precision semantics. Its `CoordinateSequence` interface is a direct precedent for a Rust trait that could have an integer-backed implementation returning `f64` on access. But JTS itself never implements integer-backed storage — it always uses `double`.

---

## The "always-dequantize" pattern dominates mainstream GIS

GDAL, PostGIS, PDAL, citygml4j, and 3DCityDB all follow the same pattern: **dequantize to float64 on read, process in float64, re-quantize on write**. Source integers are discarded at the I/O boundary.

**GDAL/OGR** stores all coordinates as `double` internally — `OGRSimpleCurve` uses `OGRRawPoint` (struct of `double x, y`) plus separate `double*` arrays for Z and M. When GDAL's MVT driver reads integer tile coordinates, it transforms them to EPSG:3857 float64 values. When writing MVT, it re-quantizes float64 to integers within the tile extent (default 4096). This roundtrip is explicitly lossy. GDAL does offer a "direct translation mode" for MBTiles-to-PMTiles container conversion that copies raw tile bytes without re-encoding, but this only works for passthrough — not for any workflow that touches geometry.

Even Rouault's **RFC 99** (GDAL 3.9+, 2024) introduced `OGRGeomCoordinatePrecision` — a metadata structure specifying XY, Z, and M resolution, attached to `OGRGeomFieldDefn` (layer-level, not geometry-level). The RFC explicitly declined to tie this to GEOS's PrecisionModel or add precision as a member of `OGRGeometry` due to "extra RAM usage implications." It provides three geometry methods: `SetPrecision()` (GEOS-backed topology-preserving rounding), `roundCoordinates()` (decimal rounding), and `roundCoordinatesIEEE754()` (bit-zeroing for compression). The bit-zeroing technique — zeroing least-significant mantissa bits while preserving values at the declared resolution — is shared with PostGIS's `ST_QuantizeCoordinates` and is relevant for compression but does not preserve source integers.

**PostGIS** stores all coordinates as `double` and has no attached precision model. `ST_QuantizeCoordinates` modifies stored float64 values by zeroing unneeded mantissa bits — it returns a new geometry, not a view. TWKB roundtrip (`ST_AsTWKB` → `ST_GeomFromTWKB`) preserves decimal values but not exact float64 bit patterns, because TWKB quantizes to `round(coord × 10^precision)` and dequantizes by dividing back.

**PDAL** (the dominant C++ point cloud pipeline) always dequantizes LAS int32 coordinates to doubles internally. Its FAQ acknowledges the roundtrip problem and provides a `forward` option to copy input scale/offset to output headers, minimizing (but not eliminating) precision drift. **las-rs** (Rust LAS crate) follows the same pattern — the primary `Point` struct stores dequantized `f64`, with raw `i32` available through a separate `raw::Point` struct in a `raw` module.

**Mapping to your options:** This "always float64" pattern is what cityjson-rs currently does, and it's what causes the roundtrip problem. Every library using this pattern accepts precision loss as the cost of API simplicity.

---

## Three libraries store source integers and compute on demand

The strongest precedent for your Option 1 comes from three libraries that face the exact same quantization problem and chose to preserve source integers.

**vector-tile-js** (Mapbox's reference MVT reader) implements the purest computed-view pattern found in this research. Its `VectorTileFeature` constructor stores only a reference to the protobuf buffer (`this._pbf`) and the byte offset of the geometry data. No geometry is parsed during construction. `loadGeometry()` re-reads the protobuf bytes on every call, returning `Point(x, y)` objects where **x and y are integers** in tile coordinate space. `toGeoJSON(x, y, z)` calls `loadGeometry()` then applies Web Mercator inverse projection to convert integers to WGS84 lon/lat floats on demand. The original bytes are never modified. This is read-only (vector-tile-js does not write MVT), but the architecture would support perfect re-serialization because the source bytes are retained.

**topojson-client** (Mike Bostock's reference TopoJSON library) stores the parsed topology — with integer arcs and a separate transform — as the canonical in-memory representation. `topojson.feature(topology, object)` dequantizes eagerly, but it creates **new** float arrays for GeoJSON output without modifying the source topology. The library explicitly provides paired methods: `mesh()` returns GeoJSON (dequantized), while `meshArcs()` returns TopoJSON objects preserving arc references to the original integers. This dual API acknowledges that sometimes you want to stay in the quantized domain. If you parse a TopoJSON and re-serialize the topology object without calling `feature()`, the integers are preserved exactly.

**laspy** (Python LAS library) is the most directly relevant precedent. It stores raw int32 coordinates internally (in `PackedPointRecord` matching the LAS binary layout) and provides dual access through a naming convention:

- `las.X`, `las.Y`, `las.Z` — raw int32 values, exactly as stored in the file
- `las.x`, `las.y`, `las.z` — dequantized float64, computed on access as `X * scale + offset`

The laspy documentation explicitly warns: "due to rounding error assignment using the scaled dimensions is not recommended." The safe pattern is to work with raw integers when possible. This is **the closest existing analogue to CityJSON's quantization model** — both use `int × scale + offset = world coordinate`, and laspy chose integers as canonical storage with computed float access.

**Mapping to your options:** All three validate a variant of Option 1 where quantized integers are the canonical storage and world coordinates are a computed view. None use a Rust-style `enum` — they use either lazy computation (vector-tile-js), eager-but-non-destructive computation (topojson-client), or property-based dual access (laspy).

---

## What CityJSON implementations actually do today

Every CityJSON implementation that interprets coordinates dequantizes destructively and loses source integers. None achieve lossless roundtrip.

**cjio** (Python, the TU Delft reference implementation) stores the raw JSON dictionary. Vertices live at `self.j["vertices"]` and remain as parsed integers after loading. But `decompress()` is destructive — it applies the transform **in-place**, replacing integer vertices with float values and then **deleting the transform object entirely**. `compress()` recomputes new scale/translate from scratch using the bounding box, producing different quantization parameters. The roundtrip `decompress()` → `compress()` is doubly lossy: float arithmetic introduces precision errors, and the new quantization grid differs from the original.

**citygml4j** (Java) uses the CityGML object model internally, which stores coordinates as double-precision floats. CityJSON reading dequantizes to doubles. CityJSON writing re-quantizes with a configurable `--vertex-precision` parameter. Source integers are not preserved. **3DCityDB** follows the same pattern — coordinates are doubles in the database, with quantization applied only during CityJSON export. Its documentation warns: "A side-effect of rounding coordinate values is that two very close but distinct vertices in the database might receive the same coordinate values in the CityJSON file."

The only CityJSON tools that preserve integers are those that **don't interpret coordinates**: **cjval** (Rust validator) works on raw `serde_json::Value` and compares integer triples directly for duplicate detection, and **cjseq** (Rust CityJSONSeq converter) passes vertices and transforms through as-is.

Importantly, **CityJSON v2.0 mandates the transform** — the `"transform"` property is required, and vertices are always integers. Hugo Ledoux's 2019 paper states that integer representation makes files "more robust, in the sense that the coordinates are not prone to rounding because of floating-point representation." This framing — integers as the robust canonical form — directly supports storing them as such.

---

## CGAL's exact kernels validate Option 2 at a steep cost

CGAL is the only production library that implements truly parallel geometry types with different coordinate representations. All geometry types are parameterized by a kernel template: `Point_2<EPICK>` uses `double` coordinates, while `Point_2<EPECK>` uses `Lazy_exact_nt<Gmpq>` (lazy-evaluated GMP rational numbers). **These are incompatible C++ types at compile time** — you cannot pass an EPICK point to a function expecting EPECK without explicit conversion. All algorithms and data structures are templates accepting a kernel parameter.

The performance cost is significant. EPECK constructions (creating new geometry objects like intersection points) build a DAG of deferred operations; when exact evaluation is triggered, individual operations can be **10–100× slower** than double arithmetic. OpenSCAD's migration from CGAL exact Nef polyhedra to surface mesh corefinement (which can sometimes use EPICK) yielded 10–100× speedups.

For I/O, CGAL's exact kernels face an inherent impedance mismatch: file formats store `double`, and converting `double` to exact rational preserves the exact double value — not the intended decimal value. CGAL's tutorial warns: when `0.1` is read from a file, it becomes the exact rational `3602879701896397/36028797018963968` (the nearest double), not `1/10`. Writing back to a file truncates exact rationals to double. **Exact arithmetic solves computational robustness but does not solve the I/O roundtrip problem.**

**Mapping to your options:** CGAL directly validates Option 2 (two parallel geometry types) and demonstrates both its power (compile-time guarantees, no accidental mixing) and its cost (code duplication across generic boundaries, performance overhead, I/O friction). For CityJSON, the exact-numeric variant of Option 3 carries CGAL's costs without solving the actual problem — CityJSON's integers are already exact, and the issue is preserving them, not achieving higher precision.

---

## No library uses a runtime storage enum

Across all libraries surveyed — JTS, GEOS, Shapely, GDAL, PostGIS, PDAL, las-rs, laspy, vector-tile-js, topojson-client, CGAL, glTF loaders, and all CityJSON implementations — **none use a runtime enum discriminating between quantized and world-coordinate storage** within a single geometry type. The observed approaches are:

- **Always float64**: GDAL, PostGIS, PDAL, GEOS, citygml4j, 3DCityDB, geozero
- **Always integer with computed float access**: vector-tile-js, laspy (internally), topojson-client
- **Compile-time type parameterization**: CGAL kernels, `geo` crate's `Coord<T>`
- **Two separate struct hierarchies**: las-rs (`Point` vs `raw::Point`)
- **Destructive in-place conversion**: cjio

The absence of a storage enum likely reflects a practical concern: every function consuming coordinates must handle both variants, either through match/dispatch overhead or by requiring callers to pre-convert. The libraries that successfully preserve integers (laspy, vector-tile-js) avoid this by making integers the single canonical storage and computing floats through accessor methods — no enum needed.

---

## How each pattern maps to your three options

**Option 1 — Storage enum (World vs Quantized{vertices, transform}), world coords as computed view:**
The enum variant specifically has no direct precedent. But the underlying principle — quantized integers as canonical storage, world coordinates as a derived view — is validated by laspy (dual-access API), vector-tile-js (lazy dequantization), and topojson-client (non-destructive feature extraction). The key insight from these libraries is that **the enum may be unnecessary**: since CityJSON v2.0 mandates the transform, you can always store integers + transform and compute world coordinates on demand. The "World" variant of the enum would only be needed for programmatically constructed (non-file-sourced) geometries. If you do use an enum, expect every coordinate-consuming function to pay the dispatch cost or require a uniform interface (trait) over both variants.

**Option 2 — Two model types (ParsedCityModel vs CityModel) with explicit conversion:**
Validated by CGAL's kernel parameterization (compile-time parallel types) and las-rs's `Point` vs `raw::Point` separation. CGAL demonstrates that this works well when the two types are used in genuinely different contexts (exact computation vs. approximate rendering), but creates friction at conversion boundaries and code duplication in generic algorithms. las-rs's approach is simpler — the `raw` module is an escape hatch for low-level access, not a parallel API surface. The risk is that most users will use only one type and find the other an unnecessary cognitive burden.

**Option 3 — Exact numeric types (decimal/rational):**
Validated only by CGAL, and not for the I/O roundtrip use case. CGAL's exact kernels solve computational geometry robustness (preventing topological failures from floating-point errors), not serialization fidelity. The 10–100× performance cost is prohibitive for a format library. More fundamentally, CityJSON's source integers are already exact — wrapping them in a rational type adds overhead without benefit. No production GIS library uses exact numerics for coordinate storage.

## Conclusion

The evidence points toward a clear architectural direction that none of the three options captures perfectly. **The strongest precedent is laspy's approach: store source integers as the single canonical representation and expose dequantized world coordinates through computed accessors** — not an enum, not two model types, not exact numerics. CityJSON v2.0's mandatory transform makes this natural: the `CityModel` always holds `Vec<[i64; 3]>` vertices plus a `Transform`, and a `fn world_vertex(&self, idx: usize) -> [f64; 3]` method computes on demand. Serialization writes the integers directly, guaranteeing byte-exact roundtrip. This is closest to a simplified Option 1 without the enum — because with a mandatory transform, the "World" variant serves no purpose for parsed data. For programmatically authored geometry, a builder API can accept `f64` world coordinates and quantize them into the integer representation immediately, using an explicit transform the caller provides.

The one gap this approach creates is ergonomic: users performing geometric operations (distance calculations, spatial queries) must either work in quantized space or accept per-access dequantization cost. topojson-client's paired API (`mesh()` vs `meshArcs()`) and laspy's uppercase/lowercase convention show how libraries have surfaced this choice without forcing users into one mode. The CityJSON ecosystem's own precedent — cjval working directly on integer triples, val3dity dequantizing for geometric validation — suggests that both modes have real use cases and both should be accessible.

# Sources — GIS Coordinate Quantization Research

Sources consulted for the report on how GIS and computational geometry libraries handle the quantized-integer vs. world-coordinate tension.

## JTS / GEOS / Shapely

- **JTS PrecisionModel (1.20.0)** — https://locationtech.github.io/jts/javadoc/org/locationtech/jts/geom/PrecisionModel.html
- **JTS PrecisionModel (1.17.0)** — https://locationtech.github.io/jts/javadoc-1.17.0/org/locationtech/jts/geom/PrecisionModel.html
- **JTS PrecisionModel (1.12, legacy Vivid Solutions)** — https://www.tsusiatsoftware.net/jts/javadoc/com/vividsolutions/jts/geom/PrecisionModel.html
- **JTS Geometry.java (source)** — https://github.com/locationtech/jts/blob/master/modules/core/src/main/java/org/locationtech/jts/geom/Geometry.java
- **JTS GeometryFactory.java (source)** — https://github.com/locationtech/jts/blob/master/modules/core/src/main/java/org/locationtech/jts/geom/GeometryFactory.java
- **JTS GeometryFactory (Javadoc)** — https://locationtech.github.io/jts/javadoc/org/locationtech/jts/geom/GeometryFactory.html
- **JTS CoordinateSequence (Javadoc)** — https://locationtech.github.io/jts/javadoc/org/locationtech/jts/geom/CoordinateSequence.html
- **JTS CoordinateSequence.java (source)** — https://github.com/locationtech/jts/blob/master/modules/core/src/main/java/org/locationtech/jts/geom/CoordinateSequence.java
- **JTS PackedCoordinateSequence (Javadoc)** — https://locationtech.github.io/jts/javadoc/org/locationtech/jts/geom/impl/PackedCoordinateSequence.html
- **History of JTS and GEOS (Martin Davis)** — https://www.tsusiatsoftware.net/jts/jtshistory.html
- **GEOS geos_c.cpp (source)** — https://github.com/libgeos/geos/blob/main/capi/geos_c.cpp
- **Shapely 2.x release notes** — https://shapely.readthedocs.io/en/stable/release/2.x.html

## GDAL / OGR

- **OGRPoint class reference** — https://gdal.org/en/stable/doxygen/classOGRPoint.html
- **OGRGeometry C++ API** — https://gdal.org/en/latest/api/ogrgeometry_cpp.html
- **RFC 99: Geometry coordinate precision** — https://gdal.org/en/stable/development/rfc/rfc99_geometry_coordinate_precision.html
- **RFC 99 discussion (gdal-dev mailing list)** — https://www.mail-archive.com/gdal-dev@lists.osgeo.org/msg40600.html
- **PMTiles driver docs** — https://gdal.org/en/stable/drivers/vector/pmtiles.html

## PostGIS / TWKB

- **Storage-Optimizing PostGIS Geometries (Dan Baston)** — http://www.danbaston.com/posts/2018/02/15/optimizing-postgis-geometries.html
- **TWKB Specification** — https://github.com/TWKB/Specification/blob/master/twkb.md

## PDAL / LAS / laspy

- **PDAL readers.copc** — https://pdal.org/en/stable/stages/readers.copc.html
- **laspy — What is a LAS file?** — https://laspy.readthedocs.io/en/latest/intro.html
- **laspy — A Complete Example** — https://laspy.readthedocs.io/en/latest/complete_tutorial.html

## Mapbox Vector Tiles / TopoJSON

- **vector-tile-js VectorTileFeature source** — https://github.com/mapbox/vector-tile-js/blob/5c453951e1a28fdfbf37f2b4535cab00125a6106/lib/vectortilefeature.js
- **vector-tile-js README** — https://github.com/mapbox/vector-tile-js/blob/main/README.md
- **topojson-client README** — https://github.com/topojson/topojson-client/blob/master/README.md
- **topojson 1.x API reference** — https://github.com/topojson/topojson-1.x-api-reference/blob/master/API-Reference.md

## CityJSON ecosystem

- **CityJSON Specifications 2.0.0** — https://www.cityjson.org/specs/2.0.0/
- **CityJSON specs releases** — https://github.com/cityjson/specs/releases
- **CityJSON: A compact and easy-to-use encoding of the CityGML data model (Ledoux et al., 2019)** — https://arxiv.org/pdf/1902.09155
- **cjio cityjson.py (source)** — https://github.com/cityjson/cjio/blob/master/cjio/cityjson.py
- **cjval (Rust validator)** — https://github.com/cityjson/cjval
- **cjseq (CityJSONSeq converter)** — https://github.com/cityjson/cjseq
- **val3dity README** — https://github.com/tudelft3d/val3dity/blob/main/README.md
- **citygml4j on GitHub** — https://github.com/citygml4j/citygml4j
- **citygml4j-cityjson on Libraries.io** — https://libraries.io/maven/org.citygml4j:citygml4j-cityjson
- **3DCityDB CityJSON export options** — https://3dcitydb-docs.readthedocs.io/en/latest/impexp/export-preferences/cityjson.html

## CGAL

- **CGAL 6.1.1 — 2D and 3D Linear Geometry Kernel User Manual** — https://doc.cgal.org/latest/Kernel_23/index.html
- **CGAL 6.0.2 — dD Geometry Kernel User Manual** — https://doc.cgal.org/latest/Kernel_d/index.html
- **CGAL Kernel Reference Manual (legacy, Stanford mirror)** — https://graphics.stanford.edu/courses/cs368-00-spring/TA/manuals/CGAL/ref-manual1/Chapter_intro_kernel.html
- **CGAL FAQ** — https://www.cgal.org/FAQ.html
- **OpenSCAD fast CSG contribution (Ochafik) — CGAL exact kernel performance** — https://ochafik.com/jekyll/update/2022/02/09/openscad-fast-csg-contibution.html

## Rust GIS ecosystem

- **geozero on crates.io** — https://crates.io/crates/geozero
- **geozero on docs.rs** — https://docs.rs/crate/geozero/0.9.8
