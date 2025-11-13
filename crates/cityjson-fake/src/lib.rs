//! # cjfake
//!
//! A library for generating fake [CityJSON](https://www.cityjson.org/) data for testing purposes.
//!
//! ## Overview
//!
//! CityJSON is a JSON-based encoding format for 3D city models. While there are many publicly
//! available datasets, they have limitations that make them unsuitable for automated testing:
//!
//! - Files are often large and slow to download/process
//! - Models contain irrelevant information for specific test cases
//! - Certain CityObject types are rare or nonexistent
//! - Advanced features like Appearances or Geometry-templates are rarely modeled
//!
//! This library allows you to generate valid CityJSON test data quickly and efficiently,
//! with precise control over the model contents and structure.
//!
//! ## Features
//!
//! - Generate complete CityJSON documents that pass validation with [cjval](https://github.com/cityjson/cjval)
//! - Control the number of vertices in surfaces (e.g. for triangulated surfaces)
//! - Support for all CityJSON object types and features
//! - Generate random but valid values for all properties
//! - Builder pattern for intuitive model construction
//!
//! Note: While the generated CityJSON is schema-valid, the geometric values are random and
//! do not represent valid real-world objects.
//!
//! ## Basic Usage
//!
//! ```rust
//! use cjfake::prelude::*;
//!
//! // Create a basic CityJSON model with defaults
//! let model: CityModel<u32, ResourceId32, OwnedStringStorage> = CityModelBuilder::default().build();
//!
//! // Create a customized model
//! let config = CJFakeConfig::default();
//! let model: CityModel<u32, ResourceId32, OwnedStringStorage> = CityModelBuilder::new(config, None)
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
//! The [`CJFakeConfig`] struct provides extensive control over the generated content:
//!
//! ```rust
//! use cjfake::prelude::*;
//!
//! let config = CJFakeConfig {
//!     // Restrict to specific CityObject types
//!     allowed_types_cityobject: Some(vec![CityObjectType::Building, CityObjectType::Bridge]),
//!
//!     // Control number of objects
//!     min_cityobjects: 5,
//!     max_cityobjects: 10,
//!
//!     // Enable parent-child relationships
//!     cityobject_hierarchy: true,
//!
//!     // Control geometry complexity
//!     min_vertices: 4,
//!     max_vertices: 20,
//!
//!     ..Default::default()
//! };
//! ```
//!
//! ## Builders
//!
//! The library provides several builders for creating different components:
//!
//! - [`CityModelBuilder`] - Main builder for complete CityJSON models
//! - [`MaterialBuilder`] - Creates material appearances with properties like color and shininess
//! - [`TextureBuilder`] - Creates texture appearances with image mappings and wrap modes
//! - [`MetadataBuilder`] - Creates metadata with contact info, dates, and references
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
//! configurable ranges to ensure they are valid according to the CityJSON specification.
//!
//! The generated CityJSON can be output as either JSON strings or UTF-8 encoded byte vectors,
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
mod cli;
pub mod material;
pub mod metadata;
pub mod texture;
pub mod vertex;

use cityjson::prelude::*;
use cityjson::v2_0::*;
use once_cell::sync::Lazy;
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

    pub use crate::cli::CJFakeConfig;
}

// TODO: use Coordinate instead of array (also implement in serde_cityjson)
// todo scj: need to use the proper coordinate type and add to CoordinateFaker
// TODO: exe/docker/server
// TODO: create a CityObjectIDFaker to generate IDs with mixed characters, not only letters
// TODO: CityObject add "address" to the type where possible
// todo: CityObject add extra
// TODO: use real EPSG codes, to get existing CRS URIs. Text file contents can be included with https://doc.rust-lang.org/std/macro.include_str.html
// todo: CityObjectTypeFaker add GenericCityObject for v2.0
// todo: CityObjectTypeFaker add CityObjectGroup
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
static CITYJSON_GEOMETRY_TYPES: Lazy<CityObjectGeometryTypes> = Lazy::new(|| {
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
static CITYOBJECTS_WITH_SEMANTICS: Lazy<CityObjectsWithSemantics> = Lazy::new(|| {
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
