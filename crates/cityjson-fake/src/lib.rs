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
use std::ops::{Range, RangeInclusive};

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
use serde_cityjson::boundary::Boundary;
use serde_cityjson::indices::{LargeIndex, LargeIndexVec};
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
        CityModelBuilder::new()
            .metadata(None)
            .vertices()
            .cityobjects(None)
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
        let cof = CityObjectFaker::new(nr_vertices as IndexType);
        let cos: Vec<CityObject> = (cof, _nr_cos).fake();
        // TODO: create a CityObjectIDFaker to generate IDs with mixed characters, not only letters
        self.cityobjects =
            Some(CityObjects::from_iter(cos.iter().map(|co| {
                (Cow::from(Word(EN).fake::<&str>()), co.to_owned())
            })));
        self
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

struct CityObjectFaker {
    nr_vertices: IndexType,
}

impl CityObjectFaker {
    fn new(nr_vertices: IndexType) -> Self {
        Self { nr_vertices }
    }
}

impl<'cm> Dummy<CityObjectFaker> for CityObject<'cm> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &CityObjectFaker, _: &mut R) -> Self {
        let cotype: CityObjectType = CityObjectTypeFaker.fake();
        // TODO: add hierarchy
        // TODO: add "address" to the type where possible
        let gf = GeometryFaker::new(config.nr_vertices, cotype.clone());
        Self::new(
            cotype,
            Some((gf, 0..=MAX_MEMBERS_CITYOBJECT_GEOMETRIES as usize).fake()),
            None,
            None,
            None,
            None,
            None,
        )
    }
}

struct CityObjectTypeFaker;

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
            // 31 => CityObjectType::GenericCityObject,
            _ => unreachable!(),
        }
    }
}

struct GeometryFaker {
    nr_vertices: IndexType,
    cotype: CityObjectType,
}

impl GeometryFaker {
    fn new(nr_vertices: IndexType, cotype: CityObjectType) -> Self {
        Self {
            nr_vertices,
            cotype,
        }
    }
}

impl Dummy<GeometryFaker> for Geometry<'_> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &GeometryFaker, rng: &mut R) -> Self {
        let lod: LoD = LoDFaker.fake();
        // todo: move this type setup to compile time
        // Choose a Geometry type that is allowed for the given CityObject type
        let mut geometry_types: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let building_types = config.cotype == CityObjectType::Building
            || config.cotype == CityObjectType::BuildingPart
            || config.cotype == CityObjectType::BuildingStorey
            || config.cotype == CityObjectType::BuildingRoom
            || config.cotype == CityObjectType::BuildingUnit;
        if config.cotype == CityObjectType::Bridge || config.cotype == CityObjectType::BridgePart {
            geometry_types = vec![2, 3, 4, 6];
        } else if building_types {
            geometry_types = vec![2, 3, 4, 6];
        // } else if config.cotype == CityObjectType::GenericCityObject {
        //     geometry_types = vec![0, 1, 2, 3, 4, 6];
        } else if config.cotype == CityObjectType::LandUse {
            geometry_types = vec![2, 3];
        } else if config.cotype == CityObjectType::PlantCover {
            geometry_types = vec![2, 3, 4, 5, 6];
        } else if config.cotype == CityObjectType::TINRelief {
            geometry_types = vec![3];
        } else if config.cotype == CityObjectType::Road
            || config.cotype == CityObjectType::Railway
            || config.cotype == CityObjectType::Waterway
        {
            geometry_types = vec![1, 2, 3];
        } else if config.cotype == CityObjectType::TransportSquare {
            geometry_types = vec![0, 1, 2, 3];
        } else if config.cotype == CityObjectType::Tunnel
            || config.cotype == CityObjectType::TunnelPart
        {
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
            } else if config.cotype == CityObjectType::Road
                || config.cotype == CityObjectType::Railway
                || config.cotype == CityObjectType::TransportSquare
            {
                generate_semantics = true;
            }
        }

        let mut boundaries: Option<Boundary> = None;
        let mut semantics: Option<Semantics> = None;
        let mut material: Option<MaterialMap> = None;
        let mut texture: Option<TextureMap> = None;
        let mut template: Option<u16> = None;
        let mut template_boundaries: Option<[usize; 1]> = None;
        let mut template_transformation_matrix: Option<[f64; 16]> = None;

        match geometry_type_chosen {
            0 => {
                let boundaries: Boundary = MultiPointFaker::new(config.nr_vertices).fake();
                let nr_points = IndexType::try_from(boundaries.vertices.len()).unwrap();
                // semantics = generate_semantics.then(|| {
                //     MultiPointSemanticsFaker::new(nr_points, config.cotype.clone()).fake()
                // });
                Geometry {
                    type_: GeometryType::MultiPoint,
                    lod: Some(lod),
                    boundaries: Some(boundaries),
                    semantics,
                    material: None,
                    texture: None,
                    template: None,
                    template_boundaries: None,
                    template_transformation_matrix: None,
                }
            }
            1 => {
                let boundaries: Boundary = MultiLineStringFaker::new(config.nr_vertices).fake();
                let nr_linestrings = IndexType::try_from(boundaries.rings.len()).unwrap();
                // semantics = generate_semantics.then(|| {
                //     MultiLineStringSemanticsFaker::new(nr_linestrings, config.cotype.clone()).fake()
                // });
                Geometry {
                    type_: GeometryType::MultiLineString,
                    lod: Some(lod),
                    boundaries: Some(boundaries),
                    semantics,
                    material: None,
                    texture: None,
                    template: None,
                    template_boundaries: None,
                    template_transformation_matrix: None,
                }
            }
            2 => {
                let boundaries: Boundary = MultiSurfaceFaker::new(config.nr_vertices).fake();
                let nr_surfaces = IndexType::try_from(boundaries.surfaces.len()).unwrap();
                // semantics = generate_semantics.then(|| {
                //     MultiSurfaceSemanticsFaker::new(nr_surfaces, config.cotype.clone()).fake()
                // });
                Geometry {
                    type_: GeometryType::MultiSurface,
                    lod: Some(lod),
                    boundaries: Some(boundaries),
                    semantics,
                    material: None,
                    texture: None,
                    template: None,
                    template_boundaries: None,
                    template_transformation_matrix: None,
                }
            }
            3 => {
                let boundaries: Boundary = MultiSurfaceFaker::new(config.nr_vertices).fake();
                let nr_surfaces = IndexType::try_from(boundaries.surfaces.len()).unwrap();
                // semantics = generate_semantics.then(|| {
                //     MultiSurfaceSemanticsFaker::new(nr_surfaces, config.cotype.clone()).fake()
                // });
                Geometry {
                    type_: GeometryType::CompositeSurface,
                    lod: Some(lod),
                    boundaries: Some(boundaries),
                    semantics,
                    material: None,
                    texture: None,
                    template: None,
                    template_boundaries: None,
                    template_transformation_matrix: None,
                }
            }
            4 => {
                let boundaries: Boundary = SolidFaker::new(config.nr_vertices).fake();
                // semantics = generate_semantics
                //     .then(|| SolidSemanticsFaker::new(&boundaries, config.cotype.clone()).fake());
                Geometry {
                    type_: GeometryType::Solid,
                    lod: Some(lod),
                    boundaries: Some(boundaries),
                    semantics,
                    material: None,
                    texture: None,
                    template: None,
                    template_boundaries: None,
                    template_transformation_matrix: None,
                }
            }
            5 => {
                let boundaries: Boundary = MultiSolidFaker::new(config.nr_vertices).fake();
                // semantics = generate_semantics.then(|| {
                //     MultiSolidSemanticsFaker::new(&boundaries, config.cotype.clone()).fake()
                // });
                Geometry {
                    type_: GeometryType::MultiSolid,
                    lod: Some(lod),
                    boundaries: Some(boundaries),
                    semantics,
                    material: None,
                    texture: None,
                    template: None,
                    template_boundaries: None,
                    template_transformation_matrix: None,
                }
            }
            6 => {
                let boundaries: Boundary = MultiSolidFaker::new(config.nr_vertices).fake();
                // semantics = generate_semantics.then(|| {
                //     MultiSolidSemanticsFaker::new(&boundaries, config.cotype.clone()).fake()
                // });
                Geometry {
                    type_: GeometryType::CompositeSolid,
                    lod: Some(lod),
                    boundaries: Some(boundaries),
                    semantics,
                    material: None,
                    texture: None,
                    template: None,
                    template_boundaries: None,
                    template_transformation_matrix: None,
                }
            }
            7 => Geometry {
                type_: GeometryType::GeometryInstance,
                lod: None,
                boundaries,
                semantics,
                material,
                texture,
                template,
                template_boundaries,
                template_transformation_matrix,
            },
            _ => unreachable!("There are only seven geometry types"),
        }
    }
}

struct LoDFaker;

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
            _ => unreachable!(),
        }
    }
}

struct MultiSolidFaker {
    nr_vertices: IndexType,
}

impl MultiSolidFaker {
    fn new(nr_vertices: IndexType) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<MultiSolidFaker> for Boundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &MultiSolidFaker, rng: &mut R) -> Self {
        let mut boundary = Boundary {
            vertices: LargeIndexVec::with_capacity(
                (MIN_MEMBERS_MULTIPOINT
                    * MAX_MEMBERS_MULTILINESTRING
                    * MAX_MEMBERS_MULTISURFACE
                    * MAX_MEMBERS_SOLID
                    * MAX_MEMBERS_MULTISOLID) as usize,
            ),
            rings: LargeIndexVec::with_capacity(
                (MAX_MEMBERS_MULTILINESTRING
                    * MAX_MEMBERS_MULTISURFACE
                    * MAX_MEMBERS_SOLID
                    * MAX_MEMBERS_MULTISOLID) as usize,
            ),
            surfaces: LargeIndexVec::with_capacity(
                (MAX_MEMBERS_MULTISURFACE * MAX_MEMBERS_SOLID * MAX_MEMBERS_MULTISOLID) as usize,
            ),
            shells: LargeIndexVec::with_capacity(
                (MAX_MEMBERS_SOLID * MAX_MEMBERS_MULTISOLID) as usize,
            ),
            solids: LargeIndexVec::with_capacity(MAX_MEMBERS_MULTISOLID as usize),
        };

        let min_linestring_len = if MIN_MEMBERS_MULTIPOINT > 1 {
            MIN_MEMBERS_MULTIPOINT
        } else {
            MIN_MEMBERS_MULTIPOINT + 1
        };

        // Counters
        let mut ring_i = 0u32;
        let mut surface_i = 0u32;
        let mut shell_i = 0u32;
        let mut solid_i = 0u32;

        let nr_solids = rng.gen_range(MIN_MEMBERS_MULTISOLID..=MAX_MEMBERS_MULTISOLID);
        for _solid in MIN_MEMBERS_MULTISOLID..=nr_solids {
            boundary.solids.push(LargeIndex::from(solid_i));
            let solid_len = rng.gen_range(MIN_MEMBERS_SOLID..=MAX_MEMBERS_SOLID);
            solid_i += solid_len;

            for _shell in MIN_MEMBERS_SOLID..=solid_len {
                boundary.shells.push(LargeIndex::from(shell_i));
                let shell_len = rng.gen_range(MIN_MEMBERS_MULTISURFACE..=MAX_MEMBERS_MULTISURFACE);
                shell_i += shell_len;

                // Add the surfaces for each shell
                for _surface in MIN_MEMBERS_MULTISURFACE..=shell_len {
                    boundary.surfaces.push(LargeIndex::from(surface_i));
                    let surface_len =
                        rng.gen_range(MIN_MEMBERS_MULTILINESTRING..=MAX_MEMBERS_MULTILINESTRING);
                    surface_i += surface_len;

                    // Add the rings for each surface
                    for _ring in MIN_MEMBERS_MULTILINESTRING..=surface_len {
                        boundary.rings.push(LargeIndex::from(ring_i));
                        let ring_len = rng.gen_range(min_linestring_len..=MAX_MEMBERS_MULTIPOINT);
                        ring_i += ring_len;

                        // Add the vertices for each ring
                        let nr_vertices: IndexType =
                            rng.gen_range(MIN_MEMBERS_MULTIPOINT..=MAX_MEMBERS_MULTIPOINT);
                        boundary.vertices.extend(
                            (0..nr_vertices)
                                .into_iter()
                                .map(|_| IndexFaker::new(config.nr_vertices).fake::<LargeIndex>()),
                        );
                    }
                }
            }
        }

        boundary
    }
}

struct SolidFaker {
    nr_vertices: IndexType,
}

impl SolidFaker {
    fn new(nr_vertices: IndexType) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<SolidFaker> for Boundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &SolidFaker, rng: &mut R) -> Self {
        let mut boundary = Boundary {
            vertices: LargeIndexVec::with_capacity(
                (MIN_MEMBERS_MULTIPOINT
                    * MAX_MEMBERS_MULTILINESTRING
                    * MAX_MEMBERS_MULTISURFACE
                    * MAX_MEMBERS_SOLID) as usize,
            ),
            rings: LargeIndexVec::with_capacity(
                (MAX_MEMBERS_MULTILINESTRING * MAX_MEMBERS_MULTISURFACE * MAX_MEMBERS_SOLID)
                    as usize,
            ),
            surfaces: LargeIndexVec::with_capacity(
                (MAX_MEMBERS_MULTISURFACE * MAX_MEMBERS_SOLID) as usize,
            ),
            shells: LargeIndexVec::with_capacity((MAX_MEMBERS_SOLID) as usize),
            solids: LargeIndexVec::default(),
        };

        let min_linestring_len = if MIN_MEMBERS_MULTIPOINT > 1 {
            MIN_MEMBERS_MULTIPOINT
        } else {
            MIN_MEMBERS_MULTIPOINT + 1
        };

        // Counters
        let mut ring_i = 0u32;
        let mut surface_i = 0u32;
        let mut shell_i = 0u32;

        let nr_shells = rng.gen_range(MIN_MEMBERS_SOLID..=MAX_MEMBERS_SOLID);
        for _shell in MIN_MEMBERS_SOLID..=nr_shells {
            boundary.shells.push(LargeIndex::from(shell_i));
            let shell_len = rng.gen_range(MIN_MEMBERS_MULTISURFACE..=MAX_MEMBERS_MULTISURFACE);
            shell_i += shell_len;

            // Add the surfaces for each shell
            for _surface in MIN_MEMBERS_MULTISURFACE..=shell_len {
                boundary.surfaces.push(LargeIndex::from(surface_i));
                let surface_len =
                    rng.gen_range(MIN_MEMBERS_MULTILINESTRING..=MAX_MEMBERS_MULTILINESTRING);
                surface_i += surface_len;

                // Add the rings for each surface
                for _ring in MIN_MEMBERS_MULTILINESTRING..=surface_len {
                    boundary.rings.push(LargeIndex::from(ring_i));
                    let ring_len = rng.gen_range(min_linestring_len..=MAX_MEMBERS_MULTIPOINT);
                    ring_i += ring_len;

                    // Add the vertices for each ring
                    let nr_vertices: IndexType =
                        rng.gen_range(MIN_MEMBERS_MULTIPOINT..=MAX_MEMBERS_MULTIPOINT);
                    boundary.vertices.extend(
                        (0..nr_vertices)
                            .into_iter()
                            .map(|_| IndexFaker::new(config.nr_vertices).fake::<LargeIndex>()),
                    );
                }
            }
        }

        boundary
    }
}

struct MultiSurfaceFaker {
    nr_vertices: IndexType,
}

impl MultiSurfaceFaker {
    fn new(nr_vertices: IndexType) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<MultiSurfaceFaker> for Boundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &MultiSurfaceFaker, rng: &mut R) -> Self {
        let mut boundary = Boundary {
            // todo scj: ::with_capacity should with the type that largeindex holds, because it doesn't make sense for largeindexvec to hold more items than max largeindex
            vertices: LargeIndexVec::with_capacity(
                (MIN_MEMBERS_MULTIPOINT * MAX_MEMBERS_MULTILINESTRING * MAX_MEMBERS_MULTISURFACE)
                    as usize,
            ),
            rings: LargeIndexVec::with_capacity(
                (MAX_MEMBERS_MULTILINESTRING * MAX_MEMBERS_MULTISURFACE) as usize,
            ),
            surfaces: LargeIndexVec::with_capacity(MAX_MEMBERS_MULTISURFACE as usize),
            shells: LargeIndexVec::default(),
            solids: LargeIndexVec::default(),
        };

        let min_linestring_len = if MIN_MEMBERS_MULTIPOINT > 1 {
            MIN_MEMBERS_MULTIPOINT
        } else {
            MIN_MEMBERS_MULTIPOINT + 1
        };

        // Counters
        let mut ring_i = 0u32;
        let mut surface_i = 0u32;

        let nr_surfaces = rng.gen_range(MIN_MEMBERS_MULTISURFACE..=MAX_MEMBERS_MULTISURFACE);
        for _surface in MIN_MEMBERS_MULTISURFACE..=nr_surfaces {
            boundary.surfaces.push(LargeIndex::from(surface_i));
            let surface_len =
                rng.gen_range(MIN_MEMBERS_MULTILINESTRING..=MAX_MEMBERS_MULTILINESTRING);
            surface_i += surface_len;

            // Add the rings for each surface
            for _ring in MIN_MEMBERS_MULTILINESTRING..=surface_len {
                boundary.rings.push(LargeIndex::from(ring_i));
                let ring_len = rng.gen_range(min_linestring_len..=MAX_MEMBERS_MULTIPOINT);
                ring_i += ring_len;

                // Add the vertices for each ring
                let nr_vertices: IndexType =
                    rng.gen_range(MIN_MEMBERS_MULTIPOINT..=MAX_MEMBERS_MULTIPOINT);
                boundary.vertices.extend(
                    (0..nr_vertices)
                        .into_iter()
                        .map(|_| IndexFaker::new(config.nr_vertices).fake::<LargeIndex>()),
                );
            }
        }
        boundary
    }
}

struct MultiLineStringFaker {
    nr_vertices: IndexType,
}

impl MultiLineStringFaker {
    fn new(nr_vertices: IndexType) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<MultiLineStringFaker> for Boundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &MultiLineStringFaker, rng: &mut R) -> Self {
        let mut boundary = Boundary {
            vertices: LargeIndexVec::with_capacity(
                (MIN_MEMBERS_MULTIPOINT * MAX_MEMBERS_MULTILINESTRING) as usize,
            ),
            rings: LargeIndexVec::with_capacity((MAX_MEMBERS_MULTILINESTRING) as usize),
            surfaces: LargeIndexVec::default(),
            shells: LargeIndexVec::default(),
            solids: LargeIndexVec::default(),
        };

        // A linestring must have at least two vertices, otherwise it's not a line.
        // Here I assume that MIN_MEMBERS_MULTIPOINT is always > 0.
        let min_linestring_len = if MIN_MEMBERS_MULTIPOINT > 1 {
            MIN_MEMBERS_MULTIPOINT
        } else {
            MIN_MEMBERS_MULTIPOINT + 1
        };

        // Counters
        let mut ring_i = 0u32;

        let nr_rings = rng.gen_range(MIN_MEMBERS_MULTILINESTRING..=MAX_MEMBERS_MULTILINESTRING);
        for _ring in MIN_MEMBERS_MULTILINESTRING..=nr_rings {
            boundary.rings.push(LargeIndex::try_from(ring_i).unwrap());
            let ring_len = rng.gen_range(min_linestring_len..=MAX_MEMBERS_MULTIPOINT);
            ring_i += ring_len;

            // Add the vertices for each ring
            let nr_vertices: IndexType =
                rng.gen_range(MIN_MEMBERS_MULTIPOINT..=MAX_MEMBERS_MULTIPOINT);
            boundary.vertices.extend(
                (0..nr_vertices)
                    .into_iter()
                    .map(|_| IndexFaker::new(config.nr_vertices).fake::<LargeIndex>()),
            );
        }
        boundary
    }
}

struct MultiPointFaker {
    nr_vertices: IndexType,
}

impl MultiPointFaker {
    fn new(nr_vertices: IndexType) -> Self {
        Self { nr_vertices }
    }
}

impl Dummy<MultiPointFaker> for Boundary {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &MultiPointFaker, _: &mut R) -> Self {
        let vf = IndexFaker::new(config.nr_vertices);
        Boundary {
            vertices: LargeIndexVecFaker(vf, MIN_MEMBERS_MULTIPOINT..=MAX_MEMBERS_MULTIPOINT)
                .fake(),
            rings: Default::default(),
            surfaces: Default::default(),
            shells: Default::default(),
            solids: Default::default(),
        }
    }
}

struct LargeIndexVecFaker(IndexFaker, RangeInclusive<u32>);

impl Dummy<LargeIndexVecFaker> for LargeIndexVec {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &LargeIndexVecFaker, _: &mut R) -> Self {
        LargeIndexVec::from(
            (
                config.0.clone(),
                *config.1.start() as usize..*config.1.end() as usize,
            )
                .fake::<Vec<u32>>(),
        )
    }
}

#[derive(Clone)]
struct IndexFaker {
    max: IndexType,
}

impl IndexFaker {
    fn new(max_vertices: IndexType) -> Self {
        Self { max: max_vertices }
    }
}

impl Dummy<IndexFaker> for IndexType {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &IndexFaker, rng: &mut R) -> Self {
        let vidx: IndexType = rng.gen_range(0..=config.max);
        vidx
    }
}

impl Dummy<IndexFaker> for LargeIndex {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &IndexFaker, rng: &mut R) -> Self {
        let vidx: IndexType = rng.gen_range(0..=config.max);
        LargeIndex::from(vidx)
    }
}

fn fake_vertices() -> Vertices {
    Faker.fake::<Vertices>()
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

    #[test]
    fn geometry() {
        let geom: Geometry = GeometryFaker::new(12, CityObjectType::Bridge).fake();
        dbg!(geom);
    }

    #[test]
    fn metadata() {
        let m = MetadataBuilder::new()
            .geographical_extent()
            .identifier()
            .point_of_contact()
            .reference_date()
            .reference_system()
            .title()
            .build();
        dbg!(m);
    }

    #[test]
    fn default() {
        let cj_str = CityModelBuilder::default().build_string().unwrap();
        println!("{}", cj_str);
    }
}
