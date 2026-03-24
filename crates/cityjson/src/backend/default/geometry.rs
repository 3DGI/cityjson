//! Core geometry storage shared across `CityJSON` versions.
//!
//! This is the crate's normalized geometry representation.
//!
//! Important invariants:
//!
//! - regular geometry and template geometry stay separate in `CityModel`
//!   storage
//! - template vertices are distinct from root vertices
//! - `GeometryInstance` is stored as one explicit value:
//!   - template handle
//!   - reference point in the root vertex store
//!   - transformation matrix
//! - public APIs can diverge from the `CityJSON` wire format as long as the
//!   meaning is preserved exactly

use crate::cityjson::core::boundary::Boundary;
use crate::cityjson::core::vertex::VertexRef;
use crate::error::Error;
use crate::resources::id::ResourceId;
use crate::resources::mapping::SemanticOrMaterialMap;
use crate::resources::mapping::textures::TextureMapCore;
use crate::resources::storage::StringStorage;
use crate::v2_0::vertex::VertexIndex;
use std::str::FromStr;

pub(crate) type ThemedMaterials<VR, RR, SS> = Vec<(
    crate::cityjson::core::appearance::ThemeName<SS>,
    SemanticOrMaterialMap<VR, RR>,
)>;
pub(crate) type ThemedTextures<VR, RR, SS> = Vec<(
    crate::cityjson::core::appearance::ThemeName<SS>,
    TextureMapCore<VR, RR>,
)>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AffineTransform3D([f64; 16]);

impl AffineTransform3D {
    #[must_use]
    pub fn new(matrix: [f64; 16]) -> Self {
        Self(matrix)
    }

    #[must_use]
    pub fn identity() -> Self {
        Self([
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[must_use]
    pub fn as_array(&self) -> &[f64; 16] {
        &self.0
    }

    #[must_use]
    pub fn into_array(self) -> [f64; 16] {
        self.0
    }
}

impl Default for AffineTransform3D {
    fn default() -> Self {
        Self::identity()
    }
}

impl From<[f64; 16]> for AffineTransform3D {
    fn from(value: [f64; 16]) -> Self {
        Self::new(value)
    }
}

impl From<AffineTransform3D> for [f64; 16] {
    fn from(value: AffineTransform3D) -> Self {
        value.into_array()
    }
}

impl AsRef<[f64; 16]> for AffineTransform3D {
    fn as_ref(&self) -> &[f64; 16] {
        self.as_array()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GeometryInstanceData<VR: VertexRef, RR: ResourceId> {
    template: RR,
    reference_point: VertexIndex<VR>,
    transformation: AffineTransform3D,
}

impl<VR: VertexRef, RR: ResourceId> GeometryInstanceData<VR, RR> {
    #[must_use]
    pub(crate) fn new(
        template: RR,
        reference_point: VertexIndex<VR>,
        transformation: AffineTransform3D,
    ) -> Self {
        Self {
            template,
            reference_point,
            transformation,
        }
    }

    pub(crate) fn template(&self) -> &RR {
        &self.template
    }

    pub(crate) fn reference_point(&self) -> &VertexIndex<VR> {
        &self.reference_point
    }

    pub(crate) fn transformation(&self) -> &AffineTransform3D {
        &self.transformation
    }
}

/// Core geometry structure that contains the data for all `CityJSON` versions.
/// Version-specific types wrap this core structure and implement methods via macros.
#[derive(Clone, Debug)]
pub(crate) struct GeometryCore<VR: VertexRef, RR: ResourceId, SS: StringStorage> {
    type_geometry: GeometryType,
    lod: Option<LoD>,
    boundaries: Option<Boundary<VR>>,
    semantics: Option<SemanticOrMaterialMap<VR, RR>>,
    materials: Option<ThemedMaterials<VR, RR, SS>>,
    textures: Option<ThemedTextures<VR, RR, SS>>,
    instance: Option<GeometryInstanceData<VR, RR>>,
}

impl<VR: VertexRef, RR: ResourceId, SS: StringStorage> GeometryCore<VR, RR, SS> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        type_geometry: GeometryType,
        lod: Option<LoD>,
        boundaries: Option<Boundary<VR>>,
        semantics: Option<SemanticOrMaterialMap<VR, RR>>,
        materials: Option<ThemedMaterials<VR, RR, SS>>,
        textures: Option<ThemedTextures<VR, RR, SS>>,
        instance: Option<GeometryInstanceData<VR, RR>>,
    ) -> Self {
        Self {
            type_geometry,
            lod,
            boundaries,
            semantics,
            materials,
            textures,
            instance,
        }
    }

    pub fn type_geometry(&self) -> &GeometryType {
        &self.type_geometry
    }

    pub fn lod(&self) -> Option<&LoD> {
        self.lod.as_ref()
    }

    pub fn boundaries(&self) -> Option<&Boundary<VR>> {
        self.boundaries.as_ref()
    }

    pub(crate) fn semantics(&self) -> Option<&SemanticOrMaterialMap<VR, RR>> {
        self.semantics.as_ref()
    }

    pub(crate) fn materials(&self) -> Option<&ThemedMaterials<VR, RR, SS>> {
        self.materials.as_ref()
    }

    pub(crate) fn textures(&self) -> Option<&ThemedTextures<VR, RR, SS>> {
        self.textures.as_ref()
    }

    pub(crate) fn instance(&self) -> Option<&GeometryInstanceData<VR, RR>> {
        self.instance.as_ref()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[non_exhaustive]
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
        write!(f, "{self:?}")
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

/// Level of Detail (`LoD`) for the geometry
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
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
