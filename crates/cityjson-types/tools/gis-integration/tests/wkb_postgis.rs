mod common;

use cityjson_types::v2_0::Boundary;
use postgres::{Client, NoTls};

#[test]
fn postgis_loads_and_roundtrips_cityjson_wkb() -> anyhow::Result<()> {
    let mut client = Client::connect(&cityjson_types_gis_integration::database_url(), NoTls)?;
    client.batch_execute("CREATE EXTENSION IF NOT EXISTS postgis")?;

    for case in common::cases() {
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


#[test]
fn postgis_loads_and_roundtrips_cityjson_ewkb() -> anyhow::Result<()> {
    let mut client = Client::connect(&cityjson_types_gis_integration::database_url(), NoTls)?;
    client.batch_execute("CREATE EXTENSION IF NOT EXISTS postgis")?;

    for case in common::cases() {
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
