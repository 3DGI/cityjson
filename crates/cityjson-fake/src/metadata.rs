/// Builder for creating CityJSON metadata with fake data.
///
/// The builder provides methods to configure different aspects of a CityJSON metadata object
/// including geographical extent, identifiers, contact information, references and titles.
/// When fields are not explicitly configured, they will receive random but valid fake data
/// when built.
///
/// # Examples
///
/// ```rust
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
use cityjson::v2_0::{Metadata, ContactRole, ContactType};
#[derive(Clone)]
pub struct MetadataBuilder(Metadata<SS, RR>);

impl<'cm> From<MetadataBuilder<'cm>> for Metadata<'cm> {
    /// Converts the builder into a Metadata object by returning the inner value.
    fn from(val: MetadataBuilder<'cm>) -> Self {
        val.0
    }
}

impl<'cm> Default for MetadataBuilder<'cm> {
    /// Creates a MetadataBuilder with all fields configured to generate random values.
    ///
    /// Equivalent to:
    /// ```
    /// # use cjfake::MetadataBuilder;
    /// MetadataBuilder::new()
    ///     .geographical_extent()
    ///     .identifier()
    ///     .point_of_contact()
    ///     .reference_date()
    ///     .reference_system()
    ///     .title();
    /// ```
    fn default() -> Self {
        MetadataBuilder::new()
            .geographical_extent()
            .identifier()
            .point_of_contact()
            .reference_date()
            .reference_system()
            .title()
    }
}

impl<'cm> MetadataBuilder<'cm> {
    /// Creates a new MetadataBuilder with an empty metadata object.
    ///
    /// # Returns
    ///
    /// A new MetadataBuilder instance
    pub fn new() -> Self {
        MetadataBuilder(Metadata::new())
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
        self.0.set_geographical_extent(Faker.fake::<BBox>());
        self
    }

    /// Sets a random UUID as the identifier.
    ///
    /// Generates a valid UUIDv1 string to uniquely identify the model.
    ///
    /// # Returns
    ///
    /// Self with identifier set
    pub fn identifier(mut self) -> Self {
        self.0.set_identifier(UUIDv1.fake::<String>());
        self
    }

    /// Sets contact information with randomly generated but realistic data.
    ///
    /// Generates and sets:
    /// - Contact name (random person name)
    /// - Email address (random but valid format)
    /// - Role (random valid CityJSON contact role)
    /// - Website (random but valid URL)
    /// - Contact type (Individual or Organization)
    /// - Physical address (random but realistic address)
    /// - Phone number (random but valid format)
    /// - Organization name (random company name)
    ///
    /// # Returns
    ///
    /// Self with contact information set
    pub fn point_of_contact(mut self) -> Self {
        self.0.set_contact_name(FakeName(EN).fake::<String>());
        self.0.set_email_address(SafeEmail(EN).fake::<String>());
        self.0.set_role(ContactRoleFaker.fake());
        self.0.set_website(format!(
            "https://www.{}.{}",
            Word(EN).fake::<String>(),
            DomainSuffix(EN).fake::<String>()
        ));
        self.0.set_contact_type(ContactTypeFaker.fake());
        self.0.set_address(format!(
            "{} {}, {}, {} {}",
            BuildingNumber(EN).fake::<String>(),
            StreetName(EN).fake::<String>(),
            PostCode(EN).fake::<String>(),
            CityName(EN).fake::<String>(),
            CountryName(EN).fake::<String>()
        ));
        self.0.set_phone(PhoneNumber(EN).fake::<String>());
        self.0.set_organization(CompanyName(EN).fake::<String>());
        self
    }

    /// Sets a random reference date.
    ///
    /// Generates and sets a date string in a valid format.
    ///
    /// # Returns
    ///
    /// Self with reference date set
    pub fn reference_date(mut self) -> Self {
        self.0.set_reference_date(FakeDate(EN).fake::<String>());
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
    pub fn reference_system(mut self) -> Self {
        let ogc_def_crs = "http://www.opengis.net/def/crs";
        let authority = *CRS_AUTHORITIES.choose(&mut thread_rng()).unwrap_or(&"EPSG");
        let version = match authority {
            "EPSG" => *CRS_EPSG_VERSIONS.choose(&mut thread_rng()).unwrap_or(&"0"),
            "OGC" => *CRS_OGC_VERSIONS.choose(&mut thread_rng()).unwrap_or(&"0"),
            _ => "0",
        };
        let code = match authority {
            "EPSG" => {
                let a = thread_rng().gen_range(2000..10500);
                a.to_string()
            }
            "OGC" => CRS_OGC_CODES
                .choose(&mut thread_rng())
                .unwrap_or(&"0")
                .to_string(),
            _ => "0".to_string(),
        };
        let crs = format!("{ogc_def_crs}/{authority}/{version}/{code}");
        self.0.set_reference_system(crs);
        self
    }

    /// Sets a random title using 1-5 words.
    ///
    /// # Returns
    ///
    /// Self with title set
    pub fn title(mut self) -> Self {
        let words: Vec<String> = Words(EN, 0..6).fake();
        self.0.set_title(words.join(" "));
        self
    }

    /// Builds the final Metadata object.
    ///
    /// Any fields that were not explicitly set will receive random but valid values.
    ///
    /// # Returns
    ///
    /// The complete Metadata object
    pub fn build(self) -> Metadata<'cm> {
        self.into()
    }
}

struct ContactRoleFaker;

impl Dummy<ContactRoleFaker> for ContactRole {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &ContactRoleFaker, rng: &mut R) -> Self {
        match rng.gen_range(0..20) {
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
        match rng.gen_range(0..2) {
            0 => ContactType::Individual,
            1 => ContactType::Organization,
            _ => unreachable!(),
        }
    }
}
