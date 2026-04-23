use std::collections::HashMap;

use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::appearance::material::Material;
use cityjson::v2_0::appearance::texture::Texture;
use cityjson::v2_0::{
    BBox, CRS, CityModel, CityModelIdentifier, Contact, Date, Extension, Extensions, Metadata, RGB,
    RGBA, RealWorldCoordinate, ThemeName, Transform, UVCoordinate,
};

use crate::de::attributes::{RawAttribute, attribute_map};
use crate::de::cityobjects::import_cityobjects;
use crate::de::geometry::{GeometryResources, import_template_geometry};
use crate::de::parse::ParseStringStorage;
use crate::de::profiling::timed;
use crate::de::root::{PreparedRoot, RawTransform};
use crate::de::sections::{
    RawAppearanceSection, RawExtension, RawGeometryTemplatesSection, RawMetadataSection,
};
use crate::de::validation::{
    parse_contact_role, parse_contact_type, parse_image_type, parse_root_header,
    parse_texture_type, parse_wrap_mode,
};
use crate::errors::Result;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub(crate) fn build_model<'de, SS>(raw: PreparedRoot<'de>) -> Result<CityModel<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let PreparedRoot {
        type_name,
        version,
        transform,
        vertices,
        metadata,
        extensions,
        cityobjects,
        appearance,
        geometry_templates,
        id,
        mut extra,
    } = raw;
    let header = timed("build.parse_root_header", || {
        parse_root_header(type_name, version)
    })?;
    let mut model = CityModel::<u32, SS>::new(header.type_citymodel);
    let transform = timed("build.apply_transform", || {
        apply_transform(transform, &mut model)
    });
    let mut resources = GeometryResources::default();

    if let Some(appearance) = appearance {
        timed("build.import_appearance", || {
            import_appearance::<SS>(appearance, &mut model, &mut resources)
        })?;
    }
    if let Some(templates) = geometry_templates {
        timed("build.import_geometry_templates", || {
            import_geometry_templates::<SS>(templates, &mut model, &mut resources)
        })?;
    }
    timed("build.import_vertices", || {
        import_vertices(&vertices, transform.as_ref(), &mut model)
    })?;

    if let Some(metadata) = metadata {
        *model.metadata_mut() = timed("build.metadata", || build_metadata::<SS>(metadata))?;
    }
    if let Some(extensions) = extensions {
        *model.extensions_mut() = timed("build.extensions", || build_extensions::<SS>(extensions));
    }
    let feature_root_id = if header.type_citymodel == cityjson::CityModelType::CityJSONFeature {
        Some(parse_feature_root_id(id.ok_or_else(|| {
            crate::errors::Error::InvalidValue("CityJSONFeature root id is required".to_owned())
        })?)?)
    } else {
        if let Some(id) = id {
            extra.insert("id", id);
        }
        None
    };
    if !extra.is_empty() {
        *model.extra_mut() = timed("build.root_extra_attributes", || {
            attribute_map::<SS>(extra, "root extra properties")
        })?;
    }

    timed("build.import_cityobjects", || {
        import_cityobjects::<SS>(cityobjects, &mut model, &resources)
    })?;
    if let Some(feature_root_id) = feature_root_id {
        model.set_id(Some(resolve_feature_root_handle(&model, &feature_root_id)?));
    }

    debug_assert_eq!(model.version(), Some(header.version));
    Ok(model)
}

fn parse_feature_root_id(raw: RawAttribute<'_>) -> Result<String> {
    match raw {
        RawAttribute::String(value) => Ok(value.into_owned()),
        _ => Err(crate::errors::Error::InvalidValue(
            "CityJSONFeature root id must be a string".to_owned(),
        )),
    }
}

fn resolve_feature_root_handle<SS: StringStorage>(
    model: &CityModel<u32, SS>,
    feature_root_id: &str,
) -> Result<cityjson::prelude::CityObjectHandle> {
    model
        .cityobjects()
        .iter()
        .find_map(|(handle, cityobject)| (cityobject.id() == feature_root_id).then_some(handle))
        .ok_or_else(|| {
            crate::errors::Error::InvalidValue(format!(
                "feature root id does not resolve to a CityObject: {feature_root_id}"
            ))
        })
}

// ---------------------------------------------------------------------------
// Transform / vertices
// ---------------------------------------------------------------------------

fn apply_transform<SS: StringStorage>(
    raw: Option<RawTransform>,
    model: &mut CityModel<u32, SS>,
) -> Option<Transform> {
    raw.map(|t| {
        let transform = model.transform_mut();
        transform.set_scale(t.scale);
        transform.set_translate(t.translate);
        transform.clone()
    })
}

fn import_vertices<SS: StringStorage>(
    vertices: &[[f64; 3]],
    transform: Option<&Transform>,
    model: &mut CityModel<u32, SS>,
) -> Result<()> {
    for vertex in vertices {
        let coordinate = match transform {
            Some(t) => {
                let scale = t.scale();
                let translate = t.translate();
                RealWorldCoordinate::new(
                    vertex[0] * scale[0] + translate[0],
                    vertex[1] * scale[1] + translate[1],
                    vertex[2] * scale[2] + translate[2],
                )
            }
            None => RealWorldCoordinate::new(vertex[0], vertex[1], vertex[2]),
        };
        model.add_vertex(coordinate)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Appearance
// ---------------------------------------------------------------------------

fn import_appearance<'de, SS>(
    section: RawAppearanceSection<'de>,
    model: &mut CityModel<u32, SS>,
    resources: &mut GeometryResources,
) -> Result<()>
where
    SS: ParseStringStorage<'de>,
{
    for material in section.materials {
        let mut mat = Material::<SS>::new(SS::store(material.name));
        mat.set_ambient_intensity(material.ambient_intensity);
        mat.set_diffuse_color(material.diffuse_color.map(RGB::from));
        mat.set_emissive_color(material.emissive_color.map(RGB::from));
        mat.set_specular_color(material.specular_color.map(RGB::from));
        mat.set_shininess(material.shininess);
        mat.set_transparency(material.transparency);
        mat.set_is_smooth(material.is_smooth);
        resources.materials.push(model.add_material(mat)?);
    }

    for texture in section.textures {
        let mut tex = Texture::<SS>::new(
            SS::store(texture.image),
            parse_image_type(texture.image_type)?,
        );
        tex.set_wrap_mode(texture.wrap_mode.map(parse_wrap_mode).transpose()?);
        tex.set_texture_type(texture.texture_type.map(parse_texture_type).transpose()?);
        tex.set_border_color(texture.border_color.map(RGBA::from));
        resources.textures.push(model.add_texture(tex)?);
    }

    for uv in section.vertices_texture {
        model.add_uv_coordinate(UVCoordinate::new(uv[0], uv[1]))?;
    }

    if let Some(theme) = section.default_theme_material {
        model.set_default_material_theme(Some(ThemeName::<SS>::new(SS::store(theme))));
    }
    if let Some(theme) = section.default_theme_texture {
        model.set_default_texture_theme(Some(ThemeName::<SS>::new(SS::store(theme))));
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Geometry templates
// ---------------------------------------------------------------------------

fn import_geometry_templates<'de, SS>(
    section: RawGeometryTemplatesSection<'de>,
    model: &mut CityModel<u32, SS>,
    resources: &mut GeometryResources,
) -> Result<()>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    for vertex in section.vertices_templates {
        model.add_template_vertex(RealWorldCoordinate::new(vertex[0], vertex[1], vertex[2]))?;
    }

    for template in section.templates {
        let handle = import_template_geometry::<SS>(template, model, resources)?;
        resources.templates.push(handle);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Metadata
// ---------------------------------------------------------------------------

fn build_metadata<'de, SS>(raw: RawMetadataSection<'de>) -> Result<Metadata<SS>>
where
    SS: ParseStringStorage<'de>,
{
    let mut metadata = Metadata::new();

    if let Some(extent) = raw.geographical_extent {
        metadata.set_geographical_extent(BBox::from(extent));
    }
    if let Some(identifier) = raw.identifier {
        metadata.set_identifier(CityModelIdentifier::new(SS::store(identifier)));
    }
    if let Some(contact) = raw.point_of_contact {
        metadata.set_point_of_contact(Some(build_contact::<SS>(contact)?));
    }
    if let Some(date) = raw.reference_date {
        metadata.set_reference_date(Date::new(SS::store(date)));
    }
    if let Some(reference_system) = raw.reference_system {
        metadata.set_reference_system(CRS::new(SS::store(reference_system)));
    }
    if let Some(title) = raw.title {
        metadata.set_title(SS::store(title));
    }
    if !raw.extra.is_empty() {
        metadata.set_extra(Some(attribute_map::<SS>(
            raw.extra,
            "metadata extra properties",
        )?));
    }

    Ok(metadata)
}

fn build_contact<'de, SS>(raw: RawContact<'de>) -> Result<Contact<SS>>
where
    SS: ParseStringStorage<'de>,
{
    let mut contact = Contact::new();

    if let Some(value) = raw.contact_name {
        contact.set_contact_name(SS::store(value));
    }
    if let Some(value) = raw.email_address {
        contact.set_email_address(SS::store(value));
    }
    if let Some(value) = raw.role {
        contact.set_role(Some(parse_contact_role(value)?));
    }
    if let Some(value) = raw.website {
        contact.set_website(Some(SS::store(value)));
    }
    if let Some(value) = raw.contact_type {
        contact.set_contact_type(Some(parse_contact_type(value)?));
    }
    if let Some(value) = raw.address {
        contact.set_address(Some(attribute_map::<SS>(value, "pointOfContact.address")?));
    }
    if let Some(value) = raw.phone {
        contact.set_phone(Some(SS::store(value)));
    }
    if let Some(value) = raw.organization {
        contact.set_organization(Some(SS::store(value)));
    }

    Ok(contact)
}

// ---------------------------------------------------------------------------
// Extensions
// ---------------------------------------------------------------------------

fn build_extensions<'de, SS>(raw: HashMap<&'de str, RawExtension<'de>>) -> Extensions<SS>
where
    SS: ParseStringStorage<'de>,
{
    let mut extensions = Extensions::new();
    for (name, extension) in raw {
        extensions.add(Extension::new(
            SS::store(name),
            SS::store(extension.url),
            SS::store(extension.version),
        ));
    }
    extensions
}

// Private re-export to avoid repeating the long path in build_contact.
use crate::de::sections::RawContact;
