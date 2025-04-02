use crate::prelude::*;
use crate::v1_0::appearance::material::Material;
use crate::v1_0::appearance::texture::Texture;
use crate::v1_0::geometry::semantic::{Semantic, SemanticType};
use crate::v1_0::geometry::Geometry;
use crate::v1_0::metadata::Metadata;
use crate::v1_0::{
    CityObject, CityObjectType, CityObjects, Extension, Extensions, Transform,
};
use crate::{format_option, CityJSONVersion, CityModelType};
use std::fmt;
use std::marker::PhantomData;

pub struct V1_0<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    _phantom_vr: PhantomData<VR>,
    _phantom_rr: PhantomData<RR>,
    _phantom_ss: PhantomData<SS>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> CityModelTypes for V1_0<VR, RR, SS> {
    type CoordinateType = FlexibleCoordinate;
    type VertexRef = VR;
    type ResourceRef = RR;
    type StringStorage = SS;
    type SemType = SemanticType;
    type Semantic = Semantic<RR, SS>;
    type Material = Material<SS>;
    type Texture = Texture<SS>;
    type Geometry = Geometry<VR, RR, SS>;
    type Metadata = Metadata<SS, RR>;
    type Transform = Transform;
    type Extension = Extension<SS>;
    type Extensions = Extensions<SS>;
    type CityObjectType = CityObjectType<SS>;
    type BBox = BBox;
    type CityObject = CityObject<SS, RR>;
    type CityObjects = CityObjects<SS, RR>;
    type GeometryPool = DefaultResourcePool<Geometry<VR, RR, SS>, RR>;
    type SemanticPool = DefaultResourcePool<Semantic<RR, SS>, RR>;
    type MaterialPool = DefaultResourcePool<Material<SS>, RR>;
    type TexturePool = DefaultResourcePool<Texture<SS>, RR>;
}

#[derive(Debug, Clone)]
pub struct CityModel<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    /// CityModel type
    type_citymodel: CityModelType,
    /// CityJSON version
    version: Option<CityJSONVersion>,
    /// CityJSON Extension declarations
    extensions: Option<Extensions<SS>>,
    /// Extra root properties for the CityModel
    extra: Option<Attributes<SS, RR>>,
    /// CityModel metadata
    metadata: Option<Metadata<SS, RR>>,
    /// Collection of CityObjects
    cityobjects: CityObjects<SS, RR>,
    /// The transform object
    transform: Option<Transform>,
    /// Pool of vertex coordinates
    vertices: Vertices<VR, FlexibleCoordinate>,
    /// Pool of geometries
    geometries: DefaultResourcePool<Geometry<VR, RR, SS>, RR>,
    /// Pool of vertex coordinates used by the geometry templates in template_geometries
    template_vertices: Vertices<VR, RealWorldCoordinate>,
    /// Pool of geometry templates
    template_geometries: DefaultResourcePool<Geometry<VR, RR, SS>, RR>,
    /// Pool of semantic objects
    semantics: DefaultResourcePool<Semantic<RR, SS>, RR>,
    /// Pool of material objects
    materials: DefaultResourcePool<Material<SS>, RR>,
    /// Pool of texture objects
    textures: DefaultResourcePool<Texture<SS>, RR>,
    /// Pool of vertex textures (UV coordinates)
    vertices_texture: Vertices<VR, UVCoordinate>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> CityModelTrait<V1_0<VR, RR, SS>>
    for CityModel<VR, RR, SS>
{
    fn new(type_citymodel: CityModelType) -> Self {
        Self {
            type_citymodel,
            version: Some(CityJSONVersion::V1_0),
            extensions: None,
            extra: None,
            metadata: None,
            cityobjects: CityObjects::new(),
            transform: None,
            vertices: Vertices::new(),
            geometries: DefaultResourcePool::new_pool(),
            template_vertices: Vertices::new(),
            template_geometries: DefaultResourcePool::new_pool(),
            semantics: DefaultResourcePool::new_pool(),
            materials: DefaultResourcePool::new_pool(),
            textures: DefaultResourcePool::new_pool(),
            vertices_texture: Vertices::new(),
        }
    }

    fn with_capacity(
        type_citymodel: CityModelType,
        cityobjects_capacity: usize,
        vertex_capacity: usize,
        semantic_capacity: usize,
        material_capacity: usize,
        texture_capacity: usize,
        geometry_capacity: usize,
    ) -> Self {
        Self {
            type_citymodel,
            version: Some(CityJSONVersion::V1_0),
            extensions: None,
            extra: None,
            metadata: None,
            cityobjects: CityObjects::with_capacity(cityobjects_capacity),
            transform: None,
            vertices: Vertices::with_capacity(vertex_capacity),
            geometries: DefaultResourcePool::with_capacity(geometry_capacity),
            template_vertices: Vertices::new(),
            template_geometries: DefaultResourcePool::new(),
            semantics: DefaultResourcePool::with_capacity(semantic_capacity),
            materials: DefaultResourcePool::with_capacity(material_capacity),
            textures: DefaultResourcePool::with_capacity(texture_capacity),
            vertices_texture: Vertices::new(),
        }
    }

    fn add_semantic(&mut self, semantic: Semantic<RR, SS>) -> RR {
        self.semantics.add(semantic)
    }

    fn get_semantic(&self, id: RR) -> Option<&Semantic<RR, SS>> {
        self.semantics.get(id)
    }

    fn get_semantic_mut(&mut self, id: RR) -> Option<&mut Semantic<RR, SS>> {
        self.semantics.get_mut(id)
    }

    fn add_material(&mut self, material: Material<SS>) -> RR {
        self.materials.add(material)
    }

    fn get_material(&self, id: RR) -> Option<&Material<SS>> {
        self.materials.get(id)
    }

    fn get_material_mut(&mut self, id: RR) -> Option<&mut Material<SS>> {
        self.materials.get_mut(id)
    }

    fn add_texture(&mut self, texture: Texture<SS>) -> RR {
        self.textures.add(texture)
    }

    fn get_texture(&self, id: RR) -> Option<&Texture<SS>> {
        self.textures.get(id)
    }

    fn get_texture_mut(&mut self, id: RR) -> Option<&mut Texture<SS>> {
        self.textures.get_mut(id)
    }

    fn add_geometry(&mut self, geometry: Geometry<VR, RR, SS>) -> RR {
        self.geometries.add(geometry)
    }

    fn geometries(&self) -> &DefaultResourcePool<Geometry<VR, RR, SS>, RR> {
        &self.geometries
    }

    fn geometries_mut(&mut self) -> &mut DefaultResourcePool<Geometry<VR, RR, SS>, RR> {
        &mut self.geometries
    }

    fn vertices(&self) -> &Vertices<VR, FlexibleCoordinate> {
        &self.vertices
    }

    fn vertices_mut(&mut self) -> &mut Vertices<VR, FlexibleCoordinate> {
        &mut self.vertices
    }

    fn add_vertex(&mut self, coordinate: FlexibleCoordinate) -> Result<VertexIndex<VR>> {
        self.vertices.push(coordinate)
    }

    fn get_vertex(&self, index: VertexIndex<VR>) -> Option<&FlexibleCoordinate> {
        self.vertices.get(index)
    }

    fn geometry_count(&self) -> usize {
        self.geometries.len()
    }

    fn semantic_count(&self) -> usize {
        self.semantics.len()
    }

    fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    fn metadata(&self) -> Option<&Metadata<SS, RR>> {
        self.metadata.as_ref()
    }

    fn metadata_mut(&mut self) -> &mut Metadata<SS, RR> {
        if self.metadata.is_none() {
            self.metadata = Some(Metadata::new());
        }
        self.metadata.as_mut().unwrap()
    }

    fn extra(&self) -> Option<&Attributes<SS, RR>> {
        self.extra.as_ref()
    }

    fn extra_mut(&mut self) -> &mut Attributes<SS, RR> {
        if self.extra.is_none() {
            self.extra = Some(Attributes::new());
        }
        self.extra.as_mut().unwrap()
    }

    fn transform(&self) -> Option<&Transform> {
        self.transform.as_ref()
    }

    fn transform_mut(&mut self) -> &mut Transform {
        if self.transform.is_none() {
            self.transform = Some(Transform::new());
        }
        self.transform.as_mut().unwrap()
    }

    fn extensions(&self) -> Option<&Extensions<SS>> {
        self.extensions.as_ref()
    }

    fn extensions_mut(&mut self) -> &mut Extensions<SS> {
        if self.extensions.is_none() {
            self.extensions = Some(Extensions::new());
        }
        self.extensions.as_mut().unwrap()
    }

    fn cityobjects(&self) -> &CityObjects<SS, RR> {
        &self.cityobjects
    }

    fn cityobjects_mut(&mut self) -> &mut CityObjects<SS, RR> {
        &mut self.cityobjects
    }

    fn add_uv_coordinate(&mut self, uvcoordinate: UVCoordinate) -> Result<VertexIndex<VR>> {
        self.vertices_texture.push(uvcoordinate)
    }

    fn get_uv_coordinate(&self, index: VertexIndex<VR>) -> Option<&UVCoordinate> {
        self.vertices_texture.get(index)
    }

    fn add_template_vertex(&mut self, coordinate: RealWorldCoordinate) -> Result<VertexIndex<VR>> {
        self.template_vertices.push(coordinate)
    }

    fn get_template_vertex(&self, index: VertexIndex<VR>) -> Option<&RealWorldCoordinate> {
        self.template_vertices.get(index)
    }

    fn add_template_geometry(&mut self, geometry: Geometry<VR, RR, SS>) -> RR {
        self.template_geometries.add(geometry)
    }

    fn template_geometries(&self) -> &DefaultResourcePool<Geometry<VR, RR, SS>, RR> {
        &self.template_geometries
    }

    fn template_geometries_mut(&mut self) -> &mut DefaultResourcePool<Geometry<VR, RR, SS>, RR> {
        &mut self.template_geometries
    }

    fn template_vertices(&self) -> &Vertices<VR, RealWorldCoordinate> {
        &self.template_vertices
    }

    fn template_vertices_mut(&mut self) -> &mut Vertices<VR, RealWorldCoordinate> {
        &mut self.template_vertices
    }
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> fmt::Display for CityModel<VR, RR, SS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "CityModel {{")?;
        writeln!(f, "\ttype: {}", self.type_citymodel)?;
        writeln!(f, "\tversion: {}", format_option(&self.version))?;
        writeln!(f, "\textensions: {{ {} }}", format_option(&self.extensions))?;
        writeln!(f, "\ttransform: {{ {} }}", format_option(&self.transform))?;
        writeln!(f, "\tmetadata: {}", format_option(&self.metadata))?;
        writeln!(
            f,
            "\tCityObjects: {{ nr. cityobjects: {}, nr. geometries: {} }}",
            self.cityobjects.len(),
            self.geometries.len()
        )?;
        writeln!(f, "\tappearance: {{ nr. materials: {}, nr. textures: {}, nr. vertices-texture: {}, default-theme-texture: {}, default-theme-material: {} }}", self.materials.len(), self.textures.len(), self.vertices_texture.len(), "not implemented", "not implemented")?;
        writeln!(f, "\tgeometry-templates: {}", "not implemented")?;
        writeln!(
            f,
            "\tvertices: {{ nr. vertices: {}, quantized coordinates: {} }}",
            self.vertices.len(),
            "not implemented"
        )?;
        writeln!(f, "\textra: {}", format_option(&self.extra))?;
        writeln!(f, "}}")
    }
}
