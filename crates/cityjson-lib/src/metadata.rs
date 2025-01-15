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
    geographical_extent: Option<BBox>,
    identifier: Option<CityModelIdentifier>,
    point_of_contact: Option<Contact>,
    reference_date: Option<Date>,
    reference_system: Option<CRS>,
    title: Option<String>,
    extra: Option<Attributes>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

    // Getters
    pub fn geographical_extent(&self) -> Option<&BBox> {
        self.geographical_extent.as_ref()
    }

    pub fn identifier(&self) -> Option<&CityModelIdentifier> {
        self.identifier.as_ref()
    }

    pub fn point_of_contact(&self) -> Option<&Contact> {
        self.point_of_contact.as_ref()
    }

    pub fn reference_date(&self) -> Option<&Date> {
        self.reference_date.as_ref()
    }

    pub fn reference_system(&self) -> Option<&CRS> {
        self.reference_system.as_ref()
    }

    pub fn title(&self) -> Option<&String> {
        self.title.as_ref()
    }

    pub fn extra(&self) -> Option<&Attributes> {
        self.extra.as_ref()
    }

    // Setters
    pub fn set_geographical_extent(&mut self, extent: BBox) {
        self.geographical_extent = Some(extent);
    }

    pub fn set_identifier(&mut self, identifier: impl Into<String>) {
        self.identifier = Some(identifier.into());
    }

    pub fn set_point_of_contact(&mut self, contact: Contact) {
        self.point_of_contact = Some(contact);
    }

    pub fn set_reference_date(&mut self, date: impl Into<String>) {
        self.reference_date = Some(date.into());
    }

    pub fn set_reference_system(&mut self, crs: impl Into<String>) {
        self.reference_system = Some(crs.into());
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    pub fn set_extra(&mut self, extra: Attributes) {
        self.extra = Some(extra);
    }

    // Mutable access
    pub fn point_of_contact_mut(&mut self) -> &mut Contact {
        self.point_of_contact.get_or_insert_with(Contact::new)
    }

    pub fn extra_mut(&mut self) -> &mut Attributes {
        self.extra.get_or_insert_with(Attributes::default)
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
        let mut new_metadata = Metadata::new();

        if let Some(extent) = metadata.geographical_extent {
            new_metadata.set_geographical_extent(extent);
        }

        if let Some(id) = metadata.identifier {
            new_metadata.set_identifier(id.to_string());
        }

        if let Some(contact) = metadata.point_of_contact {
            new_metadata.set_point_of_contact(Contact::from(contact));
        }

        if let Some(date) = metadata.reference_date {
            new_metadata.set_reference_date(date.to_string());
        }

        if let Some(crs) = metadata.reference_system {
            new_metadata.set_reference_system(crs.to_string());
        }

        if let Some(title) = metadata.title {
            new_metadata.set_title(title.to_string());
        }

        if let Some(extra) = metadata.extra {
            new_metadata.set_extra(Attributes::try_from(extra)?);
        }

        Ok(new_metadata)
    }
}

impl Contact {
    pub fn new() -> Self {
        Self {
            contact_name: String::new(),
            email_address: String::new(),
            role: None,
            website: None,
            contact_type: None,
            address: None,
            phone: None,
            organization: None,
        }
    }

    // Getters
    pub fn contact_name(&self) -> &str {
        &self.contact_name
    }

    pub fn email_address(&self) -> &str {
        &self.email_address
    }

    pub fn role(&self) -> Option<&ContactRole> {
        self.role.as_ref()
    }

    pub fn website(&self) -> Option<&str> {
        self.website.as_deref()
    }

    pub fn contact_type(&self) -> Option<&ContactType> {
        self.contact_type.as_ref()
    }

    pub fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }

    pub fn phone(&self) -> Option<&str> {
        self.phone.as_deref()
    }

    pub fn organization(&self) -> Option<&str> {
        self.organization.as_deref()
    }

    // Setters
    pub fn set_contact_name(&mut self, name: impl Into<String>) {
        self.contact_name = name.into();
    }

    pub fn set_email_address(&mut self, email: impl Into<String>) {
        self.email_address = email.into();
    }

    pub fn set_role(&mut self, role: ContactRole) {
        self.role = Some(role);
    }

    pub fn set_website(&mut self, website: impl Into<String>) {
        self.website = Some(website.into());
    }

    pub fn set_contact_type(&mut self, contact_type: ContactType) {
        self.contact_type = Some(contact_type);
    }

    pub fn set_address(&mut self, address: impl Into<String>) {
        self.address = Some(address.into());
    }

    pub fn set_phone(&mut self, phone: impl Into<String>) {
        self.phone = Some(phone.into());
    }

    pub fn set_organization(&mut self, organization: impl Into<String>) {
        self.organization = Some(organization.into());
    }
}

impl Default for Contact {
    fn default() -> Self {
        Self::new()
    }
}

impl From<cityjson::Contact> for Contact {
    fn from(contact: cityjson::Contact) -> Self {
        let mut new_contact = Contact::new();

        new_contact.set_contact_name(contact.contact_name.to_string());
        new_contact.set_email_address(contact.email_address.to_string());

        if let Some(role) = contact.role {
            new_contact.set_role(ContactRole::from(role));
        }

        if let Some(website) = contact.website {
            new_contact.set_website(website.to_string());
        }

        if let Some(contact_type) = contact.contact_type {
            new_contact.set_contact_type(ContactType::from(contact_type));
        }

        if let Some(address) = contact.address {
            new_contact.set_address(address.to_string());
        }

        if let Some(phone) = contact.phone {
            new_contact.set_phone(phone.to_string());
        }

        if let Some(organization) = contact.organization {
            new_contact.set_organization(organization.to_string());
        }

        new_contact
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

        if let Some(extent) = self.geographical_extent() {
            write!(
                f,
                "\"geographical_extent\": [{}, {}, {}, {}, {}, {}]",
                extent[0], extent[1], extent[2], extent[3], extent[4], extent[5]
            )?;
            first = false;
        }

        if let Some(id) = self.identifier() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "\"identifier\": \"{}\"", id)?;
            first = false;
        }

        if let Some(contact) = self.point_of_contact() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "\"point_of_contact\": {}", contact)?;
            first = false;
        }

        if let Some(date) = self.reference_date() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "\"reference_date\": \"{}\"", date)?;
            first = false;
        }

        if let Some(crs) = self.reference_system() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "\"reference_system\": \"{}\"", crs)?;
            first = false;
        }

        if let Some(title) = self.title() {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "\"title\": \"{}\"", title)?;
            first = false;
        }

        if let Some(extra) = self.extra() {
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
            self.contact_name(),
            self.email_address()
        )?;

        if let Some(role) = self.role() {
            write!(f, ", \"role\": \"{}\"", role)?;
        }

        if let Some(website) = self.website() {
            write!(f, ", \"website\": \"{}\"", website)?;
        }

        if let Some(contact_type) = self.contact_type() {
            write!(f, ", \"contact_type\": \"{}\"", contact_type)?;
        }

        if let Some(address) = self.address() {
            write!(f, ", \"address\": \"{}\"", address)?;
        }

        if let Some(phone) = self.phone() {
            write!(f, ", \"phone\": \"{}\"", phone)?;
        }

        if let Some(organization) = self.organization() {
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
