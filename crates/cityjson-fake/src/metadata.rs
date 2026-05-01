//! Metadata generation helpers.
//!
//! ```rust
//! use cityjson_fake::metadata::MetadataBuilder;
//! use cityjson_fake::prelude::*;
//! use rand::SeedableRng;
//! use cityjson_types::prelude::OwnedStringStorage;
//!
//! let config = CJFakeConfig::default();
//! let mut rng = rand::prelude::SmallRng::seed_from_u64(3);
//! let metadata: cityjson_types::v2_0::Metadata<OwnedStringStorage> =
//!     MetadataBuilder::new(&config, &mut rng).build();
//! let _ = metadata;
//! ```

use crate::cli::CJFakeConfig;
use crate::{CRS_AUTHORITIES, CRS_EPSG_VERSIONS, CRS_OGC_CODES, CRS_OGC_VERSIONS};
use cityjson_types::prelude::StringStorage;
use cityjson_types::v2_0::{
    BBox, CRS, CityModelIdentifier, Contact, ContactRole, ContactType, Date, Metadata,
};
use fake::RngExt;
use fake::faker::chrono::raw::Date as FakeDate;
use fake::faker::lorem::raw::Words;
use fake::locales::EN;
use fake::uuid::UUIDv1;
use fake::{Dummy, Fake, Faker};
use rand::Rng;
use rand::prelude::{IndexedRandom, SmallRng};

/// Builder for creating `CityJSON` metadata with fake data.
///
/// The builder provides methods to configure different aspects of a `CityJSON` metadata object
/// including geographical extent, identifiers, contact information, references and titles.
/// When fields are not explicitly configured, they will receive random but valid fake data
/// when built.
///
/// # Examples
///
/// ```rust
/// use cityjson_fake::prelude::*;
/// use rand::SeedableRng;
///
/// let config = CJFakeConfig::default();
/// let mut rng = rand::prelude::SmallRng::seed_from_u64(7);
/// let metadata: Metadata<OwnedStringStorage> = MetadataBuilder::new(&config, &mut rng)
///     .geographical_extent()
///     .identifier()
///     .point_of_contact()
///     .reference_date()
///     .reference_system()
///     .title()
///     .build();
///
/// assert!(metadata.identifier().is_some());
/// assert!(metadata.point_of_contact().is_some());
/// ```
pub struct MetadataBuilder<'cmbuild, SS: StringStorage> {
    rng: &'cmbuild mut SmallRng,
    #[allow(dead_code)]
    config: &'cmbuild CJFakeConfig,
    metadata: Metadata<SS>,
}

impl<SS: StringStorage> From<MetadataBuilder<'_, SS>> for Metadata<SS> {
    /// Converts the builder into a `Metadata` object by returning the inner value.
    fn from(val: MetadataBuilder<SS>) -> Self {
        val.build()
    }
}

// Note: Default implementation removed as it requires a valid lifetime and references

impl<'cmbuild, SS: StringStorage> MetadataBuilder<'cmbuild, SS> {
    /// Creates a new `MetadataBuilder` with an empty metadata object.
    ///
    /// # Returns
    ///
    /// A new `MetadataBuilder` instance
    #[must_use]
    pub fn new(config: &'cmbuild CJFakeConfig, rng: &'cmbuild mut SmallRng) -> Self {
        MetadataBuilder {
            rng,
            config,
            metadata: Metadata::new(),
        }
    }

    /// Sets the geographical extent with randomly generated coordinates.
    ///
    /// Generates a valid bounding box with random coordinates for the model extent.
    /// The coordinates represent [minx, miny, minz, maxx, maxy, maxz].
    ///
    /// # Returns
    ///
    /// Self with geographical extent set
    #[must_use]
    pub fn geographical_extent(mut self) -> Self {
        let bbox = BBoxFaker.fake_with_rng(&mut *self.rng);
        self.metadata.set_geographical_extent(bbox);
        self
    }

    /// Sets a random UUID as the identifier.
    ///
    /// Generates a valid `UUIDv1` string to uniquely identify the model.
    ///
    /// # Returns
    ///
    /// Self with identifier set
    #[must_use]
    pub fn identifier(mut self) -> Self
    where
        SS::String: From<String>,
    {
        self.metadata.set_identifier(CityModelIdentifier::new(
            UUIDv1.fake_with_rng::<String, _>(&mut *self.rng).into(),
        ));
        self
    }

    /// Sets a random reference date.
    ///
    /// Generates and sets a date string in a valid format.
    ///
    /// # Returns
    ///
    /// Self with reference date set
    #[must_use]
    pub fn reference_date(mut self) -> Self
    where
        SS::String: From<String>,
    {
        self.metadata.set_reference_date(Date::new(
            FakeDate(EN)
                .fake_with_rng::<String, _>(&mut *self.rng)
                .into(),
        ));
        self
    }

    /// Sets a random but valid coordinate reference system URI.
    ///
    /// Generates a CRS URI using either EPSG or OGC authority with valid:
    /// - Authority (EPSG or OGC)
    /// - Version
    /// - Code (valid EPSG code range or OGC CRS identifier)
    ///
    /// # Returns
    ///
    /// Self with reference system set
    #[must_use]
    pub fn reference_system(mut self) -> Self
    where
        SS::String: From<String>,
    {
        let ogc_def_crs = "http://www.opengis.net/def/crs";
        let authority = *CRS_AUTHORITIES.choose(&mut *self.rng).unwrap_or(&"EPSG");
        let version = match authority {
            "EPSG" => *CRS_EPSG_VERSIONS.choose(&mut *self.rng).unwrap_or(&"0"),
            "OGC" => *CRS_OGC_VERSIONS.choose(&mut *self.rng).unwrap_or(&"0"),
            _ => "0",
        };
        let code = match authority {
            "EPSG" => {
                let a = self.rng.random_range(2000..10500);
                a.to_string()
            }
            "OGC" => CRS_OGC_CODES
                .choose(&mut *self.rng)
                .unwrap_or(&"0")
                .to_string(),
            _ => "0".to_string(),
        };
        let crs = format!("{ogc_def_crs}/{authority}/{version}/{code}");
        self.metadata.set_reference_system(CRS::new(crs.into()));
        self
    }

    /// Sets a random title using 1-5 words.
    ///
    /// # Returns
    ///
    /// Self with title set
    #[must_use]
    pub fn title(mut self) -> Self
    where
        SS::String: From<String>,
    {
        let words: Vec<String> = Words(EN, 0..6).fake_with_rng(&mut *self.rng);
        let title: SS::String = words.join(" ").into();
        self.metadata.set_title(title);
        self
    }

    /// Sets a random point of contact.
    ///
    /// Generates a `Contact` with a random name, email, role, and optional
    /// phone and organisation fields.
    ///
    /// # Returns
    ///
    /// Self with point of contact set
    #[must_use]
    pub fn point_of_contact(mut self) -> Self
    where
        SS::String: From<String>,
    {
        use fake::faker::internet::raw::SafeEmail;
        use fake::faker::name::raw::Name;

        let mut contact = Contact::new();

        let name: String = Name(EN).fake_with_rng(&mut *self.rng);
        contact.set_contact_name(name.into());

        let email: String = SafeEmail(EN).fake_with_rng(&mut *self.rng);
        contact.set_email_address(email.into());

        let role: ContactRole = ContactRoleFaker.fake_with_rng(&mut *self.rng);
        contact.set_role(Some(role));

        let contact_type: ContactType = ContactTypeFaker.fake_with_rng(&mut *self.rng);
        contact.set_contact_type(Some(contact_type));

        if self.rng.random_bool(0.5) {
            let org: String =
                fake::faker::company::raw::CompanyName(EN).fake_with_rng(&mut *self.rng);
            contact.set_organization(Some(org.into()));
        }

        self.metadata.set_point_of_contact(Some(contact));
        self
    }

    /// Builds the final `Metadata` object.
    ///
    /// Any fields that were not explicitly set will receive random but valid values.
    ///
    /// # Returns
    ///
    /// The complete `Metadata` object
    #[must_use]
    pub fn build(self) -> Metadata<SS> {
        self.metadata
    }
}

struct ContactRoleFaker;

impl Dummy<ContactRoleFaker> for ContactRole {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &ContactRoleFaker, rng: &mut R) -> Self {
        // Extended variant list with all available ContactRole types
        match rng.random_range(0..12) {
            0 => ContactRole::Author,
            1 => ContactRole::CoAuthor,
            2 => ContactRole::Custodian,
            3 => ContactRole::Distributor,
            4 => ContactRole::Originator,
            5 => ContactRole::Owner,
            6 => ContactRole::PointOfContact,
            7 => ContactRole::PrincipalInvestigator,
            8 => ContactRole::Processor,
            9 => ContactRole::Publisher,
            10 => ContactRole::ResourceProvider,
            11 => ContactRole::User,
            _ => unreachable!(),
        }
    }
}

struct ContactTypeFaker;

impl Dummy<ContactTypeFaker> for ContactType {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &ContactTypeFaker, rng: &mut R) -> Self {
        match rng.random_range(0..2) {
            0 => ContactType::Individual,
            1 => ContactType::Organization,
            _ => unreachable!(),
        }
    }
}

struct BBoxFaker;

impl Dummy<BBoxFaker> for BBox {
    fn dummy_with_rng<R: Rng + ?Sized>(_config: &BBoxFaker, rng: &mut R) -> Self {
        let values: [f64; 6] = Faker.fake_with_rng(rng);
        BBox::from(values)
    }
}
