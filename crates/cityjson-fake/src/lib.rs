//! # cjfake
//!
//! ## Modules
//!
//! - [`attribute`] for `Attributes` builders and fakers
//! - [`citymodel`] for assembling complete models
//! - [`cli`] for command-line configuration and execution
//! - [`material`], [`metadata`], [`texture`], and [`vertex`] for lower-level generators
//!
//! Generate fake [CityJSON](https://www.cityjson.org/) data for testing.
//!
//! `cjfake` is useful when you need schema-valid `CityJSON` documents without depending on large
//! real-world datasets. It can generate complete models, or you can use the lower-level builders
//! to generate individual pieces such as metadata, materials, textures, attributes, or vertices.
//!
//! ## Quick Start
//!
//! ```rust
//! use cjfake::prelude::*;
//!
//! let model: CityModel<u32, OwnedStringStorage> = CityModelBuilder::default().build();
//! assert_eq!(model.cityobjects().len(), 1);
//! ```
//!
//! ```rust
//! use cjfake::prelude::*;
//!
//! let json = cjfake::generate_string(CJFakeConfig::default(), Some(42)).unwrap();
//! assert!(json.starts_with('{'));
//! ```
//!
//! ## What It Can Generate
//!
//! `cjfake` focuses on the `CityJSON` v2.0 generation surface that is most useful for automated
//! testing:
//!
//! - city objects and parent-child hierarchy
//! - geometry types, `LoDs`, templates, and geometry counts
//! - vertices within configurable coordinate ranges
//! - metadata, materials, textures, attributes, and semantics
//!
//! The generated documents are valid according to the `CityJSON` schema, but the geometry itself is
//! random and not intended to represent realistic city models.
//!
//! ## Public API
//!
//! The easiest entry points are:
//!
//! - [`generate_model`] for a ready-to-use `CityModel`
//! - [`generate_string`] for a serialized `CityJSON` string
//! - [`generate_vec`] for UTF-8 encoded `CityJSON` bytes
//! - [`CityModelBuilder`](citymodel::CityModelBuilder) for fine-grained control
//!
//! ## Example Config
//!
//! ```rust
//! use cjfake::prelude::*;
//!
//! let config = CJFakeConfig {
//!     cityobjects: CityObjectConfig {
//!         allowed_types_cityobject: Some(vec![CityObjectType::Building, CityObjectType::Bridge]),
//!         min_cityobjects: 5,
//!         max_cityobjects: 10,
//!         cityobject_hierarchy: true,
//!         ..Default::default()
//!     },
//!     vertices: VertexConfig {
//!         min_coordinate: -10.0,
//!         max_coordinate: 10.0,
//!         ..Default::default()
//!     },
//!     ..Default::default()
//! };
//! ```
//!
//! ## Builders
//!
//! - [`MetadataBuilder`](metadata::MetadataBuilder)
//! - [`MaterialBuilder`](material::MaterialBuilder)
//! - [`TextureBuilder`](texture::TextureBuilder)
//! - [`AttributesBuilder`](attribute::AttributesBuilder)
//!
//! ## Limitations
//!
//! - Geometry is schema-valid, but not guaranteed to be a meaningful real-world shape
//! - Semantic relationships are generated for validation, not semantic correctness
//! - Texture image paths are synthetic file paths, not actual files
//!
pub mod attribute;
pub mod citymodel;
pub mod cli;
pub mod material;
pub mod metadata;
pub mod texture;
pub mod vertex;

use cityjson::prelude::*;
use cityjson::v2_0::{CityObjectType, GeometryType, LoD, SemanticType};
use fake::Dummy;
use rand::seq::IndexedRandom;
use rand::Rng;
use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::str::FromStr;

/// Convenient re-exports for common `cjfake` use cases.
///
/// ```rust
/// use cjfake::prelude::*;
///
/// let config = CJFakeConfig::default();
/// let model = generate_model(config, Some(9));
/// assert_eq!(model.cityobjects().len(), 1);
/// ```
pub mod prelude {
    pub use cityjson::prelude::*;
    pub use cityjson::v2_0::*;

    pub use crate::attribute::AttributesBuilder;
    pub use crate::citymodel::CityModelBuilder;
    pub use crate::material::MaterialBuilder;
    pub use crate::metadata::MetadataBuilder;
    pub use crate::texture::TextureBuilder;
    pub use crate::vertex::VerticesFaker;

    pub use crate::cli::{
        AttributeConfig, CJFakeConfig, CityObjectConfig, GeometryConfig, MaterialConfig,
        MetadataConfig, SemanticConfig, TemplateConfig, TextureConfig, VertexConfig,
    };

    pub use crate::generate_model;
    #[cfg(feature = "serialize")]
    pub use crate::{generate_string, generate_vec};
}

// TODO: use Coordinate instead of array (also implement in cjlib/cityjson serialization)
// todo scj: need to use the proper coordinate type and add to CoordinateFaker
// TODO: exe/docker/server
// TODO: create a CityObjectIDFaker to generate IDs with mixed characters, not only letters
// TODO: CityObject add "address" to the type where possible
// todo: CityObject add extra
// TODO: use real EPSG codes, to get existing CRS URIs. Text file contents can be included with https://doc.rust-lang.org/std/macro.include_str.html
// todo: CityObjectTypeFaker add Extension for v2.0
// todo: CityObjectTypeFaker add Extension for v2.0
// todo scj: GeometryIndices::with_capacity should be initialized with the type that GeometryIndex holds, because it doesn't make sense for GeometryIndices to hold more items than max GeometryIndex
// todo: MultiPoint, lod 3, Building --> semantics don't make sense
// todo: scj: geometry.template_boundaries needs to be [GeometryIndex; 1] instead of [usize; 1];
// todo: if templates builder is used, make sure that at least one GeometryInstance is generated

pub(crate) const CRS_AUTHORITIES: [&str; 2] = ["EPSG", "OGC"];
pub(crate) const CRS_OGC_VERSIONS: [&str; 3] = ["0", "1.0", "1.3"];
pub(crate) const CRS_OGC_CODES: [&str; 4] = ["CRS1", "CRS27", "CRS83", "CRS84"];
pub(crate) const CRS_EPSG_VERSIONS: [&str; 5] = ["0", "1", "2", "3", "4"];

type IndexType = u32;

/// Generate a `CityModel` using the default `u32` vertex references and owned string storage.
///
/// # Examples
///
/// ```rust
/// use cjfake::cli::CJFakeConfig;
/// use cjfake::generate_model;
///
/// let model = generate_model(CJFakeConfig::default(), Some(11));
/// assert_eq!(model.cityobjects().len(), 1);
/// ```
#[must_use]
pub fn generate_model(
    config: cli::CJFakeConfig,
    seed: Option<u64>,
) -> cityjson::v2_0::CityModel<u32, OwnedStringStorage> {
    citymodel::CityModelBuilder::<u32, OwnedStringStorage>::new(config, seed)
        .metadata(None)
        .vertices()
        .materials(None)
        .textures(None)
        .attributes(None)
        .cityobjects()
        .build()
}

/// Generate a `CityJSON` string using the default `u32` vertex references and owned string storage.
///
/// # Examples
///
/// ```rust
/// use cjfake::cli::CJFakeConfig;
/// use cjfake::generate_string;
///
/// let json = generate_string(CJFakeConfig::default(), Some(12)).unwrap();
/// assert!(json.starts_with('{'));
/// ```
///
/// # Errors
///
/// Returns any serialization error from `cjlib`.
#[cfg(feature = "serialize")]
pub fn generate_string(config: cli::CJFakeConfig, seed: Option<u64>) -> cjlib::Result<String>
where
    u32: serde::Serialize,
{
    citymodel::CityModelBuilder::<u32, OwnedStringStorage>::new(config, seed)
        .metadata(None)
        .vertices()
        .materials(None)
        .textures(None)
        .attributes(None)
        .cityobjects()
        .build_string()
}

/// Generate UTF-8 encoded `CityJSON` bytes using the default `u32` vertex references and owned string storage.
///
/// # Examples
///
/// ```rust
/// use cjfake::cli::CJFakeConfig;
/// use cjfake::generate_vec;
///
/// let bytes = generate_vec(CJFakeConfig::default(), Some(13)).unwrap();
/// assert!(bytes.starts_with(b"{"));
/// ```
///
/// # Errors
///
/// Returns any serialization error from `cjlib`.
#[cfg(feature = "serialize")]
pub fn generate_vec(config: cli::CJFakeConfig, seed: Option<u64>) -> cjlib::Result<Vec<u8>>
where
    u32: serde::Serialize,
{
    citymodel::CityModelBuilder::<u32, OwnedStringStorage>::new(config, seed)
        .metadata(None)
        .vertices()
        .materials(None)
        .textures(None)
        .attributes(None)
        .cityobjects()
        .build_vec()
}

#[allow(dead_code)]
type CityObjectGeometryTypes = HashMap<CityObjectType<OwnedStringStorage>, Vec<GeometryType>>;

#[allow(dead_code)]
static CITYJSON_GEOMETRY_TYPES_BYTES: &[u8] = include_bytes!("data/cityjson_geometry_types.json");

#[allow(dead_code)]
static CITYJSON_GEOMETRY_TYPES: std::sync::LazyLock<CityObjectGeometryTypes> =
    std::sync::LazyLock::new(|| {
        let raw_data: HashMap<String, Vec<String>> =
            serde_json::from_slice(CITYJSON_GEOMETRY_TYPES_BYTES)
                .expect("Failed to deserialize cityjson_geometry_types.json");

        raw_data
            .into_iter()
            .map(|(co_type_str, geom_type_str)| {
                let co_type = CityObjectType::from_str(co_type_str.as_str()).unwrap();

                let geom_types: Vec<GeometryType> = geom_type_str
                    .into_iter()
                    .map(|v| GeometryType::from_str(v.as_str()).unwrap())
                    .collect();

                (co_type, geom_types)
            })
            .collect()
    });

#[allow(dead_code)]
type CityObjectsWithSemantics = Vec<CityObjectType<OwnedStringStorage>>;

#[allow(dead_code)]
static CITYOBJECTS_WITH_SEMANTICS_BYTES: &[u8] =
    include_bytes!("data/cityjson_semantics_allowed.json");

#[allow(dead_code)]
static CITYOBJECTS_WITH_SEMANTICS: std::sync::LazyLock<CityObjectsWithSemantics> =
    std::sync::LazyLock::new(|| {
        let raw_data: Vec<String> = serde_json::from_slice(CITYOBJECTS_WITH_SEMANTICS_BYTES)
            .expect("Failed to deserialize cityjson_semantics_allowed.json");

        raw_data
            .into_iter()
            .map(|v| CityObjectType::from_str(v.as_str()).unwrap())
            .collect()
    });

// Determine exactly how many items should we generate from a given range.
pub(crate) fn get_nr_items<R: Rng + ?Sized>(
    range: RangeInclusive<IndexType>,
    rng: &mut R,
) -> usize {
    if range.is_empty() {
        0
    } else if range.end() - range.start() == 0 {
        // e.g. MIN=2 MAX=2 should generate exactly 2 items
        *range.end() as usize
    } else {
        rng.random_range(range) as usize
    }
}

/// Indicates the hierarchy level of a `CityObject`: First (top-level) or Second (child).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum CityObjectLevel {
    #[default]
    First,
    #[allow(dead_code)]
    Second,
    #[allow(dead_code)]
    Any,
}

/// Faker for `CityObjectType` that respects hierarchy level and allowed types config.
pub(crate) struct CityObjectTypeFaker {
    pub(crate) cityobject_level: CityObjectLevel,
}

impl Dummy<CityObjectTypeFaker> for CityObjectType<OwnedStringStorage> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &CityObjectTypeFaker, rng: &mut R) -> Self {
        // Range of valid type indices for each hierarchy level
        let type_idx: u8 = match config.cityobject_level {
            CityObjectLevel::First => rng.random_range(0..16),
            CityObjectLevel::Second => rng.random_range(16..33),
            CityObjectLevel::Any => rng.random_range(0..33),
        };
        match type_idx {
            0 => CityObjectType::Bridge,
            1 => CityObjectType::Building,
            2 => CityObjectType::CityFurniture,
            3 => CityObjectType::CityObjectGroup,
            4 => CityObjectType::GenericCityObject,
            5 => CityObjectType::LandUse,
            6 => CityObjectType::OtherConstruction,
            7 => CityObjectType::PlantCover,
            8 => CityObjectType::SolitaryVegetationObject,
            9 => CityObjectType::TINRelief,
            10 => CityObjectType::TransportSquare,
            11 => CityObjectType::Railway,
            12 => CityObjectType::Road,
            13 => CityObjectType::Tunnel,
            14 => CityObjectType::WaterBody,
            15 => CityObjectType::Waterway,
            16 => CityObjectType::BridgePart,
            17 => CityObjectType::BridgeInstallation,
            18 => CityObjectType::BridgeConstructiveElement,
            19 => CityObjectType::BridgeRoom,
            20 => CityObjectType::BridgeFurniture,
            21 => CityObjectType::BuildingPart,
            22 => CityObjectType::BuildingInstallation,
            23 => CityObjectType::BuildingConstructiveElement,
            24 => CityObjectType::BuildingFurniture,
            25 => CityObjectType::BuildingStorey,
            26 => CityObjectType::BuildingRoom,
            27 => CityObjectType::BuildingUnit,
            28 => CityObjectType::TunnelPart,
            29 => CityObjectType::TunnelInstallation,
            30 => CityObjectType::TunnelConstructiveElement,
            31 => CityObjectType::TunnelHollowSpace,
            32 => CityObjectType::TunnelFurniture,
            _ => unreachable!(),
        }
    }
}

/// Maps parent `CityObject` types to their valid child types.
pub(crate) fn get_cityobject_subtype(
    cityobject_type: &CityObjectType<OwnedStringStorage>,
) -> Option<Vec<CityObjectType<OwnedStringStorage>>> {
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

const ALL_LODS: [LoD; 20] = [
    LoD::LoD0,
    LoD::LoD0_0,
    LoD::LoD0_1,
    LoD::LoD0_2,
    LoD::LoD0_3,
    LoD::LoD1,
    LoD::LoD1_0,
    LoD::LoD1_1,
    LoD::LoD1_2,
    LoD::LoD1_3,
    LoD::LoD2,
    LoD::LoD2_0,
    LoD::LoD2_1,
    LoD::LoD2_2,
    LoD::LoD2_3,
    LoD::LoD3,
    LoD::LoD3_0,
    LoD::LoD3_1,
    LoD::LoD3_2,
    LoD::LoD3_3,
];

/// Faker for random `LoD` (Level of Detail) selection.
///
/// Pass `allowed` to restrict to a specific set of `LoD` values.
#[derive(Default)]
pub(crate) struct LoDFaker<'a> {
    pub(crate) allowed: Option<&'a [LoD]>,
}

impl Dummy<LoDFaker<'_>> for LoD {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &LoDFaker<'_>, rng: &mut R) -> Self {
        use rand::seq::IndexedRandom;
        let pool = config.allowed.unwrap_or(&ALL_LODS);
        pool.choose(rng).copied().unwrap_or(LoD::LoD2)
    }
}

/// Context for semantic type generation, derived from `SemanticConfig`.
pub(crate) struct SemanticCtx<'a> {
    pub(crate) enabled: bool,
    pub(crate) allowed_types: Option<&'a [SemanticType<OwnedStringStorage>]>,
}

/// Faker for valid `SemanticType` based on `CityObjectType`
pub(crate) struct SemanticTypeFaker<'a> {
    pub(crate) city_obj_type: CityObjectType<OwnedStringStorage>,
    /// When `Some`, only pick types from this list (intersected with valid types for the CO type).
    pub(crate) allowed_types: Option<&'a [SemanticType<OwnedStringStorage>]>,
}

impl Dummy<SemanticTypeFaker<'_>> for Option<SemanticType<OwnedStringStorage>> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &SemanticTypeFaker<'_>, rng: &mut R) -> Self {
        // Only certain CityObjectTypes support semantics
        let semantic_types: Vec<SemanticType<OwnedStringStorage>> = match config.city_obj_type {
            // Building types support wall, roof, door, window, floor, etc.
            CityObjectType::Building
            | CityObjectType::BuildingPart
            | CityObjectType::BuildingStorey
            | CityObjectType::BuildingRoom
            | CityObjectType::BuildingUnit => vec![
                SemanticType::WallSurface,
                SemanticType::RoofSurface,
                SemanticType::GroundSurface,
                SemanticType::ClosureSurface,
                SemanticType::OuterCeilingSurface,
                SemanticType::OuterFloorSurface,
                SemanticType::Door,
                SemanticType::Window,
                SemanticType::InteriorWallSurface,
                SemanticType::CeilingSurface,
                SemanticType::FloorSurface,
            ],
            // Water bodies support water surfaces
            CityObjectType::WaterBody => vec![
                SemanticType::WaterSurface,
                SemanticType::WaterGroundSurface,
                SemanticType::WaterClosureSurface,
            ],
            // Transportation types support road/rail surfaces
            CityObjectType::Road | CityObjectType::Railway | CityObjectType::TransportSquare => {
                vec![
                    SemanticType::TrafficArea,
                    SemanticType::AuxiliaryTrafficArea,
                    SemanticType::TransportationMarking,
                    SemanticType::TransportationHole,
                ]
            }
            // Other types: no semantics
            _ => return None,
        };

        // If allowed_types is set, intersect with valid types for this CityObjectType
        let filtered: Vec<SemanticType<OwnedStringStorage>> =
            if let Some(allowed) = config.allowed_types {
                semantic_types
                    .into_iter()
                    .filter(|st| allowed.contains(st))
                    .collect()
            } else {
                semantic_types
            };

        filtered.choose(rng).cloned()
    }
}
