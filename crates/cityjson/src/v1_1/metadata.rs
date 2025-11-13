//! # Metadata
//!
//! This module provides types and functionality for handling CityJSON metadata.
//! It implements the [Metadata object](https://www.cityjson.org/specs/1.1.3/#metadata)
//! as specified in the CityJSON 1.1.3 standard.
//!
//! ## Overview
//!
//! The metadata module contains several key components:
//!
//! - [`Metadata`]: The main struct representing a complete metadata object
//! - [`BBox`]: A bounding box representation (geographical extent)
//! - [`Contact`]: Information about the point of contact for the dataset
//! - [`ContactRole`]: Enumeration of possible roles for a contact
//! - [`ContactType`]: Enumeration specifying the type of contact (individual or organization)
//!
//! Other specialized types include:
//! - [`CityModelIdentifier`]: A unique identifier for the city model
//! - [`Date`]: Representation of a reference date
//! - [`CRS`]: Coordinate Reference System identifier
//!
//! ## Usage Examples
//!
//! ### Creating and populating metadata
//!
//! ```rust
//! use cityjson::prelude::*;
//! use cityjson::v1_1::*;
//!
//! // Create a new metadata object
//! let mut metadata = Metadata::<OwnedStringStorage>::new();
//!
//! // Set geographical extent using BBox
//! let bbox = BBox::new(84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9);
//! metadata.set_geographical_extent(bbox);
//!
//! // Set basic metadata properties
//! metadata.set_identifier(CityModelIdentifier::new("44574905-d2d2-4f40-8e96-d39e1ae45f70".to_string()));
//! metadata.set_reference_date(Date::new("2023-06-15".to_string()));
//! metadata.set_reference_system(CRS::new("https://www.opengis.net/def/crs/EPSG/0/7415".to_string()));
//! metadata.set_title("Amsterdam City Center");
//!
//! // Configure contact information
//! metadata.set_contact_name("Jane Smith");
//! metadata.set_email_address("jane.smith@example.com");
//! metadata.set_role(ContactRole::Author);
//! metadata.set_website("https://example.com/citymodels");
//! metadata.set_contact_type(ContactType::Individual);
//! metadata.set_organization("Urban Modeling Institute");
//! ```
//!
//! ### Working with BBox
//!
//! ```rust
//! use cityjson::prelude::*;
//! use cityjson::v1_1::*;
//!
//! // Create a bounding box
//! let mut bbox = BBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 30.0);
//!
//! // Access dimensions
//! assert_eq!(bbox.width(), 100.0);
//! assert_eq!(bbox.length(), 100.0);
//! assert_eq!(bbox.height(), 30.0);
//!
//! // Convert from/to array
//! let array: [f64; 6] = [0.0, 0.0, 0.0, 100.0, 100.0, 30.0];
//! assert_eq!(BBox::from(array), bbox);
//! ```
//!
//! ## Compliance
//!
//! All types in this module are designed to comply with the
//! [CityJSON 1.1.3 specification](https://www.cityjson.org/specs/1.1.3/).
//! The module implements all required and optional metadata fields as defined in the standard.

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
    point_of_contact: Option<Contact>,
    reference_date: Option<Date<SS>>,
    reference_system: Option<CRS<SS>>,
    title: Option<String>,
    extra: Option<Attributes<SS>>,
}

impl_metadata_methods!();

impl<SS: StringStorage> Metadata<SS> {
    pub fn point_of_contact(&self) -> Option<&Contact> {
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

    // TODO: maybe this should take an Option just as the Contact.set_address does. Check for consistance of the rest of the methods too
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

    pub fn set_point_of_contact(&mut self, contact: Option<Contact>) {
        self.point_of_contact = contact;
    }
}

/// The point of contact for the city model.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#pointofcontact>
///
/// # Examples
/// ```
/// # use cityjson::v1_1::*;
/// let mut contact = Contact::new();
///
/// assert_eq!(contact.contact_name(), "");
///
/// contact.set_contact_name("Kovács János".to_string());
/// contact.set_email_address("janos.kovacs@example.hu".to_string());
/// contact.set_role(Some(ContactRole::Contributor));
/// contact.set_website(Some("https://other.example.com".to_string()));
/// contact.set_contact_type(Some(ContactType::Organization));
/// contact.set_address(Some("456 Other St, Kerek Erdő".to_string()));
/// contact.set_phone(Some("+1-555-4567".to_string()));
/// contact.set_organization(Some("Other Corp".to_string()));
///
/// assert_eq!(contact.contact_name(), "Kovács János");
/// assert_eq!(contact.email_address(), "janos.kovacs@example.hu");
/// assert_eq!(contact.role(), Some(ContactRole::Contributor));
/// assert_eq!(contact.website(), &Some("https://other.example.com".to_string()));
/// assert_eq!(contact.contact_type(), Some(ContactType::Organization));
/// assert_eq!(contact.address(), &Some("456 Other St, Kerek Erdő".to_string()));
/// assert_eq!(contact.phone(), &Some("+1-555-4567".to_string()));
/// assert_eq!(contact.organization(), &Some("Other Corp".to_string()));
/// ```
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

impl Contact {
    impl_contact_common_methods!();

    pub fn address(&self) -> &Option<String> {
        &self.address
    }

    pub fn set_address(&mut self, address: Option<String>) {
        self.address = address;
    }
}

impl Display for Contact {
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

impl Display for ContactRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

/// Metadata contact type.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#pointofcontact>
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
        metadata.set_address("Kiskőrös utca");
        metadata.set_phone("+1-555-1234");
        metadata.set_organization("Test Corp");
        println!("Metadata: {}", metadata);

        let mut contact = Contact::new();
        contact.set_contact_name("Jane Smith".to_string());
        contact.set_email_address("jane@example.com".to_string());
        contact.set_role(Some(ContactRole::Editor));
        contact.set_website(Some("https://example.net".to_string()));
        contact.set_contact_type(Some(ContactType::Organization));
        contact.set_address(Some("Kiskőrös utca".to_string()));
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
