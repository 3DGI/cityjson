use crate::cityjson::core::attributes::Attributes;
use crate::cityjson::core::coordinate::{UVCoordinate, Vertices};
use crate::cityjson::core::metadata::BBox;
use crate::cityjson::core::vertex::VertexIndex;
use crate::cityjson::traits::citymodel::{CityModelTrait, CityModelTypes};
use crate::cityjson::traits::transform::TransformTrait;
use crate::cityjson::traits::vertex::VertexRef;
use crate::prelude::{
    CityObjectsTrait, ExtensionsTrait, QuantizedCoordinate, RealWorldCoordinate, Result,
};
use crate::resources::pool::{DefaultResourcePool, ResourcePool, ResourceRef};
use crate::resources::storage::StringStorage;
use crate::v2_0::appearance::material::Material;
use crate::v2_0::appearance::texture::Texture;
use crate::v2_0::geometry::semantic::{Semantic, SemanticType};
use crate::v2_0::geometry::Geometry;
use crate::v2_0::metadata::Metadata;
use crate::v2_0::{CityObject, CityObjectType, CityObjects, Extension, Extensions, Transform};
use crate::{format_option, CityJSONVersion, CityModelType};
use std::fmt;
use std::marker::PhantomData;

pub struct V2_0<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    _phantom_vr: PhantomData<VR>,
    _phantom_rr: PhantomData<RR>,
    _phantom_ss: PhantomData<SS>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> CityModelTypes for V2_0<VR, RR, SS> {
    type CoordinateType = QuantizedCoordinate;
    type VertexRef = VR;
    type ResourceRef = RR;
    type StringStorage = SS;
    type SemType = SemanticType<SS>;
    type Semantic = Semantic<RR, SS>;
    type Material = Material<SS>;
    type Texture = Texture<SS>;
    type Geometry = Geometry<VR, RR, SS>;
    type Metadata = Metadata<RR, SS>;
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
    metadata: Option<Metadata<RR, SS>>,
    /// Collection of CityObjects
    cityobjects: CityObjects<SS, RR>,
    /// The transform object
    transform: Option<Transform>,
    /// Pool of vertex coordinates
    vertices: Vertices<VR, QuantizedCoordinate>,
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
    /// Default theme material reference
    default_theme_material: Option<RR>,
    /// Default theme texture reference
    default_theme_texture: Option<RR>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> CityModelTrait<V2_0<VR, RR, SS>>
    for CityModel<VR, RR, SS>
{
    fn new(type_citymodel: CityModelType) -> Self {
        Self {
            type_citymodel,
            version: Some(CityJSONVersion::V2_0),
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
            default_theme_material: None,
            default_theme_texture: None,
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
            version: Some(CityJSONVersion::V2_0),
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
            default_theme_material: None,
            default_theme_texture: None,
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
    fn get_or_insert_semantic(&mut self, semantic: Semantic<RR, SS>) -> RR
    where
        Semantic<RR, SS>: PartialEq,
    {
        if let Some(existing_id) = self.semantics.find(&semantic) {
            return existing_id;
        }
        self.semantics.add(semantic)
    }
    fn semantics(&self) -> &DefaultResourcePool<Semantic<RR, SS>, RR> {
        &self.semantics
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
    fn get_or_insert_material(&mut self, material: Material<SS>) -> RR
    where
        Material<SS>: PartialEq,
    {
        if let Some(existing_id) = self.materials.find(&material) {
            return existing_id;
        }
        self.materials.add(material)
    }

    fn materials(&self) -> &DefaultResourcePool<Material<SS>, RR> {
        &self.materials
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
    fn get_or_insert_texture(&mut self, texture: Texture<SS>) -> RR
    where
        Texture<SS>: PartialEq,
    {
        if let Some(existing_id) = self.textures.find(&texture) {
            return existing_id;
        }
        self.textures.add(texture)
    }

    fn textures(&self) -> &DefaultResourcePool<Texture<SS>, RR> {
        &self.textures
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
    fn clear_geometries(&mut self) {
        self.geometries.clear();
    }
    fn vertices(&self) -> &Vertices<VR, QuantizedCoordinate> {
        &self.vertices
    }

    fn vertices_mut(&mut self) -> &mut Vertices<VR, QuantizedCoordinate> {
        &mut self.vertices
    }

    fn clear_vertices(&mut self) {
        self.vertices.clear();
    }

    fn add_vertex(&mut self, coordinate: QuantizedCoordinate) -> Result<VertexIndex<VR>> {
        self.vertices.push(coordinate)
    }

    fn get_vertex(&self, index: VertexIndex<VR>) -> Option<&QuantizedCoordinate> {
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

    fn metadata(&self) -> Option<&Metadata<RR, SS>> {
        self.metadata.as_ref()
    }

    fn metadata_mut(&mut self) -> &mut Metadata<RR, SS> {
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

    fn clear_cityobjects(&mut self) {
        self.cityobjects.clear();
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

    fn clear_template_vertices(&mut self) {
        self.template_vertices.clear();
    }

    fn type_citymodel(&self) -> CityModelType {
        self.type_citymodel
    }

    fn version(&self) -> Option<CityJSONVersion> {
        self.version
    }

    fn default_theme_material(&self) -> Option<RR> {
        self.default_theme_material
    }

    fn set_default_theme_material(&mut self, material_ref: Option<RR>) {
        self.default_theme_material = material_ref;
    }

    fn default_theme_texture(&self) -> Option<RR> {
        self.default_theme_texture
    }

    fn set_default_theme_texture(&mut self, texture_ref: Option<RR>) {
        self.default_theme_texture = texture_ref;
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
        writeln!(
            f,
            "\tappearance: {{ nr. materials: {}, nr. textures: {}, nr. vertices-texture: {}, default-theme-texture: {}, default-theme-material: {} }}",
            self.materials.len(),
            self.textures.len(),
            self.vertices_texture.len(),
            format_option(&self.default_theme_texture),
            format_option(&self.default_theme_material)
        )?;
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::v2_0::geometry::semantic::{Semantic, SemanticType};
    use crate::v2_0::*;
    #[test]
    fn test_clear_cityobjects() {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Add some cityobjects
        let co1 = CityObject::new("obj-1".to_string(), CityObjectType::Building);
        let co2 = CityObject::new("obj-2".to_string(), CityObjectType::Bridge);
        model.cityobjects_mut().add(co1);
        model.cityobjects_mut().add(co2);

        assert_eq!(model.cityobjects().len(), 2);

        // Clear cityobjects
        model.clear_cityobjects();

        assert_eq!(model.cityobjects().len(), 0);
    }

    #[test]
    fn test_clear_geometries() {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Add some geometries
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiPoint, BuilderMode::Regular);
        builder.add_point(QuantizedCoordinate::new(0, 0, 0));
        builder.build().unwrap();
        let mut builder =
            GeometryBuilder::new(&mut model, GeometryType::MultiPoint, BuilderMode::Regular);
        builder.add_point(QuantizedCoordinate::new(1, 0, 0));
        builder.build().unwrap();
        assert_eq!(model.geometry_count(), 2);

        // Clear geometries
        model.clear_geometries();

        assert_eq!(model.geometry_count(), 0);
    }

    #[test]
    fn test_clear_vertices() -> Result<()> {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Add some vertices
        model.add_vertex(QuantizedCoordinate::new(100, 200, 300))?;
        model.add_vertex(QuantizedCoordinate::new(400, 500, 600))?;
        model.add_vertex(QuantizedCoordinate::new(700, 800, 900))?;

        assert_eq!(model.vertex_count(), 3);

        // Clear vertices
        model.clear_vertices();

        assert_eq!(model.vertex_count(), 0);
        Ok(())
    }

    #[test]
    fn test_clear_template_vertices() -> Result<()> {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Add some template vertices
        model.add_template_vertex(RealWorldCoordinate::new(1.0, 2.0, 3.0))?;
        model.add_template_vertex(RealWorldCoordinate::new(4.0, 5.0, 6.0))?;

        assert_eq!(model.template_vertices().len(), 2);

        // Clear template vertices
        model.clear_template_vertices();

        assert_eq!(model.template_vertices().len(), 0);
        Ok(())
    }

    #[test]
    fn test_get_or_insert_semantic() {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Create a semantic
        let semantic1 = Semantic::new(SemanticType::RoofSurface);
        let semantic2 = Semantic::new(SemanticType::RoofSurface);

        // Insert first semantic
        let id1 = model.get_or_insert_semantic(semantic1);

        assert_eq!(model.semantic_count(), 1);

        // Insert same semantic again - should return same ID
        let id2 = model.get_or_insert_semantic(semantic2);

        assert_eq!(model.semantic_count(), 1);
        assert_eq!(id1, id2);

        // Insert different semantic
        let semantic3 = Semantic::new(SemanticType::WallSurface);
        let id3 = model.get_or_insert_semantic(semantic3);

        assert_eq!(model.semantic_count(), 2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_get_or_insert_material() {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Create materials
        let material1 = Material::new("red".to_string());
        let material2 = Material::new("red".to_string());

        // Insert first material
        let id1 = model.get_or_insert_material(material1);

        // Insert same material again - should return same ID
        let id2 = model.get_or_insert_material(material2);

        assert_eq!(id1, id2);

        // Insert different material
        let material3 = Material::new("blue".to_string());
        let id3 = model.get_or_insert_material(material3);

        assert_ne!(id1, id3);
    }

    #[test]
    fn test_get_or_insert_texture() {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Create textures
        let texture1 = Texture::new("texture1.png".to_string(), ImageType::Png);
        let texture2 = Texture::new("texture1.png".to_string(), ImageType::Png);

        // Insert first texture
        let id1 = model.get_or_insert_texture(texture1);

        // Insert same texture again - should return same ID
        let id2 = model.get_or_insert_texture(texture2);

        assert_eq!(id1, id2);

        // Insert different texture
        let texture3 = Texture::new("texture2.jpg".to_string(), ImageType::Jpg);
        let id3 = model.get_or_insert_texture(texture3);

        assert_ne!(id1, id3);
    }
}
