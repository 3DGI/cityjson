use std::ops::Range;
use cjlib::indexed::*;
use fake::{Dummy, Fake, Faker};
use fake::uuid::UUIDv1;
use fake::faker::chrono::raw::Date as FakeDate;
use fake::faker::lorem::raw::{Word, Words};
use fake::faker::name::raw::Name as FakeName;
use fake::faker::internet::raw::{SafeEmail, DomainSuffix};
use fake::faker::address::raw::{CountryName, StreetName, PostCode, CityName, BuildingNumber};
use fake::faker::phone_number::raw::PhoneNumber;
use fake::faker::company::raw::CompanyName;
use fake::locales::*;
use rand::distributions::uniform::SampleRange;
use rand::Rng;
use rand::seq::SliceRandom;

// TODO: Probably should use https://docs.rs/rand/0.8.5/rand/rngs/struct.SmallRng.html for its speed

const CRS_AUTHORITIES: [&str; 2] = ["EPSG", "OGC"];
const CRS_OGC_VERSIONS: [&str; 3] = ["0", "1.0", "1.3"];
const CRS_OGC_CODES: [&str; 4] = ["CRS1", "CRS27", "CRS83", "CRS84"];
const CRS_EPSG_VERSIONS: [&str; 5] = ["0", "1", "2", "3", "4"];
// TODO: Maybe I could have this configurable, to that it'll be possible to emulate triangulated
//  surfaces with a range of min=3 max=3.
const MIN_MEMBERS_MULTIPOINT: usize = 1;
const MAX_MEMBERS_MULTIPOINT: usize = 50;
const MIN_MEMBERS_MULTILINESTRING: usize = 15;
const MAX_MEMBERS_MULTILINESTRING: usize = 15;
const MAX_MEMBERS_CITYOBJECT_GEOMETRIES: usize = 10;

struct CityModelBuilder {
    id: Option<String>,
    type_cm: Option<CityModelType>,
    version: Option<CityJSONVersion>,
    transform: Option<Transform>,
    cityobjects: Option<CityObjects>,
    vertices: Option<Vertices>,
    metadata: Option<Metadata>,
}

struct MetadataBuilder(Metadata);

struct CityObjectFaker;

struct CityObjectTypeFaker;

struct GeometryFaker;

struct LoDFaker;

struct MultiLineStringFaker {
    vertex_index_max: usize,
}

struct MultiPointFaker {
    vertex_index_max: usize,
}

struct VertexIndexFaker {
    max: usize,
}

struct ContactRoleFaker;

struct ContactTypeFaker;

impl Into<CityModel> for CityModelBuilder {
    fn into(self) -> CityModel {
        CityModel::new(self.id, self.type_cm,
                       Some(self.version.unwrap_or(CityJSONVersion::V1_1)),
                       Some(self.transform.unwrap_or_default()),
                       self.cityobjects,
                       self.vertices, self.metadata)
    }
}

impl Default for CityModelBuilder {
    fn default() -> Self {
        CityModelBuilder::new().metadata(None).vertices().cityobjects(None)
    }
}

impl CityModelBuilder {
    #[must_use]
    pub fn new() -> Self {
        CityModelBuilder {
            id: None,
            type_cm: None,
            version: None,
            transform: None,
            cityobjects: None,
            vertices: None,
            metadata: None,
        }
    }

    /// Generate 1 CityObject if `nr_cityobjects` is `None`, else generate the number of CityObjects
    /// within the provided range.
    pub fn cityobjects(mut self, nr_cityobjects: Option<Range<usize>>) -> Self {
        let _nr_cos = nr_cityobjects.unwrap_or(1..2);
        let cos: Vec<CityObject> = (CityObjectFaker, _nr_cos).fake();
        // TODO: create a CityObjectIDFaker to generate IDs with mixed characters, not only letters
        self.cityobjects = Some(CityObjects::from_iter(cos.iter().map(|co| (Word(EN).fake(), co.to_owned()))));
        self
    }

    pub fn metadata(mut self, metadata_builder: Option<MetadataBuilder>) -> Self {
        if let Some(mb) = metadata_builder {
            self.metadata = Some(mb.build());
        } else {
            self.metadata = Some(MetadataBuilder::default().build());
        }
        self
    }

    pub fn vertices(mut self) -> Self {
        self.vertices = Some(Faker.fake::<Vertices>());
        self
    }

    pub fn build_string(self) -> serde_json::Result<String> {
        serde_json::to_string::<CityModel>(&self.into())
    }

    pub fn build_vec(self) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec::<CityModel>(&self.into())
    }
}

impl Dummy<CityObjectFaker> for CityObject {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &CityObjectFaker, _: &mut R) -> Self {
        Self::new(
            CityObjectTypeFaker.fake(),
            (GeometryFaker, 0..=MAX_MEMBERS_CITYOBJECT_GEOMETRIES).fake(),
        )
    }
}

impl Dummy<CityObjectTypeFaker> for CityObjectType {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &CityObjectTypeFaker, rng: &mut R) -> Self {
        match rng.gen_range(0..25) {
            0 => CityObjectType::Bridge,
            1 => CityObjectType::BridgePart,
            2 => CityObjectType::BridgeInstallation,
            3 => CityObjectType::BridgeConstructiveElement,
            4 => CityObjectType::BridgeRoom,
            5 => CityObjectType::BridgeFurniture,
            6 => CityObjectType::Building,
            7 => CityObjectType::BuildingPart,
            8 => CityObjectType::BuildingInstallation,
            9 => CityObjectType::BuildingConstructiveElement,
            10 => CityObjectType::BuildingFurniture,
            11 => CityObjectType::BuildingStorey,
            12 => CityObjectType::BuildingRoom,
            13 => CityObjectType::BuildingUnit,
            14 => CityObjectType::CityFurniture,
            15 => CityObjectType::LandUse,
            16 => CityObjectType::OtherConstruction,
            17 => CityObjectType::PlantCover,
            18 => CityObjectType::SolitaryVegetationObject,
            19 => CityObjectType::TINRelief,
            20 => CityObjectType::WaterBody,
            21 => CityObjectType::Road,
            22 => CityObjectType::Railway,
            23 => CityObjectType::Waterway,
            24 => CityObjectType::TransportSquare,
            _ => unreachable!()
        }
    }
}

impl Dummy<GeometryFaker> for Geometry {
    /// TODO: Could be possible to restrict the type selection to certain types by providing a sub-range
    ///     for `gen_range`. The sub-range could be passed as part of the wrapped CityObjectType.
    ///     That's because certain CityObjectTypes only allow a restricted set of Boundary types.
    /// TODO: Would need to control the range of the Boundary faker so that the indices correspond
    ///     with the number of vertices.


    fn dummy_with_rng<R: Rng + ?Sized>(_: &GeometryFaker, rng: &mut R) -> Self {
        match rng.gen_range(0..7) {
            0 => Geometry::MultiPoint {
                lod: LoDFaker.fake(),
                boundaries: Faker.fake::<MultiPointBoundary>(),
                semantics: None,
            },
            1 => Geometry::MultiLineString {
                lod: LoDFaker.fake(),
                boundaries: Faker.fake::<MultiLineStringBoundary>(),
                semantics: None,
            },
            2 => Geometry::MultiSurface {
                lod: LoDFaker.fake(),
                boundaries: Faker.fake::<AggregateSurfaceBoundary>(),
                semantics: None,
            },
            3 => Geometry::CompositeSurface {
                lod: LoDFaker.fake(),
                boundaries: Faker.fake::<AggregateSurfaceBoundary>(),
                semantics: None,
            },
            4 => Geometry::Solid {
                lod: LoDFaker.fake(),
                boundaries: Faker.fake::<SolidBoundary>(),
                semantics: None,
            },
            5 => Geometry::MultiSolid {
                lod: LoDFaker.fake(),
                boundaries: Faker.fake::<AggregateSolidBoundary>(),
                semantics: None,
            },
            6 => Geometry::CompositeSolid {
                lod: LoDFaker.fake(),
                boundaries: Faker.fake::<AggregateSolidBoundary>(),
                semantics: None,
            },
            _ => unreachable!()
        }
    }
}


impl Dummy<LoDFaker> for LoD {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &LoDFaker, rng: &mut R) -> Self {
        match rng.gen_range(0..20usize) {
            0 => LoD::LoD0,
            1 => LoD::LoD0_0,
            2 => LoD::LoD0_1,
            3 => LoD::LoD0_2,
            4 => LoD::LoD0_3,
            5 => LoD::LoD1,
            6 => LoD::LoD1_0,
            7 => LoD::LoD1_1,
            8 => LoD::LoD1_2,
            9 => LoD::LoD1_3,
            10 => LoD::LoD2,
            11 => LoD::LoD2_0,
            12 => LoD::LoD2_1,
            13 => LoD::LoD2_2,
            14 => LoD::LoD2_3,
            15 => LoD::LoD3,
            16 => LoD::LoD3_0,
            17 => LoD::LoD3_1,
            18 => LoD::LoD3_2,
            19 => LoD::LoD3_3,
            _ => unreachable!()
        }
    }
}

impl MultiLineStringFaker {
    fn new(vertex_index_max: usize) -> Self {
        Self { vertex_index_max }
    }
}

impl Dummy<MultiLineStringFaker> for MultiLineStringBoundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &MultiLineStringFaker, rng: &mut R) -> Self {
        let mpf = MultiPointFaker::new(config.vertex_index_max);
        (mpf, MIN_MEMBERS_MULTILINESTRING..=MAX_MEMBERS_MULTILINESTRING).fake::<Vec<MultiPointBoundary>>()
    }
}

impl MultiPointFaker {
    fn new(vertex_index_max: usize) -> Self {
        Self { vertex_index_max }
    }
}

impl Dummy<MultiPointFaker> for MultiPointBoundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &MultiPointFaker, _: &mut R) -> Self {
        let vf = VertexIndexFaker::new(config.vertex_index_max);
        (vf, MIN_MEMBERS_MULTIPOINT..=MAX_MEMBERS_MULTIPOINT).fake::<Vec<usize>>()
    }
}

impl VertexIndexFaker {
    fn new(vertex_index_max: usize) -> Self {
        Self { max: vertex_index_max }
    }
}

impl Dummy<VertexIndexFaker> for usize {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &VertexIndexFaker, rng: &mut R) -> Self {
        let vidx: usize = rng.gen_range(0..=config.max);
        vidx
    }
}

impl Into<Metadata> for MetadataBuilder {
    fn into(self) -> Metadata {
        self.0
    }
}

impl Default for MetadataBuilder {
    fn default() -> Self {
        MetadataBuilder::new().geographical_extent().identifier().point_of_contact().reference_date().reference_system().title()
    }
}

impl MetadataBuilder {
    fn new() -> Self {
        MetadataBuilder(Metadata::new())
    }

    fn geographical_extent(mut self) -> Self {
        self.0.set_geographical_extent(Faker.fake::<BBox>());
        self
    }

    fn identifier(mut self) -> Self {
        self.0.set_identifier(UUIDv1.fake::<String>());
        self
    }

    fn point_of_contact(mut self) -> Self {
        self.0.set_contact_name(FakeName(EN).fake::<String>());
        self.0.set_email_address(SafeEmail(EN).fake::<String>());
        self.0.set_role(ContactRoleFaker.fake());
        self.0.set_website(format!("https://www.{}.{}", Word(EN).fake::<String>(), DomainSuffix(EN).fake::<String>()));
        self.0.set_contact_type(ContactTypeFaker.fake());
        self.0.set_address(format!("{} {}, {}, {} {}", BuildingNumber(EN).fake::<String>(), StreetName(EN).fake::<String>(), PostCode(EN).fake::<String>(), CityName(EN).fake::<String>(), CountryName(EN).fake::<String>()));
        self.0.set_phone(PhoneNumber(EN).fake::<String>());
        self.0.set_organization(CompanyName(EN).fake::<String>());
        self
    }

    fn reference_date(mut self) -> Self {
        self.0.set_reference_date(FakeDate(EN).fake::<String>());
        self
    }

    fn reference_system(mut self) -> Self {
        let ogc_def_crs = "http://www.opengis.net/def/crs";
        let authority = *CRS_AUTHORITIES.choose(&mut rand::thread_rng()).unwrap_or(&"EPSG");
        let version = match authority {
            "EPSG" => {
                *CRS_EPSG_VERSIONS.choose(&mut rand::thread_rng()).unwrap_or(&"0")
            }
            "OGC" => {
                *CRS_OGC_VERSIONS.choose(&mut rand::thread_rng()).unwrap_or(&"0")
            }
            _ => { "0" }
        };
        // TODO: use real EPSG codes, to get existing CRS URIs. Text file contents can be included
        //  with https://doc.rust-lang.org/std/macro.include_str.html
        let code = match authority {
            "EPSG" => {
                let a = rand::thread_rng().gen_range(2000..10500);
                let str = a.to_string();
                str
            }
            "OGC" => {
                CRS_OGC_CODES.choose(&mut rand::thread_rng()).unwrap_or(&"0").to_string()
            }
            _ => { "0".to_string() }
        };
        let crs = format!("{ogc_def_crs}/{authority}/{version}/{code}");
        self.0.set_reference_system(crs);
        self
    }

    fn title(mut self) -> Self {
        let words: Vec<String> = Words(EN, 0..6).fake();
        self.0.set_title(words.join(" "));
        self
    }

    fn build(self) -> Metadata {
        self.into()
    }
}

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
            _ => unreachable!()
        }
    }
}

impl Dummy<ContactTypeFaker> for ContactType {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &ContactTypeFaker, rng: &mut R) -> Self {
        match rng.gen_range(0..2) {
            0 => ContactType::Individual,
            1 => ContactType::Organization,
            _ => unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_custom_boundaryfaker() {
        let nr_vertices: usize = 12;

        let mpf = MultiPointFaker { vertex_index_max: nr_vertices };
        let a: MultiPointBoundary = mpf.fake();
        println!("nr points: {}", a.len());
        println!("{:?}", &a);

        let a: MultiLineStringBoundary = MultiLineStringFaker { vertex_index_max: nr_vertices }.fake();
        println!("nr linestrings: {}", a.len());
        println!("{:?}", &a);
    }

    #[test]
    fn it_works() {
        let a: LoD = LoDFaker.fake();
        println!("{:?}", &a);
        println!("{}", serde_json::to_string(&a).unwrap());

        let ag: AggregateSolidBoundary = Faker.fake::<AggregateSolidBoundary>();
        println!("{:?}", ag);

        let v: Vertices = Faker.fake::<Vertices>();
        println!("{:?}", v);
    }

    #[test]
    fn test_builder_basic() {
        let cj_str = CityModelBuilder::default().build_string().unwrap();
        println!("{}", cj_str);
    }
}
