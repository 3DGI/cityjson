use crate::attributes::Attributes;
use crate::errors;
use serde_cityjson::v1_1 as cityjson;
use std::fmt;

pub type BBox = [f32; 6];
pub type CityModelIdentifier = String;
pub type Date = String;
pub type CRS = String;

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub geographical_extent: Option<BBox>,
    pub identifier: Option<CityModelIdentifier>,
    pub point_of_contact: Option<Contact>,
    pub reference_date: Option<Date>,
    pub reference_system: Option<CRS>,
    pub title: Option<String>,
    pub extra: Option<Attributes>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Contact {
    pub contact_name: String,
    pub email_address: String,
    pub role: Option<ContactRole>,
    pub website: Option<String>,
    pub contact_type: Option<ContactType>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ContactType {
    Individual,
    Organization,
}

impl Metadata {
    pub fn new() -> Self {
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

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TryFrom<cityjson::Metadata<'a>> for Metadata {
    type Error = errors::Error;

    fn try_from(metadata: cityjson::Metadata<'a>) -> errors::Result<Self> {
        Ok(Self {
            geographical_extent: metadata.geographical_extent,
            identifier: metadata.identifier.map(|s| s.to_string()),
            point_of_contact: metadata.point_of_contact.map(Contact::from),
            reference_date: metadata.reference_date.map(|s| s.to_string()),
            reference_system: metadata.reference_system.map(|s| s.to_string()),
            title: metadata.title.map(|s| s.to_string()),
            extra: metadata
                .extra
                .map(|e| Attributes::try_from(e))
                .transpose()?,
        })
    }
}

impl From<cityjson::Contact> for Contact {
    fn from(contact: cityjson::Contact) -> Self {
        Self {
            contact_name: contact.contact_name.to_string(),
            email_address: contact.email_address.to_string(),
            role: contact.role.map(ContactRole::from),
            website: contact.website.map(|s| s.to_string()),
            contact_type: contact.contact_type.map(ContactType::from),
            address: contact.address.map(|s| s.to_string()),
            phone: contact.phone.map(|s| s.to_string()),
            organization: contact.organization.map(|s| s.to_string()),
        }
    }
}

impl From<cityjson::ContactRole> for ContactRole {
    fn from(role: cityjson::ContactRole) -> Self {
        match role {
            cityjson::ContactRole::Author => ContactRole::Author,
            cityjson::ContactRole::CoAuthor => ContactRole::CoAuthor,
            cityjson::ContactRole::Collaborator => ContactRole::Collaborator,
            cityjson::ContactRole::Contributor => ContactRole::Contributor,
            cityjson::ContactRole::Custodian => ContactRole::Custodian,
            cityjson::ContactRole::Distributor => ContactRole::Distributor,
            cityjson::ContactRole::Editor => ContactRole::Editor,
            cityjson::ContactRole::Funder => ContactRole::Funder,
            cityjson::ContactRole::Mediator => ContactRole::Mediator,
            cityjson::ContactRole::Originator => ContactRole::Originator,
            cityjson::ContactRole::Owner => ContactRole::Owner,
            cityjson::ContactRole::PointOfContact => ContactRole::PointOfContact,
            cityjson::ContactRole::PrincipalInvestigator => ContactRole::PrincipalInvestigator,
            cityjson::ContactRole::Processor => ContactRole::Processor,
            cityjson::ContactRole::Publisher => ContactRole::Publisher,
            cityjson::ContactRole::ResourceProvider => ContactRole::ResourceProvider,
            cityjson::ContactRole::RightsHolder => ContactRole::RightsHolder,
            cityjson::ContactRole::Sponsor => ContactRole::Sponsor,
            cityjson::ContactRole::Stakeholder => ContactRole::Stakeholder,
            cityjson::ContactRole::User => ContactRole::User,
        }
    }
}

impl From<cityjson::ContactType> for ContactType {
    fn from(contact_type: cityjson::ContactType) -> Self {
        match contact_type {
            cityjson::ContactType::Individual => ContactType::Individual,
            cityjson::ContactType::Organization => ContactType::Organization,
        }
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        let mut first = true;

        if let Some(extent) = &self.geographical_extent {
            write!(
                f,
                "\"geographical_extent\": [{}, {}, {}, {}, {}, {}]",
                extent[0], extent[1], extent[2], extent[3], extent[4], extent[5]
            )?;
            first = false;
        }

        if let Some(id) = &self.identifier {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "\"identifier\": \"{}\"", id)?;
            first = false;
        }

        if let Some(contact) = &self.point_of_contact {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "\"point_of_contact\": {}", contact)?;
            first = false;
        }

        if let Some(date) = &self.reference_date {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "\"reference_date\": \"{}\"", date)?;
            first = false;
        }

        if let Some(crs) = &self.reference_system {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "\"reference_system\": \"{}\"", crs)?;
            first = false;
        }

        if let Some(title) = &self.title {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "\"title\": \"{}\"", title)?;
            first = false;
        }

        if let Some(extra) = &self.extra {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "\"extra\": {}", extra)?;
        }

        write!(f, "}}")
    }
}

impl fmt::Display for Contact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        write!(
            f,
            "\"contact_name\": \"{}\", \"email_address\": \"{}\"",
            self.contact_name, self.email_address
        )?;

        if let Some(role) = &self.role {
            write!(f, ", \"role\": \"{}\"", role)?;
        }

        if let Some(website) = &self.website {
            write!(f, ", \"website\": \"{}\"", website)?;
        }

        if let Some(contact_type) = &self.contact_type {
            write!(f, ", \"contact_type\": \"{}\"", contact_type)?;
        }

        if let Some(address) = &self.address {
            write!(f, ", \"address\": \"{}\"", address)?;
        }

        if let Some(phone) = &self.phone {
            write!(f, ", \"phone\": \"{}\"", phone)?;
        }

        if let Some(organization) = &self.organization {
            write!(f, ", \"organization\": \"{}\"", organization)?;
        }

        write!(f, "}}")
    }
}

impl fmt::Display for ContactRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContactRole::Author => write!(f, "Author"),
            ContactRole::CoAuthor => write!(f, "CoAuthor"),
            ContactRole::Collaborator => write!(f, "Collaborator"),
            ContactRole::Contributor => write!(f, "Contributor"),
            ContactRole::Custodian => write!(f, "Custodian"),
            ContactRole::Distributor => write!(f, "Distributor"),
            ContactRole::Editor => write!(f, "Editor"),
            ContactRole::Funder => write!(f, "Funder"),
            ContactRole::Mediator => write!(f, "Mediator"),
            ContactRole::Originator => write!(f, "Originator"),
            ContactRole::Owner => write!(f, "Owner"),
            ContactRole::PointOfContact => write!(f, "PointOfContact"),
            ContactRole::PrincipalInvestigator => write!(f, "PrincipalInvestigator"),
            ContactRole::Processor => write!(f, "Processor"),
            ContactRole::Publisher => write!(f, "Publisher"),
            ContactRole::ResourceProvider => write!(f, "ResourceProvider"),
            ContactRole::RightsHolder => write!(f, "RightsHolder"),
            ContactRole::Sponsor => write!(f, "Sponsor"),
            ContactRole::Stakeholder => write!(f, "Stakeholder"),
            ContactRole::User => write!(f, "User"),
        }
    }
}

impl fmt::Display for ContactType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContactType::Individual => write!(f, "Individual"),
            ContactType::Organization => write!(f, "Organization"),
        }
    }
}
