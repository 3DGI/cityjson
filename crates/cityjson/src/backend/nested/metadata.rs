//! Metadata types for the nested backend.

use crate::backend::nested::attributes::Attributes;
use crate::format_option;
use crate::resources::storage::StringStorage;
use std::fmt::{Display, Formatter};

/// Bounding Box.
///
/// A wrapper around an array of 6 values: `[minx, miny, minz, maxx, maxy, maxz]`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BBox {
    values: [f64; 6],
}

impl BBox {
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            values: [min_x, min_y, min_z, max_x, max_y, max_z],
        }
    }

    pub fn as_slice(&self) -> &[f64] {
        &self.values
    }

    pub fn min_x(&self) -> f64 {
        self.values[0]
    }

    pub fn min_y(&self) -> f64 {
        self.values[1]
    }

    pub fn min_z(&self) -> f64 {
        self.values[2]
    }

    pub fn max_x(&self) -> f64 {
        self.values[3]
    }

    pub fn max_y(&self) -> f64 {
        self.values[4]
    }

    pub fn max_z(&self) -> f64 {
        self.values[5]
    }

    pub fn width(&self) -> f64 {
        self.max_x() - self.min_x()
    }

    pub fn length(&self) -> f64 {
        self.max_y() - self.min_y()
    }

    pub fn height(&self) -> f64 {
        self.max_z() - self.min_z()
    }
}

impl Default for BBox {
    fn default() -> Self {
        Self {
            values: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }
}

impl From<[f64; 6]> for BBox {
    fn from(values: [f64; 6]) -> Self {
        Self { values }
    }
}

impl From<BBox> for [f64; 6] {
    fn from(bbox: BBox) -> Self {
        bbox.values
    }
}

impl Display for BBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}, {}, {}, {}, {}, {}]",
            self.min_x(),
            self.min_y(),
            self.min_z(),
            self.max_x(),
            self.max_y(),
            self.max_z()
        )
    }
}

/// An identifier for the dataset.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct CityModelIdentifier<SS: StringStorage>(SS::String);

impl<SS: StringStorage> CityModelIdentifier<SS> {
    pub fn new(value: SS::String) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> SS::String {
        self.0
    }
}

impl<SS: StringStorage> Display for CityModelIdentifier<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The date when the dataset was compiled.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Date<SS: StringStorage>(SS::String);

impl<SS: StringStorage> Date<SS> {
    pub fn new(value: SS::String) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> SS::String {
        self.0
    }
}

impl<SS: StringStorage> Display for Date<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The coordinate reference system (CRS) of the city model.
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct CRS<SS: StringStorage>(SS::String);

impl<SS: StringStorage> CRS<SS> {
    pub fn new(value: SS::String) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> SS::String {
        self.0
    }
}

impl<SS: StringStorage> Display for CRS<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metadata<SS: StringStorage, RR> {
    geographical_extent: Option<BBox>,
    identifier: Option<CityModelIdentifier<SS>>,
    point_of_contact: Option<Contact<SS, RR>>,
    reference_date: Option<Date<SS>>,
    reference_system: Option<CRS<SS>>,
    title: Option<String>,
    extra: Option<Attributes<SS, RR>>,
}

impl<SS: StringStorage, RR> Metadata<SS, RR> {
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

    pub fn extra(&self) -> Option<&Attributes<SS, RR>> {
        self.extra.as_ref()
    }

    pub fn extra_mut(&mut self) -> &mut Option<Attributes<SS, RR>> {
        &mut self.extra
    }

    pub fn set_extra(&mut self, extra: Option<Attributes<SS, RR>>) {
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

    pub fn set_title<S: AsRef<str>>(&mut self, title: S) {
        self.title = Some(title.as_ref().to_owned());
    }

    pub fn set_phone<S: AsRef<str>>(&mut self, phone: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.phone = Some(phone.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                phone: Some(phone.as_ref().to_owned()),
                ..Contact::new()
            })
        }
    }

    pub fn set_organization<S: AsRef<str>>(&mut self, organization: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.organization = Some(organization.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                organization: Some(organization.as_ref().to_owned()),
                ..Contact::new()
            })
        }
    }

    pub fn point_of_contact(&self) -> Option<&Contact<SS, RR>> {
        self.point_of_contact.as_ref()
    }

    pub fn set_contact_name<S: AsRef<str>>(&mut self, name: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.contact_name = name.as_ref().to_owned();
        } else {
            self.point_of_contact = Some(Contact {
                contact_name: name.as_ref().to_owned(),
                ..Contact::new()
            })
        }
    }

    pub fn set_email_address<S: AsRef<str>>(&mut self, email: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.email_address = email.as_ref().to_owned();
        } else {
            self.point_of_contact = Some(Contact {
                email_address: email.as_ref().to_owned(),
                ..Contact::new()
            })
        }
    }

    pub fn set_role(&mut self, role: ContactRole) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.role = Some(role);
        } else {
            self.point_of_contact = Some(Contact {
                role: Some(role),
                ..Contact::new()
            })
        }
    }

    pub fn set_website<S: AsRef<str>>(&mut self, website: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.website = Some(website.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                website: Some(website.as_ref().to_owned()),
                ..Contact::new()
            })
        }
    }

    pub fn set_contact_type(&mut self, contact_type: ContactType) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.contact_type = Some(contact_type);
        } else {
            self.point_of_contact = Some(Contact {
                contact_type: Some(contact_type),
                ..Contact::new()
            })
        }
    }

    pub fn set_address(&mut self, address: Attributes<SS, RR>) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.address = Some(address);
        } else {
            self.point_of_contact = Some(Contact {
                address: Some(address),
                ..Contact::new()
            })
        }
    }

    pub fn address_mut(&mut self) {}

    pub fn set_point_of_contact(&mut self, contact: Option<Contact<SS, RR>>) {
        self.point_of_contact = contact;
    }
}

impl<SS: StringStorage, RR> Default for Metadata<SS, RR> {
    fn default() -> Self {
        Self {
            geographical_extent: None,
            identifier: None,
            point_of_contact: None,
            reference_date: None,
            reference_system: None,
            title: None,
            extra: None,
        }
    }
}

impl<SS: StringStorage, RR> Display for Metadata<SS, RR>
where
    Attributes<SS, RR>: Display,
    CityModelIdentifier<SS>: Display,
    Contact<SS, RR>: Display,
    CRS<SS>: Display,
    Date<SS>: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "geographical_extent: {}, identifier: {}, point_of_contact: {},
            reference_date: {}, reference_system: {}, title: {}",
            format_option(&self.geographical_extent),
            format_option(&self.identifier),
            format_option(&self.point_of_contact),
            format_option(&self.reference_date),
            format_option(&self.reference_system),
            format_option(&self.title)
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Contact<SS: StringStorage, RR> {
    contact_name: String,
    email_address: String,
    role: Option<ContactRole>,
    website: Option<String>,
    contact_type: Option<ContactType>,
    address: Option<Attributes<SS, RR>>,
    phone: Option<String>,
    organization: Option<String>,
}

impl<SS: StringStorage, RR> Contact<SS, RR> {
    pub fn new() -> Self {
        Self {
            contact_name: "".to_string(),
            email_address: "".to_string(),
            role: None,
            website: None,
            contact_type: None,
            address: None,
            phone: None,
            organization: None,
        }
    }

    pub fn contact_name(&self) -> &str {
        &self.contact_name
    }

    pub fn email_address(&self) -> &str {
        &self.email_address
    }

    pub fn role(&self) -> Option<ContactRole> {
        self.role
    }

    pub fn website(&self) -> &Option<String> {
        &self.website
    }

    pub fn contact_type(&self) -> Option<ContactType> {
        self.contact_type
    }

    pub fn phone(&self) -> &Option<String> {
        &self.phone
    }

    pub fn organization(&self) -> &Option<String> {
        &self.organization
    }

    pub fn set_contact_name(&mut self, contact_name: String) {
        self.contact_name = contact_name;
    }

    pub fn set_email_address(&mut self, email_address: String) {
        self.email_address = email_address;
    }

    pub fn set_role(&mut self, role: Option<ContactRole>) {
        self.role = role;
    }

    pub fn set_website(&mut self, website: Option<String>) {
        self.website = website;
    }

    pub fn set_contact_type(&mut self, contact_type: Option<ContactType>) {
        self.contact_type = contact_type;
    }

    pub fn set_phone(&mut self, phone: Option<String>) {
        self.phone = phone;
    }

    pub fn set_organization(&mut self, organization: Option<String>) {
        self.organization = organization;
    }

    pub fn address(&self) -> Option<&Attributes<SS, RR>> {
        self.address.as_ref()
    }

    pub fn address_mut(&mut self) -> Option<&mut Attributes<SS, RR>> {
        self.address.as_mut()
    }

    pub fn set_address(&mut self, address: Option<Attributes<SS, RR>>) {
        self.address = address;
    }
}

impl<SS: StringStorage, RR> Default for Contact<SS, RR> {
    fn default() -> Self {
        Self::new()
    }
}

impl<SS: StringStorage, RR> Display for Contact<SS, RR>
where
    Attributes<SS, RR>: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "contact_name: {}, email_address: {}, role: {}, website: {},
            contact_type: {}, address: {}, phone: {}, organization: {}",
            self.contact_name,
            self.email_address,
            format_option(&self.role),
            format_option(&self.website),
            format_option(&self.contact_type),
            format_option(&self.address),
            format_option(&self.phone),
            format_option(&self.organization)
        )
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ContactRole {
    Author,
    CoAuthor,
    Collaborator,
    Contributor,
    Custodian,
    Distributor,
    Editor,
    Funder,
    Mediator,
    Originator,
    Owner,
    PointOfContact,
    PrincipalInvestigator,
    Processor,
    Publisher,
    ResourceProvider,
    RightsHolder,
    Sponsor,
    Stakeholder,
    User,
}

impl Display for ContactRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ContactType {
    Individual,
    Organization,
}

impl Display for ContactType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
