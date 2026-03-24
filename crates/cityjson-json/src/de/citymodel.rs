use std::collections::HashMap;

use cityjson::resources::handles::CityObjectHandle;
use cityjson::v2_0::{
    BBox, BorrowedCityModel, CityModelIdentifier, CityObject, CityObjectIdentifier, CityObjectType,
    Contact, ContactRole, ContactType, Date, Extension, Extensions, Metadata, OwnedCityModel,
    RealWorldCoordinate, Transform, CRS,
};
use serde::Deserialize;
use serde_json::Value as OwnedJsonValue;
use serde_json_borrow::Value as BorrowedJsonValue;

use crate::de::attributes::{
    borrowed_attributes_from_json_owned, borrowed_attributes_from_map, owned_attributes_from_json,
};
use crate::de::geometry::{
    import_borrowed_geometries, import_owned_geometries, RawGeometryBorrowed, RawGeometryOwned,
};
use crate::de::header::parse_root_header;
use crate::errors::{Error, Result};

#[derive(Deserialize)]
struct RawTransform {
    scale: [f64; 3],
    translate: [f64; 3],
}

#[derive(Deserialize)]
struct RawExtensionOwned {
    url: String,
    version: String,
}

#[derive(Deserialize)]
struct RawContactOwned {
    #[serde(rename = "contactName", default)]
    contact_name: Option<String>,
    #[serde(rename = "emailAddress", default)]
    email_address: Option<String>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    website: Option<String>,
    #[serde(rename = "contactType", default)]
    contact_type: Option<String>,
    #[serde(default)]
    address: Option<OwnedJsonValue>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    organization: Option<String>,
}

#[derive(Deserialize)]
struct RawMetadataOwned {
    #[serde(rename = "geographicalExtent", default)]
    geographical_extent: Option<[f64; 6]>,
    #[serde(default)]
    identifier: Option<String>,
    #[serde(rename = "pointOfContact", default)]
    point_of_contact: Option<RawContactOwned>,
    #[serde(rename = "referenceDate", default)]
    reference_date: Option<String>,
    #[serde(rename = "referenceSystem", default)]
    reference_system: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, OwnedJsonValue>,
}

#[derive(Deserialize)]
struct RawCityObjectOwned {
    #[serde(rename = "type")]
    type_name: String,
    #[serde(rename = "geographicalExtent", default)]
    geographical_extent: Option<[f64; 6]>,
    #[serde(default)]
    attributes: Option<OwnedJsonValue>,
    #[serde(default)]
    parents: Vec<String>,
    #[serde(default)]
    children: Vec<String>,
    #[serde(default)]
    geometry: Vec<RawGeometryOwned>,
    #[serde(flatten)]
    extra: HashMap<String, OwnedJsonValue>,
}

#[derive(Deserialize)]
struct RawRootOwned {
    #[serde(rename = "type")]
    type_name: String,
    version: String,
    #[serde(default)]
    transform: Option<RawTransform>,
    #[serde(default)]
    metadata: Option<RawMetadataOwned>,
    #[serde(default)]
    extensions: HashMap<String, RawExtensionOwned>,
    #[serde(rename = "CityObjects")]
    cityobjects: HashMap<String, RawCityObjectOwned>,
    vertices: Vec<[f64; 3]>,
    #[serde(default)]
    appearance: Option<OwnedJsonValue>,
    #[serde(rename = "geometry-templates", default)]
    geometry_templates: Option<OwnedJsonValue>,
    #[serde(flatten)]
    extra: HashMap<String, OwnedJsonValue>,
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
struct RawExtensionBorrowed<'a> {
    #[serde(borrow)]
    url: &'a str,
    #[serde(borrow)]
    version: &'a str,
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
struct RawContactBorrowed<'a> {
    #[serde(rename = "contactName", default, borrow)]
    contact_name: Option<&'a str>,
    #[serde(rename = "emailAddress", default, borrow)]
    email_address: Option<&'a str>,
    #[serde(default, borrow)]
    role: Option<&'a str>,
    #[serde(default, borrow)]
    website: Option<&'a str>,
    #[serde(rename = "contactType", default, borrow)]
    contact_type: Option<&'a str>,
    #[serde(default, borrow)]
    address: Option<BorrowedJsonValue<'a>>,
    #[serde(default, borrow)]
    phone: Option<&'a str>,
    #[serde(default, borrow)]
    organization: Option<&'a str>,
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
struct RawMetadataBorrowed<'a> {
    #[serde(rename = "geographicalExtent", default)]
    geographical_extent: Option<[f64; 6]>,
    #[serde(default, borrow)]
    identifier: Option<&'a str>,
    #[serde(rename = "pointOfContact", default, borrow)]
    point_of_contact: Option<RawContactBorrowed<'a>>,
    #[serde(rename = "referenceDate", default, borrow)]
    reference_date: Option<&'a str>,
    #[serde(rename = "referenceSystem", default, borrow)]
    reference_system: Option<&'a str>,
    #[serde(default, borrow)]
    title: Option<&'a str>,
    #[serde(flatten, borrow)]
    extra: HashMap<&'a str, BorrowedJsonValue<'a>>,
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
struct RawCityObjectBorrowed<'a> {
    #[serde(rename = "type", borrow)]
    type_name: &'a str,
    #[serde(rename = "geographicalExtent", default)]
    geographical_extent: Option<[f64; 6]>,
    #[serde(default, borrow)]
    attributes: Option<BorrowedJsonValue<'a>>,
    #[serde(default, borrow)]
    parents: Vec<&'a str>,
    #[serde(default, borrow)]
    children: Vec<&'a str>,
    #[serde(default, borrow)]
    geometry: Vec<RawGeometryBorrowed<'a>>,
    #[serde(flatten, borrow)]
    extra: HashMap<&'a str, BorrowedJsonValue<'a>>,
}

#[derive(Deserialize)]
#[serde(bound(deserialize = "'de: 'a, 'a: 'de"))]
struct RawRootBorrowed<'a> {
    #[serde(rename = "type", borrow)]
    type_name: &'a str,
    #[serde(borrow)]
    version: &'a str,
    #[serde(default)]
    transform: Option<RawTransform>,
    #[serde(default, borrow)]
    metadata: Option<RawMetadataBorrowed<'a>>,
    #[serde(default, borrow)]
    extensions: HashMap<&'a str, RawExtensionBorrowed<'a>>,
    #[serde(rename = "CityObjects", borrow)]
    cityobjects: HashMap<&'a str, RawCityObjectBorrowed<'a>>,
    vertices: Vec<[f64; 3]>,
    #[serde(default, borrow)]
    appearance: Option<BorrowedJsonValue<'a>>,
    #[serde(rename = "geometry-templates", default, borrow)]
    geometry_templates: Option<BorrowedJsonValue<'a>>,
    #[serde(flatten, borrow)]
    extra: HashMap<&'a str, BorrowedJsonValue<'a>>,
}

struct PendingRelationsOwned {
    source_id: String,
    source_handle: CityObjectHandle,
    parents: Vec<String>,
    children: Vec<String>,
}

struct PendingRelationsBorrowed<'a> {
    source_id: &'a str,
    source_handle: CityObjectHandle,
    parents: Vec<&'a str>,
    children: Vec<&'a str>,
}

pub(crate) fn from_str_owned(input: &str) -> Result<OwnedCityModel> {
    let raw: RawRootOwned = serde_json::from_str(input)?;
    build_owned_citymodel(raw)
}

pub(crate) fn from_str_borrowed<'a>(input: &'a str) -> Result<BorrowedCityModel<'a>> {
    let raw: RawRootBorrowed<'a> = serde_json::from_str(input)?;
    build_borrowed_citymodel(raw)
}

fn build_owned_citymodel(raw: RawRootOwned) -> Result<OwnedCityModel> {
    let header = parse_root_header(&raw.type_name, &raw.version)?;
    let mut model = OwnedCityModel::new(header.type_citymodel);
    let transform = apply_transform(raw.transform, &mut model);

    reject_unsupported_root_sections_owned(
        raw.appearance.as_ref(),
        raw.geometry_templates.as_ref(),
    )?;
    import_vertices(&raw.vertices, transform.as_ref(), &mut model)?;

    if let Some(metadata) = raw.metadata {
        *model.metadata_mut() = build_owned_metadata(metadata)?;
    }

    if !raw.extensions.is_empty() {
        *model.extensions_mut() = build_owned_extensions(raw.extensions);
    }

    if !raw.extra.is_empty() {
        let value = OwnedJsonValue::Object(raw.extra.into_iter().collect());
        *model.extra_mut() = owned_attributes_from_json(&value, "root extra properties")?;
    }

    import_owned_cityobjects(raw.cityobjects, &mut model)?;
    debug_assert_eq!(model.version(), Some(header.version));
    Ok(model)
}

fn build_borrowed_citymodel<'a>(raw: RawRootBorrowed<'a>) -> Result<BorrowedCityModel<'a>> {
    let header = parse_root_header(raw.type_name, raw.version)?;
    let mut model = BorrowedCityModel::new(header.type_citymodel);
    let transform = apply_transform(raw.transform, &mut model);

    reject_unsupported_root_sections_borrowed(
        raw.appearance.as_ref(),
        raw.geometry_templates.as_ref(),
    )?;
    import_vertices(&raw.vertices, transform.as_ref(), &mut model)?;

    if let Some(metadata) = raw.metadata {
        *model.metadata_mut() = build_borrowed_metadata(metadata)?;
    }

    if !raw.extensions.is_empty() {
        *model.extensions_mut() = build_borrowed_extensions(raw.extensions);
    }

    if !raw.extra.is_empty() {
        *model.extra_mut() = borrowed_attributes_from_map(raw.extra, "root extra properties")?;
    }

    import_borrowed_cityobjects(raw.cityobjects, &mut model)?;
    debug_assert_eq!(model.version(), Some(header.version));
    Ok(model)
}

fn apply_transform<SS: cityjson::resources::storage::StringStorage>(
    raw: Option<RawTransform>,
    model: &mut cityjson::v2_0::CityModel<u32, SS>,
) -> Option<Transform> {
    raw.map(|raw| {
        let transform = model.transform_mut();
        transform.set_scale(raw.scale);
        transform.set_translate(raw.translate);
        transform.clone()
    })
}

fn import_vertices<SS: cityjson::resources::storage::StringStorage>(
    vertices: &[[f64; 3]],
    transform: Option<&Transform>,
    model: &mut cityjson::v2_0::CityModel<u32, SS>,
) -> Result<()> {
    for vertex in vertices {
        let coordinate = match transform {
            Some(transform) => {
                let scale = transform.scale();
                let translate = transform.translate();
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

fn build_owned_extensions(
    raw: HashMap<String, RawExtensionOwned>,
) -> Extensions<cityjson::prelude::OwnedStringStorage> {
    let mut extensions = Extensions::new();
    for (name, extension) in raw {
        extensions.add(Extension::new(name, extension.url, extension.version));
    }
    extensions
}

fn build_borrowed_extensions<'a>(
    raw: HashMap<&'a str, RawExtensionBorrowed<'a>>,
) -> Extensions<cityjson::prelude::BorrowedStringStorage<'a>> {
    let mut extensions = Extensions::new();
    for (name, extension) in raw {
        extensions.add(Extension::new(name, extension.url, extension.version));
    }
    extensions
}

fn build_owned_metadata(
    raw: RawMetadataOwned,
) -> Result<Metadata<cityjson::prelude::OwnedStringStorage>> {
    let mut metadata = Metadata::new();

    if let Some(extent) = raw.geographical_extent {
        metadata.set_geographical_extent(BBox::from(extent));
    }
    if let Some(identifier) = raw.identifier {
        metadata.set_identifier(CityModelIdentifier::new(identifier));
    }
    if let Some(contact) = raw.point_of_contact {
        metadata.set_point_of_contact(Some(build_owned_contact(contact)?));
    }
    if let Some(date) = raw.reference_date {
        metadata.set_reference_date(Date::new(date));
    }
    if let Some(reference_system) = raw.reference_system {
        metadata.set_reference_system(CRS::new(reference_system));
    }
    if let Some(title) = raw.title {
        metadata.set_title(title);
    }
    if !raw.extra.is_empty() {
        let value = OwnedJsonValue::Object(raw.extra.into_iter().collect());
        metadata.set_extra(Some(owned_attributes_from_json(
            &value,
            "metadata extra properties",
        )?));
    }

    Ok(metadata)
}

fn build_borrowed_metadata<'a>(
    raw: RawMetadataBorrowed<'a>,
) -> Result<Metadata<cityjson::prelude::BorrowedStringStorage<'a>>> {
    let mut metadata = Metadata::new();

    if let Some(extent) = raw.geographical_extent {
        metadata.set_geographical_extent(BBox::from(extent));
    }
    if let Some(identifier) = raw.identifier {
        metadata.set_identifier(CityModelIdentifier::new(identifier));
    }
    if let Some(contact) = raw.point_of_contact {
        metadata.set_point_of_contact(Some(build_borrowed_contact(contact)?));
    }
    if let Some(date) = raw.reference_date {
        metadata.set_reference_date(Date::new(date));
    }
    if let Some(reference_system) = raw.reference_system {
        metadata.set_reference_system(CRS::new(reference_system));
    }
    if let Some(title) = raw.title {
        metadata.set_title(title);
    }
    if !raw.extra.is_empty() {
        metadata.set_extra(Some(borrowed_attributes_from_map(
            raw.extra,
            "metadata extra properties",
        )?));
    }

    Ok(metadata)
}

fn build_owned_contact(
    raw: RawContactOwned,
) -> Result<Contact<cityjson::prelude::OwnedStringStorage>> {
    let mut contact = Contact::new();

    if let Some(value) = raw.contact_name {
        contact.set_contact_name(value);
    }
    if let Some(value) = raw.email_address {
        contact.set_email_address(value);
    }
    if let Some(value) = raw.role {
        contact.set_role(Some(parse_contact_role(&value)?));
    }
    if let Some(value) = raw.website {
        contact.set_website(Some(value));
    }
    if let Some(value) = raw.contact_type {
        contact.set_contact_type(Some(parse_contact_type(&value)?));
    }
    if let Some(value) = raw.address {
        contact.set_address(Some(owned_attributes_from_json(
            &value,
            "pointOfContact.address",
        )?));
    }
    if let Some(value) = raw.phone {
        contact.set_phone(Some(value));
    }
    if let Some(value) = raw.organization {
        contact.set_organization(Some(value));
    }

    Ok(contact)
}

fn build_borrowed_contact<'a>(
    raw: RawContactBorrowed<'a>,
) -> Result<Contact<cityjson::prelude::BorrowedStringStorage<'a>>> {
    let mut contact = Contact::new();

    if let Some(value) = raw.contact_name {
        contact.set_contact_name(value.to_owned());
    }
    if let Some(value) = raw.email_address {
        contact.set_email_address(value.to_owned());
    }
    if let Some(value) = raw.role {
        contact.set_role(Some(parse_contact_role(value)?));
    }
    if let Some(value) = raw.website {
        contact.set_website(Some(value.to_owned()));
    }
    if let Some(value) = raw.contact_type {
        contact.set_contact_type(Some(parse_contact_type(value)?));
    }
    if let Some(value) = raw.address {
        contact.set_address(Some(borrowed_attributes_from_json_owned(
            value,
            "pointOfContact.address",
        )?));
    }
    if let Some(value) = raw.phone {
        contact.set_phone(Some(value.to_owned()));
    }
    if let Some(value) = raw.organization {
        contact.set_organization(Some(value.to_owned()));
    }

    Ok(contact)
}

fn import_owned_cityobjects(
    raw_objects: HashMap<String, RawCityObjectOwned>,
    model: &mut OwnedCityModel,
) -> Result<()> {
    let mut handle_by_id = HashMap::with_capacity(raw_objects.len());
    let mut pending = Vec::with_capacity(raw_objects.len());

    for (id, raw_object) in raw_objects {
        let type_cityobject = parse_owned_cityobject_type(&raw_object.type_name)?;
        let mut cityobject =
            CityObject::new(CityObjectIdentifier::new(id.clone()), type_cityobject);

        if let Some(extent) = raw_object.geographical_extent {
            cityobject.set_geographical_extent(Some(BBox::from(extent)));
        }
        if let Some(attributes) = raw_object.attributes.as_ref() {
            *cityobject.attributes_mut() =
                owned_attributes_from_json(attributes, "CityObject.attributes")?;
        }
        if !raw_object.extra.is_empty() {
            let value = OwnedJsonValue::Object(raw_object.extra.into_iter().collect());
            *cityobject.extra_mut() = owned_attributes_from_json(&value, "CityObject extra")?;
        }
        for geometry in import_owned_geometries(raw_object.geometry, model)? {
            cityobject.add_geometry(geometry);
        }

        let handle = model.cityobjects_mut().add(cityobject)?;
        handle_by_id.insert(id.clone(), handle);
        pending.push(PendingRelationsOwned {
            source_id: id,
            source_handle: handle,
            parents: raw_object.parents,
            children: raw_object.children,
        });
    }

    resolve_owned_relations(pending, handle_by_id, model)
}

fn import_borrowed_cityobjects<'a>(
    raw_objects: HashMap<&'a str, RawCityObjectBorrowed<'a>>,
    model: &mut BorrowedCityModel<'a>,
) -> Result<()> {
    let mut handle_by_id = HashMap::with_capacity(raw_objects.len());
    let mut pending = Vec::with_capacity(raw_objects.len());

    for (id, raw_object) in raw_objects {
        let type_cityobject = parse_borrowed_cityobject_type(raw_object.type_name)?;
        let mut cityobject = CityObject::new(CityObjectIdentifier::new(id), type_cityobject);

        if let Some(extent) = raw_object.geographical_extent {
            cityobject.set_geographical_extent(Some(BBox::from(extent)));
        }
        if let Some(attributes) = raw_object.attributes {
            *cityobject.attributes_mut() =
                borrowed_attributes_from_json_owned(attributes, "CityObject.attributes")?;
        }
        if !raw_object.extra.is_empty() {
            *cityobject.extra_mut() =
                borrowed_attributes_from_map(raw_object.extra, "CityObject extra")?;
        }
        for geometry in import_borrowed_geometries(raw_object.geometry, model)? {
            cityobject.add_geometry(geometry);
        }

        let handle = model.cityobjects_mut().add(cityobject)?;
        handle_by_id.insert(id, handle);
        pending.push(PendingRelationsBorrowed {
            source_id: id,
            source_handle: handle,
            parents: raw_object.parents,
            children: raw_object.children,
        });
    }

    resolve_borrowed_relations(pending, handle_by_id, model)
}

fn resolve_owned_relations(
    pending: Vec<PendingRelationsOwned>,
    handle_by_id: HashMap<String, CityObjectHandle>,
    model: &mut OwnedCityModel,
) -> Result<()> {
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
            let handle = handle_by_id.get(parent.as_str()).copied().ok_or_else(|| {
                Error::UnresolvedCityObjectReference {
                    source_id: relation.source_id.clone(),
                    target_id: parent.clone(),
                    relation: "parent",
                }
            })?;
            cityobject.add_parent(handle);
        }

        for child in relation.children {
            let handle = handle_by_id.get(child.as_str()).copied().ok_or_else(|| {
                Error::UnresolvedCityObjectReference {
                    source_id: relation.source_id.clone(),
                    target_id: child.clone(),
                    relation: "child",
                }
            })?;
            cityobject.add_child(handle);
        }
    }

    Ok(())
}

fn resolve_borrowed_relations<'a>(
    pending: Vec<PendingRelationsBorrowed<'a>>,
    handle_by_id: HashMap<&'a str, CityObjectHandle>,
    model: &mut BorrowedCityModel<'a>,
) -> Result<()> {
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

fn reject_unsupported_root_sections_owned(
    appearance: Option<&OwnedJsonValue>,
    geometry_templates: Option<&OwnedJsonValue>,
) -> Result<()> {
    if appearance.is_some_and(value_is_present_owned) {
        return Err(Error::UnsupportedFeature(
            "appearance import is not implemented yet",
        ));
    }
    if geometry_templates.is_some_and(value_is_present_owned) {
        return Err(Error::UnsupportedFeature(
            "geometry template import is not implemented yet",
        ));
    }
    Ok(())
}

fn reject_unsupported_root_sections_borrowed(
    appearance: Option<&BorrowedJsonValue<'_>>,
    geometry_templates: Option<&BorrowedJsonValue<'_>>,
) -> Result<()> {
    if appearance.is_some_and(value_is_present_borrowed) {
        return Err(Error::UnsupportedFeature(
            "appearance import is not implemented yet",
        ));
    }
    if geometry_templates.is_some_and(value_is_present_borrowed) {
        return Err(Error::UnsupportedFeature(
            "geometry template import is not implemented yet",
        ));
    }
    Ok(())
}

fn value_is_present_owned(value: &OwnedJsonValue) -> bool {
    match value {
        OwnedJsonValue::Null => false,
        OwnedJsonValue::Array(values) => !values.is_empty(),
        OwnedJsonValue::Object(values) => !values.is_empty(),
        _ => true,
    }
}

fn value_is_present_borrowed(value: &BorrowedJsonValue<'_>) -> bool {
    match value {
        BorrowedJsonValue::Null => false,
        BorrowedJsonValue::Array(values) => !values.is_empty(),
        BorrowedJsonValue::Object(values) => !values.is_empty(),
        _ => true,
    }
}

fn parse_contact_role(value: &str) -> Result<ContactRole> {
    match value {
        "author" => Ok(ContactRole::Author),
        "processor" => Ok(ContactRole::Processor),
        "pointOfContact" => Ok(ContactRole::PointOfContact),
        "owner" => Ok(ContactRole::Owner),
        "user" => Ok(ContactRole::User),
        "distributor" => Ok(ContactRole::Distributor),
        "originator" => Ok(ContactRole::Originator),
        "custodian" => Ok(ContactRole::Custodian),
        "resourceProvider" => Ok(ContactRole::ResourceProvider),
        "rightsHolder" => Ok(ContactRole::RightsHolder),
        "sponsor" => Ok(ContactRole::Sponsor),
        "principalInvestigator" => Ok(ContactRole::PrincipalInvestigator),
        "stakeholder" => Ok(ContactRole::Stakeholder),
        "publisher" => Ok(ContactRole::Publisher),
        _ => Err(Error::InvalidValue(format!(
            "unsupported pointOfContact.role value '{value}'"
        ))),
    }
}

fn parse_contact_type(value: &str) -> Result<ContactType> {
    match value {
        "individual" => Ok(ContactType::Individual),
        "organization" => Ok(ContactType::Organization),
        _ => Err(Error::InvalidValue(format!(
            "unsupported pointOfContact.contactType value '{value}'"
        ))),
    }
}

fn parse_owned_cityobject_type(
    value: &str,
) -> Result<CityObjectType<cityjson::prelude::OwnedStringStorage>> {
    Ok(match value {
        "Bridge" => CityObjectType::Bridge,
        "BridgePart" => CityObjectType::BridgePart,
        "BridgeInstallation" => CityObjectType::BridgeInstallation,
        "BridgeConstructiveElement" => CityObjectType::BridgeConstructiveElement,
        "BridgeRoom" => CityObjectType::BridgeRoom,
        "BridgeFurniture" => CityObjectType::BridgeFurniture,
        "Building" => CityObjectType::Building,
        "BuildingPart" => CityObjectType::BuildingPart,
        "BuildingInstallation" => CityObjectType::BuildingInstallation,
        "BuildingConstructiveElement" => CityObjectType::BuildingConstructiveElement,
        "BuildingFurniture" => CityObjectType::BuildingFurniture,
        "BuildingStorey" => CityObjectType::BuildingStorey,
        "BuildingRoom" => CityObjectType::BuildingRoom,
        "BuildingUnit" => CityObjectType::BuildingUnit,
        "CityFurniture" => CityObjectType::CityFurniture,
        "CityObjectGroup" => CityObjectType::CityObjectGroup,
        "Default" => CityObjectType::Default,
        "GenericCityObject" => CityObjectType::GenericCityObject,
        "LandUse" => CityObjectType::LandUse,
        "OtherConstruction" => CityObjectType::OtherConstruction,
        "PlantCover" => CityObjectType::PlantCover,
        "SolitaryVegetationObject" => CityObjectType::SolitaryVegetationObject,
        "TINRelief" => CityObjectType::TINRelief,
        "WaterBody" => CityObjectType::WaterBody,
        "Road" => CityObjectType::Road,
        "Railway" => CityObjectType::Railway,
        "Waterway" => CityObjectType::Waterway,
        "TransportSquare" => CityObjectType::TransportSquare,
        "Tunnel" => CityObjectType::Tunnel,
        "TunnelPart" => CityObjectType::TunnelPart,
        "TunnelInstallation" => CityObjectType::TunnelInstallation,
        "TunnelConstructiveElement" => CityObjectType::TunnelConstructiveElement,
        "TunnelHollowSpace" => CityObjectType::TunnelHollowSpace,
        "TunnelFurniture" => CityObjectType::TunnelFurniture,
        _ if value.starts_with('+') => CityObjectType::Extension(value.to_owned()),
        _ => {
            return Err(Error::InvalidValue(format!(
                "invalid CityObject type '{value}'"
            )))
        }
    })
}

fn parse_borrowed_cityobject_type<'a>(
    value: &'a str,
) -> Result<CityObjectType<cityjson::prelude::BorrowedStringStorage<'a>>> {
    Ok(match value {
        "Bridge" => CityObjectType::Bridge,
        "BridgePart" => CityObjectType::BridgePart,
        "BridgeInstallation" => CityObjectType::BridgeInstallation,
        "BridgeConstructiveElement" => CityObjectType::BridgeConstructiveElement,
        "BridgeRoom" => CityObjectType::BridgeRoom,
        "BridgeFurniture" => CityObjectType::BridgeFurniture,
        "Building" => CityObjectType::Building,
        "BuildingPart" => CityObjectType::BuildingPart,
        "BuildingInstallation" => CityObjectType::BuildingInstallation,
        "BuildingConstructiveElement" => CityObjectType::BuildingConstructiveElement,
        "BuildingFurniture" => CityObjectType::BuildingFurniture,
        "BuildingStorey" => CityObjectType::BuildingStorey,
        "BuildingRoom" => CityObjectType::BuildingRoom,
        "BuildingUnit" => CityObjectType::BuildingUnit,
        "CityFurniture" => CityObjectType::CityFurniture,
        "CityObjectGroup" => CityObjectType::CityObjectGroup,
        "Default" => CityObjectType::Default,
        "GenericCityObject" => CityObjectType::GenericCityObject,
        "LandUse" => CityObjectType::LandUse,
        "OtherConstruction" => CityObjectType::OtherConstruction,
        "PlantCover" => CityObjectType::PlantCover,
        "SolitaryVegetationObject" => CityObjectType::SolitaryVegetationObject,
        "TINRelief" => CityObjectType::TINRelief,
        "WaterBody" => CityObjectType::WaterBody,
        "Road" => CityObjectType::Road,
        "Railway" => CityObjectType::Railway,
        "Waterway" => CityObjectType::Waterway,
        "TransportSquare" => CityObjectType::TransportSquare,
        "Tunnel" => CityObjectType::Tunnel,
        "TunnelPart" => CityObjectType::TunnelPart,
        "TunnelInstallation" => CityObjectType::TunnelInstallation,
        "TunnelConstructiveElement" => CityObjectType::TunnelConstructiveElement,
        "TunnelHollowSpace" => CityObjectType::TunnelHollowSpace,
        "TunnelFurniture" => CityObjectType::TunnelFurniture,
        _ if value.starts_with('+') => CityObjectType::Extension(value),
        _ => {
            return Err(Error::InvalidValue(format!(
                "invalid CityObject type '{value}'"
            )))
        }
    })
}
