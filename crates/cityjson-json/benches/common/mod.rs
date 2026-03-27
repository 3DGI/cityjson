#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use cityjson::v2_0::{GeometryType, LoD};
use cjfake::cli::{
    AttributeConfig, CJFakeConfig, CityObjectConfig, GeometryConfig, MaterialConfig,
    MetadataConfig, SemanticConfig, TemplateConfig, TextureConfig, VertexConfig,
};
use cjfake::generate_model;
use cjfake::generate_string;
use serde::Serialize;
use serde_json::Value;

use serde_cityjson::{from_str_owned, to_string, OwnedCityModel};

const REAL_DATA_DIR: &str = "tests/data/downloaded";

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum CaseSource {
    Real { filename: &'static str },
    Synthetic { config: CJFakeConfig, seed: u64 },
}

#[derive(Clone)]
pub(crate) struct CaseSpec {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) borrowed: bool,
    pub(crate) source: CaseSource,
}

pub(crate) struct PreparedCase {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) borrowed: bool,
    pub(crate) input_json: String,
    pub(crate) model: OwnedCityModel,
    pub(crate) value: Value,
    pub(crate) input_bytes: u64,
    pub(crate) output_bytes: u64,
}

#[derive(Serialize)]
struct SuiteMetadata<'a> {
    suite: &'a str,
    cases: Vec<CaseMetadata<'a>>,
}

#[derive(Serialize)]
struct CaseMetadata<'a> {
    id: &'a str,
    description: &'a str,
    borrowed: bool,
    input_bytes: u64,
    output_bytes: u64,
}

impl CaseSpec {
    pub(crate) fn prepare(&self) -> PreparedCase {
        let input_json = match &self.source {
            CaseSource::Real { filename } => read_file(filename),
            CaseSource::Synthetic { config, seed } => {
                generate_string(config.clone(), Some(*seed)).unwrap()
            }
        };
        let model = match &self.source {
            CaseSource::Real { .. } => from_str_owned(&input_json).unwrap(),
            CaseSource::Synthetic { config, seed } => generate_model(config.clone(), Some(*seed)),
        };
        let value = serde_json::from_str(&input_json).unwrap();
        let output_bytes = to_string(&model).unwrap().len() as u64;

        PreparedCase {
            name: self.name,
            description: self.description,
            borrowed: self.borrowed,
            input_bytes: input_json.len() as u64,
            output_bytes,
            input_json,
            model,
            value,
        }
    }
}

pub(crate) fn write_suite_metadata(suite: &str, prepared: &[PreparedCase]) {
    let metadata = SuiteMetadata {
        suite,
        cases: prepared
            .iter()
            .map(|case| CaseMetadata {
                id: case.name,
                description: case.description,
                borrowed: case.borrowed,
                input_bytes: case.input_bytes,
                output_bytes: case.output_bytes,
            })
            .collect(),
    };
    let output = serde_json::to_string_pretty(&metadata).unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("results")
        .join(format!("suite_metadata_{suite}.json"));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, output).unwrap();
}

pub(crate) fn read_cases() -> Vec<CaseSpec> {
    let mut cases = vec![real_3dbag(), real_3dbasis()];
    cases.extend(synthetic_read_cases());
    cases
}

pub(crate) fn write_cases() -> Vec<CaseSpec> {
    let mut cases = vec![real_3dbag(), real_3dbasis()];
    cases.extend(synthetic_write_cases());
    cases
}

pub(crate) fn real_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("downloaded")
}

fn read_file(filename: &str) -> String {
    fs::read_to_string(real_data_dir().join(filename)).unwrap()
}

fn base_config() -> CJFakeConfig {
    CJFakeConfig {
        cityobjects: CityObjectConfig::default(),
        geometry: GeometryConfig::default(),
        vertices: VertexConfig::default(),
        materials: MaterialConfig::default(),
        textures: TextureConfig::default(),
        templates: TemplateConfig::default(),
        metadata: MetadataConfig::default(),
        attributes: AttributeConfig::default(),
        semantics: SemanticConfig::default(),
        seed: None,
    }
}

fn disable_optional_surfaces(mut config: CJFakeConfig) -> CJFakeConfig {
    config.materials.materials_enabled = false;
    config.textures.textures_enabled = false;
    config.templates.use_templates = false;
    config.metadata.metadata_enabled = false;
    config.attributes.attributes_enabled = false;
    config.semantics.semantics_enabled = false;
    config
}

fn synthetic_read_cases() -> Vec<CaseSpec> {
    vec![
        CaseSpec {
            name: "geometry_flattening_best_case",
            description: "Large MultiSurface payload with no relation graph or attribute tree",
            borrowed: true,
            source: CaseSource::Synthetic {
                config: geometry_flattening_best_case(),
                seed: 11,
            },
        },
        CaseSpec {
            name: "vertex_transform_stress",
            description: "Large vertex pool with very little object-level normalization",
            borrowed: true,
            source: CaseSource::Synthetic {
                config: vertex_transform_stress(),
                seed: 19,
            },
        },
        CaseSpec {
            name: "attribute_tree_worst_case",
            description: "Deep nested attributes with minimal geometry work",
            borrowed: true,
            source: CaseSource::Synthetic {
                config: attribute_tree_worst_case(),
                seed: 23,
            },
        },
        CaseSpec {
            name: "relation_graph_worst_case",
            description: "Dense parent-child graph with small geometry payloads",
            borrowed: true,
            source: CaseSource::Synthetic {
                config: relation_graph_worst_case(),
                seed: 29,
            },
        },
        CaseSpec {
            name: "deep_boundary_stress",
            description: "Solid-heavy geometry that exercises nested boundary flattening",
            borrowed: true,
            source: CaseSource::Synthetic {
                config: deep_boundary_stress(),
                seed: 31,
            },
        },
        CaseSpec {
            name: "composite_value_favorable_worst_case",
            description: "Mixed geometry and normalization workload that is smaller but denser",
            borrowed: true,
            source: CaseSource::Synthetic {
                config: composite_value_favorable_worst_case(),
                seed: 37,
            },
        },
    ]
}

fn synthetic_write_cases() -> Vec<CaseSpec> {
    let mut cases = synthetic_read_cases();
    cases.push(CaseSpec {
        name: "appearance_and_validation_stress",
        description: "Serializer-heavy case with materials, textures, templates, and semantics",
        borrowed: true,
        source: CaseSource::Synthetic {
            config: appearance_and_validation_stress(),
            seed: 41,
        },
    });
    cases
}

fn real_3dbag() -> CaseSpec {
    CaseSpec {
        name: "3DBAG",
        description:
            "Real-world medium-size dataset with two geometries per object and parent-child links",
        borrowed: true,
        source: CaseSource::Real {
            filename: "10-356-724.city.json",
        },
    }
}

fn real_3dbasis() -> CaseSpec {
    CaseSpec {
        name: "3D Basisvoorziening",
        description: "Large real-world dataset dominated by geometry flattening and vertex import",
        borrowed: false,
        source: CaseSource::Real {
            filename: "30gz1_04.city.json",
        },
    }
}

fn geometry_flattening_best_case() -> CJFakeConfig {
    let mut config = disable_optional_surfaces(base_config());
    config.cityobjects.min_cityobjects = 8_000;
    config.cityobjects.max_cityobjects = 8_000;
    config.geometry.allowed_types_geometry = Some(vec![GeometryType::MultiSurface]);
    config.geometry.allowed_lods = Some(vec![LoD::LoD2_2]);
    config.geometry.min_members_cityobject_geometries = 1;
    config.geometry.max_members_cityobject_geometries = 1;
    config.geometry.min_members_multisurface = 12;
    config.geometry.max_members_multisurface = 12;
    config.vertices.min_vertices = 8;
    config.vertices.max_vertices = 8;
    config
}

fn vertex_transform_stress() -> CJFakeConfig {
    let mut config = disable_optional_surfaces(base_config());
    config.cityobjects.min_cityobjects = 2_000;
    config.cityobjects.max_cityobjects = 2_000;
    config.geometry.allowed_types_geometry = Some(vec![GeometryType::MultiSurface]);
    config.geometry.allowed_lods = Some(vec![LoD::LoD2_2]);
    config.geometry.min_members_cityobject_geometries = 1;
    config.geometry.max_members_cityobject_geometries = 1;
    config.geometry.min_members_multisurface = 1;
    config.geometry.max_members_multisurface = 1;
    config.vertices.min_vertices = 250_000;
    config.vertices.max_vertices = 250_000;
    config.vertices.min_coordinate = -5_000.0;
    config.vertices.max_coordinate = 5_000.0;
    config
}

fn attribute_tree_worst_case() -> CJFakeConfig {
    let mut config = disable_optional_surfaces(base_config());
    config.cityobjects.min_cityobjects = 6_000;
    config.cityobjects.max_cityobjects = 6_000;
    config.geometry.allowed_types_geometry = Some(vec![GeometryType::MultiSurface]);
    config.geometry.allowed_lods = Some(vec![LoD::LoD2_2]);
    config.geometry.min_members_cityobject_geometries = 1;
    config.geometry.max_members_cityobject_geometries = 1;
    config.geometry.min_members_multisurface = 1;
    config.geometry.max_members_multisurface = 1;
    config.attributes.attributes_enabled = true;
    config.attributes.min_attributes = 24;
    config.attributes.max_attributes = 24;
    config.attributes.attributes_max_depth = 4;
    config.attributes.attributes_random_keys = true;
    config.attributes.attributes_random_values = true;
    config.vertices.min_vertices = 32;
    config.vertices.max_vertices = 32;
    config
}

fn relation_graph_worst_case() -> CJFakeConfig {
    let mut config = disable_optional_surfaces(base_config());
    config.cityobjects.min_cityobjects = 6_000;
    config.cityobjects.max_cityobjects = 6_000;
    config.cityobjects.cityobject_hierarchy = true;
    config.cityobjects.min_children = 4;
    config.cityobjects.max_children = 4;
    config.geometry.allowed_types_geometry = Some(vec![GeometryType::MultiSurface]);
    config.geometry.allowed_lods = Some(vec![LoD::LoD2_2]);
    config.geometry.min_members_cityobject_geometries = 1;
    config.geometry.max_members_cityobject_geometries = 1;
    config.geometry.min_members_multisurface = 1;
    config.geometry.max_members_multisurface = 1;
    config.vertices.min_vertices = 32;
    config.vertices.max_vertices = 32;
    config
}

fn deep_boundary_stress() -> CJFakeConfig {
    let mut config = disable_optional_surfaces(base_config());
    config.cityobjects.min_cityobjects = 2_500;
    config.cityobjects.max_cityobjects = 2_500;
    config.geometry.allowed_types_geometry = Some(vec![
        GeometryType::Solid,
        GeometryType::MultiSolid,
        GeometryType::CompositeSolid,
    ]);
    config.geometry.allowed_lods = Some(vec![LoD::LoD2_2]);
    config.geometry.min_members_cityobject_geometries = 1;
    config.geometry.max_members_cityobject_geometries = 1;
    config.geometry.min_members_solid = 5;
    config.geometry.max_members_solid = 5;
    config.geometry.min_members_multisolid = 2;
    config.geometry.max_members_multisolid = 2;
    config.geometry.min_members_compositesolid = 2;
    config.geometry.max_members_compositesolid = 2;
    config.geometry.min_members_compositesurface = 4;
    config.geometry.max_members_compositesurface = 4;
    config.vertices.min_vertices = 64;
    config.vertices.max_vertices = 64;
    config
}

fn composite_value_favorable_worst_case() -> CJFakeConfig {
    let mut config = disable_optional_surfaces(base_config());
    config.cityobjects.min_cityobjects = 3_000;
    config.cityobjects.max_cityobjects = 3_000;
    config.cityobjects.cityobject_hierarchy = true;
    config.cityobjects.min_children = 2;
    config.cityobjects.max_children = 2;
    config.geometry.allowed_types_geometry = Some(vec![
        GeometryType::MultiSurface,
        GeometryType::Solid,
        GeometryType::CompositeSolid,
    ]);
    config.geometry.allowed_lods = Some(vec![LoD::LoD2_2]);
    config.geometry.min_members_cityobject_geometries = 2;
    config.geometry.max_members_cityobject_geometries = 2;
    config.geometry.min_members_multisurface = 2;
    config.geometry.max_members_multisurface = 3;
    config.geometry.min_members_solid = 2;
    config.geometry.max_members_solid = 3;
    config.geometry.min_members_compositesolid = 1;
    config.geometry.max_members_compositesolid = 2;
    config.attributes.attributes_enabled = true;
    config.attributes.min_attributes = 16;
    config.attributes.max_attributes = 16;
    config.attributes.attributes_max_depth = 3;
    config.vertices.min_vertices = 128;
    config.vertices.max_vertices = 128;
    config
}

fn appearance_and_validation_stress() -> CJFakeConfig {
    let mut config = base_config();
    config.cityobjects.min_cityobjects = 2_000;
    config.cityobjects.max_cityobjects = 2_000;
    config.geometry.allowed_types_geometry =
        Some(vec![GeometryType::MultiSurface, GeometryType::Solid]);
    config.geometry.allowed_lods = Some(vec![LoD::LoD2_2]);
    config.geometry.min_members_cityobject_geometries = 2;
    config.geometry.max_members_cityobject_geometries = 3;
    config.geometry.min_members_multisurface = 3;
    config.geometry.max_members_multisurface = 4;
    config.geometry.min_members_solid = 3;
    config.geometry.max_members_solid = 4;
    config.materials.materials_enabled = true;
    config.materials.min_materials = 2;
    config.materials.max_materials = 2;
    config.materials.nr_themes_materials = 2;
    config.textures.textures_enabled = true;
    config.textures.min_textures = 2;
    config.textures.max_textures = 2;
    config.textures.nr_themes_textures = 2;
    config.textures.max_vertices_texture = 8;
    config.templates.use_templates = true;
    config.templates.min_templates = 4;
    config.templates.max_templates = 4;
    config.metadata.metadata_enabled = true;
    config.attributes.attributes_enabled = true;
    config.attributes.min_attributes = 8;
    config.attributes.max_attributes = 8;
    config.attributes.attributes_max_depth = 2;
    config.attributes.attributes_random_keys = false;
    config.attributes.attributes_random_values = true;
    config.semantics.semantics_enabled = true;
    config.vertices.min_vertices = 128;
    config.vertices.max_vertices = 128;
    config
}
