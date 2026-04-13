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
