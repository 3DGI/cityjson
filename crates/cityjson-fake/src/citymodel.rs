//! City model assembly helpers.
//!
//! ```rust
//! use cityjson_fake::cli::CJFakeConfig;
//! use cityjson_fake::citymodel::CityModelBuilder;
//! use cityjson_fake::prelude::*;
//!
//! let model = CityModelBuilder::<u32, OwnedStringStorage>::new(CJFakeConfig::default(), Some(6))
//!     .metadata(None)
//!     .vertices()
//!     .materials(None)
//!     .textures(None)
//!     .attributes(None)
//!     .cityobjects()
//!     .build();
//! assert_eq!(model.cityobjects().len(), 1);
//! ```

use crate::attribute::AttributesFaker;
use crate::cli::CJFakeConfig;
use crate::material::MaterialBuilder;
use crate::metadata::MetadataBuilder;
use crate::texture::TextureBuilder;
#[allow(unused_imports)]
use crate::vertex::VerticesFaker;
use crate::{get_nr_items, CityObjectLevel, CityObjectTypeFaker, LoDFaker, SemanticCtx};
use cityjson::prelude::*;
use cityjson::v2_0::{
    AffineTransform3D, BBox, CityModel, CityObject, CityObjectIdentifier, CityObjectType,
    GeometryDraft, GeometryType, ImageType, LineStringDraft, LoD, Material, OwnedAttributeValue,
    OwnedAttributes, OwnedSemantic, PointDraft, RealWorldCoordinate, RingDraft, Semantic,
    SemanticType, ShellDraft, SolidDraft, SurfaceDraft, Texture, ThemeName, UVCoordinate, UvDraft,
    VertexDraft, VertexIndex, VertexRef, RGB,
};
#[cfg(feature = "json")]
use cityjson_json::{self, WriteOptions};
use fake::Fake;
use fake::RngExt;
use rand::prelude::SmallRng;
use rand::seq::{IndexedRandom, SliceRandom};
use rand::SeedableRng;

// ─── Internal helpers (all specialised to OwnedStringStorage) ───────────────

/// Obtain a semantic handle for `city_obj_type`, inserting into the model if needed.
fn make_semantic_handle<VR: VertexRef>(
    city_obj_type: &CityObjectType<OwnedStringStorage>,
    rng: &mut SmallRng,
    model: &mut CityModel<VR, OwnedStringStorage>,
    sem_ctx: &SemanticCtx<'_>,
) -> Option<SemanticHandle>
where
    OwnedSemantic: PartialEq,
{
    use crate::SemanticTypeFaker;
    use fake::Dummy;

    if !sem_ctx.enabled {
        return None;
    }

    let faker = SemanticTypeFaker {
        city_obj_type: city_obj_type.clone(),
        allowed_types: sem_ctx.allowed_types,
    };
    let st: Option<SemanticType<OwnedStringStorage>> =
        <Option<SemanticType<OwnedStringStorage>> as Dummy<SemanticTypeFaker>>::dummy_with_rng(
            &faker, rng,
        );
    st.and_then(|s| model.get_or_insert_semantic(Semantic::new(s)).ok())
}

/// Build one ring using freshly-created vertex coordinates (no pre-generated pool needed).
fn make_ring<VR: VertexRef>(
    num_vertices: usize,
    min_coord: f64,
    max_coord: f64,
    rng: &mut SmallRng,
    texture_info: Option<(String, TextureHandle)>,
) -> RingDraft<VR, OwnedStringStorage> {
    let verts: Vec<VertexDraft<VR>> = (0..num_vertices)
        .map(|_| {
            VertexDraft::New(RealWorldCoordinate::new(
                rng.random_range(min_coord..=max_coord),
                rng.random_range(min_coord..=max_coord),
                rng.random_range(min_coord..=max_coord),
            ))
        })
        .collect();
    let mut ring_draft = RingDraft::new(verts);
    if let Some((theme, tex)) = texture_info {
        let uvs: Vec<UvDraft<VR>> = (0..num_vertices)
            .map(|_| {
                UvDraft::New(UVCoordinate::new(
                    rng.random_range(0.0f32..=1.0f32),
                    rng.random_range(0.0f32..=1.0f32),
                ))
            })
            .collect();
        ring_draft = ring_draft.with_texture(ThemeName::new(theme), tex, uvs);
    }
    ring_draft
}

struct AppearanceCtx<'a> {
    mat_themes: &'a [String],
    mat_handles: &'a [MaterialHandle],
    tex_themes: &'a [String],
    tex_handles: &'a [TextureHandle],
    max_vertices_texture: usize,
    texture_allow_none: bool,
}

impl AppearanceCtx<'_> {
    fn pick_material(&self, rng: &mut SmallRng) -> Option<(String, MaterialHandle)> {
        if self.mat_themes.is_empty() || self.mat_handles.is_empty() || !rng.random_bool(0.7) {
            return None;
        }
        Some((
            self.mat_themes.choose(rng)?.clone(),
            *self.mat_handles.choose(rng)?,
        ))
    }

    fn pick_texture(
        &self,
        rng: &mut SmallRng,
        num_vertices: usize,
        used_vertices: usize,
    ) -> Option<(String, TextureHandle)> {
        if self.tex_themes.is_empty()
            || self.tex_handles.is_empty()
            || used_vertices.saturating_add(num_vertices) > self.max_vertices_texture
        {
            return None;
        }
        if self.texture_allow_none && !rng.random_bool(0.5) {
            return None;
        }
        Some((
            self.tex_themes.choose(rng)?.clone(),
            *self.tex_handles.choose(rng)?,
        ))
    }
}

#[derive(Clone, Copy)]
struct CoordRange {
    min_coord: f64,
    max_coord: f64,
}

struct GeometryCtx<'a> {
    config: &'a CJFakeConfig,
    coord_range: CoordRange,
    city_obj_type: &'a CityObjectType<OwnedStringStorage>,
    app: &'a AppearanceCtx<'a>,
    sem_ctx: &'a SemanticCtx<'a>,
}

fn make_surfaces<VR: VertexRef>(
    count: usize,
    ctx: &GeometryCtx<'_>,
    rng: &mut SmallRng,
    model: &mut CityModel<VR, OwnedStringStorage>,
    used_material_themes: &mut Vec<String>,
    used_texture_themes: &mut Vec<String>,
) -> Vec<SurfaceDraft<VR, OwnedStringStorage>>
where
    OwnedSemantic: PartialEq,
{
    let mut texture_vertices_used = 0usize;
    let mut material_seeded = !used_material_themes.is_empty();
    let mut texture_seeded = !used_texture_themes.is_empty();
    (0..count)
        .map(|_| {
            let sem = make_semantic_handle(ctx.city_obj_type, rng, model, ctx.sem_ctx);
            let mut mat = ctx.app.pick_material(rng);
            if mat.is_none()
                && !material_seeded
                && !ctx.app.mat_themes.is_empty()
                && !ctx.app.mat_handles.is_empty()
            {
                mat = Some((ctx.app.mat_themes[0].clone(), ctx.app.mat_handles[0]));
            }
            if mat.is_some() {
                material_seeded = true;
            }
            if let Some((theme, _)) = &mat {
                if !used_material_themes.contains(theme) {
                    used_material_themes.push(theme.clone());
                }
            }
            let n = rng.random_range(3..=8usize);
            let mut tex = ctx.app.pick_texture(rng, n, texture_vertices_used);
            if tex.is_none()
                && !texture_seeded
                && !ctx.app.tex_themes.is_empty()
                && !ctx.app.tex_handles.is_empty()
                && texture_vertices_used.saturating_add(n) <= ctx.app.max_vertices_texture
            {
                tex = Some((ctx.app.tex_themes[0].clone(), ctx.app.tex_handles[0]));
            }
            if tex.is_some() {
                texture_seeded = true;
            }
            if let Some((theme, _)) = &tex {
                if !used_texture_themes.contains(theme) {
                    used_texture_themes.push(theme.clone());
                }
            }
            if tex.is_some() {
                texture_vertices_used += n;
            }
            let ring_draft = make_ring::<VR>(
                n,
                ctx.coord_range.min_coord,
                ctx.coord_range.max_coord,
                rng,
                tex,
            );
            let mut surface = SurfaceDraft::new(ring_draft, []);
            if let Some(h) = sem {
                surface = surface.with_semantic(h);
            }
            if let Some((theme, mat)) = mat {
                surface = surface.with_material(ThemeName::new(theme), mat);
            }
            surface
        })
        .collect()
}

// ─── Per-geometry-type generators ────────────────────────────────────────────

fn gen_multisurface<VR: VertexRef>(
    ctx: &GeometryCtx<'_>,
    lod: LoD,
    rng: &mut SmallRng,
    model: &mut CityModel<VR, OwnedStringStorage>,
    used_material_themes: &mut Vec<String>,
    used_texture_themes: &mut Vec<String>,
) -> Result<GeometryHandle>
where
    OwnedSemantic: PartialEq,
{
    let n = get_nr_items(
        ctx.config.geometry.min_members_multisurface..=ctx.config.geometry.max_members_multisurface,
        rng,
    );
    let surfaces = make_surfaces(
        n,
        ctx,
        rng,
        model,
        used_material_themes,
        used_texture_themes,
    );
    GeometryDraft::multi_surface(Some(lod), surfaces).insert_into(model)
}

fn gen_multipoint<VR: VertexRef>(
    ctx: &GeometryCtx<'_>,
    lod: LoD,
    rng: &mut SmallRng,
    model: &mut CityModel<VR, OwnedStringStorage>,
    _used_material_themes: &mut Vec<String>,
    _used_texture_themes: &mut Vec<String>,
) -> Result<GeometryHandle>
where
    OwnedSemantic: PartialEq,
{
    let n = get_nr_items(
        ctx.config.geometry.min_members_multipoint..=ctx.config.geometry.max_members_multipoint,
        rng,
    );
    let points: Vec<PointDraft<VR>> = (0..n)
        .map(|_| {
            let vertex = VertexDraft::New(RealWorldCoordinate::new(
                rng.random_range(ctx.coord_range.min_coord..=ctx.coord_range.max_coord),
                rng.random_range(ctx.coord_range.min_coord..=ctx.coord_range.max_coord),
                rng.random_range(ctx.coord_range.min_coord..=ctx.coord_range.max_coord),
            ));
            let sem = make_semantic_handle(ctx.city_obj_type, rng, model, ctx.sem_ctx);
            let mut p = PointDraft::new(vertex);
            if let Some(h) = sem {
                p = p.with_semantic(h);
            }
            p
        })
        .collect();
    GeometryDraft::multi_point(Some(lod), points).insert_into(model)
}

fn gen_multilinestring<VR: VertexRef>(
    ctx: &GeometryCtx<'_>,
    lod: LoD,
    rng: &mut SmallRng,
    model: &mut CityModel<VR, OwnedStringStorage>,
    _used_material_themes: &mut Vec<String>,
    _used_texture_themes: &mut Vec<String>,
) -> Result<GeometryHandle>
where
    OwnedSemantic: PartialEq,
{
    let n_ls = get_nr_items(
        ctx.config.geometry.min_members_multilinestring
            ..=ctx.config.geometry.max_members_multilinestring,
        rng,
    );
    let linestrings: Vec<LineStringDraft<VR>> = (0..n_ls)
        .map(|_| {
            let n_v = rng.random_range(2..=8usize);
            let verts: Vec<VertexDraft<VR>> = (0..n_v)
                .map(|_| {
                    VertexDraft::New(RealWorldCoordinate::new(
                        rng.random_range(ctx.coord_range.min_coord..=ctx.coord_range.max_coord),
                        rng.random_range(ctx.coord_range.min_coord..=ctx.coord_range.max_coord),
                        rng.random_range(ctx.coord_range.min_coord..=ctx.coord_range.max_coord),
                    ))
                })
                .collect();
            let sem = make_semantic_handle(ctx.city_obj_type, rng, model, ctx.sem_ctx);
            let mut ls = LineStringDraft::new(verts);
            if let Some(h) = sem {
                ls = ls.with_semantic(h);
            }
            ls
        })
        .collect();
    GeometryDraft::multi_line_string(Some(lod), linestrings).insert_into(model)
}

fn gen_composite_surface<VR: VertexRef>(
    ctx: &GeometryCtx<'_>,
    lod: LoD,
    rng: &mut SmallRng,
    model: &mut CityModel<VR, OwnedStringStorage>,
    used_material_themes: &mut Vec<String>,
    used_texture_themes: &mut Vec<String>,
) -> Result<GeometryHandle>
where
    OwnedSemantic: PartialEq,
{
    let n = get_nr_items(
        ctx.config.geometry.min_members_compositesurface
            ..=ctx.config.geometry.max_members_compositesurface,
        rng,
    );
    let surfaces = make_surfaces(
        n,
        ctx,
        rng,
        model,
        used_material_themes,
        used_texture_themes,
    );
    GeometryDraft::composite_surface(Some(lod), surfaces).insert_into(model)
}

fn gen_composite_solid<VR: VertexRef>(
    ctx: &GeometryCtx<'_>,
    lod: LoD,
    rng: &mut SmallRng,
    model: &mut CityModel<VR, OwnedStringStorage>,
    used_material_themes: &mut Vec<String>,
    used_texture_themes: &mut Vec<String>,
) -> Result<GeometryHandle>
where
    OwnedSemantic: PartialEq,
{
    let n_solids = get_nr_items(
        ctx.config.geometry.min_members_compositesolid
            ..=ctx.config.geometry.max_members_compositesolid,
        rng,
    );
    let solids: Vec<SolidDraft<VR, OwnedStringStorage>> = (0..n_solids)
        .map(|_| {
            let n = get_nr_items(
                ctx.config.geometry.min_members_compositesurface
                    ..=ctx.config.geometry.max_members_compositesurface,
                rng,
            );
            let surfaces = make_surfaces(
                n,
                ctx,
                rng,
                model,
                used_material_themes,
                used_texture_themes,
            );
            SolidDraft::new(ShellDraft::new(surfaces), [])
        })
        .collect();
    GeometryDraft::composite_solid(Some(lod), solids).insert_into(model)
}

fn gen_solid<VR: VertexRef>(
    ctx: &GeometryCtx<'_>,
    lod: LoD,
    rng: &mut SmallRng,
    model: &mut CityModel<VR, OwnedStringStorage>,
    used_material_themes: &mut Vec<String>,
    used_texture_themes: &mut Vec<String>,
) -> Result<GeometryHandle>
where
    OwnedSemantic: PartialEq,
{
    let n = get_nr_items(
        ctx.config.geometry.min_members_solid..=ctx.config.geometry.max_members_solid,
        rng,
    );
    let surfaces = make_surfaces(
        n,
        ctx,
        rng,
        model,
        used_material_themes,
        used_texture_themes,
    );
    let outer = ShellDraft::new(surfaces);
    GeometryDraft::solid(Some(lod), outer, []).insert_into(model)
}

fn gen_multisolid<VR: VertexRef>(
    ctx: &GeometryCtx<'_>,
    lod: LoD,
    rng: &mut SmallRng,
    model: &mut CityModel<VR, OwnedStringStorage>,
    used_material_themes: &mut Vec<String>,
    used_texture_themes: &mut Vec<String>,
) -> Result<GeometryHandle>
where
    OwnedSemantic: PartialEq,
{
    let n_solids = get_nr_items(
        ctx.config.geometry.min_members_multisolid..=ctx.config.geometry.max_members_multisolid,
        rng,
    );
    let solids: Vec<SolidDraft<VR, OwnedStringStorage>> = (0..n_solids)
        .map(|_| {
            let n = get_nr_items(
                ctx.config.geometry.min_members_solid..=ctx.config.geometry.max_members_solid,
                rng,
            );
            let surfaces = make_surfaces(
                n,
                ctx,
                rng,
                model,
                used_material_themes,
                used_texture_themes,
            );
            SolidDraft::new(ShellDraft::new(surfaces), [])
        })
        .collect();
    GeometryDraft::multi_solid(Some(lod), solids).insert_into(model)
}

// ─── Schema-based geometry type allowlists ───────────────────────────────────

// Building / Bridge / Tunnel main types + their sub-objects that don't allow
// multi-point, multi-linestring, or multi-solid.
const GEOM_BUILDING_MAIN: &[GeometryType] = &[
    GeometryType::MultiSurface,
    GeometryType::CompositeSurface,
    GeometryType::Solid,
    GeometryType::CompositeSolid,
];

// Surface only (LandUse)
const GEOM_SURFACE_ONLY: &[GeometryType] =
    &[GeometryType::MultiSurface, GeometryType::CompositeSurface];

// TINRelief: schema allows CompositeSurface only
const GEOM_TIN: &[GeometryType] = &[GeometryType::CompositeSurface];

// PlantCover: surface + solid types (no point/line)
const GEOM_PLANT_COVER: &[GeometryType] = &[
    GeometryType::MultiSurface,
    GeometryType::CompositeSurface,
    GeometryType::Solid,
    GeometryType::MultiSolid,
    GeometryType::CompositeSolid,
];

// Transportation: line + surface (no point, no solid)
const GEOM_TRANSPORT: &[GeometryType] = &[
    GeometryType::MultiLineString,
    GeometryType::MultiSurface,
    GeometryType::CompositeSurface,
];

// WaterBody: line + surface + solid (no point, no multi-solid)
const GEOM_WATER_BODY: &[GeometryType] = &[
    GeometryType::MultiLineString,
    GeometryType::MultiSurface,
    GeometryType::CompositeSurface,
    GeometryType::Solid,
    GeometryType::CompositeSolid,
];

// All 7 standard geometry types (installation/furniture/etc. types)
const GEOM_ALL: &[GeometryType] = &[
    GeometryType::MultiPoint,
    GeometryType::MultiLineString,
    GeometryType::MultiSurface,
    GeometryType::CompositeSurface,
    GeometryType::Solid,
    GeometryType::MultiSolid,
    GeometryType::CompositeSolid,
];

// CityObjectGroup can reference all standard geometries except GeometryInstance.
const GEOM_GROUP: &[GeometryType] = &[
    GeometryType::MultiPoint,
    GeometryType::MultiLineString,
    GeometryType::MultiSurface,
    GeometryType::CompositeSurface,
    GeometryType::Solid,
    GeometryType::MultiSolid,
    GeometryType::CompositeSolid,
];

/// Returns true if this city object type allows `GeometryInstance` geometry.
fn allows_geometry_instance(city_obj_type: &CityObjectType<OwnedStringStorage>) -> bool {
    matches!(
        city_obj_type,
        CityObjectType::BuildingInstallation
            | CityObjectType::BuildingConstructiveElement
            | CityObjectType::BuildingFurniture
            | CityObjectType::BridgeInstallation
            | CityObjectType::BridgeConstructiveElement
            | CityObjectType::BridgeFurniture
            | CityObjectType::TunnelInstallation
            | CityObjectType::TunnelConstructiveElement
            | CityObjectType::TunnelFurniture
            | CityObjectType::SolitaryVegetationObject
            | CityObjectType::CityFurniture
            | CityObjectType::OtherConstruction
    )
}

/// Returns the geometry types allowed by the `CityJSON` schema for the given city object type.
fn schema_geom_for(city_obj_type: &CityObjectType<OwnedStringStorage>) -> &'static [GeometryType] {
    match city_obj_type {
        CityObjectType::Building
        | CityObjectType::BuildingPart
        | CityObjectType::BuildingRoom
        | CityObjectType::BuildingUnit
        | CityObjectType::BuildingStorey
        | CityObjectType::Bridge
        | CityObjectType::BridgePart
        | CityObjectType::BridgeRoom
        | CityObjectType::Tunnel
        | CityObjectType::TunnelPart
        | CityObjectType::TunnelHollowSpace => GEOM_BUILDING_MAIN,

        CityObjectType::LandUse => GEOM_SURFACE_ONLY,
        CityObjectType::TINRelief => GEOM_TIN,
        CityObjectType::PlantCover => GEOM_PLANT_COVER,

        CityObjectType::Road
        | CityObjectType::Railway
        | CityObjectType::Waterway
        | CityObjectType::TransportSquare => GEOM_TRANSPORT,

        CityObjectType::WaterBody => GEOM_WATER_BODY,
        CityObjectType::CityObjectGroup => GEOM_GROUP,

        _ => GEOM_ALL,
    }
}

/// First-level city object types — valid as top-level objects without a `parents` field.
const FIRST_LEVEL_TYPES: &[CityObjectType<OwnedStringStorage>] = &[
    CityObjectType::Bridge,
    CityObjectType::Building,
    CityObjectType::CityFurniture,
    CityObjectType::CityObjectGroup,
    CityObjectType::GenericCityObject,
    CityObjectType::LandUse,
    CityObjectType::OtherConstruction,
    CityObjectType::PlantCover,
    CityObjectType::SolitaryVegetationObject,
    CityObjectType::TINRelief,
    CityObjectType::TransportSquare,
    CityObjectType::Railway,
    CityObjectType::Road,
    CityObjectType::Tunnel,
    CityObjectType::WaterBody,
    CityObjectType::Waterway,
];

/// Returns `true` if the type is a first-level city object (valid without a `parents` field).
fn is_first_level_type(t: &CityObjectType<OwnedStringStorage>) -> bool {
    FIRST_LEVEL_TYPES.contains(t)
}

/// Returns all first-level city object types that support at least one geometry from `allowed`.
fn compatible_first_level_types(
    allowed: &[GeometryType],
) -> Vec<CityObjectType<OwnedStringStorage>> {
    FIRST_LEVEL_TYPES
        .iter()
        .filter(|t| schema_geom_for(t).iter().any(|g| allowed.contains(g)))
        .cloned()
        .collect()
}

/// Pick a geometry type compatible with both the `CityObject` schema and the config allowlist.
fn pick_geometry_type(
    config: &CJFakeConfig,
    city_obj_type: &CityObjectType<OwnedStringStorage>,
    rng: &mut SmallRng,
) -> GeometryType {
    let schema_allowed = schema_geom_for(city_obj_type);
    let candidates: Vec<GeometryType> = schema_allowed
        .iter()
        .copied()
        .filter(|gt| {
            config
                .geometry
                .allowed_types_geometry
                .as_ref()
                .is_none_or(|allowed| allowed.contains(gt))
        })
        .collect();

    // Fallback: if intersection is empty, use schema-allowed types only
    let pool = if candidates.is_empty() {
        schema_allowed.to_vec()
    } else {
        candidates
    };

    pool.choose(rng)
        .copied()
        .unwrap_or(GeometryType::MultiSurface)
}

/// Dispatch to the appropriate generator.
fn generate_geometry<VR: VertexRef>(
    ctx: &GeometryCtx<'_>,
    geom_type: GeometryType,
    lod: LoD,
    rng: &mut SmallRng,
    model: &mut CityModel<VR, OwnedStringStorage>,
    used_material_themes: &mut Vec<String>,
    used_texture_themes: &mut Vec<String>,
) -> Result<GeometryHandle>
where
    OwnedSemantic: PartialEq,
{
    match geom_type {
        GeometryType::MultiPoint => gen_multipoint(
            ctx,
            lod,
            rng,
            model,
            used_material_themes,
            used_texture_themes,
        ),
        GeometryType::MultiLineString => gen_multilinestring(
            ctx,
            lod,
            rng,
            model,
            used_material_themes,
            used_texture_themes,
        ),
        GeometryType::Solid => gen_solid(
            ctx,
            lod,
            rng,
            model,
            used_material_themes,
            used_texture_themes,
        ),
        GeometryType::CompositeSolid => gen_composite_solid(
            ctx,
            lod,
            rng,
            model,
            used_material_themes,
            used_texture_themes,
        ),
        GeometryType::MultiSolid => gen_multisolid(
            ctx,
            lod,
            rng,
            model,
            used_material_themes,
            used_texture_themes,
        ),
        GeometryType::CompositeSurface => gen_composite_surface(
            ctx,
            lod,
            rng,
            model,
            used_material_themes,
            used_texture_themes,
        ),
        _ => gen_multisurface(
            ctx,
            lod,
            rng,
            model,
            used_material_themes,
            used_texture_themes,
        ),
    }
}

/// Compute a bounding box from a contiguous slice of the model's vertex array.
fn bbox_from_vertex_range<VR: VertexRef, SS: StringStorage>(
    start: usize,
    end: usize,
    model: &CityModel<VR, SS>,
) -> Option<BBox> {
    if start >= end {
        return None;
    }
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut min_z = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut max_z = f64::MIN;
    for i in start..end {
        if let Some(c) = model.get_vertex(VertexIndex::new(VR::from_usize(i).unwrap())) {
            min_x = min_x.min(c.x());
            min_y = min_y.min(c.y());
            min_z = min_z.min(c.z());
            max_x = max_x.max(c.x());
            max_y = max_y.max(c.y());
            max_z = max_z.max(c.z());
        }
    }
    (min_x <= max_x).then(|| BBox::new(min_x, min_y, min_z, max_x, max_y, max_z))
}

// ─── Builder ─────────────────────────────────────────────────────────────────

/// Builder for creating `CityJSON` models with fake data.
///
/// The builder provides methods to configure and generate different aspects of a `CityJSON` model,
/// such as vertices, cityobjects, materials, textures, etc. The generated data is valid according
/// to the `CityJSON` specification, though the geometric values are random.
///
/// # Examples
///
/// ```rust
/// use cityjson_fake::prelude::*;
///
/// // Create a basic CityJSON model with default settings
/// let model: CityModel<u32, OwnedStringStorage> = CityModelBuilder::default().build();
///
/// // Create a customized model
/// let config = CJFakeConfig::default();
/// let model: CityModel<u32, OwnedStringStorage> = CityModelBuilder::new(config, None)
///     .metadata(None)
///     .vertices()
///     .materials(None)
///     .textures(None)
///     .attributes(None)
///     .cityobjects()
///     .build();
/// ```
/// Builder for assembling a complete `CityModel`.
///
/// # Examples
///
/// ```rust
/// use cityjson_fake::citymodel::CityModelBuilder;
/// use cityjson_fake::cli::CJFakeConfig;
/// use cityjson_fake::prelude::*;
///
/// let model = CityModelBuilder::<u32, OwnedStringStorage>::new(CJFakeConfig::default(), Some(6))
///     .metadata(None)
///     .vertices()
///     .materials(None)
///     .textures(None)
///     .attributes(None)
///     .cityobjects()
///     .build();
/// assert_eq!(model.cityobjects().len(), 1);
/// ```
pub struct CityModelBuilder<VR: VertexRef, SS: StringStorage> {
    model: CityModel<VR, SS>,
    rng: SmallRng,
    config: CJFakeConfig,

    themes_material: Vec<String>,
    themes_texture: Vec<String>,
    material_handles: Vec<MaterialHandle>,
    texture_handles: Vec<TextureHandle>,
    used_material_themes: Vec<String>,
    used_texture_themes: Vec<String>,
    /// Generated `CityObject` attributes (`OwnedStringStorage`; applied at object creation).
    attributes_cityobject: Option<OwnedAttributes>,

    progress_done_metadata: bool,
    progress_done_transform: bool,
    progress_done_vertices: bool,
}

impl<VR: VertexRef, SS: StringStorage<String = String>> From<CityModelBuilder<VR, SS>>
    for CityModel<VR, SS>
{
    fn from(val: CityModelBuilder<VR, SS>) -> Self {
        val.build()
    }
}

impl<VR: VertexRef> Default for CityModelBuilder<VR, OwnedStringStorage> {
    fn default() -> Self {
        CityModelBuilder::new(CJFakeConfig::default(), None)
            .metadata(None)
            .vertices()
            .materials(None)
            .textures(None)
            .attributes(None)
            .cityobjects()
    }
}

// ─── Generic methods (work for any SS: StringStorage<String = String>) ────────

impl<VR: VertexRef, SS: StringStorage<String = String>> CityModelBuilder<VR, SS> {
    /// Creates a new `CityModelBuilder` with the given configuration and optional random seed.
    #[must_use]
    pub fn new(config: CJFakeConfig, seed: Option<u64>) -> Self {
        let rng = if let Some(s) = seed {
            SmallRng::seed_from_u64(s)
        } else {
            SmallRng::from_rng(&mut rand::rng())
        };
        Self {
            model: CityModel::new(CityModelType::CityJSON),
            rng,
            config,
            themes_material: Vec::new(),
            themes_texture: Vec::new(),
            material_handles: Vec::new(),
            texture_handles: Vec::new(),
            used_material_themes: Vec::new(),
            used_texture_themes: Vec::new(),
            attributes_cityobject: None,
            progress_done_metadata: false,
            progress_done_transform: false,
            progress_done_vertices: false,
        }
    }

    /// Adds metadata to the model.
    #[must_use]
    pub fn metadata(mut self, _metadata_builder: Option<MetadataBuilder<SS>>) -> Self {
        if !self.progress_done_metadata {
            if self.config.metadata.metadata_enabled {
                let mc = &self.config.metadata;
                let mut builder = MetadataBuilder::new(&self.config, &mut self.rng);
                if mc.metadata_geographical_extent {
                    builder = builder.geographical_extent();
                }
                if mc.metadata_identifier {
                    builder = builder.identifier();
                }
                if mc.metadata_reference_date {
                    builder = builder.reference_date();
                }
                if mc.metadata_reference_system {
                    builder = builder.reference_system();
                }
                if mc.metadata_title {
                    builder = builder.title();
                }
                if mc.metadata_point_of_contact {
                    builder = builder.point_of_contact();
                }
                *self.model.metadata_mut() = builder.build();
            }
            self.progress_done_metadata = true;
        }
        self
    }

    /// Adds materials to the model.
    #[must_use]
    pub fn materials(
        mut self,
        _material_builder: Option<MaterialBuilder<OwnedStringStorage>>,
    ) -> Self {
        use fake::faker::lorem::raw::Word;
        use fake::locales::EN;
        use fake::Fake;

        if !self.config.materials.materials_enabled {
            return self;
        }

        let mc = self.config.materials.clone();
        let nr_materials = get_nr_items(mc.min_materials..=mc.max_materials, &mut self.rng);
        let nr_themes = get_nr_items(1..=mc.nr_themes_materials, &mut self.rng);

        let themes: Vec<String> = (0..nr_themes)
            .map(|_| Word(EN).fake_with_rng(&mut self.rng))
            .collect();
        self.themes_material.clone_from(&themes);

        for _ in 0..nr_materials {
            let name: String = Word(EN).fake_with_rng(&mut self.rng);
            let mut material = Material::new(name);
            if mc
                .generate_ambient_intensity
                .unwrap_or_else(|| self.rng.random_bool(0.5))
            {
                material.set_ambient_intensity(Some(self.rng.random_range(0.0..=1.0)));
            }
            if mc
                .generate_diffuse_color
                .unwrap_or_else(|| self.rng.random_bool(0.5))
            {
                material.set_diffuse_color(Some(RGB::new(
                    self.rng.random_range(0.0..=1.0),
                    self.rng.random_range(0.0..=1.0),
                    self.rng.random_range(0.0..=1.0),
                )));
            }
            if mc
                .generate_emissive_color
                .unwrap_or_else(|| self.rng.random_bool(0.5))
            {
                material.set_emissive_color(Some(RGB::new(
                    self.rng.random_range(0.0..=1.0),
                    self.rng.random_range(0.0..=1.0),
                    self.rng.random_range(0.0..=1.0),
                )));
            }
            if mc
                .generate_specular_color
                .unwrap_or_else(|| self.rng.random_bool(0.5))
            {
                material.set_specular_color(Some(RGB::new(
                    self.rng.random_range(0.0..=1.0),
                    self.rng.random_range(0.0..=1.0),
                    self.rng.random_range(0.0..=1.0),
                )));
            }
            if mc
                .generate_shininess
                .unwrap_or_else(|| self.rng.random_bool(0.5))
            {
                material.set_shininess(Some(self.rng.random_range(0.0..=1.0)));
            }
            if mc
                .generate_transparency
                .unwrap_or_else(|| self.rng.random_bool(0.5))
            {
                material.set_transparency(Some(self.rng.random_range(0.0..=1.0)));
            }
            if let Ok(h) = self.model.add_material(material) {
                self.material_handles.push(h);
            }
        }

        self
    }

    /// Adds textures to the model.
    #[must_use]
    pub fn textures(mut self, _texture_builder: Option<TextureBuilder>) -> Self {
        use fake::faker::filesystem::raw::FilePath;
        use fake::faker::lorem::raw::Word;
        use fake::locales::EN;
        use fake::Fake;

        if !self.config.textures.textures_enabled {
            return self;
        }

        let tc = self.config.textures.clone();
        let nr_textures = get_nr_items(tc.min_textures..=tc.max_textures, &mut self.rng);
        let nr_themes = get_nr_items(1..=tc.nr_themes_textures, &mut self.rng);

        let themes: Vec<String> = (0..nr_themes)
            .map(|_| Word(EN).fake_with_rng(&mut self.rng))
            .collect();
        self.themes_texture.clone_from(&themes);

        for _ in 0..nr_textures {
            let path: String = FilePath(EN).fake_with_rng(&mut self.rng);
            let image_type = if self.rng.random_bool(0.5) {
                ImageType::Png
            } else {
                ImageType::Jpg
            };
            if let Ok(h) = self.model.add_texture(Texture::new(path, image_type)) {
                self.texture_handles.push(h);
            }
        }

        self
    }

    /// Adds the transform member to the `CityModel`.
    #[must_use]
    pub fn transform(mut self) -> Self {
        if !self.progress_done_transform {
            let _ = self.model.transform_mut();
            self.progress_done_transform = true;
        }
        self
    }

    /// No-op kept for API compatibility. Vertices are generated on-demand by `cityobjects()`.
    ///
    /// The required `transform` property is initialized here so it appears in the serialized output.
    #[must_use]
    pub fn vertices(mut self) -> Self {
        self.progress_done_vertices = true;
        // `transform` is required in CityJSON — ensure it is always present
        if !self.progress_done_transform {
            let _ = self.model.transform_mut();
            self.progress_done_transform = true;
        }
        self
    }

    /// Builds the final `CityJSON` model.
    #[must_use]
    pub fn build(self) -> CityModel<VR, SS> {
        self.model
    }
}

#[cfg(feature = "json")]
impl CityModelBuilder<u32, OwnedStringStorage> {
    /// Serializes the built model to a `CityJSON` string.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn build_string(self) -> cityjson_json::Result<String> {
        let bytes = self.build_vec()?;
        String::from_utf8(bytes).map_err(|error| cityjson_json::Error::Utf8(error.utf8_error()))
    }

    /// Serializes the built model to a UTF-8 encoded `CityJSON` byte vector.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn build_vec(self) -> cityjson_json::Result<Vec<u8>> {
        let model = self.build();
        let options = WriteOptions::default();
        cityjson_json::to_vec(&model, &options)
    }
}

// ─── OwnedStringStorage-specific methods ─────────────────────────────────────

impl<VR: VertexRef> CityModelBuilder<VR, OwnedStringStorage> {
    /// Adds one or more geometries to a single city object.
    #[allow(clippy::too_many_arguments)]
    fn add_geometries_for_cityobject(
        &mut self,
        city_obj_type: &CityObjectType<OwnedStringStorage>,
        min_coord: f64,
        max_coord: f64,
        template_handles: &[GeometryTemplateHandle],
        app: &AppearanceCtx<'_>,
        sem_ctx: &SemanticCtx<'_>,
        cityobject: &mut CityObject<OwnedStringStorage>,
    ) where
        OwnedSemantic: PartialEq,
    {
        let vtx_start = self.model.vertices().len();
        let geometry_ctx = GeometryCtx {
            config: &self.config,
            coord_range: CoordRange {
                min_coord,
                max_coord,
            },
            city_obj_type,
            app,
            sem_ctx,
        };
        let nr_geometries = get_nr_items(
            self.config.geometry.min_members_cityobject_geometries
                ..=self.config.geometry.max_members_cityobject_geometries,
            &mut self.rng,
        );

        for _ in 0..nr_geometries {
            let geom_result = if self.config.templates.use_templates
                && !template_handles.is_empty()
                && allows_geometry_instance(city_obj_type)
            {
                let tpl = *template_handles.choose(&mut self.rng).unwrap();
                let ref_point = RealWorldCoordinate::new(
                    self.rng.random_range(min_coord..=max_coord),
                    self.rng.random_range(min_coord..=max_coord),
                    self.rng.random_range(min_coord..=max_coord),
                );
                GeometryDraft::<VR, OwnedStringStorage>::instance(
                    tpl,
                    ref_point,
                    AffineTransform3D::identity(),
                )
                .insert_into(&mut self.model)
            } else {
                let lod: LoD = LoDFaker {
                    allowed: self.config.geometry.allowed_lods.as_deref(),
                }
                .fake_with_rng(&mut self.rng);
                let gt = pick_geometry_type(&self.config, city_obj_type, &mut self.rng);
                generate_geometry::<VR>(
                    &geometry_ctx,
                    gt,
                    lod,
                    &mut self.rng,
                    &mut self.model,
                    &mut self.used_material_themes,
                    &mut self.used_texture_themes,
                )
            };

            if let Ok(h) = geom_result {
                cityobject.add_geometry(h);
            }
        }

        let vtx_end = self.model.vertices().len();
        if let Some(bbox) = bbox_from_vertex_range(vtx_start, vtx_end, &self.model) {
            cityobject.set_geographical_extent(Some(bbox));
        }
    }

    /// Populate `CityObjectGroup` members and `children_roles` after all city objects exist.
    fn wire_cityobject_groups(&mut self, group_handles: &[CityObjectHandle]) {
        use fake::faker::lorem::raw::Word;
        use fake::locales::EN;
        use fake::Fake;

        let all_handles: Vec<CityObjectHandle> = self.model.cityobjects().ids().collect();

        for &group_handle in group_handles {
            let group_type = self
                .model
                .cityobjects()
                .get(group_handle)
                .map(|group| group.type_cityobject().clone());
            if group_type != Some(CityObjectType::CityObjectGroup) {
                continue;
            }

            let mut candidates: Vec<CityObjectHandle> = all_handles
                .iter()
                .copied()
                .filter(|handle| *handle != group_handle)
                .collect();
            let non_group_candidates: Vec<CityObjectHandle> = candidates
                .iter()
                .copied()
                .filter(|handle| {
                    self.model
                        .cityobjects()
                        .get(*handle)
                        .is_some_and(|cityobject| {
                            cityobject.type_cityobject() != &CityObjectType::CityObjectGroup
                        })
                })
                .collect();
            if !non_group_candidates.is_empty() {
                candidates = non_group_candidates;
            }
            if candidates.is_empty() {
                continue;
            }

            candidates.shuffle(&mut self.rng);
            let max_members = candidates.len().min(3);
            let max_members = u32::try_from(max_members).expect("group size is capped at 3");
            let nr_members = get_nr_items(1..=max_members, &mut self.rng);
            let selected: Vec<CityObjectHandle> = candidates.into_iter().take(nr_members).collect();
            if selected.is_empty() {
                continue;
            }

            let roles: Vec<String> = (0..selected.len())
                .map(|_| Word(EN).fake_with_rng(&mut self.rng))
                .collect();

            if let Some(group) = self.model.cityobjects_mut().get_mut(group_handle) {
                for &member in &selected {
                    group.add_child(member);
                }
                group.extra_mut().insert(
                    "children_roles".to_string(),
                    OwnedAttributeValue::Vec(
                        roles.into_iter().map(OwnedAttributeValue::String).collect(),
                    ),
                );
            }

            for member in selected {
                if let Some(child) = self.model.cityobjects_mut().get_mut(member) {
                    child.add_parent(group_handle);
                }
            }
        }
    }

    /// Generates random attributes for `CityObjects`.
    #[must_use]
    pub fn attributes(
        mut self,
        _attributes_builder: Option<crate::attribute::AttributesBuilder>,
    ) -> Self {
        if !self.config.attributes.attributes_enabled {
            return self;
        }
        let ac = &self.config.attributes;
        let faker = AttributesFaker {
            random_keys: ac.attributes_random_keys,
            random_values: ac.attributes_random_values,
            max_depth: ac.attributes_max_depth,
            min_attrs: ac.min_attributes,
            max_attrs: ac.max_attributes,
        };
        self.attributes_cityobject = Some(faker.generate(&mut self.rng));
        self
    }

    /// Generates `CityObjects` for the model.
    ///
    /// # Panics
    ///
    /// Panics if templates are enabled but the template handle list is unexpectedly empty
    /// after template generation (should not occur in practice).
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn cityobjects(mut self) -> Self
    where
        OwnedSemantic: PartialEq,
    {
        use fake::faker::lorem::raw::Word;
        use fake::locales::EN;

        let nr_cityobjects = get_nr_items(
            self.config.cityobjects.min_cityobjects..=self.config.cityobjects.max_cityobjects,
            &mut self.rng,
        );

        // Ensure transform is present (vertices() may not have been called yet)
        if !self.progress_done_transform {
            let _ = self.model.transform_mut();
            self.progress_done_transform = true;
        }

        let min_coord = self.config.vertices.min_coordinate;
        let max_coord = self.config.vertices.max_coordinate;

        // Create template geometries
        let template_handles: Vec<GeometryTemplateHandle> = {
            let templates_enabled = self.config.templates.use_templates;
            let templates_can_be_used = match &self.config.cityobjects.allowed_types_cityobject {
                None => true,
                Some(allowed) => {
                    allowed
                        .iter()
                        .any(|t| is_first_level_type(t) && allows_geometry_instance(t))
                        || (self.config.cityobjects.cityobject_hierarchy
                            && allowed.iter().any(allows_geometry_instance))
                }
            };

            if templates_enabled && templates_can_be_used {
                let nr = get_nr_items(
                    self.config.templates.min_templates..=self.config.templates.max_templates,
                    &mut self.rng,
                );
                (0..nr)
                    .filter_map(|_| {
                        let n_verts = self.rng.random_range(3..=8usize);
                        let n_surf = self.rng.random_range(1..=3usize);
                        let surfaces: Vec<SurfaceDraft<VR, OwnedStringStorage>> = (0..n_surf)
                            .map(|_| {
                                let ring_verts: Vec<VertexDraft<VR>> = (0..n_verts)
                                    .map(|_| {
                                        VertexDraft::New(RealWorldCoordinate::new(
                                            self.rng.random_range(-10.0..=10.0f64),
                                            self.rng.random_range(-10.0..=10.0f64),
                                            self.rng.random_range(-10.0..=10.0f64),
                                        ))
                                    })
                                    .collect();
                                SurfaceDraft::new(RingDraft::new(ring_verts), [])
                            })
                            .collect();
                        // Templates require lod per CityJSON schema
                        let tpl_lod = LoDFaker {
                            allowed: self.config.geometry.allowed_lods.as_deref(),
                        }
                        .fake_with_rng(&mut self.rng);
                        GeometryDraft::multi_surface(Some(tpl_lod), surfaces)
                            .insert_template_into(&mut self.model)
                            .ok()
                    })
                    .collect()
            } else {
                Vec::new()
            }
        };

        let (parent_count, children_per_parent) = if self.config.cityobjects.cityobject_hierarchy {
            let parents = std::cmp::max(1, nr_cityobjects / 2);
            let children_count = get_nr_items(
                self.config.cityobjects.min_children..=self.config.cityobjects.max_children,
                &mut self.rng,
            );
            (parents, children_count)
        } else {
            (nr_cityobjects, 0)
        };

        let mat_themes = self.themes_material.clone();
        let mat_handles = self.material_handles.clone();
        let tex_themes = self.themes_texture.clone();
        let tex_handles = self.texture_handles.clone();
        let app = AppearanceCtx {
            mat_themes: &mat_themes,
            mat_handles: &mat_handles,
            tex_themes: &tex_themes,
            tex_handles: &tex_handles,
            max_vertices_texture: self.config.textures.max_vertices_texture as usize,
            texture_allow_none: self.config.textures.texture_allow_none,
        };
        let sem_allowed = self.config.semantics.allowed_types_semantic.clone();
        let sem_ctx = SemanticCtx {
            enabled: self.config.semantics.semantics_enabled,
            allowed_types: sem_allowed.as_deref(),
        };

        for parent_idx in 0..parent_count {
            let co_id = format!(
                "{}_{parent_idx}",
                Word(EN).fake_with_rng::<String, _>(&mut self.rng)
            );

            let city_obj_type: CityObjectType<OwnedStringStorage> =
                if let Some(allowed) = &self.config.cityobjects.allowed_types_cityobject {
                    // Filter to first-level types only — child types require a `parents` field
                    // and fail CityJSON schema validation when used as top-level objects.
                    let first_level: Vec<_> = allowed
                        .iter()
                        .filter(|t| is_first_level_type(t))
                        .cloned()
                        .collect();
                    // If no first-level types in the allowlist, fall back to Building
                    first_level
                        .choose(&mut self.rng)
                        .cloned()
                        .unwrap_or(CityObjectType::Building)
                } else if self.config.templates.use_templates && !template_handles.is_empty() {
                    // When templates are in use, only pick first-level city object types that
                    // allow GeometryInstance. Second-level types (installation/furniture etc.)
                    // require a `parents` field and would fail schema validation as top-level objects.
                    const GEOM_INSTANCE_FIRST_LEVEL: &[CityObjectType<OwnedStringStorage>] = &[
                        CityObjectType::SolitaryVegetationObject,
                        CityObjectType::CityFurniture,
                        CityObjectType::OtherConstruction,
                    ];
                    GEOM_INSTANCE_FIRST_LEVEL
                        .choose(&mut self.rng)
                        .cloned()
                        .unwrap_or(CityObjectType::CityFurniture)
                } else if let Some(geom_filter) = &self.config.geometry.allowed_types_geometry {
                    // When geometry types are restricted, only pick compatible city object types
                    let compatible = compatible_first_level_types(geom_filter);
                    compatible
                        .choose(&mut self.rng)
                        .cloned()
                        .unwrap_or(CityObjectType::Building)
                } else {
                    CityObjectTypeFaker {
                        cityobject_level: CityObjectLevel::First,
                    }
                    .fake_with_rng(&mut self.rng)
                };

            let mut cityobject = CityObject::new(
                CityObjectIdentifier::new(co_id.clone()),
                city_obj_type.clone(),
            );

            if let Some(attrs) = &self.attributes_cityobject {
                for (k, v) in attrs.iter() {
                    cityobject.attributes_mut().insert(k.clone(), v.clone());
                }
            }

            self.add_geometries_for_cityobject(
                &city_obj_type,
                min_coord,
                max_coord,
                &template_handles,
                &app,
                &sem_ctx,
                &mut cityobject,
            );

            let parent_handle = self.model.cityobjects_mut().add(cityobject).ok();

            // Only generate hierarchy when the parent type has valid child types.
            if self.config.cityobjects.cityobject_hierarchy && children_per_parent > 0 {
                if let Some(child_type_pool) = crate::get_cityobject_subtype(&city_obj_type) {
                    let mut child_handles: Vec<CityObjectHandle> = Vec::new();

                    for child_idx in 0..children_per_parent {
                        let child_id = format!(
                            "{}_{parent_idx}_{child_idx}",
                            Word(EN).fake_with_rng::<String, _>(&mut self.rng)
                        );

                        let child_type = child_type_pool
                            .choose(&mut self.rng)
                            .cloned()
                            .unwrap_or(CityObjectType::Building);

                        let mut child_obj = CityObject::new(
                            CityObjectIdentifier::new(child_id),
                            child_type.clone(),
                        );

                        if let Some(attrs) = &self.attributes_cityobject {
                            for (k, v) in attrs.iter() {
                                child_obj.attributes_mut().insert(k.clone(), v.clone());
                            }
                        }

                        self.add_geometries_for_cityobject(
                            &child_type,
                            min_coord,
                            max_coord,
                            &template_handles,
                            &app,
                            &sem_ctx,
                            &mut child_obj,
                        );

                        if let Ok(ch) = self.model.cityobjects_mut().add(child_obj) {
                            child_handles.push(ch);
                        }
                    }

                    // Wire parent ↔ child relationships
                    if let Some(ph) = parent_handle {
                        for &ch in &child_handles {
                            if let Some(p) = self.model.cityobjects_mut().get_mut(ph) {
                                p.add_child(ch);
                            }
                            if let Some(c) = self.model.cityobjects_mut().get_mut(ch) {
                                c.add_parent(ph);
                            }
                        }
                    }
                }
            }
        }

        // Top-up: if fewer city objects were generated than the configured minimum
        // (e.g., hierarchy was ON but the parent type had no valid subtypes), add
        // standalone first-level city objects to meet the minimum.
        while self.model.cityobjects().len() < nr_cityobjects {
            let co_id: String = format!("topup_{}", self.model.cityobjects().len());
            let topup_type: CityObjectType<OwnedStringStorage> =
                if let Some(allowed) = &self.config.cityobjects.allowed_types_cityobject {
                    let first_level: Vec<_> = allowed
                        .iter()
                        .filter(|t| is_first_level_type(t))
                        .cloned()
                        .collect();
                    first_level
                        .choose(&mut self.rng)
                        .cloned()
                        .unwrap_or(CityObjectType::Building)
                } else {
                    CityObjectTypeFaker {
                        cityobject_level: CityObjectLevel::First,
                    }
                    .fake_with_rng(&mut self.rng)
                };
            let mut topup_obj =
                CityObject::new(CityObjectIdentifier::new(co_id), topup_type.clone());
            self.add_geometries_for_cityobject(
                &topup_type,
                min_coord,
                max_coord,
                &template_handles,
                &app,
                &sem_ctx,
                &mut topup_obj,
            );
            let _ = self.model.cityobjects_mut().add(topup_obj);
        }

        let group_handles: Vec<CityObjectHandle> = self
            .model
            .cityobjects()
            .iter()
            .filter_map(|(handle, cityobject)| {
                matches!(
                    cityobject.type_cityobject(),
                    CityObjectType::CityObjectGroup
                )
                .then_some(handle)
            })
            .collect();
        if !group_handles.is_empty() {
            self.wire_cityobject_groups(&group_handles);
        }

        if let Some(theme) = self.used_material_themes.first().cloned() {
            self.model
                .set_default_material_theme(Some(ThemeName::new(theme)));
        }
        if let Some(theme) = self.used_texture_themes.first().cloned() {
            self.model
                .set_default_texture_theme(Some(ThemeName::new(theme)));
        }

        self
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform() {
        let config = CJFakeConfig {
            ..Default::default()
        };
        let cmf: CityModelBuilder<u32, OwnedStringStorage> = CityModelBuilder::new(config, None);
        let cm = cmf.transform().build();
        assert!(cm.transform().is_some());
    }

    #[test]
    fn vertex_coords_in_range() {
        let config = CJFakeConfig {
            vertices: crate::cli::VertexConfig {
                min_coordinate: 0.0,
                max_coordinate: 100.0,
                ..Default::default()
            },
            cityobjects: crate::cli::CityObjectConfig {
                min_cityobjects: 2,
                max_cityobjects: 2,
                ..Default::default()
            },
            ..Default::default()
        };
        let cm = CityModelBuilder::<u32, OwnedStringStorage>::new(config, None)
            .vertices()
            .cityobjects()
            .build();
        assert!(!cm.vertices().is_empty());
        for v in cm.vertices().as_slice() {
            assert!((0.0..=100.0).contains(&v.x()));
            assert!((0.0..=100.0).contains(&v.y()));
            assert!((0.0..=100.0).contains(&v.z()));
        }
    }

    #[test]
    fn texture_pick_respects_max_vertices() {
        let texture_cfg = crate::cli::TextureConfig {
            max_vertices_texture: 3,
            texture_allow_none: false,
            ..Default::default()
        };
        let themes = vec![String::from("winter")];
        let mut model =
            CityModel::<u32, OwnedStringStorage>::new(cityjson::CityModelType::CityJSON);
        let handles = [model
            .add_texture(Texture::new("tex.png".to_string(), ImageType::Png))
            .expect("texture handle should be created")];
        let app = AppearanceCtx {
            mat_themes: &[],
            mat_handles: &[],
            tex_themes: &themes,
            tex_handles: &handles,
            max_vertices_texture: texture_cfg.max_vertices_texture as usize,
            texture_allow_none: texture_cfg.texture_allow_none,
        };
        let mut rng = SmallRng::seed_from_u64(7);

        assert!(app.pick_texture(&mut rng, 3, 0).is_some());
        assert!(app.pick_texture(&mut rng, 4, 0).is_none());
    }

    #[test]
    fn texture_pick_can_return_none_when_allowed() {
        let texture_cfg = crate::cli::TextureConfig {
            max_vertices_texture: 10,
            texture_allow_none: true,
            ..Default::default()
        };
        let themes = vec![String::from("winter")];
        let mut model =
            CityModel::<u32, OwnedStringStorage>::new(cityjson::CityModelType::CityJSON);
        let handles = [model
            .add_texture(Texture::new("tex.png".to_string(), ImageType::Png))
            .expect("texture handle should be created")];
        let app = AppearanceCtx {
            mat_themes: &[],
            mat_handles: &[],
            tex_themes: &themes,
            tex_handles: &handles,
            max_vertices_texture: texture_cfg.max_vertices_texture as usize,
            texture_allow_none: texture_cfg.texture_allow_none,
        };
        let mut rng = SmallRng::seed_from_u64(0);

        let mut saw_none = false;
        for _ in 0..33 {
            if app.pick_texture(&mut rng, 3, 0).is_none() {
                saw_none = true;
                break;
            }
        }
        assert!(saw_none);
    }

    #[test]
    fn first_level_faker_includes_generic_cityobject() {
        let mut saw_generic = false;
        let mut rng = SmallRng::seed_from_u64(0);
        for _ in 0..128 {
            let t: CityObjectType<OwnedStringStorage> = CityObjectTypeFaker {
                cityobject_level: CityObjectLevel::First,
            }
            .fake_with_rng(&mut rng);
            if t == CityObjectType::GenericCityObject {
                saw_generic = true;
                break;
            }
        }
        assert!(saw_generic);
    }
}
