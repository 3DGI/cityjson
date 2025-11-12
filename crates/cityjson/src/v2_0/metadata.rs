use crate::cityjson;
use crate::cityjson::core::attributes::Attributes;
use crate::cityjson::core::metadata::{BBox, CRS, CityModelIdentifier, Date};
use crate::format_option;
use crate::prelude::ResourceRef;
use crate::resources::storage::StringStorage;
use std::fmt::{Display, Formatter};

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Metadata<RR: ResourceRef, SS: StringStorage> {
    geographical_extent: Option<BBox>,
    identifier: Option<CityModelIdentifier<SS>>,
    point_of_contact: Option<Contact<RR, SS>>,
    reference_date: Option<Date<SS>>,
    reference_system: Option<CRS<SS>>,
    title: Option<String>,
    extra: Option<Attributes<SS, RR>>,
}

impl<RR: ResourceRef, SS: StringStorage> Metadata<RR, SS> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn geographical_extent(&self) -> Option<&BBox> {
        self.geographical_extent.as_ref()
    }

    pub fn identifier(&self) -> Option<&CityModelIdentifier<SS>> {
        self.identifier.as_ref()
    }

    pub fn point_of_contact(&self) -> Option<&Contact<RR, SS>> {
        self.point_of_contact.as_ref()
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

    pub fn set_address(&mut self, address: Attributes<SS, RR>) {
        if let Some(poc) = self.point_of_contact.as_mut() {
            poc.address = Some(address);
        } else {
            self.point_of_contact = Some(Contact {
                address: Some(address),
                ..Default::default()
            })
        }
    }

    pub fn address_mut(&mut self) {}

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

    pub fn set_point_of_contact(&mut self, contact: Option<Contact<RR, SS>>) {
        self.point_of_contact = contact;
    }
}

impl<RR: ResourceRef, SS: StringStorage> Display for Metadata<RR, SS> {
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

// TODO: Should use StringStorage for the String values
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Contact<RR: ResourceRef, SS: StringStorage> {
    contact_name: String,
    email_address: String,
    role: Option<ContactRole>,
    website: Option<String>,
    contact_type: Option<ContactType>,
    address: Option<Attributes<SS, RR>>,
    phone: Option<String>,
    organization: Option<String>,
}

impl<RR: ResourceRef, SS: StringStorage> Contact<RR, SS> {
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

    pub fn address(&self) -> Option<&Attributes<SS, RR>> {
        self.address.as_ref()
    }

    pub fn address_mut(&mut self) -> Option<&mut Attributes<SS, RR>> {
        self.address.as_mut()
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

    pub fn set_address(&mut self, address: Option<Attributes<SS, RR>>) {
        self.address = address;
    }

    pub fn set_phone(&mut self, phone: Option<String>) {
        self.phone = phone;
    }

    pub fn set_organization(&mut self, organization: Option<String>) {
        self.organization = organization;
    }
}

impl<RR: ResourceRef, SS: StringStorage> Display for Contact<RR, SS> {
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
impl<RR: ResourceRef, SS: StringStorage> cityjson::traits::metadata::MetadataTrait<SS>
    for Metadata<RR, SS>
{
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::*;
    use crate::resources::storage::OwnedStringStorage;

    #[test]
    fn display() {
        let mut metadata = Metadata::<ResourceId32, OwnedStringStorage>::new();
        metadata.set_geographical_extent(BBox::new(1.1, 2.1, 3.1, 4.1, 5.0, 6.0));
        metadata.set_identifier(CityModelIdentifier::new("test-id".to_string()));
        metadata.set_reference_date(Date::new("2024-03-20".to_string()));
        metadata.set_reference_system(CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/7415".to_string(),
        ));
        metadata.set_title("Test Dataset");
        metadata.set_contact_name("John Doe");
        metadata.set_email_address("john@example.com");
        metadata.set_role(ContactRole::Author);
        metadata.set_website("https://example.com");
        metadata.set_contact_type(ContactType::Individual);
        let mut address = Attributes::new();
        address.insert(
            "street".to_string(),
            AttributeValue::String("Kiskőrös utca".to_string()),
        );
        metadata.set_address(address);
        metadata.set_phone("+1-555-1234");
        metadata.set_organization("Test Corp");
        println!("Metadata: {}", metadata);

        let mut contact = Contact::<ResourceId32, OwnedStringStorage>::new();
        contact.set_contact_name("Jane Smith".to_string());
        contact.set_email_address("jane@example.com".to_string());
        contact.set_role(Some(ContactRole::Editor));
        contact.set_website(Some("https://example.net".to_string()));
        contact.set_contact_type(Some(ContactType::Organization));
        let mut address = Attributes::new();
        address.insert(
            "street".to_string(),
            AttributeValue::String("Kiskőrös utca".to_string()),
        );
        contact.set_address(Some(address));
        contact.set_phone(Some("+1-555-5678".to_string()));
        contact.set_organization(Some("Sample Inc".to_string()));
        println!("Contact: {}", contact);

        println!("ContactRole: {}", ContactRole::Publisher);
        println!("ContactType: {}", ContactType::Organization);

        let bbox: BBox = BBox::from([1.1, 2.1, 3.1, 4.1, 5.0, 6.0]);
        let id: CityModelIdentifier<OwnedStringStorage> =
            CityModelIdentifier::new("test-id".to_string());
        let date: Date<OwnedStringStorage> = Date::new("2024-03-21".to_string());
        let crs: CRS<OwnedStringStorage> =
            CRS::new("https://www.opengis.net/def/crs/EPSG/0/7415".to_string());

        assert_eq!(metadata.geographical_extent(), Some(bbox).as_ref());
        assert_eq!(metadata.identifier(), Some(id.clone()).as_ref());
        assert_eq!(metadata.reference_system(), Some(crs.clone()).as_ref());

        println!("BBox: {}", bbox);
        println!("CityModelIdentifier: {}", id);
        println!("Date: {}", date);
        println!("CRS: {}", crs);
    }
}
