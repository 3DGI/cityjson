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
//! let mut metadata = Metadata::<OwnedStringStorage, ResourceId32>::new();
//!
//! // Set geographical extent using BBox
//! let bbox = BBox::new(84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9);
//! metadata.set_geographical_extent(bbox);
//!
//! // Set basic metadata properties
//! metadata.set_identifier("44574905-d2d2-4f40-8e96-d39e1ae45f70");
//! metadata.set_reference_date("2023-06-15");
//! metadata.set_reference_system("https://www.opengis.net/def/crs/EPSG/0/7415");
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
//! // Update coordinates
//! bbox.set_min_z(-10.0);
//! bbox.set_max_z(50.0);
//! assert_eq!(bbox.height(), 60.0);
//!
//! // Convert from/to array
//! let array: [f64; 6] = [0.0, 0.0, -10.0, 100.0, 100.0, 50.0];
//! assert_eq!(BBox::from(array), bbox);
//! ```
//!
//! ## Compliance
//!
//! All types in this module are designed to comply with the
//! [CityJSON 1.1.3 specification](https://www.cityjson.org/specs/1.1.3/).
//! The module implements all required and optional metadata fields as defined in the standard.

use crate::format_option;
use crate::prelude::ResourceRef;
use crate::resources::storage::StringStorage;
use crate::shared::attributes::Attributes;
use crate::traits::metadata::BBoxTrait;
use std::fmt::{Display, Formatter};

/// Metadata for a city model.
///
/// There is only structural validation for the metadata items, the metadata values are not
/// validated.
/// For instance, a contact website must be a string, but it is not
/// checked whether the string is a valid URL or not.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#metadata>
///
/// # Examples
/// ```
/// # use cityjson::prelude::*;
/// # use cityjson::v1_1::*;
/// let mut metadata = Metadata::<OwnedStringStorage, ResourceId32>::new();
///
/// metadata.set_geographical_extent(BBox::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0));
/// metadata.set_identifier("test-id");
/// metadata.set_reference_date("2024-03-20");
/// metadata.set_reference_system("https://www.opengis.net/def/crs/EPSG/0/7415");
/// metadata.set_title("Test Dataset");
/// metadata.set_contact_name("John Doe");
/// metadata.set_email_address("john@example.com");
/// metadata.set_role(ContactRole::Author);
/// metadata.set_website("https://example.com");
/// metadata.set_contact_type(ContactType::Individual);
/// metadata.set_address("123 Test St");
/// metadata.set_phone("+1-555-1234");
/// metadata.set_organization("Test Corp");
///
/// assert_eq!(metadata.geographical_extent().unwrap().min_x(), 1.0);
/// assert_eq!(metadata.identifier().unwrap(), "test-id");
/// assert_eq!(metadata.reference_date().unwrap(), "2024-03-20");
/// assert_eq!(metadata.reference_system().unwrap(), "https://www.opengis.net/def/crs/EPSG/0/7415");
/// assert_eq!(metadata.title().unwrap(), "Test Dataset");
/// assert_eq!(metadata.point_of_contact().unwrap().contact_name(), "John Doe");
/// assert_eq!(metadata.point_of_contact().unwrap().email_address(), "john@example.com");
/// assert_eq!(metadata.point_of_contact().unwrap().role(), Some(ContactRole::Author));
/// assert_eq!(metadata.point_of_contact().unwrap().website(), &Some("https://example.com".to_string()));
/// assert_eq!(metadata.point_of_contact().unwrap().contact_type(), Some(ContactType::Individual));
/// assert_eq!(metadata.point_of_contact().unwrap().address(), &Some("123 Test St".to_string()));
/// assert_eq!(metadata.point_of_contact().unwrap().phone(), &Some("+1-555-1234".to_string()));
/// assert_eq!(metadata.point_of_contact().unwrap().organization(), &Some("Test Corp".to_string()));
/// ```
#[derive(Clone, Default, Debug, PartialEq)]
pub struct Metadata<SS: StringStorage, RR: ResourceRef> {
    geographical_extent: Option<BBox>,
    identifier: Option<CityModelIdentifier>,
    point_of_contact: Option<Contact>,
    reference_date: Option<Date>,
    reference_system: Option<CRS>,
    title: Option<String>,
    extra: Option<Attributes<SS, RR>>,
}

impl<SS: StringStorage, RR: ResourceRef> Metadata<SS, RR> {
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

    pub fn extra(&self) -> Option<&Attributes<SS, RR>> {
        self.extra.as_ref()
    }

    pub fn extra_mut(&mut self) -> &mut Option<Attributes<SS, RR>> {
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

impl<SS: StringStorage, RR: ResourceRef> Display for Metadata<SS, RR> {
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

    pub fn address(&self) -> &Option<String> {
        &self.address
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

    pub fn set_address(&mut self, address: Option<String>) {
        self.address = address;
    }

    pub fn set_phone(&mut self, phone: Option<String>) {
        self.phone = phone;
    }

    pub fn set_organization(&mut self, organization: Option<String>) {
        self.organization = organization;
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
impl<SS: StringStorage, RR: ResourceRef> crate::traits::metadata::MetadataTrait<SS>
    for Metadata<SS, RR>
{
}

/// Bounding Box.
///
/// A wrapper around an array of 6 values: `[minx, miny, minz, maxx, maxy, maxz]`.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#geographicalextent-bbox>
///
/// # Examples
/// ```
/// # use cityjson::prelude::*;
/// # use cityjson::v1_1::*;
/// let bbox = BBox::new(84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9);
/// let bbox_height = bbox.height();
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BBox {
    values: [f64; 6],
}

impl BBoxTrait for BBox {
    fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            values: [min_x, min_y, min_z, max_x, max_y, max_z],
        }
    }

    fn from_array(values: [f64; 6]) -> Self {
        Self { values }
    }

    fn as_array(&self) -> &[f64; 6] {
        &self.values
    }

    fn as_array_mut(&mut self) -> &mut [f64; 6] {
        &mut self.values
    }

    fn min_x(&self) -> f64 {
        self.values[0]
    }

    fn min_y(&self) -> f64 {
        self.values[1]
    }

    fn min_z(&self) -> f64 {
        self.values[2]
    }

    fn max_x(&self) -> f64 {
        self.values[3]
    }

    fn max_y(&self) -> f64 {
        self.values[4]
    }

    fn max_z(&self) -> f64 {
        self.values[5]
    }

    fn set_min_x(&mut self, value: f64) {
        self.values[0] = value;
    }

    fn set_min_y(&mut self, value: f64) {
        self.values[1] = value;
    }

    fn set_min_z(&mut self, value: f64) {
        self.values[2] = value;
    }

    fn set_max_x(&mut self, value: f64) {
        self.values[3] = value;
    }

    fn set_max_y(&mut self, value: f64) {
        self.values[4] = value;
    }

    fn set_max_z(&mut self, value: f64) {
        self.values[5] = value;
    }

    fn width(&self) -> f64 {
        self.max_x() - self.min_x()
    }

    fn length(&self) -> f64 {
        self.max_y() - self.min_y()
    }

    fn height(&self) -> f64 {
        self.max_z() - self.min_z()
    }
}

impl Default for BBox {
    /// Creates a default BBox with all coordinates set to 0.0.
    fn default() -> Self {
        Self {
            values: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }
}

impl From<[f64; 6]> for BBox {
    /// Creates a BBox from an array of 6 values.
    fn from(values: [f64; 6]) -> Self {
        Self { values }
    }
}

impl From<BBox> for [f64; 6] {
    /// Converts a BBox into an array of 6 values.
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
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#identifier>
///
/// # Examples
/// ```
/// # use cityjson::v1_1::*;
/// let city_id = CityModelIdentifier::from("44574905-d2d2-4f40-8e96-d39e1ae45f70");
/// ```
pub type CityModelIdentifier = String;

/// The date when the dataset was compiled.
///
/// The format is a `"full-date"` per the
/// [RFC 3339, Section 5.6](https://tools.ietf.org/html/rfc3339#section-5.6).
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#referencedate>
///
/// # Examples
/// ```
/// # use cityjson::v1_1::*;
/// let date = Date::from("1977-02-28");
/// ```
pub type Date = String;

/// The coordinate reference system (CRS) of the city model.
///
/// Must be formatted as a URL, according to the
/// [OGC Name Type Specification](https://docs.opengeospatial.org/pol/09-048r5.html#_production_rule_for_specification_element_names).
/// Specs: <https://www.cityjson.org/specs/1.1.3/#referencesystem-crs>
///
/// # Examples
/// ```
/// # use cityjson::v1_1::*;
/// let crs = CRS::from("https://www.opengis.net/def/crs/EPSG/0/7415");
/// ```
pub type CRS = String;

#[cfg(test)]
mod test {
    use super::*;
    use crate::prelude::ResourceId32;
    use crate::resources::storage::OwnedStringStorage;

    #[test]
    fn display() {
        let mut metadata = Metadata::<OwnedStringStorage, ResourceId32>::new();
        metadata.set_geographical_extent(BBox::new(1.1, 2.1, 3.1, 4.1, 5.0, 6.0));
        metadata.set_identifier("test-id");
        metadata.set_reference_date("2024-03-20");
        metadata.set_reference_system("https://www.opengis.net/def/crs/EPSG/0/7415");
        metadata.set_title("Test Dataset");
        metadata.set_contact_name("John Doe");
        metadata.set_email_address("john@example.com");
        metadata.set_role(ContactRole::Author);
        metadata.set_website("https://example.com");
        metadata.set_contact_type(ContactType::Individual);
        metadata.set_address("123 Test St");
        metadata.set_phone("+1-555-1234");
        metadata.set_organization("Test Corp");
        println!("Metadata: {}", metadata);

        let mut contact = Contact::new();
        contact.set_contact_name("Jane Smith".to_string());
        contact.set_email_address("jane@example.com".to_string());
        contact.set_role(Some(ContactRole::Editor));
        contact.set_website(Some("https://example.net".to_string()));
        contact.set_contact_type(Some(ContactType::Organization));
        contact.set_address(Some("456 Sample Ave".to_string()));
        contact.set_phone(Some("+1-555-5678".to_string()));
        contact.set_organization(Some("Sample Inc".to_string()));
        println!("Contact: {}", contact);

        println!("ContactRole: {}", ContactRole::Publisher);
        println!("ContactType: {}", ContactType::Organization);

        let bbox: BBox = BBox::from_array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let id: CityModelIdentifier = "test-model-id".to_string();
        let date: Date = "2024-03-21".to_string();
        let crs: CRS = "https://www.opengis.net/def/crs/EPSG/0/4326".to_string();

        println!("BBox: {}", bbox);
        println!("CityModelIdentifier: {}", id);
        println!("Date: {}", date);
        println!("CRS: {}", crs);
    }
}
