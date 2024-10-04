//! # cjfake
//!
//! CityJSON generator with fake data.
//!
//! - You can control the number of vertices it the surfaces, for instance to fake triangulated
//!   surfaces.
//! - The generated CityJSON is valid according to the specifications. However, the generated
//!   vertices and geometries are random, they have no resemblance to real-world and they are invalid.
//! -
//!
//! See the [design doc] for details on how this crate works under the hood.
use std::borrow::Cow;
use std::ops::{Range, RangeInclusive};
use std::path::PathBuf;

use fake::faker::address::raw::{BuildingNumber, CityName, CountryName, PostCode, StreetName};
use fake::faker::chrono::raw::Date as FakeDate;
use fake::faker::company::raw::CompanyName;
use fake::faker::filesystem::raw::*;
use fake::faker::internet::raw::{DomainSuffix, SafeEmail};
use fake::faker::lorem::raw::{Word, Words};
use fake::faker::name::raw::Name as FakeName;
use fake::faker::phone_number::raw::PhoneNumber;
use fake::locales::*;
use fake::uuid::UUIDv1;
use fake::{Dummy, Fake, Faker};
use rand::distributions::{Bernoulli, Distribution};
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use serde_cityjson::attributes::Attributes;
use serde_cityjson::boundary::Boundary;
use serde_cityjson::indices::{LargeIndex, LargeIndexVec, OptionalLargeIndex};
use serde_cityjson::labels::{LabelIndex, TextureIndex};
use serde_cityjson::v1_1::*;

// TODO: Probably should use https://docs.rs/rand/0.8.5/rand/rngs/struct.SmallRng.html for its speed
// TODO: use Coordinate instead of array (also implement in serde_cityjson)
// todo: vertices unused
// TODO: exact configuration for reproducible models (same types, config etc)
// TODO: CLI/API
// TODO: exe/docker/server
// TODO: docs
// todo cj: templates are added under the 'kebab-case' root member
// TODO: create a CityObjectIDFaker to generate IDs with mixed characters, not only letters
// TODO: Maybe I could have MIN_COORDINATE/MAX_COORDINATE configurable, to that it'll be possible to emulate triangulated surfaces with a range of min=3 max=3.
// TODO: CityObject add "address" to the type where possible
// todo: CityObject add geographical_extent
// todo: CityObject add extra
// TODO: use real EPSG codes, to get existing CRS URIs. Text file contents can be included with https://doc.rust-lang.org/std/macro.include_str.html
// todo cj: need to use the proper coordinate type and add to CoordinateFaker
// todo: CityObjectTypeFaker add GenericCityObject for v2.0
// todo: CityObjectTypeFaker add CityObjectGroup
// todo scj: LargeIndexVec::with_capacity should be initialized with the type that LargeIndex holds, because it doesn't make sense for LargeIndexVec to hold more items than max LargeIndex
// todo: MultiPoint, lod 3, Building --> semantics don't make sense

const CRS_AUTHORITIES: [&str; 2] = ["EPSG", "OGC"];
const CRS_OGC_VERSIONS: [&str; 3] = ["0", "1.0", "1.3"];
const CRS_OGC_CODES: [&str; 4] = ["CRS1", "CRS27", "CRS83", "CRS84"];
const CRS_EPSG_VERSIONS: [&str; 5] = ["0", "1", "2", "3", "4"];

type IndexType = u32;
const MIN_COORDINATE: i64 = i64::MIN;
const MAX_COORDINATE: i64 = i64::MAX;
const MIN_NR_VERTICES: IndexType = 1;
const MAX_NR_VERTICES: IndexType = 30; // IndexType::MAX;
const MIN_MEMBERS_MULTIPOINT: IndexType = 1;
const MAX_MEMBERS_MULTIPOINT: IndexType = 10;
const MIN_MEMBERS_MULTILINESTRING: IndexType = 1;
const MAX_MEMBERS_MULTILINESTRING: IndexType = 5;
const MIN_MEMBERS_MULTISURFACE: IndexType = 1;
const MAX_MEMBERS_MULTISURFACE: IndexType = 5;
const MIN_MEMBERS_SOLID: IndexType = 1;
const MAX_MEMBERS_SOLID: IndexType = 3;
const MIN_MEMBERS_MULTISOLID: IndexType = 1;
const MAX_MEMBERS_MULTISOLID: IndexType = 3;
const MAX_MEMBERS_CITYOBJECT_GEOMETRIES: IndexType = 3;
const MIN_NR_MATERIALS: usize = 1;
const MAX_NR_MATERIALS: usize = 3;
// Must be >= 1
const NR_THEMES_MATERIALS: usize = 3;
const MIN_NR_TEXTURES: usize = 1;
const MAX_NR_TEXTURES: usize = 3;
// Must be >= 1
const NR_THEMES_TEXTURES: usize = 3;
const MAX_NR_VERTICES_TEXTURE: usize = 10;
const MIN_NR_TEMPLATES: usize = 1;
const MAX_NR_TEMPLATES: usize = 10;

struct CityModelBuilder<'cm> {
    id: Option<Cow<'cm, str>>,
    type_cm: Option<CityModelType>,
    version: Option<CityJSONVersion>,
    transform: Option<Transform>,
    cityobjects: Option<CityObjects<'cm>>,
    vertices: Option<Vertices>,
    metadata: Option<Metadata<'cm>>,
    appearance: Option<Appearance<'cm>>,
    geometry_templates: Option<GeometryTemplates<'cm>>,
    extra: Option<Attributes<'cm>>,
    extensions: Option<Extensions>,
    themes_material: Option<Vec<String>>,
    themes_texture: Option<Vec<String>>,
    attributes_cityobject: Option<Attributes<'cm>>,
    attributes_semantic: Option<Attributes<'cm>>,
}

impl<'cm> From<CityModelBuilder<'cm>> for CityModel<'cm> {
    fn from(val: CityModelBuilder<'cm>) -> Self {
        CityModel::new(
            val.id,
            val.type_cm,
            Some(val.version.unwrap_or(CityJSONVersion::V1_1)),
            Some(val.transform.unwrap_or_default()),
            val.cityobjects,
            val.vertices,
            val.metadata,
            val.appearance,
            val.geometry_templates,
            val.extra,
            val.extensions,
        )
    }
}

impl<'cm> Default for CityModelBuilder<'cm> {
    fn default() -> Self {
        CityModelBuilder::new()
            .metadata(None)
            .vertices()
            .materials(None)
            .textures(None)
            .attributes()
            .cityobjects(None, true)
    }
}

impl<'cm> CityModelBuilder<'cm> {
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
            themes_material: None,
            themes_texture: None,
            attributes_cityobject: None,
            attributes_semantic: None,
        }
    }

    /// Generate 1 CityObject if `nr_cityobjects` is `None`, else generate the number of CityObjects
    /// within the provided range. If the `nr_cityobjects` is 1 and `cityobject_hierarchy` is
    /// `true` and the generated CityObject has 2nd-level types, then one additional 2nd-level
    /// CityObject will be created too.
    /// If the `nr_cityobject` is set to a range and `cityobject_hierarchy` is `true`, then the
    /// total number of 1st- and 2nd-level CityObjects will be in the provided range.
    /// If the vertices haven't been generated yet, they will be created, so that the geometry
    /// boundaries can index them.
    pub fn cityobjects(
        mut self,
        nr_cityobjects: Option<Range<usize>>,
        cityobject_hierarchy: bool,
    ) -> Self {
        let use_templates = true;
        let texture_allow_none = false;
        let mut nr_cos_range = nr_cityobjects.unwrap_or(1..2);
        if nr_cos_range.is_empty() {
            nr_cos_range = 1..2;
        }
        let nr_parents_range: Range<usize> = if (nr_cos_range.end - nr_cos_range.start) == 1 {
            // Create one parent and one child.
            1..2
        } else {
            // Half of the range's end becomes parents, then for each eligible parent a child is
            // created. Some 1st-level types don't have sub-types, so they won't have children.
            // In case the total number of created objects is less than the range's start, then
            // additional objects are created so to reach the range's start.
            if cityobject_hierarchy {
                let _n = nr_cos_range.end.div_ceil(2);
                _n.._n + 1
            } else {
                nr_cos_range
            }
        };

        self = self.vertices();

        let nr_vertices = self.vertices.as_ref().unwrap().len();
        let cof_parents = CityObjectFaker::new(
            nr_vertices as IndexType,
            self.appearance.clone(),
            self.themes_material.clone(),
            self.themes_texture.clone(),
            &self.attributes_cityobject,
            &self.attributes_semantic,
            CityObjectLevel::First,
            None,
            texture_allow_none,
        );
        let cos_parents: Vec<CityObject> = (cof_parents, nr_parents_range).fake();
        let estimate_total_nr = if cityobject_hierarchy {
            cos_parents.len() * 2
        } else {
            // Hierarchy is off, so only parents are generated
            cos_parents.len()
        };
        let mut cityobjects = CityObjects::with_capacity(estimate_total_nr);
        if cityobject_hierarchy {
            for mut co_parent in cos_parents.into_iter() {
                let parent_id = Cow::from(Word(EN).fake::<&str>());
                if let Some(subtypes) = get_cityobject_subtype(&co_parent.type_co) {
                    let mut co_child: CityObject = CityObjectFaker::new(
                        nr_vertices as IndexType,
                        self.appearance.clone(),
                        self.themes_material.clone(),
                        self.themes_texture.clone(),
                        &self.attributes_cityobject,
                        &self.attributes_semantic,
                        CityObjectLevel::Second,
                        Some(subtypes),
                        texture_allow_none,
                    )
                    .fake();
                    let child_id = Cow::from(Word(EN).fake::<&str>());
                    co_child.parents = Some(vec![parent_id.clone()]);
                    co_parent.children = Some(vec![child_id.clone()]);
                    cityobjects.insert(child_id, co_child);
                }
                cityobjects.insert(parent_id, co_parent);
            }
        } else {
            cityobjects = CityObjects::from_iter(
                cos_parents
                    .into_iter()
                    .map(|co| (Cow::from(Word(EN).fake::<&str>()), co)),
            );
        }
        self.cityobjects = Some(cityobjects);
        if use_templates {
            let vertices_templates: VerticesTemplates = VerticesTemplatesFaker.fake();
            // The 8th geometry type is GeometryInstance, which cannot be a template
            let geometry_types = Some(vec![
                GeometryType::MultiPoint,
                GeometryType::MultiLineString,
                GeometryType::MultiSurface,
                GeometryType::CompositeSurface,
                GeometryType::Solid,
                GeometryType::MultiSolid,
                GeometryType::CompositeSolid,
            ]);
            let gf = GeometryFaker {
                nr_vertices: IndexType::try_from(vertices_templates.len()).unwrap(),
                // All templates are Buildings, to make our life easier, and so that semantics,
                // materials and textures can be added to them.
                cotype: CityObjectType::Building,
                appearance: self.appearance.clone(),
                themes_material: self.themes_material.clone(),
                themes_texture: self.themes_texture.clone(),
                geometry_types,
                semantics_attributes: &self.attributes_semantic,
                texture_allow_none,
            };
            self.geometry_templates = Some(GeometryTemplates {
                templates: (gf, MIN_NR_TEMPLATES..MAX_NR_TEMPLATES).fake(),
                vertices_templates,
            });
        }
        self
    }

    pub fn attributes(mut self) -> Self {
        self.attributes_cityobject = Some(
            AttributesFaker {
                random_values: false,
                random_keys: false,
            }
            .fake(),
        );
        self.attributes_semantic = Some(
            AttributesFaker {
                random_values: false,
                random_keys: false,
            }
            .fake(),
        );
        // self.extra = Some(
        //     AttributesFaker {
        //         random_values: false,
        //         random_keys: false,
        //     }
        //     .fake(),
        // );
        self
    }

    pub fn materials(mut self, material_builder: Option<MaterialBuilder<'cm>>) -> Self {
        let mat: Vec<Material>;
        if let Some(mb) = material_builder {
            mat = (MIN_NR_MATERIALS..=MAX_NR_MATERIALS)
                .map(|_| mb.clone().build())
                .collect()
        } else {
            mat = (MIN_NR_MATERIALS..=MAX_NR_MATERIALS)
                .map(|_| MaterialBuilder::default().into())
                .collect()
        }
        let themes: Vec<String> = (Word(EN), 1..=NR_THEMES_MATERIALS).fake();
        let default_theme = themes.first().map(|t| Cow::from(t.clone()));
        self.themes_material = Some(themes);
        if let Some(ref mut appearance) = self.appearance {
            appearance.materials = Some(mat);
            appearance.default_theme_material = default_theme;
        } else {
            self.appearance = Some(Appearance {
                materials: Some(mat),
                textures: None,
                vertices_texture: None,
                default_theme_texture: None,
                default_theme_material: default_theme,
            });
        }
        self
    }

    pub fn textures(mut self, texture_builder: Option<TextureBuilder<'cm>>) -> Self {
        let tex: Vec<Texture>;
        if let Some(tb) = texture_builder {
            tex = (MIN_NR_TEXTURES..=MAX_NR_TEXTURES)
                .map(|_| tb.clone().build())
                .collect()
        } else {
            tex = (MIN_NR_TEXTURES..=MAX_NR_TEXTURES)
                .map(|_| TextureBuilder::default().into())
                .collect()
        }
        let themes: Vec<String> = (Word(EN), 1..=NR_THEMES_TEXTURES).fake();
        let default_theme = themes.first().map(|t| Cow::from(t.clone()));
        self.themes_texture = Some(themes);
        let vertices_texture: VerticesTexture =
            (UVCoordinateFaker, 0..=MAX_NR_VERTICES_TEXTURE).fake();
        if let Some(ref mut appearance) = self.appearance {
            appearance.textures = Some(tex);
            appearance.vertices_texture = Some(vertices_texture);
            appearance.default_theme_texture = default_theme;
        } else {
            self.appearance = Some(Appearance {
                materials: None,
                textures: Some(tex),
                vertices_texture: Some(vertices_texture),
                default_theme_texture: default_theme,
                default_theme_material: None,
            });
        }
        self
    }

    pub fn metadata(mut self, metadata_builder: Option<MetadataBuilder<'cm>>) -> Self {
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

    #[allow(dead_code)]
    pub fn build_string(self) -> serde_json::Result<String> {
        serde_json::to_string::<CityModel>(&self.into())
    }

    #[allow(dead_code)]
    pub fn build_vec(self) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec::<CityModel>(&self.into())
    }
}

struct CityObjectFaker<'cmbuild, 'cm> {
    nr_vertices: IndexType,
    // FIXME: this should take an &Option<Appearance, referencing appearance of the CityModelBuilder but I don't know how to make it work
    appearance: Option<Appearance<'cmbuild>>,
    themes_material: Option<Vec<String>>,
    themes_texture: Option<Vec<String>>,
    attributes_cityobject: &'cmbuild Option<Attributes<'cm>>,
    attributes_semantic: &'cmbuild Option<Attributes<'cm>>,
    cityobject_level: CityObjectLevel,
    allowed_types: Option<Vec<CityObjectType>>,
    texture_allow_none: bool,
}

impl<'cm: 'cmbuild, 'cmbuild> CityObjectFaker<'cmbuild, 'cm> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        nr_vertices: IndexType,
        appearance: Option<Appearance<'cmbuild>>,
        themes_material: Option<Vec<String>>,
        themes_texture: Option<Vec<String>>,
        attributes_cityobject: &'cmbuild Option<Attributes<'cm>>,
        attributes_semantic: &'cmbuild Option<Attributes<'cm>>,
        cityobject_level: CityObjectLevel,
        allowed_types: Option<Vec<CityObjectType>>,
        texture_allow_none: bool,
    ) -> Self {
        Self {
            nr_vertices,
            appearance,
            themes_material,
            themes_texture,
            attributes_cityobject,
            attributes_semantic,
            cityobject_level,
            allowed_types,
            texture_allow_none,
        }
    }
}

impl<'cm: 'cmbuild, 'cmbuild> Dummy<CityObjectFaker<'cmbuild, 'cm>> for CityObject<'cm> {
    fn dummy_with_rng<R: Rng + ?Sized>(
        config: &CityObjectFaker<'cmbuild, 'cm>,
        rnd: &mut R,
    ) -> Self {
        let cotype: CityObjectType = if let Some(types) = &config.allowed_types {
            // Safe to unwrap, because allowed_types is never empty
            types.choose(rnd).unwrap().clone()
        } else {
            CityObjectTypeFaker {
                cityobject_level: config.cityobject_level,
            }
            .fake()
        };
        let gf = GeometryFaker::new(
            config.nr_vertices,
            cotype.clone(),
            config.appearance.clone(),
            config.themes_material.clone(),
            config.themes_texture.clone(),
            None,
            config.attributes_semantic,
            config.texture_allow_none,
        );
        let geometry = if config.nr_vertices == 0 {
            None
        } else {
            Some((gf, 0..=MAX_MEMBERS_CITYOBJECT_GEOMETRIES as usize).fake())
        };
        Self::new(
            cotype,
            geometry,
            config.attributes_cityobject.clone(),
            None,
            None,
            None,
            None,
        )
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum CityObjectLevel {
    #[default]
    First,
    Second,
    #[allow(dead_code)]
    Any,
}

fn get_cityobject_subtype(cityobject_type: &CityObjectType) -> Option<Vec<CityObjectType>> {
    match cityobject_type {
        CityObjectType::Bridge => Some(vec![
            CityObjectType::BridgePart,
            CityObjectType::BridgeInstallation,
            CityObjectType::BridgeConstructiveElement,
            CityObjectType::BridgeRoom,
            CityObjectType::BridgeFurniture,
        ]),
        CityObjectType::Building => Some(vec![
            CityObjectType::BuildingPart,
            CityObjectType::BuildingInstallation,
            CityObjectType::BuildingConstructiveElement,
            CityObjectType::BuildingFurniture,
            CityObjectType::BuildingStorey,
            CityObjectType::BuildingRoom,
            CityObjectType::BuildingUnit,
        ]),
        CityObjectType::Tunnel => Some(vec![
            CityObjectType::TunnelPart,
            CityObjectType::TunnelInstallation,
            CityObjectType::TunnelConstructiveElement,
            CityObjectType::TunnelHollowSpace,
            CityObjectType::TunnelFurniture,
        ]),
        _ => None,
    }
}

struct CityObjectTypeFaker {
    cityobject_level: CityObjectLevel,
}

impl Dummy<CityObjectTypeFaker> for CityObjectType {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &CityObjectTypeFaker, rng: &mut R) -> Self {
        let type_idx: u8 = match config.cityobject_level {
            CityObjectLevel::First => rng.gen_range(0..14),
            CityObjectLevel::Second => rng.gen_range(14..31),
            CityObjectLevel::Any => rng.gen_range(0..31),
        };
        match type_idx {
            0 => CityObjectType::Bridge,
            1 => CityObjectType::Building,
            2 => CityObjectType::CityFurniture,
            3 => CityObjectType::LandUse,
            4 => CityObjectType::OtherConstruction,
            5 => CityObjectType::PlantCover,
            6 => CityObjectType::SolitaryVegetationObject,
            7 => CityObjectType::TINRelief,
            8 => CityObjectType::TransportSquare,
            9 => CityObjectType::Railway,
            10 => CityObjectType::Road,
            11 => CityObjectType::Tunnel,
            12 => CityObjectType::WaterBody,
            13 => CityObjectType::Waterway,
            14 => CityObjectType::BridgePart,
            15 => CityObjectType::BridgeInstallation,
            16 => CityObjectType::BridgeConstructiveElement,
            17 => CityObjectType::BridgeRoom,
            18 => CityObjectType::BridgeFurniture,
            19 => CityObjectType::BuildingPart,
            20 => CityObjectType::BuildingInstallation,
            21 => CityObjectType::BuildingConstructiveElement,
            22 => CityObjectType::BuildingFurniture,
            23 => CityObjectType::BuildingStorey,
            24 => CityObjectType::BuildingRoom,
            25 => CityObjectType::BuildingUnit,
            26 => CityObjectType::TunnelPart,
            27 => CityObjectType::TunnelInstallation,
            28 => CityObjectType::TunnelConstructiveElement,
            29 => CityObjectType::TunnelHollowSpace,
            30 => CityObjectType::TunnelFurniture,
            _ => unreachable!(),
        }
    }
}

/// If `texture_allow_none` is `true`, null values are allowed in the texture UV-indices.
/// If `geometry_types` is set, choose from the provided geometry types. If `geometry_types` is
/// `None`, the generated geometry type is chosen randomly from the geometry types that are allowed
/// by the CityObject type.
struct GeometryFaker<'cmbuild, 'cm> {
    nr_vertices: IndexType,
    cotype: CityObjectType,
    appearance: Option<Appearance<'cmbuild>>,
    themes_material: Option<Vec<String>>,
    themes_texture: Option<Vec<String>>,
    geometry_types: Option<Vec<GeometryType>>,
    semantics_attributes: &'cmbuild Option<Attributes<'cm>>,
    texture_allow_none: bool,
}

impl<'cm: 'cmbuild, 'cmbuild> GeometryFaker<'cmbuild, 'cm> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        nr_vertices: IndexType,
        cotype: CityObjectType,
        appearance: Option<Appearance<'cmbuild>>,
        themes_material: Option<Vec<String>>,
        themes_texture: Option<Vec<String>>,
        geometry_types: Option<Vec<GeometryType>>,
        semantics_attributes: &'cmbuild Option<Attributes<'cm>>,
        texture_allow_none: bool,
    ) -> Self {
        Self {
            nr_vertices,
            cotype,
            appearance,
            themes_material,
            themes_texture,
            geometry_types,
            semantics_attributes,
            texture_allow_none,
        }
    }
}

impl<'cm: 'cmbuild, 'cmbuild> Dummy<GeometryFaker<'cmbuild, 'cm>> for Geometry<'cm> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &GeometryFaker<'cmbuild, 'cm>, rng: &mut R) -> Self {
        let lod: LoD = LoDFaker.fake();
        // todo: move this type setup to compile time
        let building_types = config.cotype == CityObjectType::Building
            || config.cotype == CityObjectType::BuildingPart
            || config.cotype == CityObjectType::BuildingStorey
            || config.cotype == CityObjectType::BuildingRoom
            || config.cotype == CityObjectType::BuildingUnit;

        let mut geometry_types = vec![
            GeometryType::MultiPoint,
            GeometryType::MultiLineString,
            GeometryType::MultiSurface,
            GeometryType::CompositeSurface,
            GeometryType::Solid,
            GeometryType::MultiSolid,
            GeometryType::CompositeSolid,
            GeometryType::GeometryInstance,
        ];

        if let Some(ref gt) = config.geometry_types {
            geometry_types = gt.clone()
        } else {
            // Choose a Geometry type that is allowed for the given CityObject type
            if config.cotype == CityObjectType::Bridge
                || config.cotype == CityObjectType::BridgePart
            {
                geometry_types = vec![
                    GeometryType::MultiSurface,
                    GeometryType::CompositeSurface,
                    GeometryType::Solid,
                    GeometryType::CompositeSolid,
                ];
            } else if building_types {
                geometry_types = vec![
                    GeometryType::MultiSurface,
                    GeometryType::CompositeSurface,
                    GeometryType::Solid,
                    GeometryType::CompositeSolid,
                ];
            // } else if config.cotype == CityObjectType::GenericCityObject {
            //     geometry_types = vec![0, 1, 2, 3, 4, 6];
            } else if config.cotype == CityObjectType::LandUse {
                geometry_types = vec![GeometryType::MultiSurface, GeometryType::CompositeSurface];
            } else if config.cotype == CityObjectType::PlantCover {
                geometry_types = vec![
                    GeometryType::MultiSurface,
                    GeometryType::CompositeSurface,
                    GeometryType::Solid,
                    GeometryType::MultiSolid,
                    GeometryType::CompositeSolid,
                ];
            } else if config.cotype == CityObjectType::TINRelief {
                geometry_types = vec![GeometryType::CompositeSurface];
            } else if config.cotype == CityObjectType::Road
                || config.cotype == CityObjectType::Railway
                || config.cotype == CityObjectType::Waterway
                || config.cotype == CityObjectType::TransportSquare
            {
                geometry_types = vec![
                    GeometryType::MultiLineString,
                    GeometryType::MultiSurface,
                    GeometryType::CompositeSurface,
                ];
            } else if config.cotype == CityObjectType::Tunnel
                || config.cotype == CityObjectType::TunnelPart
            {
                geometry_types = vec![
                    GeometryType::MultiSurface,
                    GeometryType::CompositeSurface,
                    GeometryType::Solid,
                    GeometryType::CompositeSolid,
                ];
            } else if config.cotype == CityObjectType::WaterBody {
                geometry_types = vec![
                    GeometryType::MultiLineString,
                    GeometryType::MultiSurface,
                    GeometryType::CompositeSurface,
                    GeometryType::Solid,
                    GeometryType::CompositeSolid,
                ];
            }
        }
        let geometry_type_chosen = geometry_types
            .choose(rng)
            .unwrap_or(&GeometryType::MultiPoint);
        // Decide if we can generate semantics for the given CityObject type
        let mut generate_semantics = false;
        if lod >= LoD::LoD2
            && (building_types
                || config.cotype == CityObjectType::BuildingInstallation
                || config.cotype == CityObjectType::WaterBody
                || config.cotype == CityObjectType::Road
                || config.cotype == CityObjectType::Railway
                || config.cotype == CityObjectType::TransportSquare)
        {
            generate_semantics = true;
        }
        // Decide if we can generate materials
        let mut generate_materials = false;
        let mut nr_materials: IndexType = 0;
        // The material themes of the geometry
        let mut themes_material: Vec<String> = Vec::new();
        // The whole geometry gets a single material
        let mut single_material = false;
        if let Some(ref appearance) = config.appearance {
            if let Some(ref materials_vec) = appearance.materials {
                nr_materials = IndexType::try_from(materials_vec.len()).unwrap();
                if nr_materials > 0 {
                    generate_materials = true;
                    // Choose the material themes from the available themes.
                    // One of the themes must be the default theme.
                    if let Some(ref all_themes_materials) = config.themes_material {
                        if let Some(ref default_theme) = appearance.default_theme_material {
                            themes_material.push(default_theme.to_string());
                            if let Some(t) = all_themes_materials[1..].choose(rng) {
                                themes_material.push(t.to_string());
                            }
                        }
                    }
                    single_material = rng.gen_bool(0.5);
                }
            }
        }
        // Decide if we can generate textures
        let mut generate_textures = false;
        let mut nr_textures: IndexType = 0;
        let mut nr_vertices_texture: IndexType = 0;
        // The texture themes of the geometry
        let mut themes_texture: Vec<String> = Vec::new();
        if let Some(ref appearance) = config.appearance {
            if let Some(ref textures_vec) = appearance.textures {
                nr_textures = IndexType::try_from(textures_vec.len()).unwrap();
                if nr_textures > 0 {
                    generate_textures = true;
                    // Choose the texture themes from the available themes.
                    // One of the themes must be the default theme.
                    if let Some(ref all_themes_textures) = config.themes_texture {
                        if let Some(ref default_theme) = appearance.default_theme_texture {
                            themes_texture.push(default_theme.to_string());
                            if let Some(t) = all_themes_textures[1..].choose(rng) {
                                themes_texture.push(t.to_string());
                            }
                        }
                    }
                    if let Some(ref vt) = appearance.vertices_texture {
                        nr_vertices_texture = IndexType::try_from(vt.len()).unwrap();
                    }
                }
            }
        }

        let boundaries: Option<Boundary> = None;
        let mut semantics: Option<Semantics> = None;
        let mut material: Option<MaterialMap> = None;
        let mut texture: Option<TextureMap> = None;
        let template: Option<u16>;
        let template_boundaries: Option<[usize; 1]>;
        let template_transformation_matrix: Option<[f64; 16]>;

        match geometry_type_chosen {
            GeometryType::MultiPoint => {
                let boundaries: Boundary = MultiPointFaker::new(config.nr_vertices).fake();
                let nr_points = IndexType::try_from(boundaries.vertices.len()).unwrap();
                semantics = generate_semantics.then(|| {
                    MultiPointSemanticsFaker::new(
                        nr_points,
                        config.cotype.clone(),
                        config.semantics_attributes,
                    )
                    .fake()
                });
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
            GeometryType::MultiLineString => {
                let boundaries: Boundary = MultiLineStringFaker::new(config.nr_vertices).fake();
                let nr_linestrings = IndexType::try_from(boundaries.rings.len()).unwrap();
                semantics = generate_semantics.then(|| {
                    MultiLineStringSemanticsFaker::new(
                        nr_linestrings,
                        config.cotype.clone(),
                        config.semantics_attributes,
                    )
                    .fake()
                });
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
            GeometryType::MultiSurface => {
                let boundaries: Boundary = MultiSurfaceFaker::new(config.nr_vertices).fake();
                let nr_surfaces = IndexType::try_from(boundaries.surfaces.len()).unwrap();
                semantics = generate_semantics.then(|| {
                    MultiSurfaceSemanticsFaker::new(
                        nr_surfaces,
                        config.cotype.clone(),
                        config.semantics_attributes,
                    )
                    .fake()
                });
                material = generate_materials.then(|| {
                    MaterialMapFaker::new(
                        nr_materials,
                        themes_material,
                        single_material,
                        &boundaries,
                    )
                    .fake()
                });
                texture = generate_textures.then(|| {
                    TextureMapFaker::new(
                        nr_textures,
                        nr_vertices_texture,
                        themes_texture,
                        &boundaries,
                        config.texture_allow_none,
                    )
                    .fake()
                });
                Geometry {
                    type_: GeometryType::MultiSurface,
                    lod: Some(lod),
                    boundaries: Some(boundaries),
                    semantics,
                    material,
                    texture,
                    template: None,
                    template_boundaries: None,
                    template_transformation_matrix: None,
                }
            }
            GeometryType::CompositeSurface => {
                let boundaries: Boundary = MultiSurfaceFaker::new(config.nr_vertices).fake();
                let nr_surfaces = IndexType::try_from(boundaries.surfaces.len()).unwrap();
                semantics = generate_semantics.then(|| {
                    MultiSurfaceSemanticsFaker::new(
                        nr_surfaces,
                        config.cotype.clone(),
                        config.semantics_attributes,
                    )
                    .fake()
                });
                material = generate_materials.then(|| {
                    MaterialMapFaker::new(
                        nr_materials,
                        themes_material,
                        single_material,
                        &boundaries,
                    )
                    .fake()
                });
                texture = generate_textures.then(|| {
                    TextureMapFaker::new(
                        nr_textures,
                        nr_vertices_texture,
                        themes_texture,
                        &boundaries,
                        config.texture_allow_none,
                    )
                    .fake()
                });
                Geometry {
                    type_: GeometryType::CompositeSurface,
                    lod: Some(lod),
                    boundaries: Some(boundaries),
                    semantics,
                    material,
                    texture,
                    template: None,
                    template_boundaries: None,
                    template_transformation_matrix: None,
                }
            }
            GeometryType::Solid => {
                let boundaries: Boundary = SolidFaker::new(config.nr_vertices).fake();
                semantics = generate_semantics.then(|| {
                    SolidSemanticsFaker::new(
                        &boundaries,
                        config.cotype.clone(),
                        config.semantics_attributes,
                    )
                    .fake()
                });
                material = generate_materials.then(|| {
                    MaterialMapFaker::new(
                        nr_materials,
                        themes_material,
                        single_material,
                        &boundaries,
                    )
                    .fake()
                });
                texture = generate_textures.then(|| {
                    TextureMapFaker::new(
                        nr_textures,
                        nr_vertices_texture,
                        themes_texture,
                        &boundaries,
                        config.texture_allow_none,
                    )
                    .fake()
                });
                Geometry {
                    type_: GeometryType::Solid,
                    lod: Some(lod),
                    boundaries: Some(boundaries),
                    semantics,
                    material,
                    texture,
                    template: None,
                    template_boundaries: None,
                    template_transformation_matrix: None,
                }
            }
            GeometryType::MultiSolid => {
                let boundaries: Boundary = MultiSolidFaker::new(config.nr_vertices).fake();
                semantics = generate_semantics.then(|| {
                    MultiSolidSemanticsFaker::new(
                        &boundaries,
                        config.cotype.clone(),
                        config.semantics_attributes,
                    )
                    .fake()
                });
                material = generate_materials.then(|| {
                    MaterialMapFaker::new(
                        nr_materials,
                        themes_material,
                        single_material,
                        &boundaries,
                    )
                    .fake()
                });
                texture = generate_textures.then(|| {
                    TextureMapFaker::new(
                        nr_textures,
                        nr_vertices_texture,
                        themes_texture,
                        &boundaries,
                        config.texture_allow_none,
                    )
                    .fake()
                });
                Geometry {
                    type_: GeometryType::MultiSolid,
                    lod: Some(lod),
                    boundaries: Some(boundaries),
                    semantics,
                    material,
                    texture,
                    template: None,
                    template_boundaries: None,
                    template_transformation_matrix: None,
                }
            }
            GeometryType::CompositeSolid => {
                let boundaries: Boundary = MultiSolidFaker::new(config.nr_vertices).fake();
                semantics = generate_semantics.then(|| {
                    MultiSolidSemanticsFaker::new(
                        &boundaries,
                        config.cotype.clone(),
                        config.semantics_attributes,
                    )
                    .fake()
                });
                material = generate_materials.then(|| {
                    MaterialMapFaker::new(
                        nr_materials,
                        themes_material,
                        single_material,
                        &boundaries,
                    )
                    .fake()
                });
                texture = generate_textures.then(|| {
                    TextureMapFaker::new(
                        nr_textures,
                        nr_vertices_texture,
                        themes_texture,
                        &boundaries,
                        config.texture_allow_none,
                    )
                    .fake()
                });
                Geometry {
                    type_: GeometryType::CompositeSolid,
                    lod: Some(lod),
                    boundaries: Some(boundaries),
                    semantics,
                    material,
                    texture,
                    template: None,
                    template_boundaries: None,
                    template_transformation_matrix: None,
                }
            }
            GeometryType::GeometryInstance => {
                let reference_point: u32 = IndexFaker {
                    max: config.nr_vertices,
                }
                .fake();
                template_boundaries = Some([reference_point as usize]);
                template = Some(0);
                template_transformation_matrix = Some((0.0..f64::MAX).fake());
                Geometry {
                    type_: GeometryType::GeometryInstance,
                    lod: None,
                    boundaries,
                    semantics,
                    material,
                    texture,
                    template,
                    template_boundaries,
                    template_transformation_matrix,
                }
            }
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
// FIXME: shouldn't have empty arrays
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
            let nr_shells = rng.gen_range(MIN_MEMBERS_SOLID..=MAX_MEMBERS_SOLID);
            solid_i += nr_shells;

            fake_solid_boundary(
                config.nr_vertices,
                rng,
                &mut boundary,
                min_linestring_len,
                &mut ring_i,
                &mut surface_i,
                &mut shell_i,
                nr_shells,
            );
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
            shells: LargeIndexVec::with_capacity(MAX_MEMBERS_SOLID as usize),
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
        fake_solid_boundary(
            config.nr_vertices,
            rng,
            &mut boundary,
            min_linestring_len,
            &mut ring_i,
            &mut surface_i,
            &mut shell_i,
            nr_shells,
        );

        boundary
    }
}

#[allow(clippy::too_many_arguments)]
fn fake_solid_boundary<R: Rng + ?Sized>(
    nr_vertices_citymodel: IndexType,
    rng: &mut R,
    boundary: &mut Boundary,
    min_linestring_len: IndexType,
    ring_i: &mut u32,
    surface_i: &mut u32,
    shell_i: &mut u32,
    nr_shells: IndexType,
) {
    for _shell in MIN_MEMBERS_SOLID..=nr_shells {
        boundary.shells.push(LargeIndex::from(*shell_i));
        let shell_len = rng.gen_range(MIN_MEMBERS_MULTISURFACE..=MAX_MEMBERS_MULTISURFACE);
        *shell_i += shell_len;

        // Add the surfaces for each shell
        for _surface in MIN_MEMBERS_MULTISURFACE..=shell_len {
            boundary.surfaces.push(LargeIndex::from(*surface_i));
            let nr_rings = rng.gen_range(MIN_MEMBERS_MULTILINESTRING..=MAX_MEMBERS_MULTILINESTRING);
            *surface_i += nr_rings;

            // Add the rings for each surface
            fake_surface_boundary(
                nr_vertices_citymodel,
                rng,
                boundary,
                min_linestring_len,
                ring_i,
                nr_rings,
            );
        }
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
            let nr_rings = rng.gen_range(MIN_MEMBERS_MULTILINESTRING..=MAX_MEMBERS_MULTILINESTRING);
            surface_i += nr_rings;

            // Add the rings for each surface
            fake_surface_boundary(
                config.nr_vertices,
                rng,
                &mut boundary,
                min_linestring_len,
                &mut ring_i,
                nr_rings,
            );
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
            rings: LargeIndexVec::with_capacity(MAX_MEMBERS_MULTILINESTRING as usize),
            surfaces: LargeIndexVec::default(),
            shells: LargeIndexVec::default(),
            solids: LargeIndexVec::default(),
        };

        // A linestring must have at least two vertices, otherwise it's not a line.
        let min_linestring_len = if MIN_MEMBERS_MULTIPOINT > 1 {
            MIN_MEMBERS_MULTIPOINT
        } else {
            2
        };

        // Counters
        let mut ring_i = 0u32;

        let nr_rings = rng.gen_range(MIN_MEMBERS_MULTILINESTRING..=MAX_MEMBERS_MULTILINESTRING);
        fake_surface_boundary(
            config.nr_vertices,
            rng,
            &mut boundary,
            min_linestring_len,
            &mut ring_i,
            nr_rings,
        );
        boundary
    }
}

/// Generate one surface and add it to the boundary.
fn fake_surface_boundary<R: Rng + ?Sized>(
    nr_vertices_citymodel: IndexType,
    rng: &mut R,
    boundary: &mut Boundary,
    min_linestring_len: IndexType,
    ring_i: &mut u32,
    nr_rings: IndexType,
) {
    for _ring in MIN_MEMBERS_MULTILINESTRING..=nr_rings {
        boundary.rings.push(LargeIndex::from(*ring_i));
        let ring_len = rng.gen_range(min_linestring_len..=MAX_MEMBERS_MULTIPOINT);
        *ring_i += ring_len;

        // Add the vertices for each ring
        // Cannot have an empty ring, so we start at 1 (https://github.com/cityjson/specs/issues/189)
        let nr_vertices: IndexType = rng.gen_range(MIN_MEMBERS_MULTIPOINT..=MAX_MEMBERS_MULTIPOINT);
        boundary.vertices.extend(
            (1..nr_vertices).map(|_| IndexFaker::new(nr_vertices_citymodel).fake::<LargeIndex>()),
        );
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
        // If the number of vertices is 0, create an empty range, which will cause
        // LargeIndexVecFaker to generate an empty vector.
        let range_members_multipoint = if config.nr_vertices == 0 {
            config.nr_vertices + 1..=config.nr_vertices
        } else {
            MIN_MEMBERS_MULTIPOINT..=MAX_MEMBERS_MULTIPOINT
        };
        Boundary {
            vertices: LargeIndexVecFaker {
                index_faker: vf,
                range: range_members_multipoint,
            }
            .fake(),
            rings: Default::default(),
            surfaces: Default::default(),
            shells: Default::default(),
            solids: Default::default(),
        }
    }
}

#[test]
fn bla() {
    let nr_vertices = 0;
    let vf = IndexFaker::new(nr_vertices);
    let range_members_multipoint = if nr_vertices == 0 {
        nr_vertices + 1..=nr_vertices
    } else {
        MIN_MEMBERS_MULTIPOINT..=MAX_MEMBERS_MULTIPOINT
    };
    dbg!(range_members_multipoint.is_empty());
    let boundary_vertices: LargeIndexVec = LargeIndexVecFaker {
        index_faker: vf,
        range: range_members_multipoint,
    }
    .fake();
    dbg!(boundary_vertices);
}

struct LargeIndexVecFaker {
    index_faker: IndexFaker,
    range: RangeInclusive<u32>,
}

impl Dummy<LargeIndexVecFaker> for LargeIndexVec {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &LargeIndexVecFaker, _: &mut R) -> Self {
        if config.range.is_empty() {
            LargeIndexVec::default()
        } else {
            LargeIndexVec::from(
                (
                    config.index_faker,
                    *config.range.start() as usize..*config.range.end() as usize,
                )
                    .fake::<Vec<u32>>(),
            )
        }
    }
}

#[derive(Clone, Copy)]
struct IndexFaker {
    max: IndexType,
}

impl IndexFaker {
    fn new(nr_vertices: IndexType) -> Self {
        Self {
            max: if nr_vertices > 0 { nr_vertices - 1 } else { 0 },
        }
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

// FIXME: this can generate an empty vertices vec. Should be able to set the amount of vertices
//  needed, with the minimum of 1.
fn fake_vertices() -> Vertices {
    Faker.fake::<Vertices>()
}

struct VerticesTemplatesFaker;
impl Dummy<VerticesTemplatesFaker> for VerticesTemplates {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &VerticesTemplatesFaker, _: &mut R) -> Self {
        (TemplateVertexFaker, 1..=MAX_NR_VERTICES as usize).fake()
    }
}

type TemplateVertex = [f64; 3];
struct TemplateVertexFaker;
impl Dummy<TemplateVertexFaker> for TemplateVertex {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &TemplateVertexFaker, _: &mut R) -> Self {
        Faker.fake()
    }
}

struct CoordinateFaker {
    min: i64,
    max: i64,
}

impl Dummy<CoordinateFaker> for [i64; 3] {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &CoordinateFaker, rng: &mut R) -> Self {
        [
            rng.gen_range(config.min..=config.max),
            rng.gen_range(0..=config.max),
            rng.gen_range(0..=config.max),
        ]
    }
}

struct VerticesFaker;
impl Dummy<VerticesFaker> for Vertices {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &VerticesFaker, _: &mut R) -> Self {
        let cf = CoordinateFaker {
            min: MIN_COORDINATE,
            max: MAX_COORDINATE,
        };
        (cf, MIN_NR_VERTICES as usize..=MAX_NR_VERTICES as usize).fake()
    }
}

struct MultiSolidSemanticsFaker<'semfaker, 'cmbuild, 'cm> {
    boundary: &'semfaker Boundary,
    cotype: CityObjectType,
    attributes: &'cmbuild Option<Attributes<'cm>>,
}

impl<'cm: 'cmbuild, 'cmbuild, 'semfaker> MultiSolidSemanticsFaker<'semfaker, 'cmbuild, 'cm> {
    fn new(
        boundary: &'semfaker Boundary,
        cotype: CityObjectType,
        attributes: &'cmbuild Option<Attributes<'cm>>,
    ) -> Self {
        Self {
            boundary,
            cotype,
            attributes,
        }
    }
}

impl<'cm: 'semfaker + 'cmbuild, 'cmbuild, 'semfaker>
    Dummy<MultiSolidSemanticsFaker<'semfaker, 'cmbuild, 'cm>> for Semantics<'cm>
{
    fn dummy_with_rng<R: Rng + ?Sized>(
        config: &MultiSolidSemanticsFaker<'semfaker, 'cmbuild, 'cm>,
        rng: &mut R,
    ) -> Self {
        if config.boundary.solids.is_empty() {
            Self::new(Vec::<Semantic>::default(), LabelIndex::default())
        } else {
            let (surfaces, values) = fake_depth_three_semantics(
                config.cotype.clone(),
                config.boundary,
                rng,
                config.attributes,
            );
            Self::new(surfaces, values)
        }
    }
}

struct SolidSemanticsFaker<'semfaker, 'cmbuild, 'cm> {
    boundary: &'semfaker Boundary,
    cotype: CityObjectType,
    attributes: &'cmbuild Option<Attributes<'cm>>,
}

impl<'cm: 'cmbuild, 'cmbuild, 'semfaker> SolidSemanticsFaker<'semfaker, 'cmbuild, 'cm> {
    fn new(
        boundary: &'semfaker Boundary,
        cotype: CityObjectType,
        attributes: &'cmbuild Option<Attributes<'cm>>,
    ) -> Self {
        Self {
            boundary,
            cotype,
            attributes,
        }
    }
}

impl<'cm: 'semfaker + 'cmbuild, 'cmbuild, 'semfaker>
    Dummy<SolidSemanticsFaker<'semfaker, 'cmbuild, 'cm>> for Semantics<'cm>
{
    fn dummy_with_rng<R: Rng + ?Sized>(
        config: &SolidSemanticsFaker<'semfaker, 'cmbuild, 'cm>,
        rng: &mut R,
    ) -> Self {
        if config.boundary.shells.is_empty() {
            Self::new(Vec::<Semantic>::default(), LabelIndex::default())
        } else {
            let (surfaces, values) = fake_depth_two_semantics(
                config.cotype.clone(),
                config.boundary,
                rng,
                config.attributes,
            );
            Self::new(surfaces, values)
        }
    }
}

struct MultiSurfaceSemanticsFaker<'cmbuild, 'cm> {
    nr_surfaces: IndexType,
    cotype: CityObjectType,
    attributes: &'cmbuild Option<Attributes<'cm>>,
}

impl<'cm: 'cmbuild, 'cmbuild> MultiSurfaceSemanticsFaker<'cmbuild, 'cm> {
    fn new(
        nr_surfaces: IndexType,
        cotype: CityObjectType,
        attributes: &'cmbuild Option<Attributes<'cm>>,
    ) -> Self {
        Self {
            nr_surfaces,
            cotype,
            attributes,
        }
    }
}

impl<'cm: 'cmbuild, 'cmbuild> Dummy<MultiSurfaceSemanticsFaker<'cmbuild, 'cm>> for Semantics<'cm> {
    fn dummy_with_rng<R: Rng + ?Sized>(
        config: &MultiSurfaceSemanticsFaker<'cmbuild, 'cm>,
        rng: &mut R,
    ) -> Self {
        if config.nr_surfaces == 0 {
            Self::new(Vec::<Semantic>::default(), LabelIndex::default())
        } else {
            let (surfaces, values) = fake_depth_one_semantics(
                config.cotype.clone(),
                config.nr_surfaces,
                rng,
                config.attributes,
            );
            Self::new(
                surfaces,
                LabelIndex {
                    points: vec![],
                    linestrings: vec![],
                    surfaces: values,
                    shells: Default::default(),
                    solids: Default::default(),
                },
            )
        }
    }
}

struct MultiLineStringSemanticsFaker<'cmbuild, 'cm> {
    nr_linestrings: IndexType,
    cotype: CityObjectType,
    attributes: &'cmbuild Option<Attributes<'cm>>,
}

impl<'cm: 'cmbuild, 'cmbuild> MultiLineStringSemanticsFaker<'cmbuild, 'cm> {
    fn new(
        nr_linestrings: IndexType,
        cotype: CityObjectType,
        attributes: &'cmbuild Option<Attributes<'cm>>,
    ) -> Self {
        Self {
            nr_linestrings,
            cotype,
            attributes,
        }
    }
}

impl<'cm: 'cmbuild, 'cmbuild> Dummy<MultiLineStringSemanticsFaker<'cmbuild, 'cm>>
    for Semantics<'cm>
{
    fn dummy_with_rng<R: Rng + ?Sized>(
        config: &MultiLineStringSemanticsFaker<'cmbuild, 'cm>,
        rng: &mut R,
    ) -> Self {
        if config.nr_linestrings == 0 {
            Self::new(Vec::<Semantic>::default(), LabelIndex::default())
        } else {
            let (surfaces, values) = fake_depth_one_semantics(
                config.cotype.clone(),
                config.nr_linestrings,
                rng,
                config.attributes,
            );
            Self::new(
                surfaces,
                LabelIndex {
                    points: vec![],
                    linestrings: values,
                    surfaces: vec![],
                    shells: Default::default(),
                    solids: Default::default(),
                },
            )
        }
    }
}

struct MultiPointSemanticsFaker<'cmbuild, 'cm> {
    nr_points: IndexType,
    cotype: CityObjectType,
    attributes: &'cmbuild Option<Attributes<'cm>>,
}

impl<'cm: 'cmbuild, 'cmbuild> MultiPointSemanticsFaker<'cmbuild, 'cm> {
    fn new(
        nr_points: IndexType,
        cotype: CityObjectType,
        attributes: &'cmbuild Option<Attributes<'cm>>,
    ) -> Self {
        Self {
            nr_points,
            cotype,
            attributes,
        }
    }
}

impl<'cm: 'cmbuild, 'cmbuild> Dummy<MultiPointSemanticsFaker<'cmbuild, 'cm>> for Semantics<'cm> {
    fn dummy_with_rng<R: Rng + ?Sized>(
        config: &MultiPointSemanticsFaker<'cmbuild, 'cm>,
        rng: &mut R,
    ) -> Self {
        if config.nr_points == 0 {
            Self::new(Vec::<Semantic>::default(), LabelIndex::default())
        } else {
            let (surfaces, values) = fake_depth_one_semantics(
                config.cotype.clone(),
                config.nr_points,
                rng,
                config.attributes,
            );
            Self::new(
                surfaces,
                LabelIndex {
                    points: values,
                    linestrings: vec![],
                    surfaces: vec![],
                    shells: Default::default(),
                    solids: Default::default(),
                },
            )
        }
    }
}

fn fake_depth_three_semantics<'cm: 'cmbuild, 'cmbuild, 'semfaker, R: Rng + ?Sized>(
    cotype: CityObjectType,
    boundary: &'semfaker Boundary,
    rng: &mut R,
    attributes: &'cmbuild Option<Attributes<'cm>>,
) -> (Vec<Semantic<'cm>>, LabelIndex) {
    // semantics.surfaces
    // The number of surfaces in the first shell determines the number of different Semantic objects
    let (nr_semantic, surfaces) = fake_semantics_surfaces(
        cotype,
        boundary.surfaces.len() as IndexType,
        rng,
        attributes,
    );
    // semantics.values
    let idxf = OptionalIndexFaker::new(nr_semantic, true);
    let surfaces_values =
        (idxf, boundary.surfaces.len()..=boundary.surfaces.len()).fake::<Vec<OptionalLargeIndex>>();
    (
        surfaces,
        LabelIndex {
            points: vec![],
            linestrings: vec![],
            surfaces: surfaces_values,
            shells: boundary.shells.clone(),
            solids: boundary.solids.clone(),
        },
    )
}

fn fake_depth_two_semantics<'cm: 'cmbuild, 'cmbuild, 'semfaker, R: Rng + ?Sized>(
    cotype: CityObjectType,
    boundary: &'semfaker Boundary,
    rng: &mut R,
    attributes: &'cmbuild Option<Attributes<'cm>>,
) -> (Vec<Semantic<'cm>>, LabelIndex) {
    // semantics.surfaces
    // The number of surfaces in the first shell determines the number of different Semantic objects
    let (nr_semantic, surfaces) = fake_semantics_surfaces(
        cotype,
        boundary.surfaces.len() as IndexType,
        rng,
        attributes,
    );
    // semantics.values
    let idxf = OptionalIndexFaker::new(nr_semantic, true);
    let surfaces_values =
        (idxf, boundary.surfaces.len()..=boundary.surfaces.len()).fake::<Vec<OptionalLargeIndex>>();
    (
        surfaces,
        LabelIndex {
            points: vec![],
            linestrings: vec![],
            surfaces: surfaces_values,
            shells: boundary.shells.clone(),
            solids: Default::default(),
        },
    )
}

fn fake_depth_one_semantics<'cm: 'cmbuild, 'cmbuild, R: Rng + ?Sized>(
    cotype: CityObjectType,
    nr_members: IndexType,
    rng: &mut R,
    attributes: &'cmbuild Option<Attributes<'cm>>,
) -> (Vec<Semantic<'cm>>, Vec<OptionalLargeIndex>) {
    let (nr_semantic, surfaces) = fake_semantics_surfaces(cotype, nr_members, rng, attributes);
    let idxf = OptionalIndexFaker::new(nr_semantic, true);
    let values =
        (idxf, nr_members as usize..=nr_members as usize).fake::<Vec<OptionalLargeIndex>>();
    (surfaces, values)
}

fn fake_semantics_surfaces<'cm: 'cmbuild, 'cmbuild, R: Rng + ?Sized>(
    cotype: CityObjectType,
    nr_members: IndexType,
    rng: &mut R,
    attributes: &'cmbuild Option<Attributes<'cm>>,
) -> (IndexType, Vec<Semantic<'cm>>) {
    let sf = SemanticFaker::new(cotype, attributes);
    // We have max. as many different Semantics as there are geometry members
    let nr_semantic: IndexType = (1..=nr_members).fake_with_rng(rng);
    let surfaces: Vec<Semantic> = (0..=nr_semantic)
        .filter_map(|_| sf.fake::<Option<Semantic>>())
        .collect();
    (nr_semantic, surfaces)
}

struct SemanticFaker<'cmbuild, 'cm> {
    cotype: CityObjectType,
    attributes: &'cmbuild Option<Attributes<'cm>>,
}

impl<'cm: 'cmbuild, 'cmbuild> SemanticFaker<'cmbuild, 'cm> {
    fn new(cotype: CityObjectType, attributes: &'cmbuild Option<Attributes<'cm>>) -> Self {
        Self { cotype, attributes }
    }
}

impl<'cm: 'cmbuild, 'cmbuild> Dummy<SemanticFaker<'cmbuild, 'cm>> for Option<Semantic<'cm>> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &SemanticFaker<'cmbuild, 'cm>, _: &mut R) -> Self {
        SemanticTypeFaker::new(config.cotype.clone())
            .fake::<Option<SemanticType>>()
            .map(|semtype| Semantic {
                type_sem: semtype,
                children: None,
                parent: None,
                attributes: config.attributes.clone(),
            })
    }
}

struct SemanticTypeFaker {
    cotype: CityObjectType,
}

impl SemanticTypeFaker {
    fn new(cotype: CityObjectType) -> Self {
        Self { cotype }
    }
}

// Not all CityObject types can have Semantics, so we return an Option
impl Dummy<SemanticTypeFaker> for Option<SemanticType> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &SemanticTypeFaker, rng: &mut R) -> Self {
        let building_types = config.cotype == CityObjectType::Building
            || config.cotype == CityObjectType::BuildingPart
            || config.cotype == CityObjectType::BuildingStorey
            || config.cotype == CityObjectType::BuildingRoom
            || config.cotype == CityObjectType::BuildingUnit
            || config.cotype == CityObjectType::BridgeInstallation;
        let transportation_types = config.cotype == CityObjectType::Road
            || config.cotype == CityObjectType::Railway
            || config.cotype == CityObjectType::TransportSquare;
        let semantic_types: Vec<usize>;
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
            0 => SemanticType::RoofSurface,
            1 => SemanticType::GroundSurface,
            2 => SemanticType::WallSurface,
            3 => SemanticType::ClosureSurface,
            4 => SemanticType::OuterCeilingSurface,
            5 => SemanticType::OuterFloorSurface,
            6 => SemanticType::Window,
            7 => SemanticType::Door,
            8 => SemanticType::InteriorWallSurface,
            9 => SemanticType::CeilingSurface,
            10 => SemanticType::FloorSurface,
            11 => SemanticType::WaterSurface,
            12 => SemanticType::WaterGroundSurface,
            13 => SemanticType::WaterClosureSurface,
            14 => SemanticType::TrafficArea,
            15 => SemanticType::AuxiliaryTrafficArea,
            16 => SemanticType::TransportationMarking,
            17 => SemanticType::TransportationHole,
            _ => unreachable!(),
        };
        Some(semantic)
    }
}

#[derive(Clone, Copy)]
struct OptionalIndexFaker {
    max: IndexType,
    allow_none: bool,
}

impl OptionalIndexFaker {
    fn new(max_index: IndexType, allow_none: bool) -> Self {
        Self {
            max: max_index,
            allow_none,
        }
    }
}

// todo: here i have to use Option<LargeIndex>, i cannot use the OptionalLargeIndex for some reason
impl Dummy<OptionalIndexFaker> for Option<LargeIndex> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &OptionalIndexFaker, rng: &mut R) -> Self {
        // Probability of having a semantic for the surface, instead of a null
        let prob = if config.allow_none { 0.8 } else { 1.0 };
        let d = Bernoulli::new(prob).unwrap();
        let has_semantic = d.sample(&mut thread_rng());
        if has_semantic {
            let idx: IndexType = rng.gen_range(0..=config.max);
            Some(LargeIndex::from(idx))
        } else {
            None
        }
    }
}

#[derive(Clone)]
struct MaterialBuilder<'cm>(Material<'cm>);

impl<'cm> From<MaterialBuilder<'cm>> for Material<'cm> {
    fn from(val: MaterialBuilder<'cm>) -> Self {
        val.0
    }
}

impl<'cm> Default for MaterialBuilder<'cm> {
    fn default() -> Self {
        Self::new()
            .name()
            .ambient_intensity()
            .diffuse_color()
            .emissive_color()
            .specular_color()
            .shininess()
            .transparency()
            .smooth()
    }
}

impl<'cm> MaterialBuilder<'cm> {
    fn new() -> Self {
        Self(Material::new())
    }

    fn name(mut self) -> Self {
        self.0.name = Cow::from(Word(EN).fake::<&str>());
        self
    }

    fn ambient_intensity(mut self) -> Self {
        self.0.ambient_intensity = Some(thread_rng().gen_range(0.0f32..=0.1));
        self
    }

    fn diffuse_color(mut self) -> Self {
        self.0.diffuse_color = Some(RgbFaker.fake());
        self
    }

    fn emissive_color(mut self) -> Self {
        self.0.emissive_color = Some(RgbFaker.fake());
        self
    }

    fn specular_color(mut self) -> Self {
        self.0.diffuse_color = Some(RgbFaker.fake());
        self
    }

    fn shininess(mut self) -> Self {
        self.0.shininess = Some(thread_rng().gen_range(0.0f32..=0.1));
        self
    }

    fn transparency(mut self) -> Self {
        self.0.transparency = Some(thread_rng().gen_range(0.0f32..=0.1));
        self
    }

    fn smooth(mut self) -> Self {
        self.0.is_smooth = Some(thread_rng().gen_bool(0.5));
        self
    }

    /// Builds a Material with new values set for the members that are configured in the builder.
    fn build(self) -> Material<'cm> {
        let mut mb = self.name();
        if mb.0.ambient_intensity.is_some() {
            mb = mb.ambient_intensity();
        }
        if mb.0.diffuse_color.is_some() {
            mb = mb.diffuse_color();
        }
        if mb.0.emissive_color.is_some() {
            mb = mb.emissive_color();
        }
        if mb.0.specular_color.is_some() {
            mb = mb.specular_color();
        }
        if mb.0.shininess.is_some() {
            mb = mb.shininess();
        }
        if mb.0.transparency.is_some() {
            mb = mb.transparency();
        }
        if mb.0.is_smooth.is_some() {
            mb = mb.smooth();
        }
        mb.into()
    }
}

type Rgb = [f32; 3];

struct RgbFaker;

impl Dummy<RgbFaker> for Rgb {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &RgbFaker, rng: &mut R) -> Self {
        let color_range = 0.0f32..=1.0;
        [
            rng.gen_range(color_range.clone()),
            rng.gen_range(color_range.clone()),
            rng.gen_range(color_range.clone()),
        ]
    }
}

type Rgba = [f32; 4];

struct RgbaFaker;

impl Dummy<RgbaFaker> for Rgba {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &RgbaFaker, rng: &mut R) -> Self {
        let color_range = 0.0f32..=1.0;
        [
            rng.gen_range(color_range.clone()),
            rng.gen_range(color_range.clone()),
            rng.gen_range(color_range.clone()),
            rng.gen_range(color_range.clone()),
        ]
    }
}

/// Fake the materials for Multi/CompositeSurface, Solid, Multi/CompositeSolid geometries.
struct MaterialMapFaker<'matmapfaker> {
    nr_materials: IndexType,
    themes_material: Vec<String>,
    single_material: bool,
    boundary: &'matmapfaker Boundary,
}

impl<'matmapfaker> MaterialMapFaker<'matmapfaker> {
    fn new(
        nr_materials: IndexType,
        themes_material: Vec<String>,
        single_material: bool,
        boundary: &'matmapfaker Boundary,
    ) -> Self {
        Self {
            nr_materials,
            themes_material,
            single_material,
            boundary,
        }
    }
}

impl Dummy<MaterialMapFaker<'_>> for MaterialMap<'_> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &MaterialMapFaker, _: &mut R) -> Self {
        let nr_surfaces = config.boundary.surfaces.len();
        if nr_surfaces == 0 {
            Self::new()
        } else {
            let max_material_idx = config.nr_materials - 1;
            let idxf = IndexFaker::new(config.nr_materials);
            let oidxf = OptionalIndexFaker::new(max_material_idx, true);
            let mut matmap = MaterialMap::new();
            for theme in &config.themes_material {
                if config.single_material {
                    matmap.insert(
                        Cow::Owned(theme.to_string()),
                        MaterialValues {
                            value: Some(idxf.fake()),
                            values: None,
                        },
                    );
                } else {
                    let values =
                        (oidxf, nr_surfaces..=nr_surfaces).fake::<Vec<OptionalLargeIndex>>();
                    // Only the surfaces vec contains the pointers to the Materials, shells and
                    // solids are just pointers to the boundary arrays. In case of
                    // Multi/CompositeSurface the empty Vec-s are cloned, for more complex geoms
                    // Vec-s contain values. Works the same way as for the LabelIndex of Semantics.
                    let labelindex = LabelIndex {
                        points: vec![],
                        linestrings: vec![],
                        surfaces: values,
                        shells: config.boundary.shells.clone(),
                        solids: config.boundary.solids.clone(),
                    };
                    let matval = MaterialValues {
                        value: None,
                        values: Some(labelindex),
                    };
                    matmap.insert(Cow::Owned(theme.to_string()), matval);
                }
            }
            matmap
        }
    }
}

#[derive(Clone)]
struct TextureBuilder<'cm>(Texture<'cm>);

impl<'cm> From<TextureBuilder<'cm>> for Texture<'cm> {
    fn from(val: TextureBuilder<'cm>) -> Self {
        val.0
    }
}

impl<'cm> Default for TextureBuilder<'cm> {
    fn default() -> Self {
        Self::new()
            .image_type()
            .image()
            .wrap_mode()
            .texture_type()
            .border_color()
    }
}

impl<'cm> TextureBuilder<'cm> {
    fn new() -> Self {
        Self(Texture::new())
    }

    fn image_type(mut self) -> Self {
        self.0.image_type = if thread_rng().gen_bool(0.5) {
            ImageType::Jpg
        } else {
            ImageType::Png
        };
        self
    }

    fn image(mut self) -> Self {
        let fp: PathBuf = FilePath(EN).fake();
        match &self.0.image_type {
            ImageType::Png => {
                if let Some(pstr) = fp.with_extension("png").to_str() {
                    self.0.image = Cow::from(pstr.to_string());
                }
            }
            ImageType::Jpg => {
                if let Some(pstr) = fp.with_extension("jpg").to_str() {
                    self.0.image = Cow::from(pstr.to_string());
                }
            }
        }
        self
    }

    fn wrap_mode(mut self) -> Self {
        self.0.wrap_mode = Some(WrapModeFaker.fake());
        self
    }

    fn texture_type(mut self) -> Self {
        self.0.texture_type = Some(TextureTypeFaker.fake());
        self
    }

    fn border_color(mut self) -> Self {
        self.0.border_color = Some(RgbaFaker.fake());
        self
    }

    /// Builds a Texture with new values set for the members that are configured in the builder.
    fn build(self) -> Texture<'cm> {
        let mut tb = self.image_type();
        tb = tb.image();
        if tb.0.wrap_mode.is_some() {
            tb = tb.wrap_mode();
        }
        if tb.0.texture_type.is_some() {
            tb = tb.texture_type();
        }
        if tb.0.border_color.is_some() {
            tb = tb.border_color();
        }
        tb.into()
    }
}

struct WrapModeFaker;

impl Dummy<WrapModeFaker> for WrapMode {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &WrapModeFaker, rng: &mut R) -> Self {
        match rng.gen_range(0..5) {
            0 => WrapMode::Wrap,
            1 => WrapMode::Mirror,
            2 => WrapMode::Clamp,
            3 => WrapMode::Border,
            4 => WrapMode::None,
            _ => {
                unreachable!()
            }
        }
    }
}

struct TextureTypeFaker;

impl Dummy<TextureTypeFaker> for TextureType {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &TextureTypeFaker, rng: &mut R) -> Self {
        match rng.gen_range(0..3) {
            0 => TextureType::Unknown,
            1 => TextureType::Typical,
            2 => TextureType::Specific,
            _ => {
                unreachable!()
            }
        }
    }
}

type UVCoordinate = [f32; 2];

struct UVCoordinateFaker;
impl Dummy<UVCoordinateFaker> for UVCoordinate {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &UVCoordinateFaker, rng: &mut R) -> Self {
        let uv_range = 0.0..=1.0;
        [
            rng.gen_range(uv_range.clone()),
            rng.gen_range(uv_range.clone()),
        ]
    }
}

/// Fake the textures for Multi/CompositeSurface, Solid, Multi/CompositeSolid geometries.
struct TextureMapFaker<'texmapfaker> {
    nr_textures: IndexType,
    nr_vertices_texture: IndexType,
    themes_texture: Vec<String>,
    boundary: &'texmapfaker Boundary,
    allow_none: bool,
}

impl<'texmapfaker> TextureMapFaker<'texmapfaker> {
    fn new(
        nr_textures: IndexType,
        nr_vertices_texture: IndexType,
        themes_texture: Vec<String>,
        boundary: &'texmapfaker Boundary,
        allow_none: bool,
    ) -> Self {
        Self {
            nr_textures,
            nr_vertices_texture,
            themes_texture,
            boundary,
            allow_none,
        }
    }
}

impl Dummy<TextureMapFaker<'_>> for TextureMap<'_> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &TextureMapFaker, _: &mut R) -> Self {
        let nr_vertices = config.boundary.vertices.len();
        let nr_rings = config.boundary.rings.len();
        let nr_surfaces = config.boundary.surfaces.len();
        if nr_surfaces == 0 {
            Self::new()
        } else {
            let tex_idx_faker = OptionalIndexFaker::new(config.nr_textures - 1, false);
            let uv_idx_faker =
                OptionalIndexFaker::new(config.nr_vertices_texture - 1, config.allow_none);
            let mut texmap = TextureMap::new();
            for theme in &config.themes_texture {
                let tex_indices =
                    (tex_idx_faker, nr_rings..=nr_rings).fake::<Vec<OptionalLargeIndex>>();
                let uv_coord_indices =
                    (uv_idx_faker, nr_vertices..=nr_vertices).fake::<Vec<OptionalLargeIndex>>();
                let textureindex = TextureIndex {
                    vertices: uv_coord_indices,
                    rings: config.boundary.rings.clone(),
                    rings_textures: tex_indices,
                    surfaces: config.boundary.surfaces.clone(),
                    shells: config.boundary.shells.clone(),
                    solids: config.boundary.solids.clone(),
                };
                let texval = TextureValues {
                    values: Some(textureindex),
                };
                texmap.insert(Cow::Owned(theme.to_string()), texval);
            }
            texmap
        }
    }
}

#[derive(Clone)]
struct MetadataBuilder<'cm>(Metadata<'cm>);

impl<'cm> From<MetadataBuilder<'cm>> for Metadata<'cm> {
    fn from(val: MetadataBuilder<'cm>) -> Self {
        val.0
    }
}

impl<'cm> Default for MetadataBuilder<'cm> {
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

    fn title(mut self) -> Self {
        let words: Vec<String> = Words(EN, 0..6).fake();
        self.0.set_title(words.join(" "));
        self
    }

    fn build(self) -> Metadata<'cm> {
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

struct AttributesFaker {
    random_keys: bool,
    random_values: bool,
}

/// Generate owned attributes.
impl<'cm> Dummy<AttributesFaker> for Attributes<'cm> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &AttributesFaker, _: &mut R) -> Self {
        let mut attributes_map = serde_json::Map::new();
        let mut key_null = "null".to_string();
        let mut key_bool = "bool".to_string();
        let mut key_number_int = "number_int".to_string();
        let mut key_number_float = "number_float".to_string();
        let mut key_string = "string".to_string();
        let mut key_array_null = "array_null".to_string();
        let mut key_array_bool = "array_bool".to_string();
        let mut key_array_number = "array_number".to_string();
        let mut key_array_string = "array_string".to_string();
        let mut key_object = "object".to_string();
        if config.random_keys {
            key_null = Word(EN).fake();
            key_bool = Word(EN).fake();
            key_number_int = Word(EN).fake();
            key_number_float = Word(EN).fake();
            key_string = Word(EN).fake();
            key_array_null = Word(EN).fake();
            key_array_bool = Word(EN).fake();
            key_array_number = Word(EN).fake();
            key_array_string = Word(EN).fake();
            key_object = Word(EN).fake();
        }
        let value_null = serde_json::Value::Null;
        let mut value_bool = serde_json::Value::Bool(true);
        let mut value_number_int = serde_json::Value::from(42_i64);
        let mut value_number_float = serde_json::Value::from(42_f64);
        let mut value_string = serde_json::Value::String("äáßüóíéöűőú".into());
        let value_array_null =
            serde_json::Value::Array(vec![serde_json::Value::Null, serde_json::Value::Null]);
        let value_array_bool = serde_json::Value::Array(vec![
            serde_json::Value::Bool(true),
            serde_json::Value::Bool(false),
        ]);
        let mut value_array_number = serde_json::Value::Array(vec![
            serde_json::Value::from(42_i64),
            serde_json::Value::from(42_f64),
        ]);
        let mut value_array_string = serde_json::Value::Array(vec![
            serde_json::Value::String("".into()),
            serde_json::Value::String("äáßüóíéöűőú".into()),
        ]);
        if config.random_values {
            value_bool = serde_json::Value::Bool(Faker.fake());
            value_number_int = serde_json::Value::from(Faker.fake::<i64>());
            value_number_float = serde_json::Value::from(Faker.fake::<f64>());
            value_string = serde_json::Value::String(Faker.fake());
            let af64: Vec<f64> = (F64Faker, 3..5).fake();
            value_array_number = serde_json::Value::Array(
                af64.into_iter()
                    .map(serde_json::Value::from)
                    .collect::<Vec<_>>(),
            );
            let astring: Vec<String> = (Word(EN), 3..5).fake();
            value_array_string = serde_json::Value::Array(
                astring
                    .into_iter()
                    .map(serde_json::Value::from)
                    .collect::<Vec<_>>(),
            );
        }

        attributes_map.insert(key_null, value_null);
        attributes_map.insert(key_bool, value_bool);
        attributes_map.insert(key_number_int, value_number_int);
        attributes_map.insert(key_number_float, value_number_float);
        attributes_map.insert(key_string, value_string);
        attributes_map.insert(key_array_null, value_array_null);
        attributes_map.insert(key_array_bool, value_array_bool);
        attributes_map.insert(key_array_number, value_array_number);
        attributes_map.insert(key_array_string, value_array_string);

        let value_object = serde_json::Value::from(attributes_map.clone());
        attributes_map.insert(key_object, value_object);

        Attributes::Owned(serde_json::Value::from(attributes_map))
    }
}

struct F64Faker;
impl Dummy<F64Faker> for f64 {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &F64Faker, rng: &mut R) -> Self {
        rng.gen()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cjval::CJValidator;
    use std::env;

    fn invalids_dir() -> PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("invalids")
    }

    fn count_invalids(invalids_dir: &PathBuf) -> usize {
        let mut count: usize = 0;
        for entry in std::fs::read_dir(invalids_dir).unwrap() {
            if entry.is_ok() {
                let p = entry.unwrap().path();
                if p.extension().is_some_and(|ext| ext == "json") {
                    let name = p.file_name().unwrap();
                    let d = name
                        .to_string_lossy()
                        .replace("cjfake_invalid_", "")
                        .replace(".city.json", "");
                    let c = d.parse::<usize>().unwrap();
                    if c > count {
                        count = c;
                    }
                }
            }
        }
        count
    }

    #[test]
    fn attributes() {
        let attributes: Attributes = AttributesFaker {
            random_keys: false,
            random_values: true,
        }
        .fake();
        dbg!(attributes);
    }

    #[test]
    fn test_fake() {
        let a: [f64; 16] = (1.0..5.0).fake();
        dbg!(a);
    }

    #[test]
    fn geometry() {
        let geom: Geometry = GeometryFaker::new(
            12,
            CityObjectType::Building,
            None,
            None,
            None,
            Some(vec![GeometryType::CompositeSurface]),
            &None,
            false,
        )
        .fake();
        // dbg!(&geom);
        let g = serde_json::to_string(&geom).unwrap();
        dbg!(g);
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
        let cm: CityModel = CityModelBuilder::default().into();
        let cj_str = serde_json::to_string::<CityModel>(&cm).unwrap();
        println!("{}", &cj_str);
        let val = CJValidator::from_str(&cj_str);
        // assert!(val.validate().iter().all(|(c, s)| s.is_valid()));
        let invalids: Vec<(String, String)> = val
            .validate()
            .into_iter()
            .filter(|(_, summary)| !summary.is_valid())
            .map(|(criterion, summary)| (criterion, summary.to_string()))
            .collect();
        if invalids.len() > 0 {
            // Serialize invalid citymodels for later analysis
            let idir = invalids_dir();
            let invalids_count = count_invalids(&idir);
            let current_invalid_nr = invalids_count + 1;
            let fname = format!("cjfake_invalid_{}.city.json", current_invalid_nr);
            std::fs::write(idir.join(fname), cj_str).unwrap();
        }
        for (criterion, summary) in val.validate().iter() {
            assert!(
                summary.is_valid(),
                "{} is not valid with {}",
                criterion,
                summary
            )
        }
    }
}
