use crate::cityjson::core::attributes::Attributes;
use crate::cityjson::core::metadata::{BBox, CityModelIdentifier, Date, CRS};
use crate::format_option;
use crate::resources::storage::StringStorage;
use std::fmt::{Display, Formatter};

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Metadata<SS: StringStorage> {
    geographical_extent: Option<BBox>,
    identifier: Option<CityModelIdentifier<SS>>,
    point_of_contact: Option<Contact<SS>>,
    reference_date: Option<Date<SS>>,
    reference_system: Option<CRS<SS>>,
    title: Option<String>,
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

    pub fn extra_mut(&mut self) -> &mut Option<Attributes<SS>> {
        &mut self.extra
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

    pub fn set_title<S: AsRef<str>>(&mut self, title: S) {
        self.title = Some(title.as_ref().to_owned());
    }

    pub fn set_phone<S: AsRef<str>>(&mut self, phone: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.phone = Some(phone.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                phone: Some(phone.as_ref().to_owned()),
                ..Default::default()
            });
        }
    }

    pub fn set_organization<S: AsRef<str>>(&mut self, organization: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.organization = Some(organization.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                organization: Some(organization.as_ref().to_owned()),
                ..Default::default()
            });
        }
    }

    pub fn point_of_contact(&self) -> Option<&Contact<SS>> {
        self.point_of_contact.as_ref()
    }

    pub fn set_contact_name<S: AsRef<str>>(&mut self, name: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.contact_name = name.as_ref().to_owned();
        } else {
            self.point_of_contact = Some(Contact {
                contact_name: name.as_ref().to_owned(),
                ..Default::default()
            });
        }
    }

    pub fn set_email_address<S: AsRef<str>>(&mut self, email: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.email_address = email.as_ref().to_owned();
        } else {
            self.point_of_contact = Some(Contact {
                email_address: email.as_ref().to_owned(),
                ..Default::default()
            });
        }
    }

    pub fn set_role(&mut self, role: ContactRole) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.role = Some(role);
        } else {
            self.point_of_contact = Some(Contact {
                role: Some(role),
                ..Default::default()
            });
        }
    }

    pub fn set_website<S: AsRef<str>>(&mut self, website: S) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.website = Some(website.as_ref().to_owned());
        } else {
            self.point_of_contact = Some(Contact {
                website: Some(website.as_ref().to_owned()),
                ..Default::default()
            });
        }
    }

    pub fn set_contact_type(&mut self, contact_type: ContactType) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.contact_type = Some(contact_type);
        } else {
            self.point_of_contact = Some(Contact {
                contact_type: Some(contact_type),
                ..Default::default()
            });
        }
    }

    pub fn set_address(&mut self, address: Attributes<SS>) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.address = Some(address);
        } else {
            self.point_of_contact = Some(Contact {
                address: Some(address),
                ..Default::default()
            });
        }
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
            format_option(&self.geographical_extent),
            format_option(&self.identifier),
            format_option(&self.point_of_contact),
            format_option(&self.reference_date),
            format_option(&self.reference_system),
            format_option(&self.title)
        )
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Contact<SS: StringStorage> {
    contact_name: String,
    email_address: String,
    role: Option<ContactRole>,
    website: Option<String>,
    contact_type: Option<ContactType>,
    address: Option<Attributes<SS>>,
    phone: Option<String>,
    organization: Option<String>,
}

impl<SS: StringStorage> Contact<SS> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn contact_name(&self) -> &str {
        &self.contact_name
    }

    #[must_use]
    pub fn email_address(&self) -> &str {
        &self.email_address
    }

    #[must_use]
    pub fn role(&self) -> Option<ContactRole> {
        self.role
    }

    #[must_use]
    pub fn website(&self) -> &Option<String> {
        &self.website
    }

    #[must_use]
    pub fn contact_type(&self) -> Option<ContactType> {
        self.contact_type
    }

    #[must_use]
    pub fn phone(&self) -> &Option<String> {
        &self.phone
    }

    #[must_use]
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

    #[must_use]
    pub fn address(&self) -> Option<&Attributes<SS>> {
        self.address.as_ref()
    }

    pub fn address_mut(&mut self) -> Option<&mut Attributes<SS>> {
        self.address.as_mut()
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
