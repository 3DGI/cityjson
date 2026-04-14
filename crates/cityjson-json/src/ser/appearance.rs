use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq};

use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::appearance::material::Material;
use cityjson::v2_0::appearance::texture::Texture;
use cityjson::v2_0::{CityModel, ImageType, RGB, RGBA, TextureType, VertexRef, WrapMode};

use crate::errors::Result;
use crate::ser::context::WriteContext;
use crate::ser::geometry::GeometrySerializer;

pub(crate) fn has_appearance<VR, SS>(model: &CityModel<VR, SS>) -> bool
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    model.material_count() > 0
        || model.texture_count() > 0
        || !model.vertices_texture().is_empty()
        || model.default_material_theme().is_some()
        || model.default_texture_theme().is_some()
}

pub(crate) fn has_geometry_templates<VR, SS>(model: &CityModel<VR, SS>) -> bool
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    model.geometry_template_count() > 0 || !model.template_vertices().is_empty()
}

pub(crate) struct AppearanceSerializer<'a, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    pub(crate) model: &'a CityModel<VR, SS>,
}

impl<VR, SS> Serialize for AppearanceSerializer<'_, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        if self.model.material_count() > 0 {
            map.serialize_entry("materials", &MaterialsSerializer { model: self.model })?;
        }
        if self.model.texture_count() > 0 {
            map.serialize_entry("textures", &TexturesSerializer { model: self.model })?;
        }
        if !self.model.vertices_texture().is_empty() {
            map.serialize_entry(
                "vertices-texture",
                &TextureVerticesSerializer { model: self.model },
            )?;
        }
        if let Some(theme) = self.model.default_material_theme() {
            map.serialize_entry("default-theme-material", theme.as_ref())?;
        }
        if let Some(theme) = self.model.default_texture_theme() {
            map.serialize_entry("default-theme-texture", theme.as_ref())?;
        }
        map.end()
    }
}

pub(crate) struct GeometryTemplatesSerializer<'a, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    pub(crate) model: &'a CityModel<VR, SS>,
    pub(crate) context: &'a WriteContext,
}

impl<VR, SS> Serialize for GeometryTemplatesSerializer<'_, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(
            "templates",
            &TemplateArraySerializer {
                model: self.model,
                context: self.context,
            },
        )?;
        map.serialize_entry(
            "vertices-templates",
            &TemplateVerticesSerializer { model: self.model },
        )?;
        map.end()
    }
}

pub(crate) fn ensure_geometry_templates_supported<VR, SS>(
    model: &CityModel<VR, SS>,
    context: &WriteContext,
) -> Result<()>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    for (_, geometry) in model.iter_geometry_templates() {
        GeometrySerializer {
            model,
            geometry,
            context,
        }
        .validate()?;
    }
    Ok(())
}

struct MaterialsSerializer<'a, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    model: &'a CityModel<VR, SS>,
}

impl<VR, SS> Serialize for MaterialsSerializer<'_, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.model.material_count()))?;
        for (_, material) in self.model.iter_materials() {
            seq.serialize_element(&MaterialSerializer(material))?;
        }
        seq.end()
    }
}

struct TexturesSerializer<'a, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    model: &'a CityModel<VR, SS>,
}

impl<VR, SS> Serialize for TexturesSerializer<'_, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.model.texture_count()))?;
        for (_, texture) in self.model.iter_textures() {
            seq.serialize_element(&TextureSerializer(texture))?;
        }
        seq.end()
    }
}

struct TextureVerticesSerializer<'a, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    model: &'a CityModel<VR, SS>,
}

impl<VR, SS> Serialize for TextureVerticesSerializer<'_, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let vertices = self.model.vertices_texture().as_slice();
        let mut seq = serializer.serialize_seq(Some(vertices.len()))?;
        for uv in vertices {
            seq.serialize_element(&NormalizedPair([uv.u(), uv.v()]))?;
        }
        seq.end()
    }
}

struct TemplateArraySerializer<'a, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    model: &'a CityModel<VR, SS>,
    context: &'a WriteContext,
}

impl<VR, SS> Serialize for TemplateArraySerializer<'_, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.model.geometry_template_count()))?;
        for (_, geometry) in self.model.iter_geometry_templates() {
            seq.serialize_element(&GeometrySerializer {
                model: self.model,
                geometry,
                context: self.context,
            })?;
        }
        seq.end()
    }
}

struct TemplateVerticesSerializer<'a, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    model: &'a CityModel<VR, SS>,
}

impl<VR, SS> Serialize for TemplateVerticesSerializer<'_, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let vertices = self.model.template_vertices().as_slice();
        let mut seq = serializer.serialize_seq(Some(vertices.len()))?;
        for vertex in vertices {
            seq.serialize_element(&Triple([vertex.x(), vertex.y(), vertex.z()]))?;
        }
        seq.end()
    }
}

struct MaterialSerializer<'a, SS>(&'a Material<SS>)
where
    SS: StringStorage;

impl<SS> Serialize for MaterialSerializer<'_, SS>
where
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let material = self.0;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("name", material.name().as_ref())?;
        if let Some(ambient_intensity) = material.ambient_intensity() {
            map.serialize_entry("ambientIntensity", &NormalizedF32(ambient_intensity))?;
        }
        if let Some(diffuse_color) = material.diffuse_color() {
            map.serialize_entry("diffuseColor", &Color3Serializer(diffuse_color))?;
        }
        if let Some(emissive_color) = material.emissive_color() {
            map.serialize_entry("emissiveColor", &Color3Serializer(emissive_color))?;
        }
        if let Some(specular_color) = material.specular_color() {
            map.serialize_entry("specularColor", &Color3Serializer(specular_color))?;
        }
        if let Some(shininess) = material.shininess() {
            map.serialize_entry("shininess", &NormalizedF32(shininess))?;
        }
        if let Some(transparency) = material.transparency() {
            map.serialize_entry("transparency", &NormalizedF32(transparency))?;
        }
        if let Some(is_smooth) = material.is_smooth() {
            map.serialize_entry("isSmooth", &is_smooth)?;
        }
        map.end()
    }
}

struct TextureSerializer<'a, SS>(&'a Texture<SS>)
where
    SS: StringStorage;

impl<SS> Serialize for TextureSerializer<'_, SS>
where
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let texture = self.0;
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", image_type_str(*texture.image_type()))?;
        map.serialize_entry("image", texture.image().as_ref())?;
        if let Some(wrap_mode) = texture.wrap_mode() {
            map.serialize_entry("wrapMode", wrap_mode_str(wrap_mode))?;
        }
        if let Some(texture_type) = texture.texture_type() {
            map.serialize_entry("textureType", texture_type_str(texture_type))?;
        }
        if let Some(border_color) = texture.border_color() {
            map.serialize_entry("borderColor", &Color4Serializer(border_color))?;
        }
        map.end()
    }
}

struct Color3Serializer(RGB);

impl Serialize for Color3Serializer {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(3))?;
        for value in self.0.to_array() {
            seq.serialize_element(&NormalizedF32(value))?;
        }
        seq.end()
    }
}

struct Color4Serializer(RGBA);

impl Serialize for Color4Serializer {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(4))?;
        for value in self.0.to_array() {
            seq.serialize_element(&NormalizedF32(value))?;
        }
        seq.end()
    }
}

struct NormalizedPair([f32; 2]);

impl Serialize for NormalizedPair {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        for value in self.0 {
            seq.serialize_element(&NormalizedF32(value))?;
        }
        seq.end()
    }
}

struct Triple([f64; 3]);

impl Serialize for Triple {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(3))?;
        for value in self.0 {
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}

struct NormalizedF32(f32);

impl Serialize for NormalizedF32 {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f64(normalize_f32(self.0))
    }
}

fn image_type_str(image_type: ImageType) -> &'static str {
    match image_type {
        ImageType::Png => "PNG",
        ImageType::Jpg => "JPG",
        _ => "unknown",
    }
}

fn wrap_mode_str(wrap_mode: WrapMode) -> &'static str {
    match wrap_mode {
        WrapMode::Wrap => "wrap",
        WrapMode::Mirror => "mirror",
        WrapMode::Clamp => "clamp",
        WrapMode::Border => "border",
        _ => "none",
    }
}

fn texture_type_str(texture_type: TextureType) -> &'static str {
    match texture_type {
        TextureType::Specific => "specific",
        TextureType::Typical => "typical",
        _ => "unknown",
    }
}

fn normalize_f32(value: f32) -> f64 {
    (f64::from(value) * 1_000_000.0).round() / 1_000_000.0
}
