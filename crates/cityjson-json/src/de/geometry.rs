use cityjson::resources::handles::{
    GeometryHandle, GeometryTemplateHandle, MaterialHandle, SemanticHandle, TextureHandle,
};
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{
    AffineTransform3D, BorrowedCityModel, CityModel, GeometryDraft, LineStringDraft, LoD,
    OwnedCityModel, OwnedSemantic, PointDraft, RingDraft, SemanticType, ShellDraft, SolidDraft,
    SurfaceDraft, VertexIndex,
};
use serde::Deserialize;
use serde_json::Value as OwnedJsonValue;
use serde_json_borrow::Value as BorrowedJsonValue;

use crate::de::attributes::owned_attributes_from_json;
use crate::errors::{Error, Result};

type MultiPointBoundary = Vec<u32>;
type MultiLineStringBoundary = Vec<MultiPointBoundary>;
type MultiSurfaceBoundary = Vec<MultiLineStringBoundary>;
type SolidBoundary = Vec<MultiSurfaceBoundary>;
type MultiSolidBoundary = Vec<SolidBoundary>;

#[derive(Clone, Debug)]
struct RingTextureAssignment {
    texture: TextureHandle,
    uvs: Vec<VertexIndex<u32>>,
}

#[derive(Debug, Default)]
struct SurfaceMappings {
    semantics: Vec<Option<SemanticHandle>>,
    materials: Vec<(String, Vec<Option<MaterialHandle>>)>,
    textures: Vec<(String, Vec<Option<RingTextureAssignment>>)>,
}

#[derive(Debug, Default)]
pub(crate) struct GeometryResources {
    pub(crate) materials: Vec<MaterialHandle>,
    pub(crate) textures: Vec<TextureHandle>,
    pub(crate) templates: Vec<GeometryTemplateHandle>,
}

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
    resources: &GeometryResources,
) -> Result<Vec<GeometryHandle>> {
    raw_geometries
        .into_iter()
        .map(|geometry| import_owned_geometry(geometry, model, resources))
        .collect()
}

pub(crate) fn import_owned_geometry_templates(
    raw_geometries: Vec<RawGeometryOwned>,
    model: &mut OwnedCityModel,
    resources: &mut GeometryResources,
) -> Result<()> {
    for geometry in raw_geometries {
        resources
            .templates
            .push(import_owned_template_geometry(geometry, model)?);
    }
    Ok(())
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
    resources: &GeometryResources,
) -> Result<GeometryHandle> {
    match raw_geometry {
        RawGeometryOwned::MultiPoint {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_owned_multi_point(
            lod.as_deref(),
            boundaries,
            semantics.as_ref(),
            material.as_ref(),
            texture.as_ref(),
            model,
        ),
        RawGeometryOwned::MultiLineString {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_owned_multi_line_string(
            lod.as_deref(),
            boundaries,
            semantics.as_ref(),
            material.as_ref(),
            texture.as_ref(),
            model,
        ),
        RawGeometryOwned::MultiSurface {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_owned_multi_surface(
            lod.as_deref(),
            boundaries,
            false,
            semantics.as_ref(),
            material.as_ref(),
            texture.as_ref(),
            model,
            resources,
        ),
        RawGeometryOwned::CompositeSurface {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_owned_multi_surface(
            lod.as_deref(),
            boundaries,
            true,
            semantics.as_ref(),
            material.as_ref(),
            texture.as_ref(),
            model,
            resources,
        ),
        RawGeometryOwned::Solid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_owned_solid(
            lod.as_deref(),
            boundaries,
            semantics.as_ref(),
            material.as_ref(),
            texture.as_ref(),
            model,
            resources,
        ),
        RawGeometryOwned::MultiSolid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_owned_multi_solid(
            lod.as_deref(),
            boundaries,
            false,
            semantics.as_ref(),
            material.as_ref(),
            texture.as_ref(),
            model,
            resources,
        ),
        RawGeometryOwned::CompositeSolid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => import_owned_multi_solid(
            lod.as_deref(),
            boundaries,
            true,
            semantics.as_ref(),
            material.as_ref(),
            texture.as_ref(),
            model,
            resources,
        ),
        RawGeometryOwned::GeometryInstance {
            lod,
            template,
            boundaries,
            transformation_matrix,
        } => import_geometry_instance(
            lod.as_deref(),
            model,
            resources,
            template,
            boundaries.as_ref(),
            transformation_matrix.as_ref(),
        ),
    }
}

fn import_owned_template_geometry(
    raw_geometry: RawGeometryOwned,
    model: &mut OwnedCityModel,
) -> Result<GeometryTemplateHandle> {
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
            GeometryDraft::multi_point(
                parse_lod(lod.as_deref())?,
                boundaries.into_iter().map(point_draft),
            )
            .insert_template_into(model)
            .map_err(Error::from)
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
            let linestrings = boundaries.into_iter().map(|linestring| {
                LineStringDraft::new(linestring.into_iter().map(VertexIndex::new))
            });
            GeometryDraft::multi_line_string(parse_lod(lod.as_deref())?, linestrings)
                .insert_template_into(model)
                .map_err(Error::from)
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
            let surfaces = boundaries
                .into_iter()
                .map(surface_draft::<cityjson::prelude::OwnedStringStorage>)
                .collect::<Result<Vec<_>>>()?;
            GeometryDraft::multi_surface(parse_lod(lod.as_deref())?, surfaces)
                .insert_template_into(model)
                .map_err(Error::from)
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
            let surfaces = boundaries
                .into_iter()
                .map(surface_draft::<cityjson::prelude::OwnedStringStorage>)
                .collect::<Result<Vec<_>>>()?;
            GeometryDraft::composite_surface(parse_lod(lod.as_deref())?, surfaces)
                .insert_template_into(model)
                .map_err(Error::from)
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
            let mut shells = boundaries
                .into_iter()
                .map(solid_shell_draft::<cityjson::prelude::OwnedStringStorage>);
            let outer = shells.next().transpose()?.ok_or_else(|| {
                Error::InvalidValue("Solid geometry requires at least one shell".to_owned())
            })?;
            let inners = shells.collect::<Result<Vec<_>>>()?;
            GeometryDraft::solid(parse_lod(lod.as_deref())?, outer, inners)
                .insert_template_into(model)
                .map_err(Error::from)
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
            let solids = boundaries
                .into_iter()
                .map(solid_draft::<cityjson::prelude::OwnedStringStorage>)
                .collect::<Result<Vec<_>>>()?;
            GeometryDraft::multi_solid(parse_lod(lod.as_deref())?, solids)
                .insert_template_into(model)
                .map_err(Error::from)
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
            let solids = boundaries
                .into_iter()
                .map(solid_draft::<cityjson::prelude::OwnedStringStorage>)
                .collect::<Result<Vec<_>>>()?;
            GeometryDraft::composite_solid(parse_lod(lod.as_deref())?, solids)
                .insert_template_into(model)
                .map_err(Error::from)
        }
        RawGeometryOwned::GeometryInstance { .. } => Err(Error::UnsupportedFeature(
            "GeometryInstance cannot be used as a geometry template",
        )),
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
        } => reject_borrowed_geometry_instance(
            lod,
            template,
            boundaries.as_ref(),
            transformation_matrix.as_ref(),
        ),
    }
}

fn import_owned_multi_point(
    lod: Option<&str>,
    boundaries: MultiPointBoundary,
    semantics: Option<&OwnedJsonValue>,
    material: Option<&OwnedJsonValue>,
    texture: Option<&OwnedJsonValue>,
    model: &mut OwnedCityModel,
) -> Result<GeometryHandle> {
    if material.is_some_and(value_is_present_owned) {
        return Err(Error::UnsupportedFeature(
            "geometry material import is not supported for MultiPoint",
        ));
    }
    if texture.is_some_and(value_is_present_owned) {
        return Err(Error::UnsupportedFeature(
            "geometry texture import is not supported for MultiPoint",
        ));
    }

    let semantic_handles = import_geometry_semantics(semantics, model)?;
    let assignments = parse_point_assignments(semantics, &semantic_handles, boundaries.len())?;
    let points = boundaries.into_iter().enumerate().map(|(index, boundary)| {
        let point = PointDraft::new(VertexIndex::new(boundary));
        if let Some(Some(semantic)) = assignments.get(index) {
            point.with_semantic(*semantic)
        } else {
            point
        }
    });

    GeometryDraft::multi_point(parse_lod(lod)?, points)
        .insert_into(model)
        .map_err(Error::from)
}

fn import_owned_multi_line_string(
    lod: Option<&str>,
    boundaries: MultiLineStringBoundary,
    semantics: Option<&OwnedJsonValue>,
    material: Option<&OwnedJsonValue>,
    texture: Option<&OwnedJsonValue>,
    model: &mut OwnedCityModel,
) -> Result<GeometryHandle> {
    if material.is_some_and(value_is_present_owned) {
        return Err(Error::UnsupportedFeature(
            "geometry material import is not supported for MultiLineString",
        ));
    }
    if texture.is_some_and(value_is_present_owned) {
        return Err(Error::UnsupportedFeature(
            "geometry texture import is not supported for MultiLineString",
        ));
    }

    let semantic_handles = import_geometry_semantics(semantics, model)?;
    let assignments = parse_linestring_assignments(semantics, &semantic_handles, boundaries.len())?;
    let linestrings = boundaries.into_iter().enumerate().map(|(index, linestring)| {
        let linestring = LineStringDraft::new(linestring.into_iter().map(VertexIndex::new));
        if let Some(Some(semantic)) = assignments.get(index) {
            linestring.with_semantic(*semantic)
        } else {
            linestring
        }
    });

    GeometryDraft::multi_line_string(parse_lod(lod)?, linestrings)
        .insert_into(model)
        .map_err(Error::from)
}

fn import_owned_multi_surface(
    lod: Option<&str>,
    boundaries: MultiSurfaceBoundary,
    is_composite: bool,
    semantics: Option<&OwnedJsonValue>,
    material: Option<&OwnedJsonValue>,
    texture: Option<&OwnedJsonValue>,
    model: &mut OwnedCityModel,
    resources: &GeometryResources,
) -> Result<GeometryHandle> {
    let mappings = parse_multi_surface_mappings(
        semantics,
        material,
        texture,
        &boundaries,
        model,
        resources,
    )?;
    let mut surface_index = 0;
    let mut ring_index = 0;
    let surfaces = boundaries
        .into_iter()
        .map(|surface| {
            mapped_surface_draft(surface, &mappings, &mut surface_index, &mut ring_index)
        })
        .collect::<Result<Vec<_>>>()?;
    let draft = if is_composite {
        GeometryDraft::composite_surface(parse_lod(lod)?, surfaces)
    } else {
        GeometryDraft::multi_surface(parse_lod(lod)?, surfaces)
    };
    draft.insert_into(model).map_err(Error::from)
}

fn import_owned_solid(
    lod: Option<&str>,
    boundaries: SolidBoundary,
    semantics: Option<&OwnedJsonValue>,
    material: Option<&OwnedJsonValue>,
    texture: Option<&OwnedJsonValue>,
    model: &mut OwnedCityModel,
    resources: &GeometryResources,
) -> Result<GeometryHandle> {
    let mappings =
        parse_solid_mappings(semantics, material, texture, &boundaries, model, resources)?;
    let mut surface_index = 0;
    let mut ring_index = 0;
    let mut shells = boundaries.into_iter().map(|shell| {
        mapped_shell_draft(shell, &mappings, &mut surface_index, &mut ring_index)
    });
    let outer = shells.next().transpose()?.ok_or_else(|| {
        Error::InvalidValue("Solid geometry requires at least one shell".to_owned())
    })?;
    let inners = shells.collect::<Result<Vec<_>>>()?;
    GeometryDraft::solid(parse_lod(lod)?, outer, inners)
        .insert_into(model)
        .map_err(Error::from)
}

fn import_owned_multi_solid(
    lod: Option<&str>,
    boundaries: MultiSolidBoundary,
    is_composite: bool,
    semantics: Option<&OwnedJsonValue>,
    material: Option<&OwnedJsonValue>,
    texture: Option<&OwnedJsonValue>,
    model: &mut OwnedCityModel,
    resources: &GeometryResources,
) -> Result<GeometryHandle> {
    let mappings =
        parse_multi_solid_mappings(semantics, material, texture, &boundaries, model, resources)?;
    let mut surface_index = 0;
    let mut ring_index = 0;
    let solids = boundaries
        .into_iter()
        .map(|solid| {
            mapped_solid_draft(solid, &mappings, &mut surface_index, &mut ring_index)
        })
        .collect::<Result<Vec<_>>>()?;
    let draft = if is_composite {
        GeometryDraft::composite_solid(parse_lod(lod)?, solids)
    } else {
        GeometryDraft::multi_solid(parse_lod(lod)?, solids)
    };
    draft.insert_into(model).map_err(Error::from)
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

fn solid_shell_draft<SS: StringStorage>(
    surfaces: MultiSurfaceBoundary,
) -> Result<ShellDraft<u32, SS>> {
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

fn import_geometry_instance(
    _lod: Option<&str>,
    model: &mut OwnedCityModel,
    resources: &GeometryResources,
    template: Option<u32>,
    boundaries: Option<&OwnedJsonValue>,
    transformation_matrix: Option<&[f64; 16]>,
) -> Result<GeometryHandle> {
    let template = template.ok_or_else(|| {
        Error::InvalidValue("GeometryInstance is missing a template index".to_owned())
    })?;
    let template = resources
        .templates
        .get(template as usize)
        .copied()
        .ok_or_else(|| {
            Error::InvalidValue(format!("invalid geometry template index '{template}'"))
        })?;

    let reference_point = boundaries
        .and_then(OwnedJsonValue::as_array)
        .and_then(|values| values.first())
        .and_then(OwnedJsonValue::as_u64)
        .ok_or_else(|| {
            Error::InvalidValue(
                "GeometryInstance boundaries must contain a single reference-point index"
                    .to_owned(),
            )
        })?;

    GeometryDraft::instance(
        template,
        VertexIndex::new(reference_point as u32),
        transformation_matrix
            .copied()
            .map(AffineTransform3D::from)
            .unwrap_or_default(),
    )
    .insert_into(model)
    .map_err(Error::from)
}

fn mapped_ring_draft(
    vertices: MultiPointBoundary,
    mappings: &SurfaceMappings,
    ring_index: &mut usize,
) -> Result<RingDraft<u32, cityjson::prelude::OwnedStringStorage>> {
    let current_ring = *ring_index;
    let mut ring = RingDraft::new(vertices.into_iter().map(VertexIndex::new));
    for (theme, assignments) in &mappings.textures {
        if let Some(Some(texture)) = assignments.get(current_ring) {
            ring = ring.with_texture(theme.clone(), texture.texture, texture.uvs.iter().copied());
        }
    }
    *ring_index += 1;
    Ok(ring)
}

fn mapped_surface_draft(
    rings: MultiLineStringBoundary,
    mappings: &SurfaceMappings,
    surface_index: &mut usize,
    ring_index: &mut usize,
) -> Result<SurfaceDraft<u32, cityjson::prelude::OwnedStringStorage>> {
    let current_surface = *surface_index;
    let mut rings = rings.into_iter();
    let outer = rings.next().ok_or_else(|| {
        Error::InvalidValue("surface boundary requires at least one ring".to_owned())
    })?;
    let outer = mapped_ring_draft(outer, mappings, ring_index)?;
    let inners = rings
        .map(|ring| mapped_ring_draft(ring, mappings, ring_index))
        .collect::<Result<Vec<_>>>()?;
    let mut surface = SurfaceDraft::new(outer, inners);
    if let Some(Some(semantic)) = mappings.semantics.get(current_surface) {
        surface = surface.with_semantic(*semantic);
    }
    for (theme, assignments) in &mappings.materials {
        if let Some(Some(material)) = assignments.get(current_surface) {
            surface = surface.with_material(
                cityjson::v2_0::ThemeName::new(theme.clone()),
                *material,
            );
        }
    }
    *surface_index += 1;
    Ok(surface)
}

fn mapped_shell_draft(
    surfaces: MultiSurfaceBoundary,
    mappings: &SurfaceMappings,
    surface_index: &mut usize,
    ring_index: &mut usize,
) -> Result<ShellDraft<u32, cityjson::prelude::OwnedStringStorage>> {
    let surfaces = surfaces
        .into_iter()
        .map(|surface| mapped_surface_draft(surface, mappings, surface_index, ring_index))
        .collect::<Result<Vec<_>>>()?;
    Ok(ShellDraft::new(surfaces))
}

fn mapped_solid_draft(
    shells: SolidBoundary,
    mappings: &SurfaceMappings,
    surface_index: &mut usize,
    ring_index: &mut usize,
) -> Result<SolidDraft<u32, cityjson::prelude::OwnedStringStorage>> {
    let mut shells = shells.into_iter().map(|shell| {
        mapped_shell_draft(shell, mappings, surface_index, ring_index)
    });
    let outer = shells.next().transpose()?.ok_or_else(|| {
        Error::InvalidValue("solid boundary requires at least one shell".to_owned())
    })?;
    let inners = shells.collect::<Result<Vec<_>>>()?;
    Ok(SolidDraft::new(outer, inners))
}

fn parse_multi_surface_mappings(
    semantics: Option<&OwnedJsonValue>,
    material: Option<&OwnedJsonValue>,
    texture: Option<&OwnedJsonValue>,
    boundaries: &MultiSurfaceBoundary,
    model: &mut OwnedCityModel,
    resources: &GeometryResources,
) -> Result<SurfaceMappings> {
    let semantic_handles = import_geometry_semantics(semantics, model)?;
    Ok(SurfaceMappings {
        semantics: parse_multi_surface_scalar_assignments(
            semantics,
            &semantic_handles,
            boundaries,
            "geometry semantics.values",
        )?,
        materials: parse_multi_surface_materials(material, boundaries, resources)?,
        textures: parse_multi_surface_textures(texture, boundaries, resources)?,
    })
}

fn parse_solid_mappings(
    semantics: Option<&OwnedJsonValue>,
    material: Option<&OwnedJsonValue>,
    texture: Option<&OwnedJsonValue>,
    boundaries: &SolidBoundary,
    model: &mut OwnedCityModel,
    resources: &GeometryResources,
) -> Result<SurfaceMappings> {
    let semantic_handles = import_geometry_semantics(semantics, model)?;
    Ok(SurfaceMappings {
        semantics: parse_solid_scalar_assignments(
            semantics,
            &semantic_handles,
            boundaries,
            "geometry semantics.values",
        )?,
        materials: parse_solid_materials(material, boundaries, resources)?,
        textures: parse_solid_textures(texture, boundaries, resources)?,
    })
}

fn parse_multi_solid_mappings(
    semantics: Option<&OwnedJsonValue>,
    material: Option<&OwnedJsonValue>,
    texture: Option<&OwnedJsonValue>,
    boundaries: &MultiSolidBoundary,
    model: &mut OwnedCityModel,
    resources: &GeometryResources,
) -> Result<SurfaceMappings> {
    let semantic_handles = import_geometry_semantics(semantics, model)?;
    Ok(SurfaceMappings {
        semantics: parse_multi_solid_scalar_assignments(
            semantics,
            &semantic_handles,
            boundaries,
            "geometry semantics.values",
        )?,
        materials: parse_multi_solid_materials(material, boundaries, resources)?,
        textures: parse_multi_solid_textures(texture, boundaries, resources)?,
    })
}

fn import_geometry_semantics(
    semantics: Option<&OwnedJsonValue>,
    model: &mut OwnedCityModel,
) -> Result<Vec<SemanticHandle>> {
    let Some(semantics) = semantics else {
        return Ok(Vec::new());
    };
    let Some(semantics_object) = semantics.as_object() else {
        return Err(Error::InvalidValue(
            "geometry semantics must be a JSON object".to_owned(),
        ));
    };
    let surfaces = semantics_object
        .get("surfaces")
        .and_then(OwnedJsonValue::as_array)
        .ok_or_else(|| {
            Error::InvalidValue("geometry semantics.surfaces must be an array".to_owned())
        })?;

    let mut pending_links = Vec::with_capacity(surfaces.len());
    let mut handles = Vec::with_capacity(surfaces.len());
    for surface in surfaces {
        let object = surface.as_object().ok_or_else(|| {
            Error::InvalidValue("geometry semantic surface must be an object".to_owned())
        })?;
        let type_value = object
            .get("type")
            .and_then(OwnedJsonValue::as_str)
            .ok_or_else(|| {
                Error::InvalidValue("geometry semantic surface is missing a type".to_owned())
            })?;
        let mut semantic = OwnedSemantic::new(parse_semantic_type(type_value)?);
        let mut attributes_object = object.clone();
        attributes_object.remove("type");
        attributes_object.remove("parent");
        attributes_object.remove("children");
        if !attributes_object.is_empty() {
            let attributes =
                owned_attributes_from_json(&OwnedJsonValue::Object(attributes_object), "semantic")?;
            *semantic.attributes_mut() = attributes;
        }
        let parent = object
            .get("parent")
            .map(parse_optional_usize)
            .transpose()?
            .flatten();
        let children = object
            .get("children")
            .map(parse_usize_array)
            .transpose()?
            .unwrap_or_default();
        handles.push(model.add_semantic(semantic).map_err(Error::from)?);
        pending_links.push((parent, children));
    }

    for (index, (parent, children)) in pending_links.into_iter().enumerate() {
        let handle = handles[index];
        let semantic = model
            .get_semantic_mut(handle)
            .ok_or_else(|| Error::InvalidValue(format!("missing semantic handle {handle}")))?;
        if let Some(parent_index) = parent {
            if let Some(parent_handle) = handles.get(parent_index).copied() {
                semantic.set_parent(parent_handle);
            }
        }
        if !children.is_empty() {
            let semantic_children = semantic.children_mut();
            semantic_children.reserve(children.len());
            for child_index in children {
                if let Some(child_handle) = handles.get(child_index).copied() {
                    semantic_children.push(child_handle);
                }
            }
        }
    }

    Ok(handles)
}

fn parse_point_assignments(
    semantics: Option<&OwnedJsonValue>,
    semantic_handles: &[SemanticHandle],
    expected_len: usize,
) -> Result<Vec<Option<SemanticHandle>>> {
    let Some(values) = semantics
        .and_then(OwnedJsonValue::as_object)
        .and_then(|object| object.get("values"))
    else {
        return Ok(vec![None; expected_len]);
    };
    parse_assignment_array(values, semantic_handles, expected_len, "geometry semantics.values")
}

fn parse_linestring_assignments(
    semantics: Option<&OwnedJsonValue>,
    semantic_handles: &[SemanticHandle],
    expected_len: usize,
) -> Result<Vec<Option<SemanticHandle>>> {
    let Some(values) = semantics
        .and_then(OwnedJsonValue::as_object)
        .and_then(|object| object.get("values"))
    else {
        return Ok(vec![None; expected_len]);
    };
    parse_assignment_array(values, semantic_handles, expected_len, "geometry semantics.values")
}

fn parse_multi_surface_scalar_assignments<T: Copy>(
    semantics: Option<&OwnedJsonValue>,
    handles: &[T],
    boundaries: &MultiSurfaceBoundary,
    context: &'static str,
) -> Result<Vec<Option<T>>> {
    let expected_len = boundaries.len();
    let Some(values) = semantics
        .and_then(OwnedJsonValue::as_object)
        .and_then(|object| object.get("values"))
    else {
        return Ok(vec![None; expected_len]);
    };
    parse_assignment_array(values, handles, expected_len, context)
}

fn parse_solid_scalar_assignments<T: Copy>(
    semantics: Option<&OwnedJsonValue>,
    handles: &[T],
    boundaries: &SolidBoundary,
    context: &'static str,
) -> Result<Vec<Option<T>>> {
    let Some(values) = semantics
        .and_then(OwnedJsonValue::as_object)
        .and_then(|object| object.get("values"))
    else {
        return Ok(vec![None; boundaries.iter().map(Vec::len).sum()]);
    };
    let expected_len = boundaries.iter().map(Vec::len).sum();
    parse_assignment_array(values, handles, expected_len, context)
}

fn parse_multi_solid_scalar_assignments<T: Copy>(
    semantics: Option<&OwnedJsonValue>,
    handles: &[T],
    boundaries: &MultiSolidBoundary,
    context: &'static str,
) -> Result<Vec<Option<T>>> {
    let Some(values) = semantics
        .and_then(OwnedJsonValue::as_object)
        .and_then(|object| object.get("values"))
    else {
        return Ok(vec![None; boundaries
            .iter()
            .map(|solid| solid.iter().map(Vec::len).sum::<usize>())
            .sum()]);
    };
    let expected_len = boundaries
        .iter()
        .map(|solid| solid.iter().map(Vec::len).sum::<usize>())
        .sum();
    parse_assignment_array(values, handles, expected_len, context)
}

fn parse_multi_surface_materials(
    material: Option<&OwnedJsonValue>,
    boundaries: &MultiSurfaceBoundary,
    resources: &GeometryResources,
) -> Result<Vec<(String, Vec<Option<MaterialHandle>>)>> {
    let expected_len = boundaries.len();
    parse_material_themes(
        material,
        &resources.materials,
        expected_len,
        |values| parse_assignment_array(values, &resources.materials, expected_len, "geometry material.values"),
    )
}

fn parse_solid_materials(
    material: Option<&OwnedJsonValue>,
    boundaries: &SolidBoundary,
    resources: &GeometryResources,
) -> Result<Vec<(String, Vec<Option<MaterialHandle>>)>> {
    let surface_count = boundaries.iter().map(Vec::len).sum();
    parse_material_themes(material, &resources.materials, surface_count, |values| {
        parse_assignment_array(values, &resources.materials, surface_count, "geometry material.values")
    })
}

fn parse_multi_solid_materials(
    material: Option<&OwnedJsonValue>,
    boundaries: &MultiSolidBoundary,
    resources: &GeometryResources,
) -> Result<Vec<(String, Vec<Option<MaterialHandle>>)>> {
    let surface_count = boundaries
        .iter()
        .map(|solid| solid.iter().map(Vec::len).sum::<usize>())
        .sum();
    parse_material_themes(material, &resources.materials, surface_count, |values| {
        parse_assignment_array(values, &resources.materials, surface_count, "geometry material.values")
    })
}

fn parse_multi_surface_textures(
    texture: Option<&OwnedJsonValue>,
    boundaries: &MultiSurfaceBoundary,
    resources: &GeometryResources,
) -> Result<Vec<(String, Vec<Option<RingTextureAssignment>>) >> {
    parse_texture_themes(texture, |values| {
        parse_multi_surface_ring_textures(values, boundaries, resources)
    })
}

fn parse_solid_textures(
    texture: Option<&OwnedJsonValue>,
    boundaries: &SolidBoundary,
    resources: &GeometryResources,
) -> Result<Vec<(String, Vec<Option<RingTextureAssignment>>) >> {
    parse_texture_themes(texture, |values| parse_solid_ring_textures(values, boundaries, resources))
}

fn parse_multi_solid_textures(
    texture: Option<&OwnedJsonValue>,
    boundaries: &MultiSolidBoundary,
    resources: &GeometryResources,
) -> Result<Vec<(String, Vec<Option<RingTextureAssignment>>) >> {
    parse_texture_themes(texture, |values| {
        parse_multi_solid_ring_textures(values, boundaries, resources)
    })
}

fn parse_material_themes<F>(
    material: Option<&OwnedJsonValue>,
    handles: &[MaterialHandle],
    surface_count: usize,
    mut parse_values: F,
) -> Result<Vec<(String, Vec<Option<MaterialHandle>>)>>
where
    F: FnMut(&OwnedJsonValue) -> Result<Vec<Option<MaterialHandle>>>,
{
    let Some(material) = material else {
        return Ok(Vec::new());
    };
    let Some(material_object) = material.as_object() else {
        return Err(Error::InvalidValue(
            "geometry material must be an object".to_owned(),
        ));
    };
    let mut materials = Vec::with_capacity(material_object.len());
    for (theme, value) in material_object {
        let value = value.as_object().ok_or_else(|| {
            Error::InvalidValue("geometry material theme must be an object".to_owned())
        })?;
        let assignments = if let Some(single_value) = value.get("value") {
            let material =
                parse_optional_handle_index(single_value, handles, "geometry material.value")?;
            vec![material; surface_count]
        } else if let Some(values) = value.get("values") {
            parse_values(values)?
        } else {
            return Err(Error::InvalidValue(format!(
                "geometry material theme '{theme}' must contain value or values"
            )));
        };
        materials.push((theme.clone(), assignments));
    }
    Ok(materials)
}

fn parse_texture_themes<F>(
    texture: Option<&OwnedJsonValue>,
    mut parse_values: F,
) -> Result<Vec<(String, Vec<Option<RingTextureAssignment>>)>>
where
    F: FnMut(&OwnedJsonValue) -> Result<Vec<Option<RingTextureAssignment>>>,
{
    let Some(texture) = texture else {
        return Ok(Vec::new());
    };
    let Some(texture_object) = texture.as_object() else {
        return Err(Error::InvalidValue(
            "geometry texture must be an object".to_owned(),
        ));
    };
    let mut textures = Vec::with_capacity(texture_object.len());
    for (theme, value) in texture_object {
        let value = value.as_object().ok_or_else(|| {
            Error::InvalidValue("geometry texture theme must be an object".to_owned())
        })?;
        let values = value.get("values").ok_or_else(|| {
            Error::InvalidValue(format!(
                "geometry texture theme '{theme}' must contain values"
            ))
        })?;
        textures.push((theme.clone(), parse_values(values)?));
    }
    Ok(textures)
}

fn parse_multi_surface_ring_textures(
    values: &OwnedJsonValue,
    boundaries: &MultiSurfaceBoundary,
    resources: &GeometryResources,
) -> Result<Vec<Option<RingTextureAssignment>>> {
    let expected_len = boundaries.iter().map(Vec::len).sum();
    parse_ring_texture_assignments(values, expected_len, resources)
}

fn parse_solid_ring_textures(
    values: &OwnedJsonValue,
    boundaries: &SolidBoundary,
    resources: &GeometryResources,
) -> Result<Vec<Option<RingTextureAssignment>>> {
    let expected_len = boundaries
        .iter()
        .map(|shell| shell.iter().map(Vec::len).sum::<usize>())
        .sum();
    parse_ring_texture_assignments(values, expected_len, resources)
}

fn parse_multi_solid_ring_textures(
    values: &OwnedJsonValue,
    boundaries: &MultiSolidBoundary,
    resources: &GeometryResources,
) -> Result<Vec<Option<RingTextureAssignment>>> {
    let expected_len = boundaries
        .iter()
        .map(|solid| {
            solid
                .iter()
                .map(|shell| shell.iter().map(Vec::len).sum::<usize>())
                .sum::<usize>()
        })
        .sum();
    parse_ring_texture_assignments(values, expected_len, resources)
}

fn parse_ring_texture_assignments(
    values: &OwnedJsonValue,
    expected_len: usize,
    resources: &GeometryResources,
) -> Result<Vec<Option<RingTextureAssignment>>> {
    let mut assignments = Vec::new();
    flatten_ring_texture_assignments(values, resources, &mut assignments)?;
    if assignments.len() < expected_len {
        assignments.resize(expected_len, None);
    }
    Ok(assignments)
}

fn flatten_ring_texture_assignments(
    value: &OwnedJsonValue,
    resources: &GeometryResources,
    out: &mut Vec<Option<RingTextureAssignment>>,
) -> Result<()> {
    if looks_like_ring_texture_assignment(value) {
        out.push(parse_ring_texture_assignment(value, resources)?);
        return Ok(());
    }
    let values = value.as_array().ok_or_else(|| {
        Error::InvalidValue(format!(
            "geometry texture.values must be a nested array, got {value}"
        ))
    })?;
    for child in values {
        flatten_ring_texture_assignments(child, resources, out)?;
    }
    Ok(())
}

fn looks_like_ring_texture_assignment(value: &OwnedJsonValue) -> bool {
    let Some(values) = value.as_array() else {
        return false;
    };
    let Some(first) = values.first() else {
        return false;
    };
    match first {
        OwnedJsonValue::Null => values.len() == 1,
        OwnedJsonValue::Number(_) => values.iter().skip(1).all(OwnedJsonValue::is_number),
        _ => false,
    }
}

fn parse_ring_texture_assignment(
    value: &OwnedJsonValue,
    resources: &GeometryResources,
) -> Result<Option<RingTextureAssignment>> {
    let values = value.as_array().ok_or_else(|| {
        Error::InvalidValue("geometry texture ring value must be an array".to_owned())
    })?;
    let Some(first) = values.first() else {
        return Err(Error::InvalidValue(
            "geometry texture ring value cannot be empty".to_owned(),
        ));
    };
    if first.is_null() {
        return Ok(None);
    }

    let texture = parse_optional_handle_index(first, &resources.textures, "geometry texture index")?
        .ok_or_else(|| Error::InvalidValue("geometry texture index cannot be null".to_owned()))?;
    let mut uvs = Vec::with_capacity(values.len().saturating_sub(1));
    for uv in values.iter().skip(1) {
        let index = uv.as_u64().ok_or_else(|| {
            Error::InvalidValue(format!(
                "geometry texture uv index must be an unsigned integer, got {uv}"
            ))
        })?;
        uvs.push(VertexIndex::new(index as u32));
    }
    Ok(Some(RingTextureAssignment { texture, uvs }))
}

fn parse_optional_usize(value: &OwnedJsonValue) -> Result<Option<usize>> {
    match value {
        OwnedJsonValue::Null => Ok(None),
        OwnedJsonValue::Number(number) => number
            .as_u64()
            .map(|value| value as usize)
            .map(Some)
            .ok_or_else(|| {
                Error::InvalidValue(format!(
                    "expected unsigned integer or null, got {value}"
                ))
            }),
        _ => Err(Error::InvalidValue(format!(
            "expected unsigned integer or null, got {value}"
        ))),
    }
}

fn parse_usize_array(value: &OwnedJsonValue) -> Result<Vec<usize>> {
    let values = value.as_array().ok_or_else(|| {
        Error::InvalidValue(format!("expected array of unsigned integers, got {value}"))
    })?;
    values
        .iter()
        .map(|value| {
            value.as_u64().map(|value| value as usize).ok_or_else(|| {
                Error::InvalidValue(format!(
                    "expected unsigned integer in array, got {value}"
                ))
            })
        })
        .collect()
}

fn parse_semantic_type(
    value: &str,
) -> Result<SemanticType<cityjson::prelude::OwnedStringStorage>> {
    Ok(match value {
        "RoofSurface" => SemanticType::RoofSurface,
        "GroundSurface" => SemanticType::GroundSurface,
        "WallSurface" => SemanticType::WallSurface,
        "ClosureSurface" => SemanticType::ClosureSurface,
        "OuterCeilingSurface" => SemanticType::OuterCeilingSurface,
        "OuterFloorSurface" => SemanticType::OuterFloorSurface,
        "Window" => SemanticType::Window,
        "Door" => SemanticType::Door,
        "InteriorWallSurface" => SemanticType::InteriorWallSurface,
        "CeilingSurface" => SemanticType::CeilingSurface,
        "FloorSurface" => SemanticType::FloorSurface,
        "WaterSurface" => SemanticType::WaterSurface,
        "WaterGroundSurface" => SemanticType::WaterGroundSurface,
        "WaterClosureSurface" => SemanticType::WaterClosureSurface,
        "TrafficArea" => SemanticType::TrafficArea,
        "AuxiliaryTrafficArea" => SemanticType::AuxiliaryTrafficArea,
        "TransportationMarking" => SemanticType::TransportationMarking,
        "TransportationHole" => SemanticType::TransportationHole,
        _ if value.starts_with('+') => SemanticType::Extension(value.to_owned()),
        _ => {
            return Err(Error::InvalidValue(format!(
                "invalid Semantic type: {value}"
            )))
        }
    })
}

fn parse_assignment_array<T: Copy>(
    value: &OwnedJsonValue,
    handles: &[T],
    expected_len: usize,
    context: &'static str,
) -> Result<Vec<Option<T>>> {
    let mut assignments = Vec::new();
    flatten_assignment_array(value, handles, &mut assignments, context)?;
    if assignments.len() < expected_len {
        assignments.resize(expected_len, None);
    }
    Ok(assignments)
}

fn flatten_assignment_array<T: Copy>(
    value: &OwnedJsonValue,
    handles: &[T],
    out: &mut Vec<Option<T>>,
    context: &'static str,
) -> Result<()> {
    match value {
        OwnedJsonValue::Null | OwnedJsonValue::Number(_) => {
            out.push(parse_optional_handle_index(value, handles, context)?);
            Ok(())
        }
        OwnedJsonValue::Array(values) => {
            for child in values {
                flatten_assignment_array(child, handles, out, context)?;
            }
            Ok(())
        }
        _ => Err(Error::InvalidValue(format!(
            "{context} must contain only nested arrays of integers or nulls"
        ))),
    }
}

fn parse_optional_handle_index<T: Copy>(
    value: &OwnedJsonValue,
    handles: &[T],
    context: &'static str,
) -> Result<Option<T>> {
    match value {
        OwnedJsonValue::Null => Ok(None),
        OwnedJsonValue::Number(number) => {
            let index = number.as_u64().ok_or_else(|| {
                Error::InvalidValue(format!("{context} index must be an unsigned integer"))
            })? as usize;
            Ok(handles.get(index).copied())
        }
        _ => Err(Error::InvalidValue(format!(
            "{context} must contain integers or nulls"
        ))),
    }
}

fn reject_borrowed_geometry_instance(
    _lod: Option<&str>,
    _template: Option<u32>,
    _boundaries: Option<&BorrowedJsonValue<'_>>,
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
