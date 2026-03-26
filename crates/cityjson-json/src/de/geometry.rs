use std::collections::HashMap;
use std::fmt;

use serde::de::{self, DeserializeSeed, SeqAccess, Visitor};
use serde::Deserialize;
use serde_json::value::RawValue;

use crate::de::attributes::attribute_map;
use crate::de::parse::ParseStringStorage;
use crate::de::sections::{RawAssignment, RawGeometry, RawMaterialTheme, RawSemantics};
use crate::de::validation::{parse_lod, parse_semantic_type};
use crate::errors::{Error, Result};
use cityjson::resources::handles::{
    GeometryHandle, GeometryTemplateHandle, MaterialHandle, SemanticHandle, TextureHandle,
};
use cityjson::resources::mapping::{MaterialMap, SemanticMap, TextureMap};
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{
    AffineTransform3D, Boundary, CityModel, Geometry, GeometryType, LoD, Semantic, SemanticType,
    StoredGeometryInstance, StoredGeometryParts, ThemeName, VertexIndex,
};

// ---------------------------------------------------------------------------
// Resource registry (material / texture / template handles)
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
pub(crate) struct GeometryResources {
    pub(crate) materials: Vec<MaterialHandle>,
    pub(crate) textures: Vec<TextureHandle>,
    pub(crate) templates: Vec<GeometryTemplateHandle>,
}

// ---------------------------------------------------------------------------
// Internal mappings built during geometry import
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(crate) struct RingTextureAssignment {
    pub(crate) texture: TextureHandle,
    pub(crate) uvs: Vec<VertexIndex<u32>>,
}

#[derive(Debug, Default)]
struct SurfaceMappings<'de> {
    semantics: Vec<Option<SemanticHandle>>,
    materials: Vec<(&'de str, Vec<Option<MaterialHandle>>)>,
    textures: Vec<(&'de str, Vec<Option<RingTextureAssignment>>)>,
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
pub(crate) struct StreamingGeometry<'a> {
    #[serde(rename = "type", borrow)]
    pub(crate) type_name: &'a str,
    #[serde(default, borrow)]
    lod: Option<&'a str>,
    #[serde(default, borrow)]
    boundaries: Option<&'a RawValue>,
    #[serde(default, borrow)]
    semantics: Option<RawSemantics<'a>>,
    #[serde(default, borrow)]
    material: Option<HashMap<&'a str, RawMaterialTheme>>,
    #[serde(default, borrow)]
    texture: Option<HashMap<&'a str, crate::de::sections::RawTextureTheme>>,
    #[serde(default)]
    template: Option<u32>,
    #[serde(rename = "transformationMatrix", default)]
    transformation_matrix: Option<[f64; 16]>,
}

// ---------------------------------------------------------------------------
// Flat boundary parsing
// ---------------------------------------------------------------------------

#[derive(Default)]
struct FlatBoundaryBuilder {
    vertices: Vec<VertexIndex<u32>>,
    rings: Vec<VertexIndex<u32>>,
    surfaces: Vec<VertexIndex<u32>>,
    shells: Vec<VertexIndex<u32>>,
    solids: Vec<VertexIndex<u32>>,
}

impl FlatBoundaryBuilder {
    fn finish(self) -> Boundary<u32> {
        let mut boundary = Boundary::with_capacity(
            self.vertices.len(),
            self.rings.len(),
            self.surfaces.len(),
            self.shells.len(),
            self.solids.len(),
        );
        boundary.set_vertices_from_iter(self.vertices);
        boundary.set_rings_from_iter(self.rings);
        boundary.set_surfaces_from_iter(self.surfaces);
        boundary.set_shells_from_iter(self.shells);
        boundary.set_solids_from_iter(self.solids);
        boundary
    }
}

fn boundary_offset<E: de::Error>(
    len: usize,
    level: &'static str,
) -> std::result::Result<VertexIndex<u32>, E> {
    u32::try_from(len)
        .map(VertexIndex::new)
        .map_err(|_| E::custom(format!("{level} boundary exceeds u32 index range")))
}

struct ExtendVertices<'a>(&'a mut FlatBoundaryBuilder);
struct ExtendVerticesVisitor<'a>(&'a mut FlatBoundaryBuilder);

impl<'de> Visitor<'de> for ExtendVerticesVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an array of vertex indices")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if let Some(size_hint) = seq.size_hint() {
            self.0.vertices.reserve(size_hint);
        }

        while let Some(vertex) = seq.next_element::<u32>()? {
            self.0.vertices.push(VertexIndex::new(vertex));
        }

        Ok(())
    }
}

impl<'de> DeserializeSeed<'de> for ExtendVertices<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendVerticesVisitor(self.0))
    }
}

struct ExtendRings<'a>(&'a mut FlatBoundaryBuilder);
struct ExtendRingsVisitor<'a>(&'a mut FlatBoundaryBuilder);

impl<'de> Visitor<'de> for ExtendRingsVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a surface boundary array")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if let Some(size_hint) = seq.size_hint() {
            self.0.rings.reserve(size_hint);
        }

        self.0
            .rings
            .push(boundary_offset(self.0.vertices.len(), "ring")?);

        while let Some(()) = seq.next_element_seed(ExtendVertices(self.0))? {
            self.0
                .rings
                .push(boundary_offset(self.0.vertices.len(), "ring")?);
        }

        self.0.rings.pop();
        Ok(())
    }
}

impl<'de> DeserializeSeed<'de> for ExtendRings<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendRingsVisitor(self.0))
    }
}

struct ExtendSurfaces<'a>(&'a mut FlatBoundaryBuilder);
struct ExtendSurfacesVisitor<'a>(&'a mut FlatBoundaryBuilder);

impl<'de> Visitor<'de> for ExtendSurfacesVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a multi-surface or shell boundary array")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if let Some(size_hint) = seq.size_hint() {
            self.0.surfaces.reserve(size_hint);
        }

        self.0
            .surfaces
            .push(boundary_offset(self.0.rings.len(), "surface")?);

        while let Some(()) = seq.next_element_seed(ExtendRings(self.0))? {
            self.0
                .surfaces
                .push(boundary_offset(self.0.rings.len(), "surface")?);
        }

        self.0.surfaces.pop();
        Ok(())
    }
}

impl<'de> DeserializeSeed<'de> for ExtendSurfaces<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendSurfacesVisitor(self.0))
    }
}

struct ExtendShells<'a>(&'a mut FlatBoundaryBuilder);
struct ExtendShellsVisitor<'a>(&'a mut FlatBoundaryBuilder);

impl<'de> Visitor<'de> for ExtendShellsVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a solid boundary array")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if let Some(size_hint) = seq.size_hint() {
            self.0.shells.reserve(size_hint);
        }

        self.0
            .shells
            .push(boundary_offset(self.0.surfaces.len(), "shell")?);

        while let Some(()) = seq.next_element_seed(ExtendSurfaces(self.0))? {
            self.0
                .shells
                .push(boundary_offset(self.0.surfaces.len(), "shell")?);
        }

        self.0.shells.pop();
        Ok(())
    }
}

impl<'de> DeserializeSeed<'de> for ExtendShells<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendShellsVisitor(self.0))
    }
}

struct ExtendSolids<'a>(&'a mut FlatBoundaryBuilder);
struct ExtendSolidsVisitor<'a>(&'a mut FlatBoundaryBuilder);

impl<'de> Visitor<'de> for ExtendSolidsVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a multi-solid boundary array")
    }

    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if let Some(size_hint) = seq.size_hint() {
            self.0.solids.reserve(size_hint);
        }

        self.0
            .solids
            .push(boundary_offset(self.0.shells.len(), "solid")?);

        while let Some(()) = seq.next_element_seed(ExtendShells(self.0))? {
            self.0
                .solids
                .push(boundary_offset(self.0.shells.len(), "solid")?);
        }

        self.0.solids.pop();
        Ok(())
    }
}

impl<'de> DeserializeSeed<'de> for ExtendSolids<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendSolidsVisitor(self.0))
    }
}

#[derive(Clone, Copy)]
enum BoundaryParseKind {
    MultiPoint,
    MultiLineString,
    MultiSurface,
    Solid,
    MultiSolid,
}

fn parse_boundary_from_raw(raw: &RawValue, kind: BoundaryParseKind) -> Result<Boundary<u32>> {
    let mut deserializer = serde_json::Deserializer::from_str(raw.get());
    let mut builder = FlatBoundaryBuilder::default();

    match kind {
        BoundaryParseKind::MultiPoint => {
            ExtendVertices(&mut builder).deserialize(&mut deserializer)?;
        }
        BoundaryParseKind::MultiLineString => {
            ExtendRings(&mut builder).deserialize(&mut deserializer)?;
        }
        BoundaryParseKind::MultiSurface => {
            ExtendSurfaces(&mut builder).deserialize(&mut deserializer)?;
        }
        BoundaryParseKind::Solid => ExtendShells(&mut builder).deserialize(&mut deserializer)?,
        BoundaryParseKind::MultiSolid => {
            ExtendSolids(&mut builder).deserialize(&mut deserializer)?;
        }
    }

    Ok(builder.finish())
}

// ---------------------------------------------------------------------------
// Top-level geometry dispatch
// ---------------------------------------------------------------------------

pub(crate) fn import_stream_geometry<'de, SS>(
    raw: StreamingGeometry<'de>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<GeometryHandle>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let geometry = build_stream_geometry(raw, model, resources)?;
    model.add_geometry_unchecked(geometry).map_err(Error::from)
}

/// Import a geometry as a template (not a regular city object geometry).
///
/// Template geometries cannot be `GeometryInstance`.
pub(crate) fn import_template_geometry<'de, SS>(
    raw: RawGeometry<'de>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<GeometryTemplateHandle>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    if matches!(raw, RawGeometry::GeometryInstance { .. }) {
        return Err(Error::UnsupportedFeature(
            "GeometryInstance cannot be used as a geometry template",
        ));
    }

    let geometry = build_geometry(raw, model, resources)?;
    model
        .add_geometry_template_unchecked(geometry)
        .map_err(Error::from)
}

#[allow(clippy::too_many_lines)]
fn build_geometry<'de, SS>(
    raw: RawGeometry<'de>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    match raw {
        RawGeometry::MultiPoint {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => build_multi_point_geometry(
            lod,
            boundaries.into(),
            semantics.as_ref(),
            material.as_ref(),
            texture.as_ref(),
            model,
        ),
        RawGeometry::MultiLineString {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => build_multi_line_string_geometry(
            lod,
            boundaries.try_into()?,
            semantics.as_ref(),
            material.as_ref(),
            texture.as_ref(),
            model,
        ),
        RawGeometry::MultiSurface {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => build_raw_surface_geometry(
            lod,
            boundaries,
            false,
            semantics.as_ref(),
            material,
            texture,
            model,
            resources,
        ),
        RawGeometry::CompositeSurface {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => build_raw_surface_geometry(
            lod,
            boundaries,
            true,
            semantics.as_ref(),
            material,
            texture,
            model,
            resources,
        ),
        RawGeometry::Solid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => build_raw_solid_geometry(
            lod,
            boundaries,
            semantics.as_ref(),
            material,
            texture,
            model,
            resources,
        ),
        RawGeometry::MultiSolid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => build_raw_multi_solid_geometry(
            lod,
            boundaries,
            false,
            semantics.as_ref(),
            material,
            texture,
            model,
            resources,
        ),
        RawGeometry::CompositeSolid {
            lod,
            boundaries,
            semantics,
            material,
            texture,
        } => build_raw_multi_solid_geometry(
            lod,
            boundaries,
            true,
            semantics.as_ref(),
            material,
            texture,
            model,
            resources,
        ),
        RawGeometry::GeometryInstance {
            template,
            boundaries,
            transformation_matrix,
        } => build_geometry_instance(
            template,
            boundaries.as_deref(),
            transformation_matrix,
            resources,
        ),
    }
}

fn build_stream_geometry<'de, SS>(
    raw: StreamingGeometry<'de>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    match raw.type_name {
        "MultiPoint" => build_stream_point_geometry(&raw, BoundaryParseKind::MultiPoint, model),
        "MultiLineString" => {
            build_stream_linestring_geometry(&raw, BoundaryParseKind::MultiLineString, model)
        }
        "MultiSurface" => build_stream_surface_geometry(
            raw,
            BoundaryParseKind::MultiSurface,
            false,
            model,
            resources,
        ),
        "CompositeSurface" => build_stream_surface_geometry(
            raw,
            BoundaryParseKind::MultiSurface,
            true,
            model,
            resources,
        ),
        "Solid" => build_stream_solid_geometry(raw, BoundaryParseKind::Solid, model, resources),
        "MultiSolid" => build_stream_multi_solid_geometry(
            raw,
            BoundaryParseKind::MultiSolid,
            false,
            model,
            resources,
        ),
        "CompositeSolid" => build_stream_multi_solid_geometry(
            raw,
            BoundaryParseKind::MultiSolid,
            true,
            model,
            resources,
        ),
        "GeometryInstance" => build_stream_geometry_instance(&raw, resources),
        _ => Err(Error::InvalidValue(format!(
            "unsupported geometry type '{}'",
            raw.type_name
        ))),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_raw_surface_geometry<'de, SS>(
    lod: Option<&'de str>,
    boundaries: crate::de::sections::MultiSurfaceBoundary,
    composite: bool,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    build_multi_surface_geometry(
        lod,
        boundaries.try_into()?,
        composite,
        semantics,
        material,
        texture,
        model,
        resources,
    )
}

fn build_raw_solid_geometry<'de, SS>(
    lod: Option<&'de str>,
    boundaries: crate::de::sections::SolidBoundary,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    build_solid_geometry(
        lod,
        boundaries.try_into()?,
        semantics,
        material,
        texture,
        model,
        resources,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_raw_multi_solid_geometry<'de, SS>(
    lod: Option<&'de str>,
    boundaries: crate::de::sections::MultiSolidBoundary,
    composite: bool,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    build_multi_solid_geometry(
        lod,
        boundaries.try_into()?,
        composite,
        semantics,
        material,
        texture,
        model,
        resources,
    )
}

fn build_stream_point_geometry<'de, SS>(
    raw: &StreamingGeometry<'de>,
    kind: BoundaryParseKind,
    model: &mut CityModel<u32, SS>,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    build_multi_point_geometry(
        raw.lod,
        parse_stream_boundary(raw, kind)?,
        raw.semantics.as_ref(),
        raw.material.as_ref(),
        raw.texture.as_ref(),
        model,
    )
}

fn build_stream_linestring_geometry<'de, SS>(
    raw: &StreamingGeometry<'de>,
    kind: BoundaryParseKind,
    model: &mut CityModel<u32, SS>,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    build_multi_line_string_geometry(
        raw.lod,
        parse_stream_boundary(raw, kind)?,
        raw.semantics.as_ref(),
        raw.material.as_ref(),
        raw.texture.as_ref(),
        model,
    )
}

fn build_stream_surface_geometry<'de, SS>(
    raw: StreamingGeometry<'de>,
    kind: BoundaryParseKind,
    composite: bool,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    build_multi_surface_geometry(
        raw.lod,
        parse_stream_boundary(&raw, kind)?,
        composite,
        raw.semantics.as_ref(),
        raw.material,
        raw.texture,
        model,
        resources,
    )
}

fn build_stream_solid_geometry<'de, SS>(
    raw: StreamingGeometry<'de>,
    kind: BoundaryParseKind,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    build_solid_geometry(
        raw.lod,
        parse_stream_boundary(&raw, kind)?,
        raw.semantics.as_ref(),
        raw.material,
        raw.texture,
        model,
        resources,
    )
}

fn build_stream_multi_solid_geometry<'de, SS>(
    raw: StreamingGeometry<'de>,
    kind: BoundaryParseKind,
    composite: bool,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    build_multi_solid_geometry(
        raw.lod,
        parse_stream_boundary(&raw, kind)?,
        composite,
        raw.semantics.as_ref(),
        raw.material,
        raw.texture,
        model,
        resources,
    )
}

fn build_stream_geometry_instance<'de, SS>(
    raw: &StreamingGeometry<'de>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let instance_boundaries = raw
        .boundaries
        .map(|boundaries| serde_json::from_str::<Vec<u32>>(boundaries.get()))
        .transpose()?;
    build_geometry_instance(
        raw.template,
        instance_boundaries.as_deref(),
        raw.transformation_matrix,
        resources,
    )
}

fn parse_stream_boundary(
    raw: &StreamingGeometry<'_>,
    kind: BoundaryParseKind,
) -> Result<Boundary<u32>> {
    parse_boundary_from_raw(required_boundaries(raw.boundaries, raw.type_name)?, kind)
}

fn required_boundaries<'de>(
    boundaries: Option<&'de RawValue>,
    type_name: &str,
) -> Result<&'de RawValue> {
    boundaries.ok_or_else(|| Error::InvalidValue(format!("{type_name} is missing boundaries")))
}

fn build_multi_point_geometry<'de, SS>(
    lod: Option<&'de str>,
    boundaries: Boundary<u32>,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<&HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<&HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    if material.is_some_and(|m| !m.is_empty()) {
        return Err(Error::UnsupportedFeature(
            "geometry material import is not supported for MultiPoint",
        ));
    }
    if texture.is_some_and(|t| !t.is_empty()) {
        return Err(Error::UnsupportedFeature(
            "geometry texture import is not supported for MultiPoint",
        ));
    }

    let point_count = boundaries.vertices().len();
    let semantic_handles = import_geometry_semantics::<SS>(semantics, model)?;
    let assignments = parse_point_assignments(semantics, &semantic_handles, point_count);
    let semantic_map = semantics.map(|_| {
        let mut map = SemanticMap::<u32>::new();
        for assignment in assignments {
            map.add_point(assignment);
        }
        map
    });

    Ok(Geometry::from_stored_parts(StoredGeometryParts {
        type_geometry: GeometryType::MultiPoint,
        lod: parse_lod(lod)?,
        boundaries: Some(boundaries),
        semantics: semantic_map,
        materials: None,
        textures: None,
        instance: None,
    }))
}

fn build_multi_line_string_geometry<'de, SS>(
    lod: Option<&'de str>,
    boundaries: Boundary<u32>,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<&HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<&HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    if material.is_some_and(|m| !m.is_empty()) {
        return Err(Error::UnsupportedFeature(
            "geometry material import is not supported for MultiLineString",
        ));
    }
    if texture.is_some_and(|t| !t.is_empty()) {
        return Err(Error::UnsupportedFeature(
            "geometry texture import is not supported for MultiLineString",
        ));
    }

    let linestring_count = boundaries.rings().len();
    let semantic_handles = import_geometry_semantics::<SS>(semantics, model)?;
    let assignments = parse_linestring_assignments(semantics, &semantic_handles, linestring_count);
    let semantic_map = semantics.map(|_| {
        let mut map = SemanticMap::<u32>::new();
        for assignment in assignments {
            map.add_linestring(assignment);
        }
        map
    });

    Ok(Geometry::from_stored_parts(StoredGeometryParts {
        type_geometry: GeometryType::MultiLineString,
        lod: parse_lod(lod)?,
        boundaries: Some(boundaries),
        semantics: semantic_map,
        materials: None,
        textures: None,
        instance: None,
    }))
}

#[allow(clippy::too_many_arguments)]
fn build_multi_surface_geometry<'de, SS>(
    lod: Option<&'de str>,
    boundaries: Boundary<u32>,
    is_composite: bool,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let has_semantics = semantics.is_some();
    let has_material = material.is_some();
    let has_texture = texture.is_some();
    let mappings =
        parse_surface_mappings(semantics, material, texture, &boundaries, model, resources)?;
    Ok(build_surface_geometry_parts(
        if is_composite {
            GeometryType::CompositeSurface
        } else {
            GeometryType::MultiSurface
        },
        parse_lod(lod)?,
        boundaries,
        mappings,
        has_semantics,
        has_material,
        has_texture,
    ))
}

fn build_solid_geometry<'de, SS>(
    lod: Option<&'de str>,
    boundaries: Boundary<u32>,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let has_semantics = semantics.is_some();
    let has_material = material.is_some();
    let has_texture = texture.is_some();
    let mappings =
        parse_surface_mappings(semantics, material, texture, &boundaries, model, resources)?;
    Ok(build_surface_geometry_parts(
        GeometryType::Solid,
        parse_lod(lod)?,
        boundaries,
        mappings,
        has_semantics,
        has_material,
        has_texture,
    ))
}

#[allow(clippy::too_many_arguments)]
fn build_multi_solid_geometry<'de, SS>(
    lod: Option<&'de str>,
    boundaries: Boundary<u32>,
    is_composite: bool,
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let has_semantics = semantics.is_some();
    let has_material = material.is_some();
    let has_texture = texture.is_some();
    let mappings =
        parse_surface_mappings(semantics, material, texture, &boundaries, model, resources)?;
    Ok(build_surface_geometry_parts(
        if is_composite {
            GeometryType::CompositeSolid
        } else {
            GeometryType::MultiSolid
        },
        parse_lod(lod)?,
        boundaries,
        mappings,
        has_semantics,
        has_material,
        has_texture,
    ))
}

fn build_geometry_instance<SS>(
    template: Option<u32>,
    boundaries: Option<&[u32]>,
    transformation_matrix: Option<[f64; 16]>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: StringStorage,
{
    let template_idx = template.ok_or_else(|| {
        Error::InvalidValue("GeometryInstance is missing a template index".to_owned())
    })?;
    let template_handle = resources
        .templates
        .get(template_idx as usize)
        .copied()
        .ok_or_else(|| {
            Error::InvalidValue(format!("invalid geometry template index '{template_idx}'"))
        })?;

    let reference_point = boundaries
        .and_then(<[u32]>::first)
        .copied()
        .ok_or_else(|| {
            Error::InvalidValue(
                "GeometryInstance boundaries must contain a single reference-point index"
                    .to_owned(),
            )
        })?;

    Ok(Geometry::from_stored_parts(StoredGeometryParts {
        type_geometry: GeometryType::GeometryInstance,
        lod: None,
        boundaries: None,
        semantics: None,
        materials: None,
        textures: None,
        instance: Some(StoredGeometryInstance {
            template: template_handle,
            reference_point: VertexIndex::new(reference_point),
            transformation: transformation_matrix
                .map(AffineTransform3D::from)
                .unwrap_or_default(),
        }),
    }))
}

fn build_surface_geometry_parts<'de, SS>(
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Boundary<u32>,
    mappings: SurfaceMappings<'de>,
    has_semantics: bool,
    has_materials: bool,
    has_textures: bool,
) -> Geometry<u32, SS>
where
    SS: ParseStringStorage<'de>,
{
    let SurfaceMappings {
        semantics: surface_semantics,
        materials: surface_materials,
        textures: surface_textures,
    } = mappings;

    let semantics = has_semantics.then(|| {
        let mut map = SemanticMap::<u32>::new();
        for assignment in surface_semantics {
            map.add_surface(assignment);
        }
        map
    });
    let materials = has_materials.then(|| {
        surface_materials
            .into_iter()
            .map(|(theme, assignments)| {
                let mut map = MaterialMap::<u32>::new();
                for assignment in assignments {
                    map.add_surface(assignment);
                }
                (ThemeName::<SS>::new(SS::store(theme)), map)
            })
            .collect::<Vec<_>>()
    });
    let textures = has_textures.then(|| {
        surface_textures
            .into_iter()
            .map(|(theme, assignments)| {
                (
                    ThemeName::<SS>::new(SS::store(theme)),
                    build_texture_map(&boundaries, &assignments),
                )
            })
            .collect::<Vec<_>>()
    });

    Geometry::from_stored_parts(StoredGeometryParts {
        type_geometry,
        lod,
        boundaries: Some(boundaries),
        semantics,
        materials,
        textures,
        instance: None,
    })
}

fn build_texture_map(
    boundary: &Boundary<u32>,
    assignments: &[Option<RingTextureAssignment>],
) -> TextureMap<u32> {
    let mut map = TextureMap::<u32>::new();
    for (ring_index, &ring_start) in boundary.rings().iter().enumerate() {
        let ring_end = boundary
            .rings()
            .get(ring_index + 1)
            .map_or(boundary.vertices().len(), VertexIndex::to_usize);
        let ring_vertices = ring_end.saturating_sub(ring_start.to_usize());
        let assignment = assignments
            .get(ring_index)
            .and_then(|assignment| assignment.as_ref());

        map.add_ring(ring_start);
        map.add_ring_texture(assignment.map(|assignment| assignment.texture));

        if let Some(assignment) = assignment {
            for uv in assignment.uvs.iter().copied().take(ring_vertices) {
                map.add_vertex(Some(uv));
            }
            for _ in assignment.uvs.len().min(ring_vertices)..ring_vertices {
                map.add_vertex(None);
            }
        } else {
            for _ in 0..ring_vertices {
                map.add_vertex(None);
            }
        }
    }

    map
}

// ---------------------------------------------------------------------------
// Semantic import
// ---------------------------------------------------------------------------

pub(crate) fn import_geometry_semantics<'de, SS>(
    raw: Option<&RawSemantics<'de>>,
    model: &mut CityModel<u32, SS>,
) -> Result<Vec<SemanticHandle>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };

    let mut pending_links: Vec<(Option<u64>, Vec<u64>)> = Vec::with_capacity(raw.surfaces.len());
    let mut handles = Vec::with_capacity(raw.surfaces.len());

    for surface in &raw.surfaces {
        let sem_type: SemanticType<SS> = parse_semantic_type::<SS>(surface.type_name)?;
        let mut semantic = Semantic::<SS>::new(sem_type);

        if !surface.attributes.is_empty() {
            // Remove known keys that are not attributes
            let attrs: HashMap<&'de str, _> = surface
                .attributes
                .iter()
                .filter(|(k, _)| **k != "type" && **k != "parent" && **k != "children")
                .map(|(k, v)| (*k, v))
                .collect::<HashMap<_, _>>();

            if !attrs.is_empty() {
                let attrs_cloned: HashMap<&'de str, _> = attrs
                    .into_iter()
                    .map(|(k, v)| (k, clone_raw_attribute(v)))
                    .collect();
                *semantic.attributes_mut() =
                    attribute_map::<SS>(attrs_cloned, "semantic attributes")?;
            }
        }

        handles.push(model.add_semantic(semantic).map_err(Error::from)?);
        pending_links.push((surface.parent, surface.children.clone()));
    }

    // Resolve parent/child links after all handles are known.
    for (index, (parent, children)) in pending_links.into_iter().enumerate() {
        let handle = handles[index];
        let semantic = model
            .get_semantic_mut(handle)
            .ok_or_else(|| Error::InvalidValue(format!("missing semantic handle {handle}")))?;
        if let Some(parent_index) = parent {
            if let Some(&parent_handle) = usize::try_from(parent_index)
                .ok()
                .and_then(|i| handles.get(i))
            {
                semantic.set_parent(parent_handle);
            }
        }
        if !children.is_empty() {
            let sem_children = semantic.children_mut();
            sem_children.reserve(children.len());
            for child_index in children {
                if let Some(&child_handle) = usize::try_from(child_index)
                    .ok()
                    .and_then(|i| handles.get(i))
                {
                    sem_children.push(child_handle);
                }
            }
        }
    }

    Ok(handles)
}

/// Clone a `RawAttribute` reference for building the attributes map.
///
/// This is needed because `import_geometry_semantics` borrows `raw.surfaces`
/// but needs to produce owned data for `attribute_map`.
fn clone_raw_attribute<'de>(
    attr: &crate::de::attributes::RawAttribute<'de>,
) -> crate::de::attributes::RawAttribute<'de> {
    use crate::de::attributes::RawAttribute;
    use std::borrow::Cow;
    match attr {
        RawAttribute::Null => RawAttribute::Null,
        RawAttribute::Bool(b) => RawAttribute::Bool(*b),
        RawAttribute::Number(n) => RawAttribute::Number(n.clone()),
        RawAttribute::String(cow) => RawAttribute::String(match cow {
            Cow::Borrowed(s) => Cow::Borrowed(s),
            Cow::Owned(s) => Cow::Owned(s.clone()),
        }),
        RawAttribute::Array(v) => RawAttribute::Array(v.iter().map(clone_raw_attribute).collect()),
        RawAttribute::Object(m) => RawAttribute::Object(
            m.iter()
                .map(|(k, v)| (*k, clone_raw_attribute(v)))
                .collect(),
        ),
    }
}

// ---------------------------------------------------------------------------
// Assignment helpers
// ---------------------------------------------------------------------------

fn flatten_assignment(raw: &RawAssignment, out: &mut Vec<Option<u64>>) {
    match raw {
        RawAssignment::Null => out.push(None),
        RawAssignment::Index(i) => out.push(Some(*i)),
        RawAssignment::Nested(vec) => {
            for child in vec {
                flatten_assignment(child, out);
            }
        }
    }
}

fn resolve_assignments<T: Copy>(
    raw: &RawAssignment,
    handles: &[T],
    expected_len: usize,
) -> Vec<Option<T>> {
    let mut indices = Vec::new();
    flatten_assignment(raw, &mut indices);
    indices.resize(expected_len, None);
    indices
        .into_iter()
        .map(|idx| {
            idx.and_then(|i| {
                usize::try_from(i)
                    .ok()
                    .and_then(|i| handles.get(i))
                    .copied()
            })
        })
        .collect()
}

fn parse_point_assignments(
    semantics: Option<&RawSemantics<'_>>,
    handles: &[SemanticHandle],
    expected_len: usize,
) -> Vec<Option<SemanticHandle>> {
    match semantics {
        None => vec![None; expected_len],
        Some(s) => resolve_assignments(&s.values, handles, expected_len),
    }
}

fn parse_linestring_assignments(
    semantics: Option<&RawSemantics<'_>>,
    handles: &[SemanticHandle],
    expected_len: usize,
) -> Vec<Option<SemanticHandle>> {
    match semantics {
        None => vec![None; expected_len],
        Some(s) => resolve_assignments(&s.values, handles, expected_len),
    }
}

fn parse_surface_scalar_assignments(
    semantics: Option<&RawSemantics<'_>>,
    handles: &[SemanticHandle],
    surface_count: usize,
) -> Vec<Option<SemanticHandle>> {
    match semantics {
        None => vec![None; surface_count],
        Some(s) => resolve_assignments(&s.values, handles, surface_count),
    }
}

// ---------------------------------------------------------------------------
// SurfaceMappings builders
// ---------------------------------------------------------------------------

fn parse_surface_mappings<'de, SS>(
    semantics: Option<&RawSemantics<'de>>,
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    texture: Option<HashMap<&'de str, crate::de::sections::RawTextureTheme>>,
    boundaries: &Boundary<u32>,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<SurfaceMappings<'de>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let surface_count = boundaries.surfaces().len();
    let ring_count = boundaries.rings().len();
    let semantic_handles = import_geometry_semantics::<SS>(semantics, model)?;
    Ok(SurfaceMappings {
        semantics: parse_surface_scalar_assignments(semantics, &semantic_handles, surface_count),
        materials: parse_material_themes(material, &resources.materials, surface_count)?,
        textures: parse_texture_themes(texture, |values| {
            parse_ring_texture_assignments(values, ring_count, resources)
        })?,
    })
}

// ---------------------------------------------------------------------------
// Material theme parsing
// ---------------------------------------------------------------------------

type MaterialThemes<'de> = Vec<(&'de str, Vec<Option<MaterialHandle>>)>;
type TextureThemes<'de> = Vec<(&'de str, Vec<Option<RingTextureAssignment>>)>;

fn parse_material_themes<'de>(
    material: Option<HashMap<&'de str, RawMaterialTheme>>,
    handles: &[MaterialHandle],
    surface_count: usize,
) -> Result<MaterialThemes<'de>> {
    let Some(material) = material else {
        return Ok(Vec::new());
    };
    let mut result = Vec::with_capacity(material.len());
    for (theme, entry) in material {
        let assignments = if let Some(single) = entry.value.as_ref() {
            let idx = match single {
                RawAssignment::Null => None,
                RawAssignment::Index(i) => usize::try_from(*i)
                    .ok()
                    .and_then(|i| handles.get(i))
                    .copied(),
                RawAssignment::Nested(_) => {
                    return Err(Error::InvalidValue(
                        "geometry material.value must be a scalar, not an array".to_owned(),
                    ))
                }
            };
            vec![idx; surface_count]
        } else if let Some(values) = entry.values.as_ref() {
            resolve_assignments(values, handles, surface_count)
        } else {
            return Err(Error::InvalidValue(format!(
                "geometry material theme '{theme}' must contain value or values"
            )));
        };
        result.push((theme, assignments));
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// Texture theme parsing
// ---------------------------------------------------------------------------

fn parse_texture_themes<F>(
    texture: Option<HashMap<&str, crate::de::sections::RawTextureTheme>>,
    mut parse_values: F,
) -> Result<TextureThemes<'_>>
where
    F: FnMut(&serde_json::Value) -> Result<Vec<Option<RingTextureAssignment>>>,
{
    let Some(texture) = texture else {
        return Ok(Vec::new());
    };
    let mut result = Vec::with_capacity(texture.len());
    for (theme, entry) in texture {
        result.push((theme, parse_values(&entry.values)?));
    }
    Ok(result)
}

fn parse_ring_texture_assignments(
    values: &serde_json::Value,
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
    value: &serde_json::Value,
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

fn looks_like_ring_texture_assignment(value: &serde_json::Value) -> bool {
    let Some(values) = value.as_array() else {
        return false;
    };
    let Some(first) = values.first() else {
        return false;
    };
    match first {
        serde_json::Value::Null => values.len() == 1,
        serde_json::Value::Number(_) => values.iter().skip(1).all(serde_json::Value::is_number),
        _ => false,
    }
}

fn parse_ring_texture_assignment(
    value: &serde_json::Value,
    resources: &GeometryResources,
) -> Result<Option<RingTextureAssignment>> {
    let values = value.as_array().ok_or_else(|| {
        Error::InvalidValue("geometry texture ring value must be an array".to_owned())
    })?;
    let first = values.first().ok_or_else(|| {
        Error::InvalidValue("geometry texture ring value cannot be empty".to_owned())
    })?;
    if first.is_null() {
        return Ok(None);
    }
    let tex_u64 = first.as_u64().ok_or_else(|| {
        Error::InvalidValue("geometry texture index must be an unsigned integer".to_owned())
    })?;
    let tex_index = usize::try_from(tex_u64)
        .map_err(|_| Error::InvalidValue("geometry texture index out of range".to_owned()))?;
    let texture = resources.textures.get(tex_index).copied().ok_or_else(|| {
        Error::InvalidValue(format!("invalid geometry texture index '{tex_index}'"))
    })?;
    let mut uvs = Vec::with_capacity(values.len().saturating_sub(1));
    for uv in values.iter().skip(1) {
        let index = uv.as_u64().ok_or_else(|| {
            Error::InvalidValue(format!(
                "geometry texture uv index must be an unsigned integer, got {uv}"
            ))
        })?;
        uvs.push(VertexIndex::new(u32::try_from(index).map_err(|_| {
            Error::InvalidValue(format!("geometry texture uv index {index} out of range"))
        })?));
    }
    Ok(Some(RingTextureAssignment { texture, uvs }))
}
