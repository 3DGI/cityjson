pub mod materials;
pub mod semantics;
pub mod textures;

use crate::cityjson::core::boundary::BoundaryType;
use crate::cityjson::core::vertex::{VertexIndex, VertexRef};
use crate::resources::pool::ResourceRef;

pub use materials::MaterialMap;
pub use semantics::SemanticMap;
pub use textures::TextureMap;

#[repr(C)]
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct SemanticOrMaterialMap<VR: VertexRef, RR: ResourceRef> {
    pub(crate) points: Vec<Option<RR>>,
    pub(crate) linestrings: Vec<Option<RR>>,
    pub(crate) surfaces: Vec<Option<RR>>,
    pub(crate) shells: Vec<VertexIndex<VR>>,
    pub(crate) solids: Vec<VertexIndex<VR>>,
}

impl<VR: VertexRef, RR: ResourceRef> SemanticOrMaterialMap<VR, RR> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.points.is_empty()
            && self.linestrings.is_empty()
            && self.surfaces.is_empty()
            && self.shells.is_empty()
            && self.solids.is_empty()
    }

    pub(crate) fn add_point(&mut self, resource: Option<RR>) {
        self.points.push(resource);
    }

    pub(crate) fn add_linestring(&mut self, resource: Option<RR>) {
        self.linestrings.push(resource);
    }

    pub(crate) fn add_surface(&mut self, resource: Option<RR>) {
        self.surfaces.push(resource);
    }

    pub(crate) fn add_shell(&mut self, shell_index: VertexIndex<VR>) {
        self.shells.push(shell_index);
    }

    pub(crate) fn add_solid(&mut self, solid_index: VertexIndex<VR>) {
        self.solids.push(solid_index);
    }

    pub(crate) fn points(&self) -> &[Option<RR>] {
        &self.points
    }

    pub(crate) fn linestrings(&self) -> &[Option<RR>] {
        &self.linestrings
    }

    pub(crate) fn surfaces(&self) -> &[Option<RR>] {
        &self.surfaces
    }

    pub(crate) fn shells(&self) -> &[VertexIndex<VR>] {
        &self.shells
    }

    pub(crate) fn solids(&self) -> &[VertexIndex<VR>] {
        &self.solids
    }

    pub(crate) fn check_type(&self) -> BoundaryType {
        if !self.solids.is_empty() {
            BoundaryType::MultiOrCompositeSolid
        } else if !self.shells.is_empty() {
            BoundaryType::Solid
        } else if !self.surfaces.is_empty() {
            BoundaryType::MultiOrCompositeSurface
        } else if !self.linestrings.is_empty() {
            BoundaryType::MultiLineString
        } else if !self.points.is_empty() {
            BoundaryType::MultiPoint
        } else {
            BoundaryType::None
        }
    }
}

macro_rules! define_typed_resource_map {
    ($name:ident, $handle:ty) => {
        #[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
        pub struct $name<VR: crate::cityjson::core::vertex::VertexRef> {
            inner: crate::resources::mapping::SemanticOrMaterialMap<
                VR,
                crate::resources::pool::ResourceId32,
            >,
        }

        impl<VR: crate::cityjson::core::vertex::VertexRef> $name<VR> {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn is_empty(&self) -> bool {
                self.inner.is_empty()
            }

            pub fn add_point(&mut self, resource: Option<$handle>) {
                self.inner.add_point(resource.map(|r| r.to_raw()));
            }

            pub fn add_linestring(&mut self, resource: Option<$handle>) {
                self.inner.add_linestring(resource.map(|r| r.to_raw()));
            }

            pub fn add_surface(&mut self, resource: Option<$handle>) {
                self.inner.add_surface(resource.map(|r| r.to_raw()));
            }

            pub fn add_shell(
                &mut self,
                shell_index: crate::cityjson::core::vertex::VertexIndex<VR>,
            ) {
                self.inner.add_shell(shell_index);
            }

            pub fn add_solid(
                &mut self,
                solid_index: crate::cityjson::core::vertex::VertexIndex<VR>,
            ) {
                self.inner.add_solid(solid_index);
            }

            pub fn points(&self) -> Vec<Option<$handle>> {
                self.inner
                    .points()
                    .iter()
                    .copied()
                    .map(|r| r.map(|x| <$handle>::from_raw(x)))
                    .collect()
            }

            pub fn linestrings(&self) -> Vec<Option<$handle>> {
                self.inner
                    .linestrings()
                    .iter()
                    .copied()
                    .map(|r| r.map(|x| <$handle>::from_raw(x)))
                    .collect()
            }

            pub fn surfaces(&self) -> Vec<Option<$handle>> {
                self.inner
                    .surfaces()
                    .iter()
                    .copied()
                    .map(|r| r.map(|x| <$handle>::from_raw(x)))
                    .collect()
            }

            pub fn shells(&self) -> &[crate::cityjson::core::vertex::VertexIndex<VR>] {
                self.inner.shells()
            }

            pub fn solids(&self) -> &[crate::cityjson::core::vertex::VertexIndex<VR>] {
                self.inner.solids()
            }

            pub fn check_type(&self) -> crate::cityjson::core::boundary::BoundaryType {
                self.inner.check_type()
            }

            pub(crate) fn from_raw(
                inner: crate::resources::mapping::SemanticOrMaterialMap<
                    VR,
                    crate::resources::pool::ResourceId32,
                >,
            ) -> Self {
                Self { inner }
            }

            pub(crate) fn to_raw(
                &self,
            ) -> &crate::resources::mapping::SemanticOrMaterialMap<
                VR,
                crate::resources::pool::ResourceId32,
            > {
                &self.inner
            }
        }
    };
}

pub(crate) use define_typed_resource_map;
