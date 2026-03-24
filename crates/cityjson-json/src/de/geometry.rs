use cityjson::resources::handles::GeometryHandle;
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{
    BorrowedCityModel, CityModel, GeometryDraft, LineStringDraft, LoD, OwnedCityModel, PointDraft,
    RingDraft, ShellDraft, SolidDraft, SurfaceDraft, VertexIndex,
};
use serde::Deserialize;
use serde_json::Value as OwnedJsonValue;
use serde_json_borrow::Value as BorrowedJsonValue;

use crate::errors::{Error, Result};

type MultiPointBoundary = Vec<u32>;
type MultiLineStringBoundary = Vec<MultiPointBoundary>;
type MultiSurfaceBoundary = Vec<MultiLineStringBoundary>;
type SolidBoundary = Vec<MultiSurfaceBoundary>;
type MultiSolidBoundary = Vec<SolidBoundary>;

#[derive(Deserialize)]
#[serde(tag = "type")]
pub(crate) enum RawGeometryOwned {
    MultiPoint {
        #[serde(default)]
        lod: Option<String>,
        boundaries: MultiPointBoundary,
        #[serde(default)]
        semantics: Option<OwnedJsonValue>,
        #[serde(default)]
        material: Option<OwnedJsonValue>,
        #[serde(default)]
        texture: Option<OwnedJsonValue>,
    },
    MultiLineString {
        #[serde(default)]
        lod: Option<String>,
        boundaries: MultiLineStringBoundary,
        #[serde(default)]
        semantics: Option<OwnedJsonValue>,
        #[serde(default)]
        material: Option<OwnedJsonValue>,
        #[serde(default)]
        texture: Option<OwnedJsonValue>,
    },
    MultiSurface {
        #[serde(default)]
        lod: Option<String>,
        boundaries: MultiSurfaceBoundary,
        #[serde(default)]
        semantics: Option<OwnedJsonValue>,
        #[serde(default)]
        material: Option<OwnedJsonValue>,
        #[serde(default)]
        texture: Option<OwnedJsonValue>,
    },
    CompositeSurface {
        #[serde(default)]
        lod: Option<String>,
        boundaries: MultiSurfaceBoundary,
        #[serde(default)]
        semantics: Option<OwnedJsonValue>,
        #[serde(default)]
        material: Option<OwnedJsonValue>,
        #[serde(default)]
        texture: Option<OwnedJsonValue>,
    },
    Solid {
        #[serde(default)]
        lod: Option<String>,
        boundaries: SolidBoundary,
        #[serde(default)]
        semantics: Option<OwnedJsonValue>,
        #[serde(default)]
        material: Option<OwnedJsonValue>,
        #[serde(default)]
        texture: Option<OwnedJsonValue>,
    },
    MultiSolid {
        #[serde(default)]
        lod: Option<String>,
        boundaries: MultiSolidBoundary,
        #[serde(default)]
        semantics: Option<OwnedJsonValue>,
        #[serde(default)]
        material: Option<OwnedJsonValue>,
        #[serde(default)]
        texture: Option<OwnedJsonValue>,
    },
    CompositeSolid {
        #[serde(default)]
        lod: Option<String>,
        boundaries: MultiSolidBoundary,
        #[serde(default)]
        semantics: Option<OwnedJsonValue>,
        #[serde(default)]
        material: Option<OwnedJsonValue>,
        #[serde(default)]
        texture: Option<OwnedJsonValue>,
    },
    GeometryInstance {
        #[serde(default)]
        lod: Option<String>,
        #[serde(default)]
        template: Option<u32>,
        #[serde(default)]
        boundaries: Option<OwnedJsonValue>,
        #[serde(rename = "transformationMatrix", default)]
        transformation_matrix: Option<[f64; 16]>,
    },
}

#[derive(Deserialize)]
#[serde(tag = "type", bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) enum RawGeometryBorrowed<'a> {
    MultiPoint {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: MultiPointBoundary,
        #[serde(default, borrow)]
        semantics: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        material: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        texture: Option<BorrowedJsonValue<'a>>,
    },
    MultiLineString {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: MultiLineStringBoundary,
        #[serde(default, borrow)]
        semantics: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        material: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        texture: Option<BorrowedJsonValue<'a>>,
    },
    MultiSurface {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: MultiSurfaceBoundary,
        #[serde(default, borrow)]
        semantics: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        material: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        texture: Option<BorrowedJsonValue<'a>>,
    },
    CompositeSurface {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: MultiSurfaceBoundary,
        #[serde(default, borrow)]
        semantics: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        material: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        texture: Option<BorrowedJsonValue<'a>>,
    },
    Solid {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: SolidBoundary,
        #[serde(default, borrow)]
        semantics: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        material: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        texture: Option<BorrowedJsonValue<'a>>,
    },
    MultiSolid {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: MultiSolidBoundary,
        #[serde(default, borrow)]
        semantics: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        material: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        texture: Option<BorrowedJsonValue<'a>>,
    },
    CompositeSolid {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        boundaries: MultiSolidBoundary,
        #[serde(default, borrow)]
        semantics: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        material: Option<BorrowedJsonValue<'a>>,
        #[serde(default, borrow)]
        texture: Option<BorrowedJsonValue<'a>>,
    },
    GeometryInstance {
        #[serde(default, borrow)]
        lod: Option<&'a str>,
        #[serde(default)]
        template: Option<u32>,
        #[serde(default, borrow)]
        boundaries: Option<BorrowedJsonValue<'a>>,
        #[serde(rename = "transformationMatrix", default)]
        transformation_matrix: Option<[f64; 16]>,
    },
}

pub(crate) fn import_owned_geometries(
    raw_geometries: Vec<RawGeometryOwned>,
    model: &mut OwnedCityModel,
) -> Result<Vec<GeometryHandle>> {
    raw_geometries
        .into_iter()
        .map(|geometry| import_owned_geometry(geometry, model))
        .collect()
}

pub(crate) fn import_borrowed_geometries<'a>(
    raw_geometries: Vec<RawGeometryBorrowed<'a>>,
    model: &mut BorrowedCityModel<'a>,
) -> Result<Vec<GeometryHandle>> {
    raw_geometries
        .into_iter()
        .map(|geometry| import_borrowed_geometry(geometry, model))
        .collect()
}

fn import_owned_geometry(
    raw_geometry: RawGeometryOwned,
    model: &mut OwnedCityModel,
) -> Result<GeometryHandle> {
    match raw_geometry {
        RawGeometryOwned::MultiPoint {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_owned(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_multi_point(lod.as_deref(), boundaries, model)
        }
        RawGeometryOwned::MultiLineString {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_owned(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_multi_line_string(lod.as_deref(), boundaries, model)
        }
        RawGeometryOwned::MultiSurface {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_owned(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_multi_surface(lod.as_deref(), boundaries, false, model)
        }
        RawGeometryOwned::CompositeSurface {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_owned(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_multi_surface(lod.as_deref(), boundaries, true, model)
        }
        RawGeometryOwned::Solid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_owned(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_solid(lod.as_deref(), boundaries, model)
        }
        RawGeometryOwned::MultiSolid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_owned(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_multi_solid(lod.as_deref(), boundaries, false, model)
        }
        RawGeometryOwned::CompositeSolid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_owned(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_multi_solid(lod.as_deref(), boundaries, true, model)
        }
        RawGeometryOwned::GeometryInstance {
            lod,
            template,
            boundaries,
            transformation_matrix,
        } => reject_geometry_instance(
            lod.as_deref(),
            template,
            boundaries.as_ref(),
            transformation_matrix.as_ref(),
        ),
    }
}

fn import_borrowed_geometry<'a>(
    raw_geometry: RawGeometryBorrowed<'a>,
    model: &mut BorrowedCityModel<'a>,
) -> Result<GeometryHandle> {
    match raw_geometry {
        RawGeometryBorrowed::MultiPoint {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_borrowed(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_multi_point(lod, boundaries, model)
        }
        RawGeometryBorrowed::MultiLineString {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_borrowed(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_multi_line_string(lod, boundaries, model)
        }
        RawGeometryBorrowed::MultiSurface {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_borrowed(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_multi_surface(lod, boundaries, false, model)
        }
        RawGeometryBorrowed::CompositeSurface {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_borrowed(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_multi_surface(lod, boundaries, true, model)
        }
        RawGeometryBorrowed::Solid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_borrowed(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_solid(lod, boundaries, model)
        }
        RawGeometryBorrowed::MultiSolid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_borrowed(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_multi_solid(lod, boundaries, false, model)
        }
        RawGeometryBorrowed::CompositeSolid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => {
            reject_unsupported_mappings_borrowed(
                semantics.as_ref(),
                material.as_ref(),
                texture.as_ref(),
            )?;
            import_multi_solid(lod, boundaries, true, model)
        }
        RawGeometryBorrowed::GeometryInstance {
            lod,
            template,
            boundaries,
            transformation_matrix,
        } => reject_geometry_instance(
            lod,
            template,
            boundaries.as_ref(),
            transformation_matrix.as_ref(),
        ),
    }
}

fn import_multi_point<SS: StringStorage>(
    lod: Option<&str>,
    boundaries: MultiPointBoundary,
    model: &mut CityModel<u32, SS>,
) -> Result<GeometryHandle> {
    GeometryDraft::multi_point(parse_lod(lod)?, boundaries.into_iter().map(point_draft))
        .insert_into(model)
        .map_err(Error::from)
}

fn import_multi_line_string<SS: StringStorage>(
    lod: Option<&str>,
    boundaries: MultiLineStringBoundary,
    model: &mut CityModel<u32, SS>,
) -> Result<GeometryHandle> {
    let linestrings = boundaries
        .into_iter()
        .map(|linestring| LineStringDraft::new(linestring.into_iter().map(VertexIndex::new)));
    GeometryDraft::multi_line_string(parse_lod(lod)?, linestrings)
        .insert_into(model)
        .map_err(Error::from)
}

fn import_multi_surface<SS: StringStorage>(
    lod: Option<&str>,
    boundaries: MultiSurfaceBoundary,
    is_composite: bool,
    model: &mut CityModel<u32, SS>,
) -> Result<GeometryHandle> {
    let surfaces = boundaries
        .into_iter()
        .map(surface_draft::<SS>)
        .collect::<Result<Vec<_>>>()?;
    let draft = if is_composite {
        GeometryDraft::composite_surface(parse_lod(lod)?, surfaces)
    } else {
        GeometryDraft::multi_surface(parse_lod(lod)?, surfaces)
    };
    draft.insert_into(model).map_err(Error::from)
}

fn import_solid<SS: StringStorage>(
    lod: Option<&str>,
    boundaries: SolidBoundary,
    model: &mut CityModel<u32, SS>,
) -> Result<GeometryHandle> {
    let mut shells = boundaries.into_iter().map(shell_draft::<SS>);
    let outer = shells.next().transpose()?.ok_or_else(|| {
        Error::InvalidValue("Solid geometry requires at least one shell".to_owned())
    })?;
    let inners = shells.collect::<Result<Vec<_>>>()?;
    GeometryDraft::solid(parse_lod(lod)?, outer, inners)
        .insert_into(model)
        .map_err(Error::from)
}

fn import_multi_solid<SS: StringStorage>(
    lod: Option<&str>,
    boundaries: MultiSolidBoundary,
    is_composite: bool,
    model: &mut CityModel<u32, SS>,
) -> Result<GeometryHandle> {
    let solids = boundaries
        .into_iter()
        .map(solid_draft::<SS>)
        .collect::<Result<Vec<_>>>()?;
    let draft = if is_composite {
        GeometryDraft::composite_solid(parse_lod(lod)?, solids)
    } else {
        GeometryDraft::multi_solid(parse_lod(lod)?, solids)
    };
    draft.insert_into(model).map_err(Error::from)
}

fn point_draft(index: u32) -> PointDraft<u32> {
    PointDraft::new(VertexIndex::new(index))
}

fn ring_draft<SS: StringStorage>(vertices: MultiPointBoundary) -> RingDraft<u32, SS> {
    RingDraft::new(vertices.into_iter().map(VertexIndex::new))
}

fn surface_draft<SS: StringStorage>(
    rings: MultiLineStringBoundary,
) -> Result<SurfaceDraft<u32, SS>> {
    let mut rings = rings.into_iter();
    let outer = rings.next().map(ring_draft::<SS>).ok_or_else(|| {
        Error::InvalidValue("surface boundary requires at least one ring".to_owned())
    })?;
    let inners = rings.map(ring_draft::<SS>).collect::<Vec<_>>();
    Ok(SurfaceDraft::new(outer, inners))
}

fn shell_draft<SS: StringStorage>(surfaces: MultiSurfaceBoundary) -> Result<ShellDraft<u32, SS>> {
    let surfaces = surfaces
        .into_iter()
        .map(surface_draft::<SS>)
        .collect::<Result<Vec<_>>>()?;
    Ok(ShellDraft::new(surfaces))
}

fn solid_draft<SS: StringStorage>(shells: SolidBoundary) -> Result<SolidDraft<u32, SS>> {
    let mut shells = shells.into_iter().map(shell_draft::<SS>);
    let outer = shells.next().transpose()?.ok_or_else(|| {
        Error::InvalidValue("solid boundary requires at least one shell".to_owned())
    })?;
    let inners = shells.collect::<Result<Vec<_>>>()?;
    Ok(SolidDraft::new(outer, inners))
}

fn reject_geometry_instance(
    _lod: Option<&str>,
    _template: Option<u32>,
    _boundaries: Option<&impl Sized>,
    _transformation_matrix: Option<&[f64; 16]>,
) -> Result<GeometryHandle> {
    Err(Error::UnsupportedFeature(
        "geometry template import is not implemented yet",
    ))
}

fn reject_unsupported_mappings_owned(
    semantics: Option<&OwnedJsonValue>,
    material: Option<&OwnedJsonValue>,
    texture: Option<&OwnedJsonValue>,
) -> Result<()> {
    if semantics.is_some_and(value_is_present_owned) {
        return Err(Error::UnsupportedFeature(
            "geometry semantics import is not implemented yet",
        ));
    }
    if material.is_some_and(value_is_present_owned) {
        return Err(Error::UnsupportedFeature(
            "geometry material import is not implemented yet",
        ));
    }
    if texture.is_some_and(value_is_present_owned) {
        return Err(Error::UnsupportedFeature(
            "geometry texture import is not implemented yet",
        ));
    }
    Ok(())
}

fn reject_unsupported_mappings_borrowed(
    semantics: Option<&BorrowedJsonValue<'_>>,
    material: Option<&BorrowedJsonValue<'_>>,
    texture: Option<&BorrowedJsonValue<'_>>,
) -> Result<()> {
    if semantics.is_some_and(value_is_present_borrowed) {
        return Err(Error::UnsupportedFeature(
            "geometry semantics import is not implemented yet",
        ));
    }
    if material.is_some_and(value_is_present_borrowed) {
        return Err(Error::UnsupportedFeature(
            "geometry material import is not implemented yet",
        ));
    }
    if texture.is_some_and(value_is_present_borrowed) {
        return Err(Error::UnsupportedFeature(
            "geometry texture import is not implemented yet",
        ));
    }
    Ok(())
}

fn parse_lod(value: Option<&str>) -> Result<Option<LoD>> {
    match value {
        None => Ok(None),
        Some("0") => Ok(Some(LoD::LoD0)),
        Some("0.0") => Ok(Some(LoD::LoD0_0)),
        Some("0.1") => Ok(Some(LoD::LoD0_1)),
        Some("0.2") => Ok(Some(LoD::LoD0_2)),
        Some("0.3") => Ok(Some(LoD::LoD0_3)),
        Some("1") => Ok(Some(LoD::LoD1)),
        Some("1.0") => Ok(Some(LoD::LoD1_0)),
        Some("1.1") => Ok(Some(LoD::LoD1_1)),
        Some("1.2") => Ok(Some(LoD::LoD1_2)),
        Some("1.3") => Ok(Some(LoD::LoD1_3)),
        Some("2") => Ok(Some(LoD::LoD2)),
        Some("2.0") => Ok(Some(LoD::LoD2_0)),
        Some("2.1") => Ok(Some(LoD::LoD2_1)),
        Some("2.2") => Ok(Some(LoD::LoD2_2)),
        Some("2.3") => Ok(Some(LoD::LoD2_3)),
        Some("3") => Ok(Some(LoD::LoD3)),
        Some("3.0") => Ok(Some(LoD::LoD3_0)),
        Some("3.1") => Ok(Some(LoD::LoD3_1)),
        Some("3.2") => Ok(Some(LoD::LoD3_2)),
        Some("3.3") => Ok(Some(LoD::LoD3_3)),
        Some(other) => Err(Error::InvalidValue(format!(
            "unsupported geometry lod value '{other}'"
        ))),
    }
}

fn value_is_present_owned(value: &OwnedJsonValue) -> bool {
    match value {
        OwnedJsonValue::Null => false,
        OwnedJsonValue::Array(values) => !values.is_empty(),
        OwnedJsonValue::Object(values) => !values.is_empty(),
        _ => true,
    }
}

fn value_is_present_borrowed(value: &BorrowedJsonValue<'_>) -> bool {
    match value {
        BorrowedJsonValue::Null => false,
        BorrowedJsonValue::Array(values) => !values.is_empty(),
        BorrowedJsonValue::Object(values) => !values.is_empty(),
        _ => true,
    }
}
