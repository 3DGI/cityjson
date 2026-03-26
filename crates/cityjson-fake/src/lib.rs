//! # cjfake
//!
//! A library for generating fake [CityJSON](https://www.cityjson.org/) data for testing purposes.
//!
//! ## Overview
//!
//! `CityJSON` is a JSON-based encoding format for 3D city models. While there are many publicly
//! available datasets, they have limitations that make them unsuitable for automated testing:
//!
//! - Files are often large and slow to download/process
//! - Models contain irrelevant information for specific test cases
//! - Certain `CityObject` types are rare or nonexistent
//! - Advanced features like Appearances or Geometry-templates are rarely modeled
//!
//! This library allows you to generate valid `CityJSON` test data quickly and efficiently,
//! with precise control over the model contents and structure.
//!
//! ## Features
//!
//! - Generate complete `CityJSON` documents that pass validation with [cjval](https://github.com/cityjson/cjval)
//! - Control the number of vertices in surfaces (e.g. for triangulated surfaces)
//! - Support for all `CityJSON` object types and features
//! - Generate random but valid values for all properties
//! - Builder pattern for intuitive model construction
//!
//! Note: While the generated `CityJSON` is schema-valid, the geometric values are random and
//! do not represent valid real-world objects.
//!
//! ## Basic Usage
//!
//! ```rust
//! use cjfake::prelude::*;
//!
//! // Create a basic CityJSON model with defaults
//! let model: CityModel<u32, OwnedStringStorage> = CityModelBuilder::default().build();
//!
//! // Create a customized model
//! let config = CJFakeConfig::default();
//! let model: CityModel<u32, OwnedStringStorage> = CityModelBuilder::new(config, None)
//!     .metadata(None)
//!     .vertices()
//!     .materials(None)
//!     .textures(None)
//!     .attributes(None)
//!     .cityobjects()
//!     .build();
//! ```
//!
//! ## Configuration
//!
//! The [`CJFakeConfig`](prelude::CJFakeConfig) struct provides extensive control over the generated content:
//!
//! ```rust
//! use cjfake::prelude::*;
//!
//! let config = CJFakeConfig {
//!     cityobjects: CityObjectConfig {
//!         // Restrict to specific CityObject types
//!         allowed_types_cityobject: Some(vec![CityObjectType::Building, CityObjectType::Bridge]),
//!
//!         // Control number of objects
//!         min_cityobjects: 5,
//!         max_cityobjects: 10,
//!
//!         // Enable parent-child relationships
//!         cityobject_hierarchy: true,
//!
//!         ..Default::default()
//!     },
//!     vertices: VertexConfig {
//!         // Control geometry coordinate range
//!         min_coordinate: -10.0,
//!         max_coordinate: 10.0,
//!
//!         ..Default::default()
//!     },
//!
//!     ..Default::default()
//! };
//! ```
//!
//! ## Builders
//!
//! The library provides several builders for creating different components:
//!
//! - [`CityModelBuilder`](citymodel::CityModelBuilder) - Main builder for complete `CityJSON` models
//! - [`MaterialBuilder`](material::MaterialBuilder) - Creates material appearances with properties like color and shininess
//! - [`TextureBuilder`](texture::TextureBuilder) - Creates texture appearances with image mappings and wrap modes
//! - [`MetadataBuilder`](metadata::MetadataBuilder) - Creates metadata with contact info, dates, and references
//!
//! ### Material Example
//!
//! ```rust,ignore
//! use cjfake::prelude::*;
//!
//! let material = MaterialBuilder::new()
//!     .name()
//!     .diffuse_color()
//!     .shininess()
//!     .transparency()
//!     .build();
//! ```
//!
//! ### Texture Example
//!
//! ```rust,ignore
//! use cjfake::prelude::*;
//!
//! let texture = TextureBuilder::new()
//!     .image_type()
//!     .image()
//!     .wrap_mode()
//!     .border_color()
//!     .build();
//! ```
//!
//! ### Metadata Example
//!
//! ```rust,ignore
//! use cjfake::prelude::*;
//!
//! let metadata = MetadataBuilder::new()
//!     .geographical_extent()
//!     .identifier()
//!     .point_of_contact()
//!     .reference_system()
//!     .build();
//! ```
//!
//! ## Implementation Details
//!
//! The library uses the [fake](https://docs.rs/fake/) crate to generate random but realistic
//! values for properties like names, dates, and colors. Geometric values are generated within
//! configurable ranges to ensure they are valid according to the `CityJSON` specification.
//!
//! The generated `CityJSON` can be output as either JSON strings or UTF-8 encoded byte vectors,
//! making it suitable for both file-based and in-memory testing scenarios.
//!
//! ## Limitations
//!
//! - Generated geometric values are random and do not form valid 3D shapes
//! - Semantic relationships may not make real-world sense
//! - Generated file paths for textures do not point to real files
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
}

// TODO: use Coordinate instead of array (also implement in serde_cityjson)
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
