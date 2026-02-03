//! Geometry types for the nested backend.
//!

use crate::backend::nested::appearance::{MaterialValues, TextureValues};
use crate::backend::nested::nested_boundary::Boundary;
use crate::backend::nested::semantics::Semantics;
use crate::cityjson::core::vertex::VertexRef;
use crate::cityjson::traits::coordinate::Coordinate;
use crate::error::Error;
use crate::prelude::{RealWorldCoordinate, StringStorage, UVCoordinate, VertexIndex, Vertices};
use crate::resources::pool::ResourceRef;
use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

pub use crate::backend::nested::geometry_builder::{BuilderMode, GeometryBuilder};

// Local trait to decouple GeometryBuilder from any global CityModel traits.
pub trait GeometryModelOps<VR, RR, C, Semantic, Material, Texture, Geometry, SS>
where
    VR: VertexRef,
    RR: ResourceRef,
    C: Coordinate,
    SS: StringStorage,
{
    fn add_semantic(&mut self, semantic: Semantic) -> RR;
    fn get_or_insert_semantic(&mut self, semantic: Semantic) -> RR;
    fn add_material(&mut self, material: Material) -> RR;
    fn get_or_insert_material(&mut self, material: Material) -> RR;
    fn add_texture(&mut self, texture: Texture) -> RR;
    fn get_or_insert_texture(&mut self, texture: Texture) -> RR;
    fn add_uv_coordinate(&mut self, uvcoordinate: UVCoordinate) -> crate::prelude::Result<VertexIndex<VR>>;

    fn add_geometry(&mut self, geometry: Geometry) -> RR;
    fn add_template_geometry(&mut self, geometry: Geometry) -> RR;

    fn add_vertex(&mut self, coordinate: C) -> crate::prelude::Result<VertexIndex<VR>>;
    fn vertices_mut(&mut self) -> &mut Vertices<VR, C>;

    fn add_template_vertex(&mut self, coordinate: RealWorldCoordinate)
        -> crate::prelude::Result<VertexIndex<VR>>;
    fn template_vertices_mut(&mut self) -> &mut Vertices<VR, RealWorldCoordinate>;
}

// Trait for geometry construction (used by versioned Geometry wrappers).
pub trait GeometryConstructor<VR: VertexRef, RR: ResourceRef, SS> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        type_geometry: GeometryType,
        lod: Option<LoD>,
        boundaries: Option<crate::cityjson::core::boundary::Boundary<VR>>,
        semantics: Option<crate::resources::mapping::SemanticMap<VR, RR>>,
        materials: Option<Vec<(SS, crate::resources::mapping::MaterialMap<VR, RR>)>>,
        textures: Option<Vec<(SS, crate::resources::mapping::TextureMap<VR, RR>)>>,
        instance_template: Option<RR>,
        instance_reference_point: Option<VertexIndex<VR>>,
        instance_transformation_matrix: Option<[f64; 16]>,
    ) -> Self;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Geometry<SS: StringStorage, RR> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary>,
    semantics: Option<Semantics<SS, RR>>,
    materials: Option<HashMap<String, MaterialValues>>,
    textures: Option<HashMap<String, TextureValues>>,
    instance_template: Option<usize>,
    instance_reference_point: Option<RealWorldCoordinate>,
    instance_transformation_matrix: Option<[f64; 16]>,
    _marker: PhantomData<RR>,
}

impl<SS: StringStorage, RR> Geometry<SS, RR> {
    // Constructor
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        type_geometry: GeometryType,
        lod: Option<LoD>,
        boundaries: Option<Boundary>,
        semantics: Option<Semantics<SS, RR>>,
        materials: Option<HashMap<String, MaterialValues>>,
        textures: Option<HashMap<String, TextureValues>>,
        instance_template: Option<usize>,
        instance_reference_point: Option<RealWorldCoordinate>,
        instance_transformation_matrix: Option<[f64; 16]>,
    ) -> Self {
        Self {
            type_geometry,
            lod,
            boundaries,
            semantics,
            materials,
            textures,
            instance_template,
            instance_reference_point,
            instance_transformation_matrix,
            _marker: PhantomData,
        }
    }

    // Getters
    pub fn type_geometry(&self) -> &GeometryType {
        &self.type_geometry
    }

    pub fn lod(&self) -> Option<&LoD> {
        self.lod.as_ref()
    }

    pub fn boundaries(&self) -> Option<&Boundary> {
        self.boundaries.as_ref()
    }

    pub fn semantics(&self) -> Option<&Semantics<SS, RR>> {
        self.semantics.as_ref()
    }

    pub fn materials(&self) -> Option<&HashMap<String, MaterialValues>> {
        self.materials.as_ref()
    }

    pub fn textures(&self) -> Option<&HashMap<String, TextureValues>> {
        self.textures.as_ref()
    }

    pub fn instance_template(&self) -> Option<usize> {
        self.instance_template
    }

    pub fn instance_reference_point(&self) -> Option<&RealWorldCoordinate> {
        self.instance_reference_point.as_ref()
    }

    pub fn instance_transformation_matrix(&self) -> Option<&[f64; 16]> {
        self.instance_transformation_matrix.as_ref()
    }
}

impl<SS: StringStorage + std::fmt::Debug, RR: std::fmt::Debug> Display for Geometry<SS, RR> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ==================== CORE GEOMETRY ENUMS ====================

#[repr(C)]
#[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum GeometryType {
    MultiPoint,
    MultiLineString,
    MultiSurface,
    CompositeSurface,
    Solid,
    MultiSolid,
    CompositeSolid,
    GeometryInstance,
}

impl std::fmt::Display for GeometryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for GeometryType {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "MultiPoint" => Ok(GeometryType::MultiPoint),
            "MultiLineString" => Ok(GeometryType::MultiLineString),
            "MultiSurface" => Ok(GeometryType::MultiSurface),
            "CompositeSurface" => Ok(GeometryType::CompositeSurface),
            "Solid" => Ok(GeometryType::Solid),
            "MultiSolid" => Ok(GeometryType::MultiSolid),
            "CompositeSolid" => Ok(GeometryType::CompositeSolid),
            "GeometryInstance" => Ok(GeometryType::GeometryInstance),
            &_ => Err(Error::InvalidGeometryType {
                expected: "one of MultiPoint, MultiLineString, MultiSurface, CompositeSurface, Solid, MultiSolid, CompositeSolid, GeometryInstance"
                    .to_string(),
                found: s.to_string(),
            }),
        }
    }
}

/// Level of Detail (LoD) for the geometry.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LoD {
    LoD0,
    LoD0_0,
    LoD0_1,
    LoD0_2,
    LoD0_3,
    LoD1,
    LoD1_0,
    LoD1_1,
    LoD1_2,
    LoD1_3,
    LoD2,
    LoD2_0,
    LoD2_1,
    LoD2_2,
    LoD2_3,
    LoD3,
    LoD3_0,
    LoD3_1,
    LoD3_2,
    LoD3_3,
}

impl std::fmt::Display for LoD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            LoD::LoD0 => write!(f, "0"),
            LoD::LoD0_0 => write!(f, "0.0"),
            LoD::LoD0_1 => write!(f, "0.1"),
            LoD::LoD0_2 => write!(f, "0.2"),
            LoD::LoD0_3 => write!(f, "0.3"),
            LoD::LoD1 => write!(f, "1"),
            LoD::LoD1_0 => write!(f, "1.0"),
            LoD::LoD1_1 => write!(f, "1.1"),
            LoD::LoD1_2 => write!(f, "1.2"),
            LoD::LoD1_3 => write!(f, "1.3"),
            LoD::LoD2 => write!(f, "2"),
            LoD::LoD2_0 => write!(f, "2.0"),
            LoD::LoD2_1 => write!(f, "2.1"),
            LoD::LoD2_2 => write!(f, "2.2"),
            LoD::LoD2_3 => write!(f, "2.3"),
            LoD::LoD3 => write!(f, "3"),
            LoD::LoD3_0 => write!(f, "3.0"),
            LoD::LoD3_1 => write!(f, "3.1"),
            LoD::LoD3_2 => write!(f, "3.2"),
            LoD::LoD3_3 => write!(f, "3.3"),
        }
    }
}
