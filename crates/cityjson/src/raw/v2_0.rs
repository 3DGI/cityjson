use crate::raw::{RawAccess, RawPoolView};
use crate::resources::storage::StringStorage;
use crate::v2_0::appearance::material::Material;
use crate::v2_0::appearance::texture::Texture;
use crate::v2_0::coordinate::{RealWorldCoordinate, UVCoordinate};
use crate::v2_0::geometry::semantic::Semantic;
use crate::v2_0::vertex::VertexRef;
use crate::v2_0::{CityModel, CityObjects, Geometry};

/// Raw accessor for zero-copy access to internal `CityModel` storage.
pub struct CityModelRawAccessor<'a, VR: VertexRef, SS: StringStorage> {
    model: &'a CityModel<VR, SS>,
}

impl<'a, VR: VertexRef, SS: StringStorage> CityModelRawAccessor<'a, VR, SS> {
    pub(crate) fn new(model: &'a CityModel<VR, SS>) -> Self {
        Self { model }
    }

    #[inline]
    #[must_use]
    pub fn vertices(&self) -> &'a [RealWorldCoordinate] {
        self.model.vertices().as_slice()
    }

    #[inline]
    #[must_use]
    pub fn geometries(&self) -> RawPoolView<'a, Geometry<VR, SS>> {
        self.model.geometries_raw()
    }

    #[inline]
    #[must_use]
    pub fn semantics(&self) -> RawPoolView<'a, Semantic<SS>> {
        self.model.semantics_raw()
    }

    #[inline]
    #[must_use]
    pub fn materials(&self) -> RawPoolView<'a, Material<SS>> {
        self.model.materials_raw()
    }

    #[inline]
    #[must_use]
    pub fn textures(&self) -> RawPoolView<'a, Texture<SS>> {
        self.model.textures_raw()
    }

    #[inline]
    #[must_use]
    pub fn cityobjects(&self) -> &'a CityObjects<SS> {
        self.model.cityobjects()
    }

    #[inline]
    #[must_use]
    pub fn template_vertices(&self) -> &'a [RealWorldCoordinate] {
        self.model.template_vertices().as_slice()
    }

    #[inline]
    #[must_use]
    pub fn uv_coordinates(&self) -> &'a [UVCoordinate] {
        self.model.vertices_texture().as_slice()
    }
}
