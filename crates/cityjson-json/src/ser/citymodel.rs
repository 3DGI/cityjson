use std::collections::HashMap;

use cityjson::resources::handles::{CityObjectHandle, GeometryTemplateHandle};
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{
    BBox, CityModel, CityObject, ContactRole, ContactType, Extension, Metadata, VertexRef,
};
use serde_json::{Map, Number, Value};

use crate::errors::{Error, Result};
use crate::ser::appearance::{appearance_to_json_value, geometry_templates_to_json_value};
use crate::ser::attributes::attributes_to_json_map;
use crate::ser::geometry::geometries_to_json_value;

pub(crate) fn citymodel_to_json_value<VR, SS>(model: &CityModel<VR, SS>) -> Result<Value>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
    SS::String: AsRef<str>,
{
    let mut root = Map::new();
    root.insert(
        "type".to_owned(),
        Value::String(model.type_citymodel().to_string()),
    );
    let mut template_indices = HashMap::<GeometryTemplateHandle, usize>::new();
    if model.type_citymodel().to_string() != "CityJSONFeature" {
        root.insert(
            "version".to_owned(),
            Value::String(model.version().unwrap_or_default().to_string()),
        );
    }

    if let Some(transform) = model.transform() {
        root.insert(
            "transform".to_owned(),
            serde_json::json!({
                "scale": transform.scale(),
                "translate": transform.translate(),
            }),
        );
    }

    if let Some(metadata) = model.metadata() {
        root.insert("metadata".to_owned(), metadata_to_json_value(metadata)?);
    }

    if let Some(extensions) = model.extensions() {
        let mut value = Map::new();
        for extension in extensions {
            value.insert(
                extension.name().as_ref().to_owned(),
                extension_to_json_value(extension),
            );
        }
        root.insert("extensions".to_owned(), Value::Object(value));
    }

    let id_by_handle = collect_cityobject_ids(model);
    root.insert("vertices".to_owned(), vertices_to_json_value(model)?);
    root.insert("appearance".to_owned(), appearance_to_json_value(model));

    if model.type_citymodel().to_string() == "CityJSON" {
        let (geometry_templates, indices) = geometry_templates_to_json_value(model)?;
        template_indices = indices;
        root.insert("geometry-templates".to_owned(), geometry_templates);
    }
    root.insert(
        "CityObjects".to_owned(),
        cityobjects_to_json_value(model, &id_by_handle, &template_indices)?,
    );

    if let Some(extra) = model.extra() {
        root.extend(attributes_to_json_map(extra)?);
    }

    Ok(Value::Object(root))
}

fn metadata_to_json_value<SS>(metadata: &Metadata<SS>) -> Result<Value>
where
    SS: StringStorage,
    SS::String: AsRef<str>,
{
    let mut value = Map::new();

    if let Some(extent) = metadata.geographical_extent() {
        value.insert("geographicalExtent".to_owned(), bbox_to_json_value(extent));
    }
    if let Some(identifier) = metadata.identifier() {
        value.insert(
            "identifier".to_owned(),
            Value::String(identifier.to_string()),
        );
    }
    if let Some(contact) = metadata.point_of_contact() {
        let mut contact_value = Map::new();
        if !contact.contact_name().is_empty() {
            contact_value.insert(
                "contactName".to_owned(),
                Value::String(contact.contact_name().to_owned()),
            );
        }
        if !contact.email_address().is_empty() {
            contact_value.insert(
                "emailAddress".to_owned(),
                Value::String(contact.email_address().to_owned()),
            );
        }
        if let Some(role) = contact.role() {
            contact_value.insert(
                "role".to_owned(),
                Value::String(contact_role_to_str(role).to_owned()),
            );
        }
        if let Some(website) = contact.website().as_ref() {
            contact_value.insert("website".to_owned(), Value::String(website.to_string()));
        }
        if let Some(kind) = contact.contact_type() {
            contact_value.insert(
                "contactType".to_owned(),
                Value::String(contact_type_to_str(kind).to_owned()),
            );
        }
        if let Some(address) = contact.address() {
            contact_value.insert(
                "address".to_owned(),
                Value::Object(attributes_to_json_map(address)?),
            );
        }
        if let Some(phone) = contact.phone().as_ref() {
            contact_value.insert("phone".to_owned(), Value::String(phone.to_string()));
        }
        if let Some(organization) = contact.organization().as_ref() {
            contact_value.insert(
                "organization".to_owned(),
                Value::String(organization.to_string()),
            );
        }
        value.insert("pointOfContact".to_owned(), Value::Object(contact_value));
    }
    if let Some(reference_date) = metadata.reference_date() {
        value.insert(
            "referenceDate".to_owned(),
            Value::String(reference_date.to_string()),
        );
    }
    if let Some(reference_system) = metadata.reference_system() {
        value.insert(
            "referenceSystem".to_owned(),
            Value::String(reference_system.to_string()),
        );
    }
    if let Some(title) = metadata.title() {
        value.insert("title".to_owned(), Value::String(title.to_owned()));
    }
    if let Some(extra) = metadata.extra() {
        value.extend(attributes_to_json_map(extra)?);
    }

    Ok(Value::Object(value))
}

fn cityobjects_to_json_value<VR, SS>(
    model: &CityModel<VR, SS>,
    id_by_handle: &HashMap<CityObjectHandle, String>,
    template_indices: &HashMap<GeometryTemplateHandle, usize>,
) -> Result<Value>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    let mut value = Map::new();

    for (handle, cityobject) in model.cityobjects().iter() {
        value.insert(
            id_by_handle.get(&handle).cloned().ok_or_else(|| {
                Error::InvalidValue(format!("missing id for CityObject {handle}"))
            })?,
            cityobject_to_json_value(model, cityobject, id_by_handle, template_indices)?,
        );
    }

    Ok(Value::Object(value))
}

fn cityobject_to_json_value<VR, SS>(
    model: &CityModel<VR, SS>,
    cityobject: &CityObject<SS>,
    id_by_handle: &HashMap<CityObjectHandle, String>,
    template_indices: &HashMap<GeometryTemplateHandle, usize>,
) -> Result<Value>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    let mut value = Map::new();
    value.insert(
        "type".to_owned(),
        Value::String(cityobject.type_cityobject().to_string()),
    );

    if let Some(extent) = cityobject.geographical_extent() {
        value.insert("geographicalExtent".to_owned(), bbox_to_json_value(extent));
    }
    if let Some(attributes) = cityobject.attributes() {
        if !attributes.is_empty() {
            value.insert(
                "attributes".to_owned(),
                Value::Object(attributes_to_json_map(attributes)?),
            );
        }
    }
    if let Some(geometry) = cityobject.geometry() {
        value.insert(
            "geometry".to_owned(),
            geometries_to_json_value(model, geometry, template_indices)?,
        );
    }
    if let Some(parents) = cityobject.parents() {
        if !parents.is_empty() {
            value.insert(
                "parents".to_owned(),
                Value::Array(
                    parents
                        .iter()
                        .map(|handle| {
                            id_by_handle
                                .get(handle)
                                .cloned()
                                .map(Value::String)
                                .ok_or_else(|| Error::UnresolvedCityObjectReference {
                                    source_id: cityobject.id().to_owned(),
                                    target_id: handle.to_string(),
                                    relation: "parent",
                                })
                        })
                        .collect::<Result<Vec<_>>>()?,
                ),
            );
        }
    }
    if let Some(children) = cityobject.children() {
        if !children.is_empty() {
            value.insert(
                "children".to_owned(),
                Value::Array(
                    children
                        .iter()
                        .map(|handle| {
                            id_by_handle
                                .get(handle)
                                .cloned()
                                .map(Value::String)
                                .ok_or_else(|| Error::UnresolvedCityObjectReference {
                                    source_id: cityobject.id().to_owned(),
                                    target_id: handle.to_string(),
                                    relation: "child",
                                })
                        })
                        .collect::<Result<Vec<_>>>()?,
                ),
            );
        }
    }
    if let Some(extra) = cityobject.extra() {
        value.extend(attributes_to_json_map(extra)?);
    }

    Ok(Value::Object(value))
}

fn vertices_to_json_value<VR, SS>(model: &CityModel<VR, SS>) -> Result<Value>
where
    VR: VertexRef,
    SS: StringStorage,
{
    let transform = model.transform();
    let mut vertices = Vec::with_capacity(model.vertices().len());

    for vertex in model.vertices().as_slice() {
        let values = match transform {
            Some(transform) => {
                let scale = transform.scale();
                let translate = transform.translate();
                vec![
                    quantized_number_value((vertex.x() - translate[0]) / scale[0])?,
                    quantized_number_value((vertex.y() - translate[1]) / scale[1])?,
                    quantized_number_value((vertex.z() - translate[2]) / scale[2])?,
                ]
            }
            None => vec![
                quantized_number_value(vertex.x())?,
                quantized_number_value(vertex.y())?,
                quantized_number_value(vertex.z())?,
            ],
        };
        vertices.push(Value::Array(values));
    }

    Ok(Value::Array(vertices))
}

fn collect_cityobject_ids<VR, SS>(model: &CityModel<VR, SS>) -> HashMap<CityObjectHandle, String>
where
    VR: VertexRef,
    SS: StringStorage,
{
    model
        .cityobjects()
        .iter()
        .map(|(handle, cityobject)| (handle, cityobject.id().to_owned()))
        .collect()
}

fn extension_to_json_value<SS>(extension: &Extension<SS>) -> Value
where
    SS: StringStorage,
{
    serde_json::json!({
        "url": extension.url().as_ref(),
        "version": extension.version().as_ref(),
    })
}

fn bbox_to_json_value(extent: &BBox) -> Value {
    let extent: [f64; 6] = (*extent).into();
    Value::Array(
        extent
            .into_iter()
            .map(|value| Value::Number(Number::from_f64(value).unwrap()))
            .collect(),
    )
}

fn contact_role_to_str(role: ContactRole) -> &'static str {
    match role {
        ContactRole::Author => "author",
        ContactRole::CoAuthor => "co-author",
        ContactRole::Processor => "processor",
        ContactRole::PointOfContact => "pointOfContact",
        ContactRole::Owner => "owner",
        ContactRole::User => "user",
        ContactRole::Distributor => "distributor",
        ContactRole::Originator => "originator",
        ContactRole::Custodian => "custodian",
        ContactRole::ResourceProvider => "resourceProvider",
        ContactRole::RightsHolder => "rightsHolder",
        ContactRole::Sponsor => "sponsor",
        ContactRole::PrincipalInvestigator => "principalInvestigator",
        ContactRole::Stakeholder => "stakeholder",
        ContactRole::Publisher => "publisher",
    }
}

fn contact_type_to_str(kind: ContactType) -> &'static str {
    match kind {
        ContactType::Individual => "individual",
        ContactType::Organization => "organization",
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn quantized_number_value(value: f64) -> Result<Value> {
    let rounded = value.round();
    if rounded >= i64::MIN as f64 && rounded <= i64::MAX as f64 {
        Ok(Value::Number(Number::from(rounded as i64)))
    } else if rounded >= 0.0 && rounded <= u64::MAX as f64 {
        Ok(Value::Number(Number::from(rounded as u64)))
    } else {
        Err(Error::InvalidValue(format!(
            "cannot serialize quantized coordinate '{value}'"
        )))
    }
}
