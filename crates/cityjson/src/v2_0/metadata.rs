use crate::cityjson::core::attributes::Attributes;
use crate::cityjson::core::metadata::{BBox, CRS, CityModelIdentifier, Date};
use crate::format_option;
use crate::macros::{impl_contact_common_methods, impl_metadata_methods};
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

impl_metadata_methods!();

impl<SS: StringStorage> Metadata<SS> {
    pub fn point_of_contact(&self) -> Option<&Contact<SS>> {
        self.point_of_contact.as_ref()
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

    pub fn set_address(&mut self, address: Attributes<SS>) {
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

    pub fn set_point_of_contact(&mut self, contact: Option<Contact<SS>>) {
        self.point_of_contact = contact;
    }
}

// TODO: Should use StringStorage for the String values
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
    impl_contact_common_methods!();

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

#[cfg(test)]
mod test {
    use super::*;
    use crate::cityjson::core::attributes::{AttributeOwnerType, OwnedAttributePool};
    use crate::resources::storage::OwnedStringStorage;

    #[test]
    fn display() {
        let mut metadata = Metadata::<OwnedStringStorage>::new();
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

        // Create attribute pool and attributes container for address
        let mut pool = OwnedAttributePool::new();
        let street_id = pool.add_string(
            "street".to_string(),
            true,
            "Kiskőrös utca".to_string(),
            AttributeOwnerType::Element,
            None,
        );
        let mut address = Attributes::new();
        address.insert("street".to_string(), street_id);

        metadata.set_address(address);
        metadata.set_phone("+1-555-1234");
        metadata.set_organization("Test Corp");
        println!("Metadata: {}", metadata);

        let mut contact = Contact::<OwnedStringStorage>::new();
        contact.set_contact_name("Jane Smith".to_string());
        contact.set_email_address("jane@example.com".to_string());
        contact.set_role(Some(ContactRole::Editor));
        contact.set_website(Some("https://example.net".to_string()));
        contact.set_contact_type(Some(ContactType::Organization));

        // Create attributes for contact address
        let street_id2 = pool.add_string(
            "street".to_string(),
            true,
            "Kiskőrös utca".to_string(),
            AttributeOwnerType::Element,
            None,
        );
        let mut address2 = Attributes::new();
        address2.insert("street".to_string(), street_id2);

        contact.set_address(Some(address2));
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
