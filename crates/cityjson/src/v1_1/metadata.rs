//! # Metadata
//!
//! Represents a [Metadata object](https://www.cityjson.org/specs/1.1.3/#metadata).

use crate::cityjson;
use crate::cityjson::attributes::Attributes;
use crate::resources::storage::StringStorage;
use std::fmt::{Display, Formatter};

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Metadata<SS: StringStorage> {
    pub geographical_extent: Option<BBox>,
    pub identifier: Option<CityModelIdentifier>,
    pub point_of_contact: Option<Contact>,
    pub reference_date: Option<Date>,
    pub reference_system: Option<CRS>,
    pub title: Option<String>,
    pub extra: Option<Attributes<SS>>,
}

pub type BBox = [f32; 6];
pub type CityModelIdentifier = String;

#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Contact {
    contact_name: String,
    email_address: String,
    role: Option<ContactRole>,
    website: Option<String>,
    contact_type: Option<ContactType>,
    address: Option<String>,
    phone: Option<String>,
    organization: Option<String>,
}

/// Metadata contact role.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#pointofcontact>
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

/// Metadata contact type.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#pointofcontact>
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ContactType {
    Individual,
    Organization,
}

pub type Date = String;

pub type CRS = String;

impl<SS: StringStorage> Metadata<SS> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn geographical_extent(&self) -> Option<&BBox> {
        self.geographical_extent.as_ref()
    }

    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    pub fn point_of_contact(&self) -> Option<&Contact> {
        self.point_of_contact.as_ref()
    }

    pub fn reference_date(&self) -> Option<&str> {
        self.reference_date.as_deref()
    }

    pub fn reference_system(&self) -> Option<&str> {
        self.reference_system.as_deref()
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn extra(&self) -> Option<&Attributes<SS>> {
        self.extra.as_ref()
    }

    pub fn extra_mut(&mut self) -> &mut Option<Attributes<SS>> {
        &mut self.extra
    }

    pub fn set_geographical_extent(&mut self, bbox: BBox) {
        self.geographical_extent = Some(bbox);
    }

    pub fn set_identifier<S: AsRef<str>>(&mut self, identifier: S) {
        self.identifier = Some(identifier.as_ref().to_owned());
    }

    pub fn set_reference_date<S: AsRef<str>>(&mut self, date: S) {
        self.reference_date = Some(date.as_ref().to_owned());
    }

    pub fn set_reference_system<S: AsRef<str>>(&mut self, crs: S) {
        self.reference_system = Some(crs.as_ref().to_owned());
    }

    pub fn set_title<S: AsRef<str>>(&mut self, title: S) {
        self.title = Some(title.as_ref().to_owned());
    }

    pub fn set_contact_name<S: AsRef<str>>(&mut self, name: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.contact_name = name.as_ref().to_owned()
        } else {
            self.point_of_contact = Some(Contact {
                contact_name: name.as_ref().to_owned(),
                ..Default::default()
            })
        }
    }

    pub fn set_email_address<S: AsRef<str>>(&mut self, email: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.email_address = email.as_ref().to_owned()
        } else {
            self.point_of_contact = Some(Contact {
                email_address: email.as_ref().to_owned(),
                ..Default::default()
            })
        }
    }

    pub fn set_role(&mut self, role: ContactRole) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.role = Some(role);
        } else {
            self.point_of_contact = Some(Contact {
                role: Some(role),
                ..Default::default()
            })
        }
    }

    pub fn set_website<S: AsRef<str>>(&mut self, website: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.website = Some(website.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                website: Some(website.as_ref().to_owned()),
                ..Default::default()
            })
        }
    }

    pub fn set_contact_type(&mut self, contact_type: ContactType) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.contact_type = Some(contact_type);
        } else {
            self.point_of_contact = Some(Contact {
                contact_type: Some(contact_type),
                ..Default::default()
            })
        }
    }

    pub fn set_address<S: AsRef<str>>(&mut self, address: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.address = Some(address.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                address: Some(address.as_ref().to_owned()),
                ..Default::default()
            })
        }
    }

    pub fn set_phone<S: AsRef<str>>(&mut self, phone: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.phone = Some(phone.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                phone: Some(phone.as_ref().to_owned()),
                ..Default::default()
            })
        }
    }

    pub fn set_organization<S: AsRef<str>>(&mut self, organization: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.organization = Some(organization.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                organization: Some(organization.as_ref().to_owned()),
                ..Default::default()
            })
        }
    }
}

impl<SS: StringStorage> Display for Metadata<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "geographical_extent: {:#?}, identifier: {:#?}, point_of_contact: {:#?},
        reference_date: {:#?}, reference_system: {:#?}, title: {:#?}",
            self.geographical_extent,
            self.identifier,
            self.point_of_contact,
            self.reference_date,
            self.reference_system,
            self.title
        )
    }
}

impl Display for Contact {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "contact_name: {}, email_address: {}, role: {:?}, website: {:#?},
        contact_type: {:#?}, address: {:#?}, phone: {:#?}, organization: {:#?},",
            self.contact_name,
            self.email_address,
            self.role,
            self.website,
            self.contact_type,
            self.address,
            self.phone,
            self.organization
        )
    }
}

impl Display for ContactRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Display for ContactType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
impl<SS: StringStorage> cityjson::metadata::Metadata<SS> for Metadata<SS> {}
