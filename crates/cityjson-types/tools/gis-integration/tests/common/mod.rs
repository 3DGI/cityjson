use cityjson_types::v2_0::{
    Boundary, BoundaryNestedMultiLineString32, BoundaryNestedMultiOrCompositeSurface32,
    BoundaryNestedMultiPoint32, BoundaryNestedSolid32, EwkbType, EwktType, RealWorldCoordinate,
    Vertices,
};

#[allow(dead_code)]
pub struct Case {
    pub name: &'static str,
    pub boundary: Boundary<u32>,
    pub expected_type: &'static str,
    pub expected_ndims: i32,
    pub expected_geometries: i32,
    pub expected_first_interior_rings: i32,
    pub assert_planar_valid: bool,
}

impl Case {
    pub fn wkb(&self) -> anyhow::Result<Vec<u8>> {
        Ok(self.boundary.to_wkb(&vertices())?)
    }

    pub fn wkt(&self) -> anyhow::Result<String> {
        Ok(self.boundary.to_wkt(&vertices())?)
    }

    #[allow(dead_code)]
    pub fn ewkb(&self, srid: Option<u32>) -> anyhow::Result<Vec<u8>> {
        let ewkb_type = match self.name {
            "polyhedralsurface_z" => EwkbType::PolyhedralSurface,
            "tinz" => EwkbType::Tin,
            _ => match self.boundary.check_type() {
                cityjson_types::v2_0::BoundaryType::MultiPoint => EwkbType::MultiPoint,
                cityjson_types::v2_0::BoundaryType::MultiLineString => EwkbType::MultiLineString,
                cityjson_types::v2_0::BoundaryType::MultiOrCompositeSurface
                | cityjson_types::v2_0::BoundaryType::Solid
                | cityjson_types::v2_0::BoundaryType::MultiOrCompositeSolid => {
                    EwkbType::MultiPolygon
                }
                cityjson_types::v2_0::BoundaryType::None => anyhow::bail!("empty boundary"),
                boundary_type => anyhow::bail!("unsupported EWKB boundary type: {boundary_type:?}"),
            },
        };
        Ok(self.boundary.to_ewkb(&vertices(), ewkb_type, srid)?)
    }

    #[allow(dead_code)]
    pub fn ewkt(&self, srid: Option<u32>) -> anyhow::Result<String> {
        let ewkt_type = match self.name {
            "polyhedralsurface_z" => EwktType::PolyhedralSurface,
            "tinz" => EwktType::Tin,
            _ => match self.boundary.check_type() {
                cityjson_types::v2_0::BoundaryType::MultiPoint => EwktType::MultiPoint,
                cityjson_types::v2_0::BoundaryType::MultiLineString => EwktType::MultiLineString,
                cityjson_types::v2_0::BoundaryType::MultiOrCompositeSurface
                | cityjson_types::v2_0::BoundaryType::Solid
                | cityjson_types::v2_0::BoundaryType::MultiOrCompositeSolid => {
                    EwktType::MultiPolygon
                }
                cityjson_types::v2_0::BoundaryType::None => anyhow::bail!("empty boundary"),
                boundary_type => anyhow::bail!("unsupported EWKT boundary type: {boundary_type:?}"),
            },
        };
        Ok(self.boundary.to_ewkt(&vertices(), ewkt_type, srid)?)
    }
}

pub fn iso_cases() -> Vec<Case> {
    extended_cases()
        .into_iter()
        .filter(|case| {
            matches!(
                case.name,
                "multipoint_z" | "multilinestring_z" | "multipolygon_z"
            )
        })
        .collect()
}

pub fn extended_cases() -> Vec<Case> {
    vec![
        Case {
            name: "multipoint_z",
            boundary: multipoint(),
            expected_type: "ST_MultiPoint",
            expected_ndims: 3,
            expected_geometries: 3,
            expected_first_interior_rings: 0,
            assert_planar_valid: true,
        },
        Case {
            name: "multilinestring_z",
            boundary: multilinestring(),
            expected_type: "ST_MultiLineString",
            expected_ndims: 3,
            expected_geometries: 2,
            expected_first_interior_rings: 0,
            assert_planar_valid: true,
        },
        Case {
            name: "multipolygon_z",
            boundary: single_polygon(),
            expected_type: "ST_MultiPolygon",
            expected_ndims: 3,
            expected_geometries: 1,
            expected_first_interior_rings: 0,
            assert_planar_valid: true,
        },
        Case {
            name: "polyhedralsurface_z",
            boundary: solid(),
            expected_type: "ST_PolyhedralSurface",
            expected_ndims: 3,
            expected_geometries: 2,
            expected_first_interior_rings: 0,
            assert_planar_valid: false,
        },
        Case {
            name: "tinz",
            boundary: tin(),
            expected_type: "ST_Tin",
            expected_ndims: 3,
            expected_geometries: 1,
            expected_first_interior_rings: 0,
            assert_planar_valid: true,
        },
    ]
}

fn vertices() -> Vertices<u32, RealWorldCoordinate> {
    Vertices::from(vec![
        RealWorldCoordinate::new(0.0, 0.0, 0.0),
        RealWorldCoordinate::new(10.0, 0.0, 1.0),
        RealWorldCoordinate::new(10.0, 10.0, 2.0),
        RealWorldCoordinate::new(0.0, 10.0, 3.0),
        RealWorldCoordinate::new(2.0, 2.0, 4.0),
        RealWorldCoordinate::new(4.0, 2.0, 5.0),
        RealWorldCoordinate::new(4.0, 4.0, 6.0),
        RealWorldCoordinate::new(2.0, 4.0, 7.0),
        RealWorldCoordinate::new(20.0, 0.0, 8.0),
        RealWorldCoordinate::new(30.0, 0.0, 9.0),
        RealWorldCoordinate::new(30.0, 10.0, 10.0),
        RealWorldCoordinate::new(20.0, 10.0, 11.0),
    ])
}

fn multipoint() -> Boundary<u32> {
    let nested: BoundaryNestedMultiPoint32 = vec![0, 1, 2];
    nested.into()
}

fn multilinestring() -> Boundary<u32> {
    let nested: BoundaryNestedMultiLineString32 = vec![vec![0, 1, 2], vec![4, 5, 6]];
    nested.try_into().unwrap()
}

fn single_polygon() -> Boundary<u32> {
    let nested: BoundaryNestedMultiOrCompositeSurface32 = vec![vec![vec![0, 1, 2, 3]]];
    nested.try_into().unwrap()
}

fn solid() -> Boundary<u32> {
    let nested: BoundaryNestedSolid32 =
        vec![vec![vec![vec![0, 1, 2, 3]]], vec![vec![vec![4, 5, 6, 7]]]];
    nested.try_into().unwrap()
}

fn tin() -> Boundary<u32> {
    let nested: BoundaryNestedMultiOrCompositeSurface32 = vec![vec![vec![0, 1, 2]]];
    nested.try_into().unwrap()
}
