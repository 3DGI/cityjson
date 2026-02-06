use crate::cityjson::core::vertex::{VertexIndex, VertexRef};
use crate::resources::handles::TextureRef;
use crate::resources::pool::{ResourceId32, ResourceRef};

#[repr(C)]
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct TextureMapCore<VR: VertexRef, RR: ResourceRef> {
    vertices: Vec<Option<VertexIndex<VR>>>,
    rings: Vec<VertexIndex<VR>>,
    ring_textures: Vec<Option<RR>>,
    surfaces: Vec<VertexIndex<VR>>,
    shells: Vec<VertexIndex<VR>>,
    solids: Vec<VertexIndex<VR>>,
}

impl<VR: VertexRef, RR: ResourceRef> TextureMapCore<VR, RR> {
    pub(crate) fn with_capacity(
        vertex_capacity: usize,
        ring_capacity: usize,
        ring_texture_capacity: usize,
        surface_capacity: usize,
        shell_capacity: usize,
        solid_capacity: usize,
    ) -> Self {
        Self {
            vertices: Vec::with_capacity(vertex_capacity),
            rings: Vec::with_capacity(ring_capacity),
            ring_textures: Vec::with_capacity(ring_texture_capacity),
            surfaces: Vec::with_capacity(surface_capacity),
            shells: Vec::with_capacity(shell_capacity),
            solids: Vec::with_capacity(solid_capacity),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.vertices.is_empty()
            && self.rings.is_empty()
            && self.ring_textures.is_empty()
            && self.surfaces.is_empty()
            && self.shells.is_empty()
            && self.solids.is_empty()
    }

    pub(crate) fn add_vertex(&mut self, vertex: Option<VertexIndex<VR>>) {
        self.vertices.push(vertex);
    }

    pub(crate) fn add_ring(&mut self, ring_start: VertexIndex<VR>) {
        self.rings.push(ring_start);
    }

    pub(crate) fn add_ring_texture(&mut self, texture: Option<RR>) {
        self.ring_textures.push(texture);
    }

    pub(crate) fn add_surface(&mut self, surface_start: VertexIndex<VR>) {
        self.surfaces.push(surface_start);
    }

    pub(crate) fn add_shell(&mut self, shell_start: VertexIndex<VR>) {
        self.shells.push(shell_start);
    }

    pub(crate) fn add_solid(&mut self, solid_start: VertexIndex<VR>) {
        self.solids.push(solid_start);
    }

    pub(crate) fn vertices(&self) -> &[Option<VertexIndex<VR>>] {
        &self.vertices
    }

    pub(crate) fn vertices_mut(&mut self) -> &mut [Option<VertexIndex<VR>>] {
        &mut self.vertices
    }

    pub(crate) fn rings(&self) -> &[VertexIndex<VR>] {
        &self.rings
    }

    pub(crate) fn rings_mut(&mut self) -> &mut [VertexIndex<VR>] {
        &mut self.rings
    }

    pub(crate) fn ring_textures(&self) -> &[Option<RR>] {
        &self.ring_textures
    }

    pub(crate) fn ring_textures_mut(&mut self) -> &mut [Option<RR>] {
        &mut self.ring_textures
    }

    pub(crate) fn surfaces(&self) -> &[VertexIndex<VR>] {
        &self.surfaces
    }

    pub(crate) fn surfaces_mut(&mut self) -> &mut [VertexIndex<VR>] {
        &mut self.surfaces
    }

    pub(crate) fn shells(&self) -> &[VertexIndex<VR>] {
        &self.shells
    }

    pub(crate) fn shells_mut(&mut self) -> &mut [VertexIndex<VR>] {
        &mut self.shells
    }

    pub(crate) fn solids(&self) -> &[VertexIndex<VR>] {
        &self.solids
    }

    pub(crate) fn solids_mut(&mut self) -> &mut [VertexIndex<VR>] {
        &mut self.solids
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct TextureMap<VR: VertexRef> {
    inner: TextureMapCore<VR, ResourceId32>,
}

impl<VR: VertexRef> TextureMap<VR> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(
        vertex_capacity: usize,
        ring_capacity: usize,
        ring_texture_capacity: usize,
        surface_capacity: usize,
        shell_capacity: usize,
        solid_capacity: usize,
    ) -> Self {
        Self {
            inner: TextureMapCore::with_capacity(
                vertex_capacity,
                ring_capacity,
                ring_texture_capacity,
                surface_capacity,
                shell_capacity,
                solid_capacity,
            ),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn add_vertex(&mut self, vertex: Option<VertexIndex<VR>>) {
        self.inner.add_vertex(vertex);
    }

    pub fn add_ring(&mut self, ring_start: VertexIndex<VR>) {
        self.inner.add_ring(ring_start);
    }

    pub fn add_ring_texture(&mut self, texture: Option<TextureRef>) {
        self.inner.add_ring_texture(texture.map(|t| t.to_raw()));
    }

    pub fn add_surface(&mut self, surface_start: VertexIndex<VR>) {
        self.inner.add_surface(surface_start);
    }

    pub fn add_shell(&mut self, shell_start: VertexIndex<VR>) {
        self.inner.add_shell(shell_start);
    }

    pub fn add_solid(&mut self, solid_start: VertexIndex<VR>) {
        self.inner.add_solid(solid_start);
    }

    pub fn vertices(&self) -> &[Option<VertexIndex<VR>>] {
        self.inner.vertices()
    }

    pub fn vertices_mut(&mut self) -> &mut [Option<VertexIndex<VR>>] {
        self.inner.vertices_mut()
    }

    pub fn rings(&self) -> &[VertexIndex<VR>] {
        self.inner.rings()
    }

    pub fn rings_mut(&mut self) -> &mut [VertexIndex<VR>] {
        self.inner.rings_mut()
    }

    pub fn ring_textures(&self) -> Vec<Option<TextureRef>> {
        self.inner
            .ring_textures()
            .iter()
            .copied()
            .map(|r| r.map(TextureRef::from_raw))
            .collect()
    }

    pub fn set_ring_texture(&mut self, ring_index: usize, texture: Option<TextureRef>) -> bool {
        let Some(slot) = self.inner.ring_textures_mut().get_mut(ring_index) else {
            return false;
        };
        *slot = texture.map(|t| t.to_raw());
        true
    }

    pub fn surfaces(&self) -> &[VertexIndex<VR>] {
        self.inner.surfaces()
    }

    pub fn surfaces_mut(&mut self) -> &mut [VertexIndex<VR>] {
        self.inner.surfaces_mut()
    }

    pub fn shells(&self) -> &[VertexIndex<VR>] {
        self.inner.shells()
    }

    pub fn shells_mut(&mut self) -> &mut [VertexIndex<VR>] {
        self.inner.shells_mut()
    }

    pub fn solids(&self) -> &[VertexIndex<VR>] {
        self.inner.solids()
    }

    pub fn solids_mut(&mut self) -> &mut [VertexIndex<VR>] {
        self.inner.solids_mut()
    }

    pub(crate) fn from_raw(inner: TextureMapCore<VR, ResourceId32>) -> Self {
        Self { inner }
    }

    pub(crate) fn to_raw(&self) -> &TextureMapCore<VR, ResourceId32> {
        &self.inner
    }
}
