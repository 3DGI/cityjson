use crate::cityjson::core::coordinate::{UVCoordinate, Vertices};
use crate::cityjson::core::geometry::GeometryModelOps;
use crate::cityjson::core::vertex::VertexIndex;
use crate::cityjson::core::vertex::VertexRef;
use crate::prelude::{QuantizedCoordinate, RealWorldCoordinate, Result};
use crate::resources::pool::{ResourcePool, ResourceRef};
use crate::resources::storage::StringStorage;
use crate::v2_0::appearance::material::Material;
use crate::v2_0::appearance::texture::Texture;
use crate::v2_0::geometry::Geometry;
use crate::v2_0::geometry::semantic::Semantic;
use crate::v2_0::metadata::Metadata;
use crate::v2_0::{CityObjects, Extensions, Transform};
use crate::{CityJSONVersion, format_option};
use std::fmt;

#[derive(Debug, Clone)]
pub struct CityModel<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    #[allow(clippy::type_complexity)]
    inner: crate::cityjson::core::citymodel::CityModelCore<
        QuantizedCoordinate,
        VR,
        RR,
        SS,
        Semantic<RR, SS>,
        Material<SS>,
        Texture<SS>,
        Geometry<VR, RR, SS>,
        Metadata<RR, SS>,
        Transform,
        Extensions<SS>,
        CityObjects<SS, RR>,
    >,
}

crate::macros::impl_citymodel_methods!(QuantizedCoordinate, CityJSONVersion::V2_0, Metadata<RR, SS>);

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> fmt::Display for CityModel<VR, RR, SS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "CityModel {{")?;
        writeln!(f, "\ttype: {}", self.type_citymodel())?;
        writeln!(f, "\tversion: {}", format_option(&self.version()))?;
        writeln!(
            f,
            "\textensions: {{ {} }}",
            format_option(&self.extensions())
        )?;
        writeln!(f, "\ttransform: {{ {} }}", format_option(&self.transform()))?;
        writeln!(f, "\tmetadata: {}", format_option(&self.metadata()))?;
        writeln!(
            f,
            "\tCityObjects: {{ nr. cityobjects: {}, nr. geometries: {} }}",
            self.cityobjects().len(),
            self.geometries().len()
        )?;
        writeln!(
            f,
            "\tappearance: {{ nr. materials: {}, nr. textures: {}, nr. vertices-texture: {}, default-theme-texture: {}, default-theme-material: {} }}",
            self.materials().len(),
            self.textures().len(),
            self.uv_coordinate_count(),
            format_option(&self.default_theme_texture()),
            format_option(&self.default_theme_material())
        )?;
        writeln!(f, "\tgeometry-templates: not implemented")?;
        writeln!(
            f,
            "\tvertices: {{ nr. vertices: {}, quantized coordinates: not implemented }}",
            self.vertices().len()
        )?;
        writeln!(f, "\textra: {}", format_option(&self.extra()))?;
        writeln!(f, "}}")
    }
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage>
    GeometryModelOps<
        VR,
        RR,
        QuantizedCoordinate,
        Semantic<RR, SS>,
        Material<SS>,
        Texture<SS>,
        Geometry<VR, RR, SS>,
        SS,
    > for CityModel<VR, RR, SS>
{
    fn get_or_insert_semantic(&mut self, semantic: Semantic<RR, SS>) -> RR {
        self.get_or_insert_semantic(semantic)
    }

    fn get_or_insert_material(&mut self, material: Material<SS>) -> RR {
        self.get_or_insert_material(material)
    }

    fn get_or_insert_texture(&mut self, texture: Texture<SS>) -> RR {
        self.get_or_insert_texture(texture)
    }

    fn add_uv_coordinate(&mut self, uvcoordinate: UVCoordinate) -> Result<VertexIndex<VR>> {
        self.add_uv_coordinate(uvcoordinate)
    }

    fn add_geometry(&mut self, geometry: Geometry<VR, RR, SS>) -> RR {
        self.add_geometry(geometry)
    }

    fn add_template_geometry(&mut self, geometry: Geometry<VR, RR, SS>) -> RR {
        self.add_template_geometry(geometry)
    }

    fn add_vertex(&mut self, coordinate: QuantizedCoordinate) -> Result<VertexIndex<VR>> {
        self.add_vertex(coordinate)
    }

    fn vertices_mut(&mut self) -> &mut Vertices<VR, QuantizedCoordinate> {
        self.vertices_mut()
    }

    fn add_template_vertex(&mut self, coordinate: RealWorldCoordinate) -> Result<VertexIndex<VR>> {
        self.add_template_vertex(coordinate)
    }

    fn template_vertices_mut(&mut self) -> &mut Vertices<VR, RealWorldCoordinate> {
        self.template_vertices_mut()
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
