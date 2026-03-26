use std::collections::HashMap;

use cityjson::resources::handles::CityObjectHandle;
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::appearance::material::Material;
use cityjson::v2_0::appearance::texture::Texture;
use cityjson::v2_0::{
    BBox, CityModel, CityModelIdentifier, CityObject, CityObjectIdentifier, Contact, Date,
    Extension, Extensions, Metadata, RealWorldCoordinate, ThemeName, Transform, UVCoordinate, CRS,
    RGB, RGBA,
};
use serde_json::value::RawValue;

use crate::de::attributes::attribute_map;
use crate::de::geometry::{import_geometry, import_template_geometry, GeometryResources};
use crate::de::parse::ParseStringStorage;
use crate::de::root::{RawRoot, RawTransform};
use crate::de::sections::{
    RawAppearanceSection, RawCityObject, RawExtension, RawGeometryTemplatesSection,
    RawMetadataSection,
};
use crate::de::validation::{
    parse_cityobject_type, parse_contact_role, parse_contact_type, parse_image_type,
    parse_root_header, parse_texture_type, parse_wrap_mode,
};
use crate::errors::{Error, Result};

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub(crate) fn build_model<'de, SS>(raw: RawRoot<'de>) -> Result<CityModel<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let header = parse_root_header(raw.type_name, raw.version)?;
    let mut model = CityModel::<u32, SS>::new(header.type_citymodel);
    let transform = apply_transform(raw.transform, &mut model);
    let mut resources = GeometryResources::default();

    if let Some(appearance_raw) = raw.appearance {
        import_appearance::<SS>(appearance_raw, &mut model, &mut resources)?;
    }
    if let Some(templates_raw) = raw.geometry_templates {
        import_geometry_templates::<SS>(templates_raw, &mut model, &mut resources)?;
    }
    import_vertices(&raw.vertices, transform.as_ref(), &mut model)?;

    if let Some(metadata_raw) = raw.metadata {
        let meta: RawMetadataSection<'de> = serde_json::from_str(metadata_raw.get())?;
        *model.metadata_mut() = build_metadata::<SS>(meta)?;
    }
    if let Some(extensions_raw) = raw.extensions {
        let exts: HashMap<&'de str, RawExtension<'de>> =
            serde_json::from_str(extensions_raw.get())?;
        *model.extensions_mut() = build_extensions::<SS>(exts);
    }
    if !raw.extra.is_empty() {
        *model.extra_mut() = attribute_map::<SS>(raw.extra, "root extra properties")?;
    }

    import_cityobjects::<SS>(raw.cityobjects, &mut model, &resources)?;

    debug_assert_eq!(model.version(), Some(header.version));
    Ok(model)
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
    raw: &'de RawValue,
    model: &mut CityModel<u32, SS>,
    resources: &mut GeometryResources,
) -> Result<()>
where
    SS: ParseStringStorage<'de>,
{
    let section: RawAppearanceSection<'de> = serde_json::from_str(raw.get())?;

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
    raw: &'de RawValue,
    model: &mut CityModel<u32, SS>,
    resources: &mut GeometryResources,
) -> Result<()>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let section: RawGeometryTemplatesSection<'de> = serde_json::from_str(raw.get())?;

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

// ---------------------------------------------------------------------------
// City objects
// ---------------------------------------------------------------------------

struct PendingRelations<'de> {
    source_id: &'de str,
    source_handle: CityObjectHandle,
    parents: Vec<&'de str>,
    children: Vec<&'de str>,
}

fn import_cityobjects<'de, SS>(
    raw: &'de RawValue,
    model: &mut CityModel<u32, SS>,
    resources: &GeometryResources,
) -> Result<()>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    let city_objects: HashMap<&'de str, RawCityObject<'de>> = serde_json::from_str(raw.get())?;

    let mut handle_by_id = HashMap::with_capacity(city_objects.len());
    let mut pending = Vec::with_capacity(city_objects.len());

    for (id, raw_object) in city_objects {
        let type_cityobject = parse_cityobject_type::<SS>(raw_object.type_name)?;
        let mut cityobject =
            CityObject::new(CityObjectIdentifier::new(SS::store(id)), type_cityobject);

        if let Some(extent) = raw_object.geographical_extent {
            cityobject.set_geographical_extent(Some(BBox::from(extent)));
        }
        if let Some(attributes) = raw_object.attributes {
            *cityobject.attributes_mut() =
                attribute_map::<SS>(attributes, "CityObject.attributes")?;
        }
        if !raw_object.extra.is_empty() {
            *cityobject.extra_mut() = attribute_map::<SS>(raw_object.extra, "CityObject extra")?;
        }
        if let Some(raw_geometry) = raw_object.geometry {
            if raw_geometry.is_empty() {
                cityobject.clear_geometry();
            } else {
                for geom in raw_geometry {
                    let handle = import_geometry::<SS>(geom, model, resources)?;
                    cityobject.add_geometry(handle);
                }
            }
        }

        let handle = model.cityobjects_mut().add(cityobject)?;
        handle_by_id.insert(id, handle);
        pending.push(PendingRelations {
            source_id: id,
            source_handle: handle,
            parents: raw_object.parents,
            children: raw_object.children,
        });
    }

    resolve_relations(pending, &handle_by_id, model)
}

fn resolve_relations<'de, SS>(
    pending: Vec<PendingRelations<'de>>,
    handle_by_id: &HashMap<&'de str, CityObjectHandle>,
    model: &mut CityModel<u32, SS>,
) -> Result<()>
where
    SS: StringStorage,
{
    for relation in pending {
        let cityobject = model
            .cityobjects_mut()
            .get_mut(relation.source_handle)
            .ok_or_else(|| {
                Error::InvalidValue(format!(
                    "missing inserted CityObject for '{}'",
                    relation.source_id
                ))
            })?;

        for parent in relation.parents {
            let handle = handle_by_id.get(parent).copied().ok_or_else(|| {
                Error::UnresolvedCityObjectReference {
                    source_id: relation.source_id.to_owned(),
                    target_id: parent.to_owned(),
                    relation: "parent",
                }
            })?;
            cityobject.add_parent(handle);
        }

        for child in relation.children {
            let handle = handle_by_id.get(child).copied().ok_or_else(|| {
                Error::UnresolvedCityObjectReference {
                    source_id: relation.source_id.to_owned(),
                    target_id: child.to_owned(),
                    relation: "child",
                }
            })?;
            cityobject.add_child(handle);
        }
    }

    Ok(())
}

// Private re-export to avoid repeating the long path in build_contact.
use crate::de::sections::RawContact;
