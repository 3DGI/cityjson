//! GEOS interoperability checks for the CityJSON ISO WKB codec.

mod common;

use cityjson_types::v2_0::Boundary;
use geos::{CoordDimensions, Geom, Geometry, WKBWriter, WKTWriter};

#[test]
fn geos_loads_cityjson_wkb() -> anyhow::Result<()> {
    for case in common::iso_cases() {
        let wkb = case.wkb()?;
        let geometry = Geometry::new_from_wkb(&wkb)?;

        if case.assert_planar_valid {
            assert!(geometry.is_valid()?, "{} should be GEOS-valid", case.name);
        }
    }

    Ok(())
}

/// Purpose: verify GEOS emits CityJSON-readable EWKB after consuming ISO WKB.
///
/// Input: every ISO WKB geometry with a GEOS-assigned EPSG:7415 SRID.
///
/// Assertions: the decoded CityJSON boundary has the original semantic structure.
#[test]
fn geos_roundtrips_cityjson_wkb_through_ewkb() -> anyhow::Result<()> {
    for case in common::iso_cases() {
        let mut geometry = Geometry::new_from_wkb(&case.wkb()?)?;
        geometry.set_srid(7415);
        let mut writer = WKBWriter::new()?;
        writer.set_output_dimension(CoordDimensions::ThreeD);
        writer.set_include_SRID(true);
        let ewkb = writer.write_wkb(&geometry)?;
        let decoded = Boundary::<u32>::from_ewkb(&ewkb, case.boundary.check_type())?;
        assert_eq!(decoded.srid, Some(7415), "{}", case.name);
        assert_eq!(
            decoded.boundary.check_type(),
            case.boundary.check_type(),
            "{}",
            case.name
        );
        assert_eq!(
            decoded.boundary.to_wkt(&decoded.vertices)?,
            case.wkt()?,
            "{}",
            case.name
        );
    }
    Ok(())
}

/// Purpose: retain GEOS WKT semantic round trips for ISO geometry cases.
///
/// Input: CityJSON ISO WKT generated from each ISO case.
///
/// Assertions: parsing the GEOS WKT result reconstructs the same WKT semantics.
#[test]
fn geos_roundtrips_cityjson_wkt() -> anyhow::Result<()> {
    for case in common::iso_cases() {
        let wkt = case.wkt()?;
        let geometry = Geometry::new_from_wkt(&wkt)?;
        let mut writer = WKTWriter::new()?;
        writer.set_output_dimension(CoordDimensions::ThreeD);
        writer.set_trim(true);
        let roundtrip_wkt = writer.write(&geometry)?;
        let (boundary, vertices) = Boundary::<u32>::from_wkt(&roundtrip_wkt)?;
        assert_eq!(boundary.to_wkt(&vertices)?, wkt, "{}", case.name);
        if case.assert_planar_valid {
            assert!(geometry.is_valid()?, "{} should be GEOS-valid", case.name);
        }
    }
    Ok(())
}
