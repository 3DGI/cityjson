mod common;

use geos::{Geom, Geometry};

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
