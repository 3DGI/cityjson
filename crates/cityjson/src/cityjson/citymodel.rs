use crate::cityjson::attributes::Attributes;
use crate::cityjson::coordinate::{RealWorldCoordinate, UVCoordinate, Vertices};
use crate::cityjson::geometry::GeometryTrait;
use crate::cityjson::index::{VertexIndex, VertexRef};
use crate::cityjson::material::Material;
use crate::cityjson::semantic::Semantic;
use crate::cityjson::storage::StringStorage;
use crate::cityjson::texture::Texture;
use crate::errors;
use crate::resources::pool::{ResourcePool, ResourceRef};
use std::marker::PhantomData;

#[derive(Debug)]
pub struct GenericCityModel<VR, RR, RPS, RPM, RPT, SS, Geo, Mat, Sem, Tex>
where
    VR: VertexRef,
    RR: ResourceRef,
    RPS: ResourcePool<Sem, RR>,
    RPM: ResourcePool<Mat, RR>,
    RPT: ResourcePool<Tex, RR>,
    SS: StringStorage,
    Geo: GeometryTrait<VR, RR, SS>,
    Sem: Semantic<RR, SS>,
    Mat: Material<SS>,
    Tex: Texture<SS>,
{
    /// Pool of vertex coordinates
    vertices: Vertices<VR, RealWorldCoordinate>,
    /// Pool of semantic objects
    semantics: RPS,
    /// Pool of material objects
    materials: RPM,
    /// Pool of texture objects
    textures: RPT,
    vertices_texture: Vertices<VR, UVCoordinate>,
    /// Collection of geometries
    pub(crate) geometries: Vec<Geo>,
    extra: Option<Attributes<SS>>,
    _phantom_rr: PhantomData<RR>,
    _phantom_sem: PhantomData<Sem>,
    _phantom_mat: PhantomData<Mat>,
    _phantom_tex: PhantomData<Tex>,
}

impl<VR, RR, RPS, RPM, RPT, SS, Geo, Mat, Sem, Tex>
    GenericCityModel<VR, RR, RPS, RPM, RPT, SS, Geo, Mat, Sem, Tex>
where
    VR: VertexRef,
    RR: ResourceRef,
    RPS: ResourcePool<Sem, RR>,
    RPM: ResourcePool<Mat, RR>,
    RPT: ResourcePool<Tex, RR>,
    SS: StringStorage,
    Geo: GeometryTrait<VR, RR, SS>,
    Mat: Material<SS>,
    Sem: Semantic<RR, SS>,
    Tex: Texture<SS>,
{
    /// Create a new empty CityModel
    pub fn new() -> Self {
        Self {
            vertices: Vertices::new(),
            semantics: RPS::new(),
            materials: RPM::new(),
            textures: RPT::new(),
            vertices_texture: Vertices::new(),
            geometries: Vec::new(),
            extra: None,
            _phantom_rr: Default::default(),
            _phantom_sem: Default::default(),
            _phantom_mat: Default::default(),
            _phantom_tex: Default::default(),
        }
    }

    /// Create a new CityModel with the specified capacity
    pub fn with_capacity(
        _vertex_capacity: usize,
        semantic_capacity: usize,
        material_capacity: usize,
        texture_capacity: usize,
        geometry_capacity: usize,
    ) -> Self {
        Self {
            vertices: Vertices::new(),
            semantics: RPS::with_capacity(semantic_capacity),
            materials: RPM::with_capacity(material_capacity),
            textures: RPT::with_capacity(texture_capacity),
            vertices_texture: Vertices::new(),
            geometries: Vec::with_capacity(geometry_capacity),
            extra: None,
            _phantom_rr: Default::default(),
            _phantom_sem: Default::default(),
            _phantom_mat: Default::default(),
            _phantom_tex: Default::default(),
        }
    }

    /// Add a semantic object to the pool
    pub fn add_semantic(&mut self, semantic: Sem) -> RR {
        self.semantics.add(semantic)
    }

    /// Get a reference to a semantic object
    pub fn get_semantic(&self, id: RR) -> Option<&Sem> {
        self.semantics.get(id)
    }

    /// Get a mutable reference to a semantic object
    pub fn get_semantic_mut(&mut self, id: RR) -> Option<&mut Sem> {
        self.semantics.get_mut(id)
    }

    pub fn add_material(&mut self, material: Mat) -> RR {
        self.materials.add(material)
    }

    pub fn get_material(&self, id: RR) -> Option<&Mat> {
        self.materials.get(id)
    }

    pub fn get_material_mut(&mut self, id: RR) -> Option<&mut Mat> {
        self.materials.get_mut(id)
    }

    pub fn add_texture(&mut self, texture: Tex) -> RR {
        self.textures.add(texture)
    }

    pub fn get_texture(&self, id: RR) -> Option<&Tex> {
        self.textures.get(id)
    }

    pub fn get_texture_mut(&mut self, id: RR) -> Option<&mut Tex> {
        self.textures.get_mut(id)
    }

    /// Add a geometry to the model
    pub fn add_geometry(&mut self, geometry: Geo) {
        self.geometries.push(geometry);
    }

    /// Add a vertex coordinate
    pub fn add_vertex(
        &mut self,
        coordinate: RealWorldCoordinate,
    ) -> errors::Result<VertexIndex<VR>> {
        self.vertices.push(coordinate)
    }

    /// Get a reference to a vertex coordinate
    pub fn get_vertex(&self, index: VertexIndex<VR>) -> Option<&RealWorldCoordinate> {
        self.vertices.get(index)
    }

    /// Get the number of geometries
    pub fn geometry_count(&self) -> usize {
        self.geometries.len()
    }

    /// Get the number of semantics
    pub fn semantic_count(&self) -> usize {
        self.semantics.iter().count()
    }

    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.as_slice().len()
    }
}

// Implement default for convenience
impl<VR, RR, RPS, RPM, RPT, SS, Geo, Mat, Sem, Tex>
    GenericCityModel<VR, RR, RPS, RPM, RPT, SS, Geo, Mat, Sem, Tex>
where
    VR: VertexRef,
    RR: ResourceRef,
    RPS: ResourcePool<Sem, RR>,
    RPM: ResourcePool<Mat, RR>,
    RPT: ResourcePool<Tex, RR>,
    SS: StringStorage,
    Geo: GeometryTrait<VR, RR, SS>,
    Mat: Material<SS>,
    Sem: Semantic<RR, SS>,
    Tex: Texture<SS>,
{
    fn default() -> Self {
        Self::new()
    }
}
