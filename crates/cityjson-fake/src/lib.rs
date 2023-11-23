//! # cjmock
//!
//! CityJSON generator with fake data.
//!
//! - You can control the number of vertices it the surfaces, for instance to fake triangulated
//! surfaces.
//! - The generated CityJSON is valid according to the specifications. However, the generated
//! vertices and geometries are random, they have no resemblance to real-world and they are invalid.
//! -
//!
//! See the [design doc] for details on how this crate works under the hood.
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
use rand::distributions::{Bernoulli, Distribution};
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
const MIN_MEMBERS_MULTI_COMPOSITESURFACE: usize = 1;
const MAX_MEMBERS_MULTI_COMPOSITESURFACE: usize = 100;
const MIN_MEMBERS_SOLID: usize = 1;
const MAX_MEMBERS_SOLID: usize = 10;
const MIN_MEMBERS_MULTI_COMPOSITESOLID: usize = 1;
const MAX_MEMBERS_MULTI_COMPOSITESOLID: usize = 10;
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

struct CityObjectFaker {
    nr_vertices: usize,
}

struct CityObjectTypeFaker;

struct GeometryFaker {
    nr_vertices: usize,
    cotype: CityObjectType,
}

struct LoDFaker;

struct CompositeSolidFaker {
    nr_vertices: usize,
}

struct MultiSolidFaker {
    nr_vertices: usize,
}

struct SolidFaker {
    nr_vertices: usize,
}

struct CompositeSurfaceFaker {
    nr_vertices: usize,
}

struct MultiSurfaceFaker {
    nr_vertices: usize,
}

struct MultiLineStringFaker {
    nr_vertices: usize,
}

struct MultiPointFaker {
    nr_vertices: usize,
}

struct VertexIndexFaker {
    max: usize,
}

struct MultiPointSemanticsFaker {
    nr_points: usize,
    cotype: CityObjectType,
}

struct SemanticFaker {
    cotype: CityObjectType,
}

struct OptionalIndexFaker {
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
    /// If the vertices haven't been generated yet, they will be created, so that the geometry
    /// boundaries can index them.
    pub fn cityobjects(mut self, nr_cityobjects: Option<Range<usize>>) -> Self {
        let _nr_cos = nr_cityobjects.unwrap_or(1..2);
        if self.vertices.is_none() {
            self.vertices = Some(fake_vertices());
        }
        let nr_vertices = self.vertices.as_ref().unwrap().len();
        let cof = CityObjectFaker::new(nr_vertices);
        let cos: Vec<CityObject> = (cof, _nr_cos).fake();
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

    /// If the vertices are already set so `Some(Vertices)`, then this method does nothing.
    pub fn vertices(mut self) -> Self {
        if self.vertices.is_none() {
            self.vertices = Some(fake_vertices());
        }
        self
    }

    pub fn build_string(self) -> serde_json::Result<String> {
        serde_json::to_string::<CityModel>(&self.into())
    }

    pub fn build_vec(self) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec::<CityModel>(&self.into())
    }
}

impl CityObjectFaker {
    fn new(nr_vertices: usize) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<CityObjectFaker> for CityObject {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &CityObjectFaker, _: &mut R) -> Self {
        let cotype: CityObjectType = CityObjectTypeFaker.fake();
        // TODO: add hierarchy
        // TODO: add "address" to the type where possible
        let gf = GeometryFaker::new(config.nr_vertices, cotype);
        Self::new(
            cotype,
            (gf, 0..=MAX_MEMBERS_CITYOBJECT_GEOMETRIES).fake(),
        )
    }
}

impl Dummy<CityObjectTypeFaker> for CityObjectType {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &CityObjectTypeFaker, rng: &mut R) -> Self {
        match rng.gen_range(0..=31) {
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
            25 => CityObjectType::Tunnel,
            26 => CityObjectType::TunnelPart,
            27 => CityObjectType::TunnelInstallation,
            28 => CityObjectType::TunnelConstructiveElement,
            29 => CityObjectType::TunnelHollowSpace,
            30 => CityObjectType::TunnelFurniture,
            31 => CityObjectType::GenericCityObject,
            _ => unreachable!()
        }
    }
}

impl GeometryFaker {
    fn new(nr_vertices: usize, cotype: CityObjectType) -> Self {
        Self { nr_vertices, cotype }
    }
}

impl Dummy<GeometryFaker> for Geometry {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &GeometryFaker, rng: &mut R) -> Self {
        let lod: LoD = LoDFaker.fake();
        // Choose a Geometry type that is allowed for the given CityObject type
        let mut geometry_types: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let building_types = config.cotype == CityObjectType::Building || config.cotype == CityObjectType::BuildingPart || config.cotype == CityObjectType::BuildingStorey || config.cotype == CityObjectType::BuildingRoom || config.cotype == CityObjectType::BuildingUnit;
        if config.cotype == CityObjectType::Bridge || config.cotype == CityObjectType::BridgePart {
            geometry_types = vec![2, 3, 4, 6];
        } else if building_types {
            geometry_types = vec![2, 3, 4, 6];
        } else if config.cotype == CityObjectType::GenericCityObject {
            geometry_types = vec![0, 1, 2, 3, 4, 6];
        } else if config.cotype == CityObjectType::LandUse {
            geometry_types = vec![2, 3];
        } else if config.cotype == CityObjectType::PlantCover {
            geometry_types = vec![2, 3, 4, 5, 6];
        } else if config.cotype == CityObjectType::TINRelief {
            geometry_types = vec![3];
        } else if config.cotype == CityObjectType::Road || config.cotype == CityObjectType::Railway || config.cotype == CityObjectType::Waterway {
            geometry_types = vec![1, 2, 3];
        } else if config.cotype == CityObjectType::TransportSquare {
            geometry_types = vec![0, 1, 2, 3];
        } else if config.cotype == CityObjectType::Tunnel || config.cotype == CityObjectType::TunnelPart {
            geometry_types = vec![2, 3, 4, 6];
        } else if config.cotype == CityObjectType::WaterBody {
            geometry_types = vec![1, 2, 3, 4, 6];
        }
        let geometry_type_chosen = geometry_types.choose(rng).unwrap_or(&0_usize);
        // Decide if we can generate semantics for the given CityObject type
        let mut generate_semantics = false;
        if lod >= LoD::LoD2 {
            if building_types || config.cotype == CityObjectType::BuildingInstallation {
                generate_semantics = true;
            } else if config.cotype == CityObjectType::WaterBody {
                generate_semantics = true;
            } else if config.cotype == CityObjectType::Road || config.cotype == CityObjectType::Railway || config.cotype == CityObjectType::TransportSquare {
                generate_semantics = true;
            }
        }


        match geometry_type_chosen {
            0 => {
                let boundaries: MultiPointBoundary = MultiPointFaker::new(config.nr_vertices).fake();
                let nr_points = boundaries.len();
                Geometry::MultiPoint {
                    lod,
                    boundaries,
                    semantics: generate_semantics.then(|| MultiPointSemanticsFaker::new(nr_points, config.cotype).fake()),
                }
            }
            1 => Geometry::MultiLineString {
                lod,
                boundaries: MultiLineStringFaker::new(config.nr_vertices).fake(),
                semantics: None,
            },
            2 => Geometry::MultiSurface {
                lod,
                boundaries: MultiSurfaceFaker::new(config.nr_vertices).fake(),
                semantics: None,
            },
            3 => Geometry::CompositeSurface {
                lod,
                boundaries: CompositeSurfaceFaker::new(config.nr_vertices).fake(),
                semantics: None,
            },
            4 => Geometry::Solid {
                lod,
                boundaries: SolidFaker::new(config.nr_vertices).fake(),
                semantics: None,
            },
            5 => Geometry::MultiSolid {
                lod,
                boundaries: MultiSolidFaker::new(config.nr_vertices).fake(),
                semantics: None,
            },
            6 => Geometry::CompositeSolid {
                lod,
                boundaries: CompositeSolidFaker::new(config.nr_vertices).fake(),
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

impl CompositeSolidFaker {
    fn new(nr_vertices: usize) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<CompositeSolidFaker> for AggregateSolidBoundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &CompositeSolidFaker, _: &mut R) -> Self {
        let sof = SolidFaker::new(config.nr_vertices);
        (sof, MIN_MEMBERS_MULTI_COMPOSITESOLID..=MAX_MEMBERS_MULTI_COMPOSITESOLID).fake::<Vec<SolidBoundary>>()
    }
}


impl MultiSolidFaker {
    fn new(nr_vertices: usize) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<MultiSolidFaker> for AggregateSolidBoundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &MultiSolidFaker, _: &mut R) -> Self {
        let sof = SolidFaker::new(config.nr_vertices);
        (sof, MIN_MEMBERS_MULTI_COMPOSITESOLID..=MAX_MEMBERS_MULTI_COMPOSITESOLID).fake::<Vec<SolidBoundary>>()
    }
}

impl SolidFaker {
    fn new(nr_vertices: usize) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<SolidFaker> for SolidBoundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &SolidFaker, _: &mut R) -> Self {
        let csrff = CompositeSurfaceFaker::new(config.nr_vertices);
        (csrff, MIN_MEMBERS_SOLID..=MAX_MEMBERS_SOLID).fake::<Vec<AggregateSurfaceBoundary>>()
    }
}

impl CompositeSurfaceFaker {
    fn new(nr_vertices: usize) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<CompositeSurfaceFaker> for AggregateSurfaceBoundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &CompositeSurfaceFaker, _: &mut R) -> Self {
        let mlsf = MultiLineStringFaker::new(config.nr_vertices);
        (mlsf, MIN_MEMBERS_MULTI_COMPOSITESURFACE..=MAX_MEMBERS_MULTI_COMPOSITESURFACE).fake::<Vec<MultiLineStringBoundary>>()
    }
}

impl MultiSurfaceFaker {
    fn new(nr_vertices: usize) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<MultiSurfaceFaker> for AggregateSurfaceBoundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &MultiSurfaceFaker, _: &mut R) -> Self {
        let mlsf = MultiLineStringFaker::new(config.nr_vertices);
        (mlsf, MIN_MEMBERS_MULTI_COMPOSITESURFACE..=MAX_MEMBERS_MULTI_COMPOSITESURFACE).fake::<Vec<MultiLineStringBoundary>>()
    }
}

impl MultiLineStringFaker {
    fn new(nr_vertices: usize) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<MultiLineStringFaker> for MultiLineStringBoundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &MultiLineStringFaker, _: &mut R) -> Self {
        let mpf = MultiPointFaker::new(config.nr_vertices);
        (mpf, MIN_MEMBERS_MULTILINESTRING..=MAX_MEMBERS_MULTILINESTRING).fake::<Vec<MultiPointBoundary>>()
    }
}

impl MultiPointFaker {
    fn new(nr_vertices: usize) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<MultiPointFaker> for MultiPointBoundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &MultiPointFaker, _: &mut R) -> Self {
        let vf = VertexIndexFaker::new(config.nr_vertices);
        (vf, MIN_MEMBERS_MULTIPOINT..=MAX_MEMBERS_MULTIPOINT).fake::<Vec<usize>>()
    }
}

impl VertexIndexFaker {
    fn new(max_vertices: usize) -> Self {
        Self { max: max_vertices }
    }
}

impl Dummy<VertexIndexFaker> for usize {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &VertexIndexFaker, rng: &mut R) -> Self {
        let vidx: usize = rng.gen_range(0..=config.max);
        vidx
    }
}

fn fake_vertices() -> Vertices {
    Faker.fake::<Vertices>()
}

impl MultiPointSemanticsFaker {
    fn new(nr_points: usize, cotype: CityObjectType) -> Self {
        Self { nr_points, cotype }
    }
}

impl Dummy<MultiPointSemanticsFaker> for MultiPointSemantics {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &MultiPointSemanticsFaker, rng: &mut R) -> Self {
        if config.nr_points == 0 {
            Self::new(Vec::<Semantic>::default(), MultiPointSemanticsValues::default())
        } else {
            let sf = SemanticFaker::new(config.cotype);
            let nr_semantic: usize = (1..config.nr_points).fake_with_rng(rng);
            let mut surfaces: Vec<Semantic> = Vec::with_capacity(nr_semantic);
            for _ in 0..nr_semantic {
                if let Some(_sem) = sf.fake::<Option<Semantic>>() {
                    surfaces.push(_sem);
                }
            }
            let idxf = OptionalIndexFaker::new(config.nr_points);
            let values: MultiPointSemanticsValues = (idxf, config.nr_points..config.nr_points + 1).fake::<Vec<OptionalIndex>>();
            Self::new(surfaces, values)

        }
    }
}

impl SemanticFaker {
    fn new(cotype: CityObjectType) -> Self {
        Self { cotype }
    }
}

impl Dummy<SemanticFaker> for Option<Semantic> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &SemanticFaker, rng: &mut R) -> Self {
        let building_types = config.cotype == CityObjectType::Building || config.cotype == CityObjectType::BuildingPart || config.cotype == CityObjectType::BuildingStorey || config.cotype == CityObjectType::BuildingRoom || config.cotype == CityObjectType::BuildingUnit || config.cotype == CityObjectType::BridgeInstallation;
        let transportation_types = config.cotype == CityObjectType::Road || config.cotype == CityObjectType::Railway || config.cotype == CityObjectType::TransportSquare;
        let mut semantic_types: Vec<usize> = (0..=17).collect();
        if building_types {
            semantic_types = (0..11).collect();
        } else if config.cotype == CityObjectType::WaterBody {
            semantic_types = (11..14).collect();
        } else if transportation_types {
            semantic_types = (14..18).collect();
        } else {
            return None;
        }
        let semantic_type_chosen = semantic_types.choose(rng).unwrap_or(&0_usize);
        let semantic = match semantic_type_chosen {
            0 => Semantic::RoofSurface,
            1 => Semantic::GroundSurface,
            2 => Semantic::WallSurface,
            3 => Semantic::ClosureSurface,
            4 => Semantic::OuterCeilingSurface,
            5 => Semantic::OuterFloorSurface,
            6 => Semantic::Window,
            7 => Semantic::Door,
            8 => Semantic::InteriorWallSurface,
            9 => Semantic::CeilingSurface,
            10 => Semantic::FloorSurface,
            11 => Semantic::WaterSurface,
            12 => Semantic::WaterGroundSurface,
            13 => Semantic::WaterClosureSurface,
            14 => Semantic::TrafficArea,
            15 => Semantic::AuxiliaryTrafficArea,
            16 => Semantic::TransportationMarking,
            17 => Semantic::TransportationHole,
            _ => unreachable!()
        };
        Some(semantic)
    }
}

impl OptionalIndexFaker {
    fn new(max_index: usize) -> Self {
        Self { max: max_index }
    }
}

impl Dummy<OptionalIndexFaker> for Option<usize> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &OptionalIndexFaker, rng: &mut R) -> Self {
        // Probability of having a semantic for the surface, instead of a null
        let prob = 0.8;
        let d = Bernoulli::new(prob).unwrap();
        let has_semantic = d.sample(&mut rand::thread_rng());
        if has_semantic {
            let idx: usize = rng.gen_range(0..=config.max);
            Some(idx)
        } else {
            None
        }
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

        let mpf = MultiPointFaker { nr_vertices: nr_vertices };
        let a: MultiPointBoundary = mpf.fake();
        println!("nr points: {}", a.len());
        println!("{:?}", &a);

        let a: MultiLineStringBoundary = MultiLineStringFaker { nr_vertices: nr_vertices }.fake();
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
