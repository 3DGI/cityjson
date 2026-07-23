//! PostGIS interoperability checks for CityJSON ISO WKB/WKT and extended EWKB/EWKT codecs.

mod common;

use cityjson_types::v2_0::Boundary;
use postgres::{Client, NoTls};

/// Purpose: verify an external GIS codec accepts the selected CityJSON dialect.
///
/// Input: the documented fixture set and EPSG:7415 where the extended dialect requires it.
///
/// Assertions: geometry type, XYZ dimensionality, semantic round-trip structure, and SRID are retained.
#[test]
fn postgis_loads_and_roundtrips_cityjson_wkb() -> anyhow::Result<()> {
    let mut client = Client::connect(&cityjson_types_gis_integration::database_url(), NoTls)?;
    client.batch_execute("CREATE EXTENSION IF NOT EXISTS postgis")?;

    for case in common::iso_cases() {
        let wkb = case.wkb()?;
        let row = client.query_one(
            "
            WITH geom AS (
                SELECT ST_GeomFromWKB($1) AS g
            )
            SELECT
                ST_GeometryType(g) AS geometry_type,
                ST_NDims(g)::integer AS ndims,
                ST_NumGeometries(g)::integer AS geometries,
                CASE
                    WHEN GeometryType(ST_GeometryN(g, 1)) = 'POLYGON'
                    THEN ST_NumInteriorRings(ST_GeometryN(g, 1))::integer
                    ELSE 0
                END::integer AS first_interior_rings,
                ST_IsValid(g) AS valid,
                ST_AsBinary(g, 'NDR') AS roundtrip_wkb
            FROM geom
            ",
            &[&wkb],
        )?;

        let geometry_type: String = row.get("geometry_type");
        let ndims: i32 = row.get("ndims");
        let geometries: i32 = row.get("geometries");
        let first_interior_rings: i32 = row.get("first_interior_rings");
        let valid: bool = row.get("valid");
        let roundtrip_wkb: Vec<u8> = row.get("roundtrip_wkb");

        assert_eq!(geometry_type, case.expected_type, "{}", case.name);
        assert_eq!(ndims, case.expected_ndims, "{}", case.name);
        assert_eq!(geometries, case.expected_geometries, "{}", case.name);
        assert_eq!(
            first_interior_rings, case.expected_first_interior_rings,
            "{}",
            case.name
        );
        if case.assert_planar_valid {
            assert!(valid, "{} should be PostGIS-valid", case.name);
        }

        Boundary::<u32>::from_wkb(&roundtrip_wkb)?;
    }

    Ok(())
}

/// Purpose: verify an external GIS codec accepts the selected CityJSON dialect.
///
/// Input: the documented fixture set and EPSG:7415 where the extended dialect requires it.
///
/// Assertions: geometry type, XYZ dimensionality, semantic round-trip structure, and SRID are retained.
#[test]
fn postgis_loads_and_roundtrips_cityjson_ewkb() -> anyhow::Result<()> {
    let mut client = Client::connect(&cityjson_types_gis_integration::database_url(), NoTls)?;
    client.batch_execute("CREATE EXTENSION IF NOT EXISTS postgis")?;

    for case in common::extended_cases() {
        let ewkb = case.ewkb(Some(7415))?;
        let row = client.query_one(
            "
            WITH geom AS (
                SELECT ST_GeomFromEWKB($1) AS g
            )
            SELECT
                ST_GeometryType(g) AS geometry_type,
                ST_NDims(g)::integer AS ndims,
                ST_NumGeometries(g)::integer AS geometries,
                ST_SRID(g)::integer AS srid,
                ST_AsEWKB(g, 'NDR') AS roundtrip_ewkb
            FROM geom
            ",
            &[&ewkb],
        )?;

        let geometry_type: String = row.get("geometry_type");
        let ndims: i32 = row.get("ndims");
        let geometries: i32 = row.get("geometries");
        let srid: i32 = row.get("srid");
        let roundtrip_ewkb: Vec<u8> = row.get("roundtrip_ewkb");

        assert_eq!(geometry_type, case.expected_type, "{}", case.name);
        assert_eq!(ndims, case.expected_ndims, "{}", case.name);
        assert_eq!(geometries, case.expected_geometries, "{}", case.name);
        assert_eq!(srid, 7415, "{}", case.name);

        Boundary::<u32>::from_ewkb(&roundtrip_ewkb, case.boundary.check_type())?;
    }

    Ok(())
}

/// Purpose: verify an external GIS codec accepts the selected CityJSON dialect.
///
/// Input: the documented fixture set and EPSG:7415 where the extended dialect requires it.
///
/// Assertions: geometry type, XYZ dimensionality, semantic round-trip structure, and SRID are retained.
#[test]
fn postgis_loads_and_roundtrips_cityjson_wkt() -> anyhow::Result<()> {
    let mut client = Client::connect(&cityjson_types_gis_integration::database_url(), NoTls)?;
    client.batch_execute("CREATE EXTENSION IF NOT EXISTS postgis")?;
    for case in common::iso_cases() {
        let wkt = case.wkt()?;
        let row = client.query_one("WITH geom AS (SELECT ST_GeomFromText($1) AS g) SELECT ST_GeometryType(g) AS geometry_type, ST_NDims(g)::integer AS ndims, ST_NumGeometries(g)::integer AS geometries, ST_AsText(g) AS roundtrip_wkt FROM geom", &[&wkt])?;
        assert_eq!(
            row.get::<_, String>("geometry_type"),
            case.expected_type,
            "{}",
            case.name
        );
        assert_eq!(
            row.get::<_, i32>("ndims"),
            case.expected_ndims,
            "{}",
            case.name
        );
        assert_eq!(
            row.get::<_, i32>("geometries"),
            case.expected_geometries,
            "{}",
            case.name
        );
        let (boundary, vertices) =
            Boundary::<u32>::from_wkt(&row.get::<_, String>("roundtrip_wkt"))?;
        assert_eq!(boundary.to_wkt(&vertices)?, wkt, "{}", case.name);
    }
    Ok(())
}

/// Purpose: verify an external GIS codec accepts the selected CityJSON dialect.
///
/// Input: the documented fixture set and EPSG:7415 where the extended dialect requires it.
///
/// Assertions: geometry type, XYZ dimensionality, semantic round-trip structure, and SRID are retained.
#[test]
fn postgis_loads_and_roundtrips_cityjson_ewkt() -> anyhow::Result<()> {
    let mut client = Client::connect(&cityjson_types_gis_integration::database_url(), NoTls)?;
    client.batch_execute("CREATE EXTENSION IF NOT EXISTS postgis")?;
    for case in common::extended_cases() {
        let ewkt = case.ewkt(Some(7415))?;
        let row = client.query_one("WITH geom AS (SELECT ST_GeomFromEWKT($1) AS g) SELECT ST_GeometryType(g) AS geometry_type, ST_NDims(g)::integer AS ndims, ST_NumGeometries(g)::integer AS geometries, ST_SRID(g)::integer AS srid, ST_AsEWKT(g) AS roundtrip_ewkt FROM geom", &[&ewkt])?;
        assert_eq!(
            row.get::<_, String>("geometry_type"),
            case.expected_type,
            "{}",
            case.name
        );
        assert_eq!(
            row.get::<_, i32>("ndims"),
            case.expected_ndims,
            "{}",
            case.name
        );
        assert_eq!(
            row.get::<_, i32>("geometries"),
            case.expected_geometries,
            "{}",
            case.name
        );
        assert_eq!(row.get::<_, i32>("srid"), 7415, "{}", case.name);
        let parsed = Boundary::<u32>::from_ewkt(
            &row.get::<_, String>("roundtrip_ewkt"),
            case.boundary.check_type(),
        )?;
        assert_eq!(parsed.srid, Some(7415), "{}", case.name);
        assert_eq!(
            parsed
                .boundary
                .to_ewkt(&parsed.vertices, parsed.ewkt_type, parsed.srid)?,
            ewkt,
            "{}",
            case.name
        );
    }
    Ok(())
}
