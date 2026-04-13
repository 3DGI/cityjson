//! Command-line configuration for `cityjson-fake`.
//!
//! ```rust
//! use cityjson_fake::cli::{CJFakeConfig, Cli};
//!
//! let config = CJFakeConfig::default();
//! let cli = Cli {
//!     config,
//!     manifest: None,
//!     schema: None,
//!     case: None,
//!     check_manifest: false,
//!     output: None,
//!     count: 1,
//! };
//! assert_eq!(cli.count, 1);
//! ```

use cityjson::prelude::OwnedStringStorage;
use cityjson::v2_0::{CityObjectType, GeometryType, LoD, SemanticType};
use clap::{Args, Parser};
#[cfg(feature = "serialize")]
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
type IndexType = u32;

// ─── Sub-configs ─────────────────────────────────────────────────────────────

/// Configuration for `CityObject` generation.
#[cfg_attr(feature = "serialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default))]
#[derive(Args, Debug, Clone)]
pub struct CityObjectConfig {
    /// Restrict the `CityObject` types to the provided types
    #[arg(long, value_delimiter = ',', value_parser = parse_cityobject_type)]
    #[cfg_attr(
        feature = "serialize",
        serde(deserialize_with = "deserialize_cityobject_types")
    )]
    pub allowed_types_cityobject: Option<Vec<CityObjectType<OwnedStringStorage>>>,

    /// Minimum number of `CityObjects` to generate
    #[arg(long, default_value_t = 1)]
    pub min_cityobjects: IndexType,

    /// Maximum number of `CityObjects` to generate
    #[arg(long, default_value_t = 1)]
    pub max_cityobjects: IndexType,

    /// Whether to generate hierarchical `CityObjects` (parent-child relationships)
    #[arg(long, default_value_t = false)]
    pub cityobject_hierarchy: bool,

    /// Minimum number of child `CityObjects` per parent (when hierarchy is enabled)
    #[arg(long, default_value_t = 1)]
    pub min_children: IndexType,

    /// Maximum number of child `CityObjects` per parent (when hierarchy is enabled)
    #[arg(long, default_value_t = 3)]
    pub max_children: IndexType,
}

impl Default for CityObjectConfig {
    fn default() -> Self {
        Self {
            allowed_types_cityobject: None,
            min_cityobjects: 1,
            max_cityobjects: 1,
            cityobject_hierarchy: false,
            min_children: 1,
            max_children: 3,
        }
    }
}

/// Configuration for geometry generation.
#[cfg_attr(feature = "serialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default))]
#[derive(Args, Debug, Clone)]
pub struct GeometryConfig {
    /// Restrict the Geometry types to the provided types
    #[arg(long, value_delimiter = ',', value_parser = parse_geometry_type)]
    #[cfg_attr(
        feature = "serialize",
        serde(deserialize_with = "deserialize_geometry_types")
    )]
    pub allowed_types_geometry: Option<Vec<GeometryType>>,

    /// Restrict the `LoD` values to the provided values
    #[arg(long, value_delimiter = ',', value_parser = parse_lod)]
    #[cfg_attr(feature = "serialize", serde(deserialize_with = "deserialize_lods"))]
    pub allowed_lods: Option<Vec<LoD>>,

    /// Minimum number of points in `MultiPoint` geometries
    #[arg(long, default_value_t = 11)]
    pub min_members_multipoint: IndexType,

    /// Maximum number of points in `MultiPoint` geometries
    #[arg(long, default_value_t = 11)]
    pub max_members_multipoint: IndexType,

    /// Minimum number of linestrings in `MultiLineString` geometries
    #[arg(long, default_value_t = 1)]
    pub min_members_multilinestring: IndexType,

    /// Maximum number of linestrings in `MultiLineString` geometries
    #[arg(long, default_value_t = 1)]
    pub max_members_multilinestring: IndexType,

    /// Minimum number of surfaces in `MultiSurface` geometries
    #[arg(long, default_value_t = 1)]
    pub min_members_multisurface: IndexType,

    /// Maximum number of surfaces in `MultiSurface` geometries
    #[arg(long, default_value_t = 1)]
    pub max_members_multisurface: IndexType,

    /// Minimum number of shells in Solid geometries
    #[arg(long, default_value_t = 1)]
    pub min_members_solid: IndexType,

    /// Maximum number of shells in Solid geometries
    #[arg(long, default_value_t = 3)]
    pub max_members_solid: IndexType,

    /// Minimum number of solids in `MultiSolid` geometries
    #[arg(long, default_value_t = 1)]
    pub min_members_multisolid: IndexType,

    /// Maximum number of solids in `MultiSolid` geometries
    #[arg(long, default_value_t = 3)]
    pub max_members_multisolid: IndexType,

    /// Minimum number of surfaces in `CompositeSurface` geometries
    #[arg(long, default_value_t = 1)]
    pub min_members_compositesurface: IndexType,

    /// Maximum number of surfaces in `CompositeSurface` geometries
    #[arg(long, default_value_t = 3)]
    pub max_members_compositesurface: IndexType,

    /// Minimum number of solids in `CompositeSolid` geometries
    #[arg(long, default_value_t = 1)]
    pub min_members_compositesolid: IndexType,

    /// Maximum number of solids in `CompositeSolid` geometries
    #[arg(long, default_value_t = 3)]
    pub max_members_compositesolid: IndexType,

    /// Minimum number of geometries per `CityObject`
    #[arg(long, default_value_t = 1)]
    pub min_members_cityobject_geometries: IndexType,

    /// Maximum number of geometries per `CityObject`
    #[arg(long, default_value_t = 1)]
    pub max_members_cityobject_geometries: IndexType,
}

impl Default for GeometryConfig {
    fn default() -> Self {
        Self {
            allowed_types_geometry: None,
            allowed_lods: None,
            min_members_multipoint: 11,
            max_members_multipoint: 11,
            min_members_multilinestring: 1,
            max_members_multilinestring: 1,
            min_members_multisurface: 1,
            max_members_multisurface: 1,
            min_members_solid: 1,
            max_members_solid: 3,
            min_members_multisolid: 1,
            max_members_multisolid: 3,
            min_members_compositesurface: 1,
            max_members_compositesurface: 3,
            min_members_compositesolid: 1,
            max_members_compositesolid: 3,
            min_members_cityobject_geometries: 1,
            max_members_cityobject_geometries: 1,
        }
    }
}

/// Configuration for generated vertices.
#[cfg_attr(feature = "serialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default))]
#[derive(Args, Debug, Clone)]
pub struct VertexConfig {
    /// Minimum coordinate value for geometry vertices
    #[arg(long, default_value_t = -1000.0)]
    pub min_coordinate: f64,

    /// Maximum coordinate value for geometry vertices
    #[arg(long, default_value_t = 1000.0)]
    pub max_coordinate: f64,

    /// Minimum number of vertices in geometry objects
    #[arg(long, default_value_t = 8)]
    pub min_vertices: IndexType,

    /// Maximum number of vertices in geometry objects
    #[arg(long, default_value_t = 8)]
    pub max_vertices: IndexType,
}

impl Default for VertexConfig {
    fn default() -> Self {
        Self {
            min_coordinate: -1000.0,
            max_coordinate: 1000.0,
            min_vertices: 8,
            max_vertices: 8,
        }
    }
}

/// Configuration for material generation.
#[cfg_attr(feature = "serialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default))]
#[derive(Args, Debug, Clone)]
pub struct MaterialConfig {
    /// Whether to generate materials (default: true)
    #[arg(long, default_value_t = true)]
    pub materials_enabled: bool,

    /// Minimum number of materials
    #[arg(long, default_value_t = 1)]
    pub min_materials: IndexType,

    /// Maximum number of materials
    #[arg(long, default_value_t = 3)]
    pub max_materials: IndexType,

    /// Number of material themes
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(IndexType).range(1..))]
    pub nr_themes_materials: IndexType,

    /// Whether to generate ambient intensity (None=random, true=always, false=never)
    #[arg(long)]
    pub generate_ambient_intensity: Option<bool>,

    /// Whether to generate diffuse color (None=random, true=always, false=never)
    #[arg(long)]
    pub generate_diffuse_color: Option<bool>,

    /// Whether to generate emissive color (None=random, true=always, false=never)
    #[arg(long)]
    pub generate_emissive_color: Option<bool>,

    /// Whether to generate specular color (None=random, true=always, false=never)
    #[arg(long)]
    pub generate_specular_color: Option<bool>,

    /// Whether to generate shininess (None=random, true=always, false=never)
    #[arg(long)]
    pub generate_shininess: Option<bool>,

    /// Whether to generate transparency (None=random, true=always, false=never)
    #[arg(long)]
    pub generate_transparency: Option<bool>,
}

impl Default for MaterialConfig {
    fn default() -> Self {
        Self {
            materials_enabled: true,
            min_materials: 1,
            max_materials: 3,
            nr_themes_materials: 3,
            generate_ambient_intensity: None,
            generate_diffuse_color: None,
            generate_emissive_color: None,
            generate_specular_color: None,
            generate_shininess: None,
            generate_transparency: None,
        }
    }
}

/// Configuration for texture generation.
#[cfg_attr(feature = "serialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default))]
#[derive(Args, Debug, Clone)]
pub struct TextureConfig {
    /// Whether to generate textures (default: true)
    #[arg(long, default_value_t = true)]
    pub textures_enabled: bool,

    /// Minimum number of textures
    #[arg(long, default_value_t = 2)]
    pub min_textures: IndexType,

    /// Maximum number of textures
    #[arg(long, default_value_t = 2)]
    pub max_textures: IndexType,

    /// Number of texture themes
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(IndexType).range(1..))]
    pub nr_themes_textures: IndexType,

    /// Maximum number of vertices in texture coordinates
    #[arg(long, default_value_t = 10)]
    pub max_vertices_texture: IndexType,

    /// Allow null in the texture values
    #[arg(long, default_value_t = false)]
    pub texture_allow_none: bool,
}

impl Default for TextureConfig {
    fn default() -> Self {
        Self {
            textures_enabled: true,
            min_textures: 2,
            max_textures: 2,
            nr_themes_textures: 3,
            max_vertices_texture: 10,
            texture_allow_none: false,
        }
    }
}

/// Configuration for template generation.
#[cfg_attr(feature = "serialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default))]
#[derive(Args, Debug, Clone)]
pub struct TemplateConfig {
    /// Generate `GeometryInstances` (templates)
    #[arg(long, default_value_t = false)]
    pub use_templates: bool,

    /// Minimum number of templates
    #[arg(long, default_value_t = 1)]
    pub min_templates: IndexType,

    /// Maximum number of templates
    #[arg(long, default_value_t = 10)]
    pub max_templates: IndexType,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            use_templates: false,
            min_templates: 1,
            max_templates: 10,
        }
    }
}

/// Configuration for metadata generation.
#[allow(clippy::struct_excessive_bools)]
#[cfg_attr(feature = "serialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default))]
#[derive(Args, Debug, Clone)]
pub struct MetadataConfig {
    /// Whether to generate metadata (default: true)
    #[arg(long, default_value_t = true)]
    pub metadata_enabled: bool,

    /// Whether to generate geographical extent in metadata
    #[arg(long, default_value_t = true)]
    pub metadata_geographical_extent: bool,

    /// Whether to generate identifier in metadata
    #[arg(long, default_value_t = true)]
    pub metadata_identifier: bool,

    /// Whether to generate reference date in metadata
    #[arg(long, default_value_t = true)]
    pub metadata_reference_date: bool,

    /// Whether to generate reference system in metadata
    #[arg(long, default_value_t = true)]
    pub metadata_reference_system: bool,

    /// Whether to generate title in metadata
    #[arg(long, default_value_t = true)]
    pub metadata_title: bool,

    /// Whether to generate point of contact in metadata
    #[arg(long, default_value_t = true)]
    pub metadata_point_of_contact: bool,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            metadata_enabled: true,
            metadata_geographical_extent: true,
            metadata_identifier: true,
            metadata_reference_date: true,
            metadata_reference_system: true,
            metadata_title: true,
            metadata_point_of_contact: true,
        }
    }
}

/// Configuration for attribute generation.
#[cfg_attr(feature = "serialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default))]
#[derive(Args, Debug, Clone)]
pub struct AttributeConfig {
    /// Whether to generate attributes (default: true)
    #[arg(long, default_value_t = true)]
    pub attributes_enabled: bool,

    /// Minimum number of attributes per `CityObject`
    #[arg(long, default_value_t = 3)]
    pub min_attributes: IndexType,

    /// Maximum number of attributes per `CityObject`
    #[arg(long, default_value_t = 8)]
    pub max_attributes: IndexType,

    /// Maximum nesting depth for attribute objects
    #[arg(long, default_value_t = 2)]
    pub attributes_max_depth: u8,

    /// Whether to generate random attribute keys
    #[arg(long, default_value_t = true)]
    pub attributes_random_keys: bool,

    /// Whether to generate random attribute values
    #[arg(long, default_value_t = true)]
    pub attributes_random_values: bool,
}

impl Default for AttributeConfig {
    fn default() -> Self {
        Self {
            attributes_enabled: true,
            min_attributes: 3,
            max_attributes: 8,
            attributes_max_depth: 2,
            attributes_random_keys: true,
            attributes_random_values: true,
        }
    }
}

/// Configuration for semantic generation.
#[cfg_attr(feature = "serialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default))]
#[derive(Args, Debug, Clone)]
pub struct SemanticConfig {
    /// Whether to generate semantics (default: true)
    #[arg(long, default_value_t = true)]
    pub semantics_enabled: bool,

    /// Restrict semantic types to the provided types
    #[arg(long, value_delimiter = ',', value_parser = parse_semantic_type)]
    #[cfg_attr(
        feature = "serialize",
        serde(deserialize_with = "deserialize_semantic_types")
    )]
    pub allowed_types_semantic: Option<Vec<SemanticType<OwnedStringStorage>>>,
}

impl Default for SemanticConfig {
    fn default() -> Self {
        Self {
            semantics_enabled: true,
            allowed_types_semantic: None,
        }
    }
}

/// Top-level configuration for `CityJSON` fake data generation.
#[cfg_attr(feature = "serialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default))]
#[derive(Args, Debug, Clone, Default)]
pub struct CJFakeConfig {
    /// Random seed for deterministic output
    #[arg(long)]
    pub seed: Option<u64>,

    #[clap(flatten)]
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub cityobjects: CityObjectConfig,

    #[clap(flatten)]
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub geometry: GeometryConfig,

    #[clap(flatten)]
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub vertices: VertexConfig,

    #[clap(flatten)]
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub materials: MaterialConfig,

    #[clap(flatten)]
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub textures: TextureConfig,

    #[clap(flatten)]
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub templates: TemplateConfig,

    #[clap(flatten)]
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub metadata: MetadataConfig,

    #[clap(flatten)]
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub attributes: AttributeConfig,

    #[clap(flatten)]
    #[cfg_attr(feature = "serialize", serde(flatten))]
    pub semantics: SemanticConfig,
}

/// Command-line interface for generating `CityJSON`.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub struct Cli {
    #[command(flatten)]
    pub config: CJFakeConfig,

    /// Optional manifest file that defines one or more generation cases.
    ///
    /// When present, the manifest supplies the output configuration and the
    /// regular flag-based config is ignored.
    #[arg(long)]
    pub manifest: Option<PathBuf>,

    /// Optional JSON Schema file used to validate `--manifest`.
    ///
    /// When omitted, `cityjson-fake` uses its bundled `cityjson-fake-manifest.schema.json`.
    #[arg(long)]
    pub schema: Option<PathBuf>,

    /// Optional case id to select from a manifest.
    ///
    /// If omitted, all manifest cases are generated.
    #[arg(long)]
    pub case: Option<String>,

    /// Validate the manifest and exit without generating output.
    #[arg(long)]
    pub check_manifest: bool,

    /// Optional output path.
    ///
    /// If `--count 1` is used, this is treated as a file path.
    /// If `--count > 1`, this is treated as a directory and multiple files are written there.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Number of `CityJSON` documents to generate.
    #[arg(long, default_value_t = 1)]
    pub count: usize,
}

// ─── Parsers ──────────────────────────────────────────────────────────────────

fn parse_cityobject_type(s: &str) -> Result<CityObjectType<OwnedStringStorage>, String> {
    CityObjectType::from_str(s).map_err(|e| format!("Failed to parse CityObjectType: {e}"))
}

fn parse_geometry_type(s: &str) -> Result<GeometryType, String> {
    GeometryType::from_str(s).map_err(|e| format!("Failed to parse GeometryType: {e}"))
}

fn parse_lod(s: &str) -> Result<LoD, String> {
    match s {
        "0" => Ok(LoD::LoD0),
        "0.0" => Ok(LoD::LoD0_0),
        "0.1" => Ok(LoD::LoD0_1),
        "0.2" => Ok(LoD::LoD0_2),
        "0.3" => Ok(LoD::LoD0_3),
        "1" => Ok(LoD::LoD1),
        "1.0" => Ok(LoD::LoD1_0),
        "1.1" => Ok(LoD::LoD1_1),
        "1.2" => Ok(LoD::LoD1_2),
        "1.3" => Ok(LoD::LoD1_3),
        "2" => Ok(LoD::LoD2),
        "2.0" => Ok(LoD::LoD2_0),
        "2.1" => Ok(LoD::LoD2_1),
        "2.2" => Ok(LoD::LoD2_2),
        "2.3" => Ok(LoD::LoD2_3),
        "3" => Ok(LoD::LoD3),
        "3.0" => Ok(LoD::LoD3_0),
        "3.1" => Ok(LoD::LoD3_1),
        "3.2" => Ok(LoD::LoD3_2),
        "3.3" => Ok(LoD::LoD3_3),
        _ => Err(format!(
            "Unknown LoD: {s}. Valid values: 0, 0.0–0.3, 1, 1.0–1.3, 2, 2.0–2.3, 3, 3.0–3.3"
        )),
    }
}

fn parse_semantic_type(s: &str) -> Result<SemanticType<OwnedStringStorage>, String> {
    match s {
        "RoofSurface" => Ok(SemanticType::RoofSurface),
        "GroundSurface" => Ok(SemanticType::GroundSurface),
        "WallSurface" => Ok(SemanticType::WallSurface),
        "ClosureSurface" => Ok(SemanticType::ClosureSurface),
        "OuterCeilingSurface" => Ok(SemanticType::OuterCeilingSurface),
        "OuterFloorSurface" => Ok(SemanticType::OuterFloorSurface),
        "Window" => Ok(SemanticType::Window),
        "Door" => Ok(SemanticType::Door),
        "InteriorWallSurface" => Ok(SemanticType::InteriorWallSurface),
        "CeilingSurface" => Ok(SemanticType::CeilingSurface),
        "FloorSurface" => Ok(SemanticType::FloorSurface),
        "WaterSurface" => Ok(SemanticType::WaterSurface),
        "WaterGroundSurface" => Ok(SemanticType::WaterGroundSurface),
        "WaterClosureSurface" => Ok(SemanticType::WaterClosureSurface),
        "TrafficArea" => Ok(SemanticType::TrafficArea),
        "AuxiliaryTrafficArea" => Ok(SemanticType::AuxiliaryTrafficArea),
        "TransportationMarking" => Ok(SemanticType::TransportationMarking),
        "TransportationHole" => Ok(SemanticType::TransportationHole),
        _ => Err(format!("Unknown SemanticType: {s}")),
    }
}

#[cfg(feature = "serialize")]
fn deserialize_cityobject_types<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<CityObjectType<OwnedStringStorage>>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = Option::<Vec<String>>::deserialize(deserializer)?;
    values
        .map(|values| {
            values
                .into_iter()
                .map(|value| parse_cityobject_type(&value).map_err(serde::de::Error::custom))
                .collect()
        })
        .transpose()
}

#[cfg(feature = "serialize")]
fn deserialize_geometry_types<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<GeometryType>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = Option::<Vec<String>>::deserialize(deserializer)?;
    values
        .map(|values| {
            values
                .into_iter()
                .map(|value| parse_geometry_type(&value).map_err(serde::de::Error::custom))
                .collect()
        })
        .transpose()
}

#[cfg(feature = "serialize")]
fn deserialize_lods<'de, D>(deserializer: D) -> Result<Option<Vec<LoD>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = Option::<Vec<String>>::deserialize(deserializer)?;
    values
        .map(|values| {
            values
                .into_iter()
                .map(|value| parse_lod(&value).map_err(serde::de::Error::custom))
                .collect()
        })
        .transpose()
}

#[cfg(feature = "serialize")]
fn deserialize_semantic_types<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<SemanticType<OwnedStringStorage>>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = Option::<Vec<String>>::deserialize(deserializer)?;
    values
        .map(|values| {
            values
                .into_iter()
                .map(|value| parse_semantic_type(&value).map_err(serde::de::Error::custom))
                .collect()
        })
        .transpose()
}

// ─── CLI runner ──────────────────────────────────────────────────────────────

fn resolve_manifest_output_path(
    output_dir: Option<&Path>,
    case_output: Option<&Path>,
    manifest_dir: Option<&Path>,
    case_id: &str,
) -> PathBuf {
    if let Some(case_output) = case_output {
        if case_output.is_absolute() {
            return case_output.to_path_buf();
        }

        if let Some(output_dir) = output_dir {
            return output_dir.join(case_output);
        }

        if let Some(manifest_dir) = manifest_dir {
            return manifest_dir.join(case_output);
        }

        return case_output.to_path_buf();
    }

    if let Some(output_dir) = output_dir {
        return output_dir.join(format!("{case_id}.city.json"));
    }

    PathBuf::from(format!("{case_id}.city.json"))
}

#[cfg(feature = "serialize")]
/// Run the CLI and write the generated `CityJSON` to stdout or files.
///
/// # Errors
///
/// Returns an error if generation fails, output paths cannot be created, or writing fails.
pub fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Write};

    if let Some(manifest_path) = cli.manifest {
        if cli.check_manifest {
            match cli.schema.as_deref() {
                Some(schema_path) => {
                    crate::manifest::validate_manifest(&manifest_path, schema_path)?;
                }
                None => crate::manifest::validate_manifest_with_bundled_schema(&manifest_path)?,
            }
            return Ok(());
        }
        let manifest = match cli.schema.as_deref() {
            Some(schema_path) => {
                crate::manifest::load_manifest_validated(&manifest_path, schema_path)?
            }
            None => crate::manifest::load_manifest_validated_with_bundled_schema(&manifest_path)?,
        };
        let manifest_dir = manifest_path.parent();
        let mut cases: Vec<&crate::manifest::GenerationCase> = if let Some(case_id) = cli.case {
            vec![manifest
                .case(&case_id)
                .ok_or_else(|| format!("case '{case_id}' not found in manifest"))?]
        } else {
            manifest.cases().collect()
        };

        if cases.is_empty() {
            return Err("manifest does not contain any cases".into());
        }

        if cases.len() > 1 && cli.output.is_none() {
            return Err("multiple manifest cases require --output <DIR>".into());
        }

        if let Some(output) = cli.output {
            if cases.len() == 1 {
                let case = cases
                    .pop()
                    .ok_or_else(|| io::Error::other("single manifest case should exist"))?;
                let json = crate::generate_string(case.config.clone(), case.seed)?;
                let resolved = output;
                if let Some(parent) = resolved.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(resolved, json)?;
            } else {
                fs::create_dir_all(&output)?;
                for case in cases {
                    let json = crate::generate_string(case.config.clone(), case.seed)?;
                    let resolved = resolve_manifest_output_path(
                        Some(output.as_path()),
                        case.output.as_deref(),
                        manifest_dir,
                        &case.id,
                    );
                    if let Some(parent) = resolved.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(resolved, json)?;
                }
            }
            return Ok(());
        }

        if cases.len() != 1 {
            return Err("a manifest with multiple cases requires --output <DIR>".into());
        }

        let case = cases
            .pop()
            .ok_or_else(|| io::Error::other("single manifest case should exist"))?;
        let json = crate::generate_string(case.config.clone(), case.seed)?;
        let mut stdout = io::stdout().lock();
        stdout.write_all(json.as_bytes())?;
        stdout.write_all(b"\n")?;
        stdout.flush()?;
        return Ok(());
    }

    let seed = cli.config.seed;
    let config = cli.config;

    if cli.count > 1 && cli.output.is_none() {
        return Err("multiple documents require --output <DIR>".into());
    }

    if let Some(output) = cli.output {
        if cli.count == 1 {
            let json = crate::generate_string(config, seed)?;
            fs::write(output, json)?;
        } else {
            fs::create_dir_all(&output)?;
            for idx in 0..cli.count {
                let json = crate::generate_string(config.clone(), seed)?;
                let file_name = format!("cityjson-fake-{idx:04}.city.json");
                fs::write(output.join(file_name), json)?;
            }
        }
        return Ok(());
    }

    let json = crate::generate_string(config, seed)?;
    let mut stdout = io::stdout().lock();
    stdout.write_all(json.as_bytes())?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

#[cfg(not(feature = "serialize"))]
/// Run the CLI when serialization support is unavailable.
///
/// # Errors
///
/// Always returns an error because the `serialize` feature is required.
pub fn run(_cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    Err("serialize feature required for the CLI".into())
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_cli_defaults() {
        let cli = Cli::parse_from(["cityjson-fake"]);
        let config = cli.config;

        assert_eq!(config.seed, None);
        assert_eq!(config.cityobjects.allowed_types_cityobject, None);
        assert_eq!(config.geometry.allowed_types_geometry, None);
        assert_eq!(config.cityobjects.min_cityobjects, 1);
        assert_eq!(config.cityobjects.max_cityobjects, 1);
        assert!(!config.cityobjects.cityobject_hierarchy);
        assert_eq!(config.vertices.min_coordinate, -1000.0f64);
        assert_eq!(config.vertices.max_coordinate, 1000.0f64);
        assert_eq!(config.vertices.min_vertices, 8);
        assert_eq!(config.vertices.max_vertices, 8);
        assert_eq!(config.geometry.min_members_multipoint, 11);
        assert_eq!(config.geometry.max_members_multipoint, 11);
        assert_eq!(config.geometry.min_members_multilinestring, 1);
        assert_eq!(config.geometry.max_members_multilinestring, 1);
        assert_eq!(config.geometry.min_members_multisurface, 1);
        assert_eq!(config.geometry.max_members_multisurface, 1);
        assert_eq!(config.geometry.min_members_solid, 1);
        assert_eq!(config.geometry.max_members_solid, 3);
        assert_eq!(config.geometry.min_members_multisolid, 1);
        assert_eq!(config.geometry.max_members_multisolid, 3);
        assert_eq!(config.geometry.min_members_cityobject_geometries, 1);
        assert_eq!(config.geometry.max_members_cityobject_geometries, 1);
        assert_eq!(config.materials.min_materials, 1);
        assert_eq!(config.materials.max_materials, 3);
        assert_eq!(config.materials.nr_themes_materials, 3);
        assert_eq!(config.textures.min_textures, 2);
        assert_eq!(config.textures.max_textures, 2);
        assert_eq!(config.textures.nr_themes_textures, 3);
        assert_eq!(config.textures.max_vertices_texture, 10);
        assert_eq!(config.templates.min_templates, 1);
        assert_eq!(config.templates.max_templates, 10);
        assert!(!config.templates.use_templates);
        assert!(!config.textures.texture_allow_none);
        assert!(cli.output.is_none());
        assert_eq!(cli.count, 1);
        assert!(cli.manifest.is_none());
        assert!(cli.schema.is_none());
        assert!(cli.case.is_none());
        assert!(!cli.check_manifest);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::float_cmp)]
    fn test_cli_argument_parsing() {
        let args = vec![
            "cityjson-fake",
            "--allowed-types-cityobject",
            "Building,Bridge",
            "--allowed-types-geometry",
            "MultiSurface,Solid",
            "--min-cityobjects",
            "5",
            "--max-cityobjects",
            "10",
            "--cityobject-hierarchy",
            "--min-coordinate=-1000",
            "--max-coordinate=1000",
            "--min-vertices",
            "4",
            "--max-vertices",
            "20",
            "--min-members-multipoint",
            "2",
            "--max-members-multipoint",
            "5",
            "--min-members-multilinestring",
            "3",
            "--max-members-multilinestring",
            "6",
            "--min-members-multisurface",
            "1",
            "--max-members-multisurface",
            "3",
            "--min-members-solid",
            "2",
            "--max-members-solid",
            "4",
            "--min-members-multisolid",
            "1",
            "--max-members-multisolid",
            "2",
            "--min-members-cityobject-geometries",
            "1",
            "--max-members-cityobject-geometries",
            "3",
            "--min-materials",
            "1",
            "--max-materials",
            "2",
            "--nr-themes-materials",
            "2",
            "--min-textures",
            "1",
            "--max-textures",
            "3",
            "--nr-themes-textures",
            "2",
            "--max-vertices-texture",
            "15",
            "--min-templates",
            "2",
            "--max-templates",
            "5",
            "--use-templates",
            "--texture-allow-none",
            "--output",
            "output.city.json",
            "--count",
            "3",
        ];

        let cli = Cli::parse_from(args);
        let config = cli.config;

        assert_eq!(
            config.cityobjects.allowed_types_cityobject,
            Some(vec![CityObjectType::Building, CityObjectType::Bridge])
        );
        assert_eq!(
            config.geometry.allowed_types_geometry,
            Some(vec![GeometryType::MultiSurface, GeometryType::Solid])
        );
        assert_eq!(config.cityobjects.min_cityobjects, 5);
        assert_eq!(config.cityobjects.max_cityobjects, 10);
        assert!(config.cityobjects.cityobject_hierarchy);
        assert_eq!(config.vertices.min_coordinate, -1000.0f64);
        assert_eq!(config.vertices.max_coordinate, 1000.0f64);
        assert_eq!(config.vertices.min_vertices, 4);
        assert_eq!(config.vertices.max_vertices, 20);
        assert_eq!(config.geometry.min_members_multipoint, 2);
        assert_eq!(config.geometry.max_members_multipoint, 5);
        assert_eq!(config.geometry.min_members_multilinestring, 3);
        assert_eq!(config.geometry.max_members_multilinestring, 6);
        assert_eq!(config.geometry.min_members_multisurface, 1);
        assert_eq!(config.geometry.max_members_multisurface, 3);
        assert_eq!(config.geometry.min_members_solid, 2);
        assert_eq!(config.geometry.max_members_solid, 4);
        assert_eq!(config.geometry.min_members_multisolid, 1);
        assert_eq!(config.geometry.max_members_multisolid, 2);
        assert_eq!(config.geometry.min_members_cityobject_geometries, 1);
        assert_eq!(config.geometry.max_members_cityobject_geometries, 3);
        assert_eq!(config.materials.min_materials, 1);
        assert_eq!(config.materials.max_materials, 2);
        assert_eq!(config.materials.nr_themes_materials, 2);
        assert_eq!(config.textures.min_textures, 1);
        assert_eq!(config.textures.max_textures, 3);
        assert_eq!(config.textures.nr_themes_textures, 2);
        assert_eq!(config.textures.max_vertices_texture, 15);
        assert_eq!(config.templates.min_templates, 2);
        assert_eq!(config.templates.max_templates, 5);
        assert!(config.templates.use_templates);
        assert!(config.textures.texture_allow_none);
        assert_eq!(cli.output, Some(PathBuf::from("output.city.json")));
        assert_eq!(cli.count, 3);
        assert!(cli.manifest.is_none());
        assert!(cli.schema.is_none());
        assert!(cli.case.is_none());
        assert!(!cli.check_manifest);
    }

    #[test]
    fn test_cli_run_writes_file() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cityjson-fake-cli-{stamp}"));
        fs::create_dir_all(&dir).expect("temp dir should be creatable");
        let output = dir.join("model.city.json");

        let cli = Cli {
            config: CJFakeConfig::default(),
            manifest: None,
            schema: None,
            case: None,
            check_manifest: false,
            output: Some(output.clone()),
            count: 1,
        };

        run(cli).expect("CLI should succeed");
        let json = fs::read_to_string(&output).expect("output should exist");
        assert!(json.starts_with('{'));
    }

    #[test]
    fn test_manifest_loads_case_config() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cityjson-fake-manifest-{stamp}"));
        fs::create_dir_all(&dir).expect("temp dir should be creatable");
        let manifest_path = dir.join("manifest.json");

        let manifest = r#"
        {
          "version": 1,
          "purpose": "test",
          "cases": [
            {
              "id": "spec_complete_omnibus",
              "seed": 42
            }
          ]
        }
        "#;
        fs::write(&manifest_path, manifest).expect("manifest should be writable");

        let loaded = crate::manifest::load_manifest(&manifest_path).expect("manifest should load");
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.purpose.as_deref(), Some("test"));
        let case = loaded
            .case("spec_complete_omnibus")
            .expect("case should exist");
        assert_eq!(case.id, "spec_complete_omnibus");
        assert_eq!(case.seed, Some(42));
        assert_eq!(case.config.cityobjects.min_cityobjects, 1);
        assert_eq!(case.config.geometry.max_members_solid, 3);
    }

    #[test]
    fn test_cli_run_writes_manifest_case() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cityjson-fake-manifest-run-{stamp}"));
        let output = dir.join("spec_complete_omnibus.city.json");
        fs::create_dir_all(&dir).expect("temp dir should be creatable");
        let manifest_path = dir.join("manifest.json");
        let manifest = r#"
        {
          "version": 1,
          "cases": [
            {
              "id": "spec_complete_omnibus",
              "seed": 11,
              "min_cityobjects": 2,
              "max_cityobjects": 2
            }
          ]
        }
        "#;
        fs::write(&manifest_path, manifest).expect("manifest should be writable");

        let cli = Cli {
            config: CJFakeConfig::default(),
            manifest: Some(manifest_path),
            schema: None,
            case: None,
            check_manifest: false,
            output: Some(output.clone()),
            count: 1,
        };

        run(cli).expect("manifest-driven CLI should succeed");
        let generated = fs::read_to_string(output).expect("manifest output should exist");
        assert!(generated.starts_with('{'));
    }

    #[test]
    fn test_cli_run_writes_manifest_directory() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cityjson-fake-manifest-dir-{stamp}"));
        let output = dir.join("out");
        let schema_path = dir.join("schema.json");
        fs::create_dir_all(&dir).expect("temp dir should be creatable");
        let manifest_path = dir.join("manifest.json");
        let schema = r#"
        {
          "$schema": "https://json-schema.org/draft/2020-12/schema",
          "type": "object",
          "additionalProperties": false,
          "required": ["version", "cases"],
          "properties": {
            "version": { "type": "integer", "const": 1 },
            "cases": {
              "type": "array",
              "minItems": 1,
              "items": {
                "type": "object",
                "additionalProperties": true,
                "required": ["id"],
                "properties": {
                  "id": { "type": "string", "minLength": 1 },
                  "seed": { "type": ["integer", "null"] }
                }
              }
            }
          }
        }
        "#;
        fs::write(&schema_path, schema).expect("schema should be writable");
        let manifest = r#"
        {
          "version": 1,
          "cases": [
            {
              "id": "spec_complete_omnibus",
              "seed": 11
            },
            {
              "id": "geometry_matrix",
              "seed": 17
            }
          ]
        }
        "#;
        fs::write(&manifest_path, manifest).expect("manifest should be writable");

        let cli = Cli {
            config: CJFakeConfig::default(),
            manifest: Some(manifest_path),
            schema: Some(schema_path),
            case: None,
            check_manifest: false,
            output: Some(output.clone()),
            count: 1,
        };

        run(cli).expect("manifest-driven CLI should succeed");
        let generated_one = fs::read_to_string(output.join("spec_complete_omnibus.city.json"))
            .expect("first manifest output should exist");
        let generated_two = fs::read_to_string(output.join("geometry_matrix.city.json"))
            .expect("second manifest output should exist");
        assert!(generated_one.starts_with('{'));
        assert!(generated_two.starts_with('{'));
    }
}
