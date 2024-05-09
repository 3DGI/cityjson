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
use std::borrow::Cow;

use fake::faker::address::raw::{BuildingNumber, CityName, CountryName, PostCode, StreetName};
use fake::faker::chrono::raw::Date as FakeDate;
use fake::faker::company::raw::CompanyName;
use fake::faker::internet::raw::{DomainSuffix, SafeEmail};
use fake::faker::lorem::raw::{Word, Words};
use fake::faker::name::raw::Name as FakeName;
use fake::faker::phone_number::raw::PhoneNumber;
use fake::locales::*;
use fake::uuid::UUIDv1;
use fake::{Dummy, Fake, Faker};
use rand::seq::SliceRandom;
use rand::Rng;
use serde_cityjson::v1_1::*;

// TODO: Probably should use https://docs.rs/rand/0.8.5/rand/rngs/struct.SmallRng.html for its speed

const CRS_AUTHORITIES: [&str; 2] = ["EPSG", "OGC"];
const CRS_OGC_VERSIONS: [&str; 3] = ["0", "1.0", "1.3"];
const CRS_OGC_CODES: [&str; 4] = ["CRS1", "CRS27", "CRS83", "CRS84"];
const CRS_EPSG_VERSIONS: [&str; 5] = ["0", "1", "2", "3", "4"];

type IndexType = u32;
// TODO: Maybe I could have this configurable, to that it'll be possible to emulate triangulated
//  surfaces with a range of min=3 max=3.
const MIN_MEMBERS_MULTIPOINT: IndexType = 1;
const MAX_MEMBERS_MULTIPOINT: IndexType = 50;
const MIN_MEMBERS_MULTILINESTRING: IndexType = 1;
const MAX_MEMBERS_MULTILINESTRING: IndexType = 5;
const MIN_MEMBERS_MULTISURFACE: IndexType = 1;
const MAX_MEMBERS_MULTISURFACE: IndexType = 10;
const MIN_MEMBERS_SOLID: IndexType = 1;
const MAX_MEMBERS_SOLID: IndexType = 5;
const MIN_MEMBERS_MULTISOLID: IndexType = 1;
const MAX_MEMBERS_MULTISOLID: IndexType = 5;
const MAX_MEMBERS_CITYOBJECT_GEOMETRIES: IndexType = 10;

struct CityModelBuilder<'cmbuild> {
    id: Option<Cow<'cmbuild, str>>,
    type_cm: Option<CityModelType>,
    version: Option<CityJSONVersion>,
    transform: Option<Transform>,
    cityobjects: Option<CityObjects<'cmbuild>>,
    vertices: Option<Vertices>,
    metadata: Option<Metadata<'cmbuild>>,
    appearance: Option<Appearance<'cmbuild>>,
    geometry_templates: Option<GeometryTemplates<'cmbuild>>,
    extra: Option<Attributes<'cmbuild>>,
    extensions: Option<Extensions>,
}

impl<'cmbuild: 'cm, 'cm> Into<CityModel<'cm>> for CityModelBuilder<'cmbuild> {
    fn into(self) -> CityModel<'cm> {
        CityModel::new(
            self.id,
            self.type_cm,
            Some(self.version.unwrap_or(CityJSONVersion::V1_1)),
            Some(self.transform.unwrap_or_default()),
            self.cityobjects,
            self.vertices,
            self.metadata,
            self.appearance,
            self.geometry_templates,
            self.extra,
            self.extensions,
        )
    }
}

impl<'cmbuild> Default for CityModelBuilder<'cmbuild> {
    fn default() -> Self {
        CityModelBuilder::new().metadata(None)
    }
}

impl<'cmbuild> CityModelBuilder<'cmbuild> {
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
            appearance: None,
            geometry_templates: None,
            extra: None,
            extensions: None,
        }
    }

    pub fn metadata<'mdbuild: 'cmbuild>(
        mut self,
        metadata_builder: Option<MetadataBuilder<'mdbuild>>,
    ) -> Self {
        if let Some(mb) = metadata_builder {
            self.metadata = Some(mb.build());
        } else {
            self.metadata = Some(MetadataBuilder::default().build());
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

struct MetadataBuilder<'mdbuild>(Metadata<'mdbuild>);

struct ContactRoleFaker;

struct ContactTypeFaker;

impl<'mdbuild: 'md, 'md> Into<Metadata<'md>> for MetadataBuilder<'mdbuild> {
    fn into(self) -> Metadata<'md> {
        self.0
    }
}

impl<'mdbuild> Default for MetadataBuilder<'mdbuild> {
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

impl<'mdbuild> MetadataBuilder<'mdbuild> {
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

    fn reference_date(mut self) -> Self {
        self.0.set_reference_date(FakeDate(EN).fake::<String>());
        self
    }

    fn reference_system(mut self) -> Self {
        let ogc_def_crs = "http://www.opengis.net/def/crs";
        let authority = *CRS_AUTHORITIES
            .choose(&mut rand::thread_rng())
            .unwrap_or(&"EPSG");
        let version = match authority {
            "EPSG" => *CRS_EPSG_VERSIONS
                .choose(&mut rand::thread_rng())
                .unwrap_or(&"0"),
            "OGC" => *CRS_OGC_VERSIONS
                .choose(&mut rand::thread_rng())
                .unwrap_or(&"0"),
            _ => "0",
        };
        // TODO: use real EPSG codes, to get existing CRS URIs. Text file contents can be included
        //  with https://doc.rust-lang.org/std/macro.include_str.html
        let code = match authority {
            "EPSG" => {
                let a = rand::thread_rng().gen_range(2000..10500);
                let str = a.to_string();
                str
            }
            "OGC" => CRS_OGC_CODES
                .choose(&mut rand::thread_rng())
                .unwrap_or(&"0")
                .to_string(),
            _ => "0".to_string(),
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

    fn build(self) -> Metadata<'mdbuild> {
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
            _ => unreachable!(),
        }
    }
}

impl Dummy<ContactTypeFaker> for ContactType {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &ContactTypeFaker, rng: &mut R) -> Self {
        match rng.gen_range(0..2) {
            0 => ContactType::Individual,
            1 => ContactType::Organization,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_custom_boundaryfaker() {
    //     let nr_vertices: usize = 12;
    //
    //     let mpf = MultiPointFaker {
    //         nr_vertices: nr_vertices,
    //     };
    //     let a: MultiPointBoundary = mpf.fake();
    //     println!("nr points: {}", a.len());
    //     println!("{:?}", &a);
    //
    //     let a: MultiLineStringBoundary = MultiLineStringFaker {
    //         nr_vertices: nr_vertices,
    //     }
    //     .fake();
    //     println!("nr linestrings: {}", a.len());
    //     println!("{:?}", &a);
    // }

    // #[test]
    // fn it_works() {
    //     let a: LoD = LoDFaker.fake();
    //     println!("{:?}", &a);
    //     println!("{}", serde_json::to_string(&a).unwrap());
    //
    //     let ag: AggregateSolidBoundary = Faker.fake::<AggregateSolidBoundary>();
    //     println!("{:?}", ag);
    //
    //     let v: Vertices = Faker.fake::<Vertices>();
    //     println!("{:?}", v);
    // }

    #[test]
    fn default() {
        let cj_str = CityModelBuilder::default().build_string().unwrap();
        println!("{}", cj_str);
    }
}
