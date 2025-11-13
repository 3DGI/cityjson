use crate::cli::CJFakeConfig;
use crate::{CRS_AUTHORITIES, CRS_EPSG_VERSIONS, CRS_OGC_CODES, CRS_OGC_VERSIONS};
use cityjson::prelude::{BBox, CityModelIdentifier, Date, StringStorage, CRS};
use cityjson::v2_0::{ContactRole, ContactType, Metadata};
use fake::faker::chrono::raw::Date as FakeDate;
use fake::faker::lorem::raw::Words;
use fake::locales::EN;
use fake::uuid::UUIDv1;
use fake::{Dummy, Fake, Faker};
use rand::prelude::{IndexedRandom, SmallRng};
use rand::{rng, Rng};

/// Builder for creating CityJSON metadata with fake data.
///
/// The builder provides methods to configure different aspects of a CityJSON metadata object
/// including geographical extent, identifiers, contact information, references and titles.
/// When fields are not explicitly configured, they will receive random but valid fake data
/// when built.
///
/// # Examples
///
/// ```rust,ignore
/// use cjfake::prelude::MetadataBuilder;
///
/// // Create metadata with all default fake values
/// let metadata = MetadataBuilder::default().build();
///
/// // Create metadata with custom values
/// let metadata = MetadataBuilder::new()
///     .geographical_extent()
///     .identifier()
///     .point_of_contact()
///     .reference_date()
///     .reference_system()
///     .title()
///     .build();
/// ```
pub struct MetadataBuilder<'cmbuild, SS: StringStorage> {
    rng: &'cmbuild mut SmallRng,
    #[allow(dead_code)]
    cjfake: &'cmbuild CJFakeConfig,
    metadata: Metadata<SS>,
}

impl<SS: StringStorage> From<MetadataBuilder<'_, SS>> for Metadata<SS> {
    /// Converts the builder into a Metadata object by returning the inner value.
    fn from(val: MetadataBuilder<SS>) -> Self {
        val.build()
    }
}

// Note: Default implementation removed as it requires a valid lifetime and references

impl<'cmbuild, SS: StringStorage> MetadataBuilder<'cmbuild, SS> {
    /// Creates a new MetadataBuilder with an empty metadata object.
    ///
    /// # Returns
    ///
    /// A new MetadataBuilder instance
    pub fn new(cjfake_config: &'cmbuild CJFakeConfig, rng: &'cmbuild mut SmallRng) -> Self {
        MetadataBuilder {
            rng,
            cjfake: cjfake_config,
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
    pub fn geographical_extent(mut self) -> Self {
        let bbox = BBoxFaker.fake_with_rng(&mut *self.rng);
        self.metadata.set_geographical_extent(bbox);
        self
    }

    /// Sets a random UUID as the identifier.
    ///
    /// Generates a valid UUIDv1 string to uniquely identify the model.
    ///
    /// # Returns
    ///
    /// Self with identifier set
    pub fn identifier(mut self) -> Self
    where
        SS::String: From<String>,
    {
        self.metadata
            .set_identifier(CityModelIdentifier::new(UUIDv1.fake::<String>().into()));
        self
    }

    /// Sets a random reference date.
    ///
    /// Generates and sets a date string in a valid format.
    ///
    /// # Returns
    ///
    /// Self with reference date set
    pub fn reference_date(mut self) -> Self
    where
        SS::String: From<String>,
    {
        self.metadata
            .set_reference_date(Date::new(FakeDate(EN).fake::<String>().into()));
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
    pub fn reference_system(mut self) -> Self
    where
        SS::String: From<String>,
    {
        let ogc_def_crs = "http://www.opengis.net/def/crs";
        let authority = *CRS_AUTHORITIES.choose(&mut rng()).unwrap_or(&"EPSG");
        let version = match authority {
            "EPSG" => *CRS_EPSG_VERSIONS.choose(&mut rng()).unwrap_or(&"0"),
            "OGC" => *CRS_OGC_VERSIONS.choose(&mut rng()).unwrap_or(&"0"),
            _ => "0",
        };
        let code = match authority {
            "EPSG" => {
                let a = rng().random_range(2000..10500);
                a.to_string()
            }
            "OGC" => CRS_OGC_CODES.choose(&mut rng()).unwrap_or(&"0").to_string(),
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
    pub fn title(mut self) -> Self
    where
        SS::String: From<String>,
    {
        let words: Vec<String> = Words(EN, 0..6).fake();
        let title: SS::String = words.join(" ").into();
        self.metadata.set_title(title);
        self
    }

    // Note: point_of_contact method temporarily removed due to API changes
    // The new API uses AttributePool for contact information
    // This needs to be reimplemented according to the new flattened attributes system

    /// Builds the final Metadata object.
    ///
    /// Any fields that were not explicitly set will receive random but valid values.
    ///
    /// # Returns
    ///
    /// The complete Metadata object
    pub fn build(self) -> Metadata<SS> {
        self.metadata
    }
}

struct ContactRoleFaker;

impl Dummy<ContactRoleFaker> for ContactRole {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &ContactRoleFaker, rng: &mut R) -> Self {
        match rng.random_range(0..20) {
            0 => ContactRole::Author,
            1 => ContactRole::CoAuthor,
            2 => ContactRole::Collaborator,
            3 => ContactRole::Contributor,
            4 => ContactRole::Custodian,
            5 => ContactRole::Distributor,
            6 => ContactRole::Editor,
            7 => ContactRole::Funder,
            8 => ContactRole::Mediator,
            9 => ContactRole::Originator,
            10 => ContactRole::Owner,
            11 => ContactRole::PointOfContact,
            12 => ContactRole::PrincipalInvestigator,
            13 => ContactRole::Processor,
            14 => ContactRole::Publisher,
            15 => ContactRole::ResourceProvider,
            16 => ContactRole::RightsHolder,
            17 => ContactRole::Sponsor,
            18 => ContactRole::Stakeholder,
            19 => ContactRole::User,
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
