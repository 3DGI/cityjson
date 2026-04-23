use std::collections::HashMap;
use std::fmt;

use serde::Deserialize;
use serde::de::{self, MapAccess, Visitor};
use serde_json::value::RawValue;

use crate::de::attributes::attribute_map;
use crate::de::parse::ParseStringStorage;
use crate::de::profiling::timed;
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

pub(crate) struct StreamingGeometry<'a> {
    kind: GeometryKind,
    lod: Option<&'a str>,
    boundaries: Option<Boundary<u32>>,
    instance_boundaries: Option<Vec<u32>>,
    semantics: Option<RawSemantics<'a>>,
    material: Option<HashMap<&'a str, RawMaterialTheme>>,
    texture: Option<HashMap<&'a str, crate::de::sections::RawTextureTheme>>,
    template: Option<u32>,
    transformation_matrix: Option<[f64; 16]>,
}

#[derive(Clone, Copy, Debug)]
enum GeometryKind {
    MultiPoint,
    MultiLineString,
    MultiSurface { composite: bool },
    Solid,
    MultiSolid { composite: bool },
    GeometryInstance,
}

impl GeometryKind {
    fn from_type_name(type_name: &str) -> Result<Self> {
        match type_name {
            "MultiPoint" => Ok(Self::MultiPoint),
            "MultiLineString" => Ok(Self::MultiLineString),
            "MultiSurface" => Ok(Self::MultiSurface { composite: false }),
            "CompositeSurface" => Ok(Self::MultiSurface { composite: true }),
            "Solid" => Ok(Self::Solid),
            "MultiSolid" => Ok(Self::MultiSolid { composite: false }),
            "CompositeSolid" => Ok(Self::MultiSolid { composite: true }),
            "GeometryInstance" => Ok(Self::GeometryInstance),
            _ => Err(Error::InvalidValue(format!(
                "unsupported geometry type '{type_name}'"
            ))),
        }
    }

    fn type_name(self) -> &'static str {
        match self {
            Self::MultiPoint => "MultiPoint",
            Self::MultiLineString => "MultiLineString",
            Self::MultiSurface { composite: false } => "MultiSurface",
            Self::MultiSurface { composite: true } => "CompositeSurface",
            Self::Solid => "Solid",
            Self::MultiSolid { composite: false } => "MultiSolid",
            Self::MultiSolid { composite: true } => "CompositeSolid",
            Self::GeometryInstance => "GeometryInstance",
        }
    }

    fn boundary_kind(self) -> Option<BoundaryParseKind> {
        match self {
            Self::MultiPoint => Some(BoundaryParseKind::MultiPoint),
            Self::MultiLineString => Some(BoundaryParseKind::MultiLineString),
            Self::MultiSurface { .. } => Some(BoundaryParseKind::MultiSurface),
            Self::Solid => Some(BoundaryParseKind::Solid),
            Self::MultiSolid { .. } => Some(BoundaryParseKind::MultiSolid),
            Self::GeometryInstance => None,
        }
    }
}

impl<'de> Deserialize<'de> for StreamingGeometry<'de> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(StreamingGeometryVisitor)
    }
}

struct StreamingGeometryVisitor;

impl<'de> Visitor<'de> for StreamingGeometryVisitor {
    type Value = StreamingGeometry<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a CityJSON geometry object")
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut kind = None;
        let mut lod = None;
        let mut boundaries = None;
        let mut instance_boundaries = None;
        let mut semantics = None;
        let mut material = None;
        let mut texture = None;
        let mut template = None;
        let mut transformation_matrix = None;
        let mut raw_boundaries = None;

        while let Some(key) = map.next_key::<&'de str>()? {
            match key {
                "type" => {
                    let parsed_kind = GeometryKind::from_type_name(map.next_value::<&'de str>()?)
                        .map_err(de::Error::custom)?;
                    if let Some(raw) = raw_boundaries.take() {
                        parse_geometry_boundaries(
                            raw,
                            parsed_kind,
                            &mut boundaries,
                            &mut instance_boundaries,
                        )
                        .map_err(de::Error::custom)?;
                    }
                    kind = Some(parsed_kind);
                }
                "lod" => lod = map.next_value()?,
                "boundaries" => {
                    let raw = map.next_value::<&'de RawValue>()?;
                    if let Some(parsed_kind) = kind {
                        parse_geometry_boundaries(
                            raw,
                            parsed_kind,
                            &mut boundaries,
                            &mut instance_boundaries,
                        )
                        .map_err(de::Error::custom)?;
                    } else {
                        raw_boundaries = Some(raw);
                    }
                }
                "semantics" => semantics = map.next_value()?,
                "material" => material = map.next_value()?,
                "texture" => texture = map.next_value()?,
                "template" => template = map.next_value()?,
                "transformationMatrix" => transformation_matrix = map.next_value()?,
                _ => {
                    let _: de::IgnoredAny = map.next_value()?;
                }
            }
        }

        let kind = kind.ok_or_else(|| de::Error::missing_field("type"))?;
        if let Some(raw) = raw_boundaries {
            parse_geometry_boundaries(raw, kind, &mut boundaries, &mut instance_boundaries)
                .map_err(de::Error::custom)?;
        }

        match kind {
            GeometryKind::GeometryInstance => {}
            _ if boundaries.is_none() => {
                return Err(de::Error::custom(format!(
                    "{} is missing boundaries",
                    kind.type_name()
                )));
            }
            _ => {}
        }

        Ok(StreamingGeometry {
            kind,
            lod,
            boundaries,
            instance_boundaries,
            semantics,
            material,
            texture,
            template,
            transformation_matrix,
        })
    }
}

// ---------------------------------------------------------------------------
// Flat boundary parsing
// ---------------------------------------------------------------------------

#[derive(Default)]
struct BoundaryParts {
    vertices: Vec<VertexIndex<u32>>,
    rings: Vec<VertexIndex<u32>>,
    surfaces: Vec<VertexIndex<u32>>,
    shells: Vec<VertexIndex<u32>>,
    solids: Vec<VertexIndex<u32>>,
}

fn boundary_offset(len: usize, level: &'static str) -> Result<VertexIndex<u32>> {
    u32::try_from(len)
        .map(VertexIndex::new)
        .map_err(|_| Error::InvalidValue(format!("{level} boundary exceeds u32 index range")))
}

fn parse_geometry_boundaries(
    raw: &RawValue,
    kind: GeometryKind,
    boundaries: &mut Option<Boundary<u32>>,
    instance_boundaries: &mut Option<Vec<u32>>,
) -> Result<()> {
    match kind.boundary_kind() {
        Some(boundary_kind) => {
            *boundaries = Some(timed("geometry.parse_boundary", || {
                parse_boundary_from_raw(raw, boundary_kind)
            })?);
        }
        None => {
            *instance_boundaries = Some(serde_json::from_str(raw.get())?);
        }
    }
    Ok(())
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
    BoundaryParser::new(raw.get().as_bytes()).parse(kind)
}

struct BoundaryParser<'a> {
    input: &'a [u8],
    cursor: usize,
    parts: BoundaryParts,
}

impl<'a> BoundaryParser<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            cursor: 0,
            parts: BoundaryParts::default(),
        }
    }

    fn parse(mut self, kind: BoundaryParseKind) -> Result<Boundary<u32>> {
        match kind {
            BoundaryParseKind::MultiPoint => self.parse_vertices_array()?,
            BoundaryParseKind::MultiLineString => self.parse_rings_array()?,
            BoundaryParseKind::MultiSurface => self.parse_surfaces_array()?,
            BoundaryParseKind::Solid => self.parse_shells_array()?,
            BoundaryParseKind::MultiSolid => self.parse_solids_array()?,
        }
        self.skip_ws();
        if self.cursor != self.input.len() {
            return Err(self.error("unexpected trailing characters in geometry boundaries"));
        }
        Boundary::from_parts(
            self.parts.vertices,
            self.parts.rings,
            self.parts.surfaces,
            self.parts.shells,
            self.parts.solids,
        )
        .map_err(Error::from)
    }

    fn parse_vertices_array(&mut self) -> Result<()> {
        self.parse_array(|this| {
            let vertex = this.parse_u32()?;
            this.parts.vertices.push(VertexIndex::new(vertex));
            Ok(())
        })
    }

    fn parse_rings_array(&mut self) -> Result<()> {
        self.parse_array(|this| {
            this.parts
                .rings
                .push(boundary_offset(this.parts.vertices.len(), "ring")?);
            this.parse_vertices_array()
        })
    }

    fn parse_surfaces_array(&mut self) -> Result<()> {
        self.parse_array(|this| {
            this.parts
                .surfaces
                .push(boundary_offset(this.parts.rings.len(), "surface")?);
            this.parse_rings_array()
        })
    }

    fn parse_shells_array(&mut self) -> Result<()> {
        self.parse_array(|this| {
            this.parts
                .shells
                .push(boundary_offset(this.parts.surfaces.len(), "shell")?);
            this.parse_surfaces_array()
        })
    }

    fn parse_solids_array(&mut self) -> Result<()> {
        self.parse_array(|this| {
            this.parts
                .solids
                .push(boundary_offset(this.parts.shells.len(), "solid")?);
            this.parse_shells_array()
        })
    }

    fn parse_array<F>(&mut self, mut parse_element: F) -> Result<()>
    where
        F: FnMut(&mut Self) -> Result<()>,
    {
        self.expect_byte(b'[')?;
        self.skip_ws();
        if self.consume_if(b']') {
            return Ok(());
        }

        loop {
            parse_element(self)?;
            self.skip_ws();
            if self.consume_if(b',') {
                continue;
            }
            self.expect_byte(b']')?;
            return Ok(());
        }
    }

    fn parse_u32(&mut self) -> Result<u32> {
        self.skip_ws();
        let start = self.cursor;
        let mut value = 0u32;

        while let Some(byte) = self.peek() {
            if !byte.is_ascii_digit() {
                break;
            }
            value = value
                .checked_mul(10)
                .and_then(|acc| acc.checked_add(u32::from(byte - b'0')))
                .ok_or_else(|| self.error("vertex index exceeds u32 range"))?;
            self.cursor += 1;
        }

        if self.cursor == start {
            return Err(self.error("expected vertex index"));
        }

        Ok(value)
    }

    fn expect_byte(&mut self, expected: u8) -> Result<()> {
        self.skip_ws();
        match self.peek() {
            Some(byte) if byte == expected => {
                self.cursor += 1;
                Ok(())
            }
            Some(_) => Err(self.error(&format!("expected '{}'", char::from(expected)))),
            None => Err(self.error("unexpected end of geometry boundaries")),
        }
    }

    fn consume_if(&mut self, expected: u8) -> bool {
        self.skip_ws();
        if self.peek() == Some(expected) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.cursor += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.cursor).copied()
    }

    fn error(&self, message: &str) -> Error {
        Error::InvalidValue(format!(
            "invalid geometry boundaries at byte {}: {message}",
            self.cursor
        ))
    }
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
    let geometry = timed("geometry.build_stream_geometry", || {
        build_stream_geometry(raw, model, resources)
    })?;
    validate_root_vertex_references(&geometry, model.vertices().len())?;
    timed("geometry.add_geometry", || {
        model.add_geometry_unchecked(geometry).map_err(Error::from)
    })
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

fn validate_root_vertex_references<VR, SS>(
    geometry: &Geometry<VR, SS>,
    vertex_count: usize,
) -> Result<()>
where
    VR: cityjson::v2_0::VertexRef,
    SS: StringStorage,
{
    if let Some(instance) = geometry.instance() {
        let reference_point = instance.reference_point().to_usize();
        if reference_point >= vertex_count {
            return Err(Error::InvalidValue(format!(
                "geometry reference point index {reference_point} out of range for {vertex_count} root vertices"
            )));
        }
        return Ok(());
    }

    if let Some(boundaries) = geometry.boundaries() {
        for vertex in boundaries.vertices() {
            let index = vertex.to_usize();
            if index >= vertex_count {
                return Err(Error::InvalidValue(format!(
                    "geometry vertex index {index} out of range for {vertex_count} root vertices"
                )));
            }
        }
    }

    Ok(())
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
    match raw.kind {
        GeometryKind::MultiPoint => build_stream_point_geometry(raw, model),
        GeometryKind::MultiLineString => build_stream_linestring_geometry(raw, model),
        GeometryKind::MultiSurface { composite } => {
            build_stream_surface_geometry(raw, composite, model, resources)
        }
        GeometryKind::Solid => build_stream_solid_geometry(raw, model, resources),
        GeometryKind::MultiSolid { composite } => {
            build_stream_multi_solid_geometry(raw, composite, model, resources)
        }
        GeometryKind::GeometryInstance => build_stream_geometry_instance(&raw, resources),
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
    raw: StreamingGeometry<'de>,
    model: &mut CityModel<u32, SS>,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let boundaries = required_stream_boundaries(raw.boundaries, raw.kind)?;
    build_multi_point_geometry(
        raw.lod,
        boundaries,
        raw.semantics.as_ref(),
        raw.material.as_ref(),
        raw.texture.as_ref(),
        model,
    )
}

fn build_stream_linestring_geometry<'de, SS>(
    raw: StreamingGeometry<'de>,
    model: &mut CityModel<u32, SS>,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let boundaries = required_stream_boundaries(raw.boundaries, raw.kind)?;
    build_multi_line_string_geometry(
        raw.lod,
        boundaries,
        raw.semantics.as_ref(),
        raw.material.as_ref(),
        raw.texture.as_ref(),
        model,
    )
}

fn build_stream_surface_geometry<'de, SS>(
    raw: StreamingGeometry<'de>,
    composite: bool,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let boundaries = required_stream_boundaries(raw.boundaries, raw.kind)?;
    build_multi_surface_geometry(
        raw.lod,
        boundaries,
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
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let boundaries = required_stream_boundaries(raw.boundaries, raw.kind)?;
    build_solid_geometry(
        raw.lod,
        boundaries,
        raw.semantics.as_ref(),
        raw.material,
        raw.texture,
        model,
        resources,
    )
}

fn build_stream_multi_solid_geometry<'de, SS>(
    raw: StreamingGeometry<'de>,
    composite: bool,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<Geometry<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let boundaries = required_stream_boundaries(raw.boundaries, raw.kind)?;
    build_multi_solid_geometry(
        raw.lod,
        boundaries,
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
    build_geometry_instance(
        raw.template,
        raw.instance_boundaries.as_deref(),
        raw.transformation_matrix,
        resources,
    )
}

fn required_stream_boundaries(
    boundaries: Option<Boundary<u32>>,
    kind: GeometryKind,
) -> Result<Boundary<u32>> {
    boundaries
        .ok_or_else(|| Error::InvalidValue(format!("{} is missing boundaries", kind.type_name())))
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
        timed("geometry.build_texture_maps", || {
            surface_textures
                .into_iter()
                .map(|(theme, assignments)| {
                    (
                        ThemeName::<SS>::new(SS::store(theme)),
                        build_texture_map(&boundaries, &assignments),
                    )
                })
                .collect::<Vec<_>>()
        })
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
            let attrs: HashMap<&'de str, _> = timed("geometry.semantic_attr_filter", || {
                surface
                    .attributes
                    .iter()
                    .filter(|(k, _)| **k != "type" && **k != "parent" && **k != "children")
                    .map(|(k, v)| (*k, v))
                    .collect::<HashMap<_, _>>()
            });

            if !attrs.is_empty() {
                let attrs_cloned: HashMap<&'de str, _> =
                    timed("geometry.semantic_attr_clone", || {
                        attrs
                            .into_iter()
                            .map(|(k, v)| (k, clone_raw_attribute(v)))
                            .collect()
                    });
                *semantic.attributes_mut() = timed("geometry.semantic_attr_map", || {
                    attribute_map::<SS>(attrs_cloned, "semantic attributes")
                })?;
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
        if let Some(parent_index) = parent
            && let Some(&parent_handle) = usize::try_from(parent_index)
                .ok()
                .and_then(|i| handles.get(i))
        {
            semantic.set_parent(parent_handle);
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
    let semantic_handles = timed("geometry.import_semantics", || {
        import_geometry_semantics::<SS>(semantics, model)
    })?;
    Ok(SurfaceMappings {
        semantics: parse_surface_scalar_assignments(semantics, &semantic_handles, surface_count),
        materials: timed("geometry.parse_material_themes", || {
            parse_material_themes(material, &resources.materials, surface_count)
        })?,
        textures: timed("geometry.parse_texture_themes", || {
            parse_texture_themes(texture, |values| {
                parse_ring_texture_assignments(values, ring_count, resources)
            })
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
                    ));
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

#[cfg(test)]
mod tests {
    use super::{BoundaryParseKind, GeometryKind, StreamingGeometry, parse_boundary_from_raw};
    use serde_json::value::RawValue;

    #[test]
    fn parse_multisurface_boundary_flattens_offsets() {
        let raw = RawValue::from_string("[[[0,1,2,0]],[[3,4,5,3],[6,7,8,6]]]".to_owned())
            .expect("valid raw boundary");
        let boundary =
            parse_boundary_from_raw(&raw, BoundaryParseKind::MultiSurface).expect("parse boundary");

        assert_eq!(
            &*boundary.vertices_raw(),
            &[0, 1, 2, 0, 3, 4, 5, 3, 6, 7, 8, 6]
        );
        assert_eq!(&*boundary.rings_raw(), &[0, 4, 8]);
        assert_eq!(&*boundary.surfaces_raw(), &[0, 1]);
        assert!(boundary.shells().is_empty());
        assert!(boundary.solids().is_empty());
    }

    #[test]
    fn deserialize_streaming_geometry_accepts_boundaries_before_type() {
        let raw = r#"{
            "boundaries": [[[0,1,2,0]]],
            "lod": "2",
            "type": "MultiSurface"
        }"#;
        let geometry: StreamingGeometry<'_> =
            serde_json::from_str(raw).expect("parse streamed geometry");

        assert!(matches!(
            geometry.kind,
            GeometryKind::MultiSurface { composite: false }
        ));
        assert_eq!(geometry.lod, Some("2"));
        let boundary = geometry.boundaries.expect("parsed boundary");
        assert_eq!(&*boundary.vertices_raw(), &[0, 1, 2, 0]);
        assert_eq!(&*boundary.rings_raw(), &[0]);
        assert_eq!(&*boundary.surfaces_raw(), &[0]);
    }
}
