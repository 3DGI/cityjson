use std::collections::HashMap;

use crate::errors::Result;
use cityjson::resources::handles::GeometryTemplateHandle;
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::appearance::material::Material;
use cityjson::v2_0::appearance::texture::Texture;
use cityjson::v2_0::{CityModel, ImageType, TextureType, VertexRef, WrapMode, RGB, RGBA};
use serde_json::{Map, Value};

use crate::ser::geometry::geometry_to_json_value;

pub(crate) fn appearance_to_json_value<VR, SS>(model: &CityModel<VR, SS>) -> Value
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    let mut value = Map::new();
    let has_any_appearance = model.material_count() > 0
        || model.texture_count() > 0
        || !model.vertices_texture().is_empty()
        || model.default_material_theme().is_some()
        || model.default_texture_theme().is_some();

    if has_any_appearance {
        value.insert(
            "materials".to_owned(),
            Value::Array(
                model
                    .iter_materials()
                    .map(|(_, material)| material_to_json_value(material))
                    .collect(),
            ),
        );
    }

    if has_any_appearance {
        value.insert(
            "textures".to_owned(),
            Value::Array(
                model
                    .iter_textures()
                    .map(|(_, texture)| texture_to_json_value(texture))
                    .collect(),
            ),
        );
    }

    if has_any_appearance {
        value.insert(
            "vertices-texture".to_owned(),
            Value::Array(
                model
                    .vertices_texture()
                    .as_slice()
                    .iter()
                    .map(|uv| serde_json::json!([normalize_f32(uv.u()), normalize_f32(uv.v())]))
                    .collect(),
            ),
        );
    }

    if let Some(theme) = model.default_material_theme() {
        value.insert(
            "default-theme-material".to_owned(),
            Value::String(theme.as_ref().to_owned()),
        );
    }

    if let Some(theme) = model.default_texture_theme() {
        value.insert(
            "default-theme-texture".to_owned(),
            Value::String(theme.as_ref().to_owned()),
        );
    }

    Value::Object(value)
}

pub(crate) fn geometry_templates_to_json_value<VR, SS>(
    model: &CityModel<VR, SS>,
) -> Result<(Value, HashMap<GeometryTemplateHandle, usize>)>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    let mut templates = Vec::with_capacity(model.geometry_template_count());
    let mut indices = HashMap::with_capacity(model.geometry_template_count());

    for (dense_index, (handle, geometry)) in model.iter_geometry_templates().enumerate() {
        indices.insert(handle, dense_index);
        templates.push(geometry_to_json_value(model, geometry, Some(&indices))?);
    }

    Ok((
        serde_json::json!({
            "templates": templates,
            "vertices-templates": model
                .template_vertices()
                .as_slice()
                .iter()
                .map(|vertex| [vertex.x(), vertex.y(), vertex.z()])
                .collect::<Vec<_>>(),
        }),
        indices,
    ))
}

fn material_to_json_value<SS>(material: &Material<SS>) -> Value
where
    SS: StringStorage,
{
    let mut value = Map::new();
    value.insert(
        "name".to_owned(),
        Value::String(material.name().as_ref().to_owned()),
    );

    if let Some(ambient_intensity) = material.ambient_intensity() {
        value.insert(
            "ambientIntensity".to_owned(),
            serde_json::json!(normalize_f32(ambient_intensity)),
        );
    }
    if let Some(diffuse_color) = material.diffuse_color() {
        value.insert("diffuseColor".to_owned(), color3_to_json(diffuse_color));
    }
    if let Some(emissive_color) = material.emissive_color() {
        value.insert("emissiveColor".to_owned(), color3_to_json(emissive_color));
    }
    if let Some(specular_color) = material.specular_color() {
        value.insert("specularColor".to_owned(), color3_to_json(specular_color));
    }
    if let Some(shininess) = material.shininess() {
        value.insert(
            "shininess".to_owned(),
            serde_json::json!(normalize_f32(shininess)),
        );
    }
    if let Some(transparency) = material.transparency() {
        value.insert(
            "transparency".to_owned(),
            serde_json::json!(normalize_f32(transparency)),
        );
    }
    if let Some(is_smooth) = material.is_smooth() {
        value.insert("isSmooth".to_owned(), Value::Bool(is_smooth));
    }

    Value::Object(value)
}

fn texture_to_json_value<SS>(texture: &Texture<SS>) -> Value
where
    SS: StringStorage,
{
    let mut value = Map::new();
    value.insert(
        "type".to_owned(),
        Value::String(match texture.image_type() {
            ImageType::Png => "PNG".to_owned(),
            ImageType::Jpg => "JPG".to_owned(),
            _ => texture.image_type().to_string(),
        }),
    );
    value.insert(
        "image".to_owned(),
        Value::String(texture.image().as_ref().to_owned()),
    );

    if let Some(wrap_mode) = texture.wrap_mode() {
        value.insert(
            "wrapMode".to_owned(),
            Value::String(match wrap_mode {
                WrapMode::Wrap => "wrap".to_owned(),
                WrapMode::Mirror => "mirror".to_owned(),
                WrapMode::Clamp => "clamp".to_owned(),
                WrapMode::Border => "border".to_owned(),
                WrapMode::None => "none".to_owned(),
                _ => wrap_mode.to_string(),
            }),
        );
    }
    if let Some(texture_type) = texture.texture_type() {
        value.insert(
            "textureType".to_owned(),
            Value::String(match texture_type {
                TextureType::Unknown => "unknown".to_owned(),
                TextureType::Specific => "specific".to_owned(),
                TextureType::Typical => "typical".to_owned(),
                _ => texture_type.to_string(),
            }),
        );
    }
    if let Some(border_color) = texture.border_color() {
        value.insert("borderColor".to_owned(), color4_to_json(border_color));
    }

    Value::Object(value)
}

fn color3_to_json(color: RGB) -> Value {
    Value::Array(
        color
            .to_array()
            .into_iter()
            .map(|value| serde_json::json!(normalize_f32(value)))
            .collect(),
    )
}

fn color4_to_json(color: RGBA) -> Value {
    Value::Array(
        color
            .to_array()
            .into_iter()
            .map(|value| serde_json::json!(normalize_f32(value)))
            .collect(),
    )
}

fn normalize_f32(value: f32) -> f64 {
    (f64::from(value) * 1_000_000.0).round() / 1_000_000.0
}
