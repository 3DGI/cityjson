//! `CityJSON` metadata fields.
//!
//! [`Metadata`] corresponds to the optional `metadata` member of a `CityJSON` object.
//! All fields are optional. The spec defines these fields, aligned with ISO 19115:
//!
//! | Field | Type | Notes |
//! |---|---|---|
//! | `geographicalExtent` | [`BBox`] | `[minx, miny, minz, maxx, maxy, maxz]` |
//! | `identifier` | [`CityModelIdentifier`] | e.g. a UUID |
//! | `referenceDate` | [`Date`] | `YYYY-MM-DD` (RFC 3339 full-date) |
//! | `referenceSystem` | [`CRS`] | OGC CRS URL, e.g. `https://www.opengis.net/def/crs/EPSG/0/7415` |
//! | `title` | `String` | Human-readable dataset name |
//! | `pointOfContact` | [`Contact`] | Contact information |
//!
//! ```rust
//! use cityjson::CityModelType;
//! use cityjson::v2_0::{BBox, CRS, CityModelIdentifier, Date, OwnedCityModel};
//!
//! let mut model = OwnedCityModel::new(CityModelType::CityJSON);
//! let meta = model.metadata_mut();
//!
//! meta.set_geographical_extent(BBox::new(84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9));
//! meta.set_identifier(CityModelIdentifier::new("eaeceeaa-3f66-429a-b81d-bbc6140b8c1c".to_string()));
//! meta.set_reference_date(Date::new("1977-02-28".to_string()));
//! meta.set_reference_system(CRS::new("https://www.opengis.net/def/crs/EPSG/0/7415".to_string()));
//! meta.set_title("Amsterdam buildings LoD2".to_string());
//! ```

use crate::format_option;
use crate::resources::storage::StringStorage;
use crate::v2_0::attributes::Attributes;
use std::fmt::{Display, Formatter};

pub use crate::cityjson::core::metadata::{BBox, CRS, CityModelIdentifier, Date};

/// Metadata for a `CityJSON` document. See the [module docs](self) for field descriptions.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Metadata<SS: StringStorage> {
    geographical_extent: Option<BBox>,
    identifier: Option<CityModelIdentifier<SS>>,
    point_of_contact: Option<Contact<SS>>,
    reference_date: Option<Date<SS>>,
    reference_system: Option<CRS<SS>>,
    title: Option<SS::String>,
    extra: Option<Attributes<SS>>,
}

impl<SS: StringStorage> Metadata<SS> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn geographical_extent(&self) -> Option<&BBox> {
        self.geographical_extent.as_ref()
    }

    pub fn identifier(&self) -> Option<&CityModelIdentifier<SS>> {
        self.identifier.as_ref()
    }

    pub fn reference_date(&self) -> Option<&Date<SS>> {
        self.reference_date.as_ref()
    }

    pub fn reference_system(&self) -> Option<&CRS<SS>> {
        self.reference_system.as_ref()
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn extra(&self) -> Option<&Attributes<SS>> {
        self.extra.as_ref()
    }

    /// Returns a mutable reference to the extra attributes, inserting an empty map if absent.
    pub fn extra_mut(&mut self) -> &mut Attributes<SS> {
        self.extra.get_or_insert_with(Attributes::new)
    }

    pub fn set_extra(&mut self, extra: Option<Attributes<SS>>) {
        self.extra = extra;
    }

    pub fn set_geographical_extent(&mut self, bbox: BBox) {
        self.geographical_extent = Some(bbox);
    }

    pub fn set_identifier(&mut self, identifier: CityModelIdentifier<SS>) {
        self.identifier = Some(identifier);
    }

    pub fn set_reference_date(&mut self, date: Date<SS>) {
        self.reference_date = Some(date);
    }

    pub fn set_reference_system(&mut self, crs: CRS<SS>) {
        self.reference_system = Some(crs);
    }

    pub fn set_title(&mut self, title: SS::String) {
        self.title = Some(title);
    }

    pub fn point_of_contact(&self) -> Option<&Contact<SS>> {
        self.point_of_contact.as_ref()
    }

    pub fn set_point_of_contact(&mut self, contact: Option<Contact<SS>>) {
        self.point_of_contact = contact;
    }
}

impl<SS: StringStorage> std::fmt::Display for Metadata<SS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "geographical_extent: {}, identifier: {}, point_of_contact: {}, reference_date: {}, reference_system: {}, title: {}",
            format_option(self.geographical_extent.as_ref()),
            format_option(self.identifier.as_ref()),
            format_option(self.point_of_contact.as_ref()),
            format_option(self.reference_date.as_ref()),
            format_option(self.reference_system.as_ref()),
            format_option(self.title.as_ref())
        )
    }
}

/// Point-of-contact information within [`Metadata`].
///
/// Corresponds to the `pointOfContact` field in the `CityJSON` spec.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Contact<SS: StringStorage> {
    name: SS::String,
    email_address: SS::String,
    role: Option<ContactRole>,
    website: Option<SS::String>,
    kind: Option<ContactType>,
    address: Option<Attributes<SS>>,
    phone: Option<SS::String>,
    organization: Option<SS::String>,
}

impl<SS: StringStorage> Contact<SS> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn contact_name(&self) -> &str {
        self.name.as_ref()
    }

    #[must_use]
    pub fn email_address(&self) -> &str {
        self.email_address.as_ref()
    }

    #[must_use]
    pub fn role(&self) -> Option<ContactRole> {
        self.role
    }

    #[must_use]
    pub fn website(&self) -> &Option<SS::String> {
        &self.website
    }

    #[must_use]
    pub fn contact_type(&self) -> Option<ContactType> {
        self.kind
    }

    #[must_use]
    pub fn phone(&self) -> &Option<SS::String> {
        &self.phone
    }

    #[must_use]
    pub fn organization(&self) -> &Option<SS::String> {
        &self.organization
    }

    pub fn set_contact_name(&mut self, contact_name: SS::String) {
        self.name = contact_name;
    }

    pub fn set_email_address(&mut self, email_address: SS::String) {
        self.email_address = email_address;
    }

    pub fn set_role(&mut self, role: Option<ContactRole>) {
        self.role = role;
    }

    pub fn set_website(&mut self, website: Option<SS::String>) {
        self.website = website;
    }

    pub fn set_contact_type(&mut self, contact_type: Option<ContactType>) {
        self.kind = contact_type;
    }

    pub fn set_phone(&mut self, phone: Option<SS::String>) {
        self.phone = phone;
    }

    pub fn set_organization(&mut self, organization: Option<SS::String>) {
        self.organization = organization;
    }

    #[must_use]
    pub fn address(&self) -> Option<&Attributes<SS>> {
        self.address.as_ref()
    }

    pub fn address_mut(&mut self) -> &mut Attributes<SS> {
        self.address.get_or_insert_with(Attributes::new)
    }

    pub fn set_address(&mut self, address: Option<Attributes<SS>>) {
        self.address = address;
    }
}

impl<SS: StringStorage> Display for Contact<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "contact_name: {}, email_address: {}, role: {}, website: {}, contact_type: {}, address: {}, phone: {}, organization: {}",
            self.name,
            self.email_address,
            format_option(self.role.as_ref()),
            format_option(self.website.as_ref()),
            format_option(self.kind.as_ref()),
            format_option(self.address.as_ref()),
            format_option(self.phone.as_ref()),
            format_option(self.organization.as_ref())
        )
    }
}

/// Role of the point of contact, as defined in ISO 19115.
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ContactRole {
    Author,
    CoAuthor,
    Processor,
    PointOfContact,
    Owner,
    User,
    Distributor,
    Originator,
    Custodian,
    ResourceProvider,
    RightsHolder,
    Sponsor,
    PrincipalInvestigator,
    Stakeholder,
    Publisher,
}

impl Display for ContactRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Whether the point of contact is an individual or an organization.
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ContactType {
    Individual,
    Organization,
}

impl Display for ContactType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
