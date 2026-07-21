mod common;

use cityjson_types::v2_0::Boundary;
use geos::{CoordDimensions, Geom, Geometry, WKTWriter};

#[test]
fn geos_loads_cityjson_wkb() -> anyhow::Result<()> {
    for case in common::cases() {
        let wkb = case.wkb()?;
        let geometry = Geometry::new_from_wkb(&wkb)?;

        if case.assert_planar_valid {
            assert!(geometry.is_valid()?, "{} should be GEOS-valid", case.name);
        }
    }

    Ok(())
}

#[test]
fn geos_roundtrips_cityjson_wkt() -> anyhow::Result<()> {
    for case in common::cases() {
        let wkt = case.wkt()?;
        let geometry = Geometry::new_from_wkt(&wkt)?;
        let mut writer = WKTWriter::new()?;
        writer.set_output_dimension(CoordDimensions::ThreeD);
        writer.set_trim(true);
        let roundtrip_wkt = writer.write(&geometry)?;
        let (boundary, vertices) = Boundary::<u32>::from_wkt(&roundtrip_wkt)?;
        assert_eq!(boundary.to_wkt(&vertices)?, wkt, "{}", case.name);
        if case.assert_planar_valid { assert!(geometry.is_valid()?, "{} should be GEOS-valid", case.name); }
    }
    Ok(())
}
