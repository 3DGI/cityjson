#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use cityjson::prelude::OwnedStringStorage;
use cityjson::v2_0::{CityObjectType, GeometryType, LoD};
use cjfake::cli::{
    AttributeConfig, CJFakeConfig, CityObjectConfig, GeometryConfig, MaterialConfig,
    MetadataConfig, SemanticConfig, TemplateConfig, TextureConfig, VertexConfig,
};
use cjfake::{generate_model, generate_string};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use serde_cityjson::{as_json, from_str_owned, to_string, to_string_validated, OwnedCityModel};

const REAL_DATA_DIR: &str = "tests/data/downloaded";
const MANIFEST_PATH: &str = "tests/data/generated/manifest.json";

pub(crate) const READ_BENCH_SERDE_CITYJSON_OWNED: &str = "serde_cityjson/owned";
pub(crate) const READ_BENCH_SERDE_CITYJSON_BORROWED: &str = "serde_cityjson/borrowed";
pub(crate) const READ_BENCH_SERDE_JSON_VALUE: &str = "serde_json::Value";

pub(crate) const WRITE_BENCH_SERDE_CITYJSON_AS_JSON_TO_VALUE: &str =
    "serde_cityjson/as_json_to_value";
pub(crate) const WRITE_BENCH_SERDE_CITYJSON_TO_STRING: &str = "serde_cityjson/to_string";
pub(crate) const WRITE_BENCH_SERDE_CITYJSON_TO_STRING_VALIDATED: &str =
    "serde_cityjson/to_string_validated";
pub(crate) const WRITE_BENCH_SERDE_JSON_TO_STRING: &str = "serde_json::to_string";

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum CaseSource {
    Real { path: PathBuf },
    Synthetic { config: CJFakeConfig, seed: u64 },
}

#[derive(Clone)]
pub(crate) struct CaseSpec {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) borrowed: bool,
    pub(crate) source: CaseSource,
}

pub(crate) struct PreparedReadCase {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) borrowed: bool,
    pub(crate) input_json: String,
    pub(crate) input_bytes: u64,
}

pub(crate) struct PreparedWriteCase {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) borrowed: bool,
    pub(crate) model: OwnedCityModel,
    pub(crate) canonical_value: Value,
    pub(crate) benchmark_bytes: BTreeMap<String, u64>,
}

#[derive(Serialize)]
struct SuiteMetadata {
    suite: String,
    cases: Vec<CaseMetadata>,
}

#[derive(Serialize)]
struct CaseMetadata {
    id: String,
    description: String,
    borrowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    benchmark_bytes: BTreeMap<String, u64>,
}

#[derive(Deserialize)]
struct Manifest {
    cases: Vec<ManifestCase>,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum CaseKind {
    Real,
    Synthetic,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum SuiteKind {
    Read,
    Write,
}

#[derive(Deserialize)]
struct ManifestCase {
    id: String,
    kind: CaseKind,
    suites: Vec<SuiteKind>,
    borrowed: bool,
    description: String,
    #[serde(default)]
    source: Option<ManifestSource>,
    #[serde(default)]
    profile_path: Option<PathBuf>,
    #[serde(default)]
    seed: Option<u64>,
}

#[derive(Deserialize)]
struct ManifestSource {
    path: PathBuf,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct SyntheticProfile {
    cityobjects: CityObjectProfile,
    geometry: GeometryProfile,
    vertices: VertexProfile,
    materials: MaterialProfile,
    textures: TextureProfile,
    templates: TemplateGenerationProfile,
    metadata: MetadataProfile,
    attributes: AttributeProfile,
    semantics: SemanticProfile,
}

#[derive(Deserialize)]
#[serde(default)]
struct CityObjectProfile {
    allowed_types_cityobject: Option<Vec<String>>,
    min_cityobjects: u32,
    max_cityobjects: u32,
    cityobject_hierarchy: bool,
    min_children: u32,
    max_children: u32,
}

impl Default for CityObjectProfile {
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

#[derive(Deserialize)]
#[serde(default)]
struct GeometryProfile {
    allowed_types_geometry: Option<Vec<String>>,
    allowed_lods: Option<Vec<String>>,
    min_members_multipoint: u32,
    max_members_multipoint: u32,
    min_members_multilinestring: u32,
    max_members_multilinestring: u32,
    min_members_multisurface: u32,
    max_members_multisurface: u32,
    min_members_solid: u32,
    max_members_solid: u32,
    min_members_multisolid: u32,
    max_members_multisolid: u32,
    min_members_compositesurface: u32,
    max_members_compositesurface: u32,
    min_members_compositesolid: u32,
    max_members_compositesolid: u32,
    min_members_cityobject_geometries: u32,
    max_members_cityobject_geometries: u32,
}

impl Default for GeometryProfile {
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

#[derive(Deserialize)]
#[serde(default)]
struct VertexProfile {
    min_coordinate: f64,
    max_coordinate: f64,
    min_vertices: u32,
    max_vertices: u32,
}

impl Default for VertexProfile {
    fn default() -> Self {
        Self {
            min_coordinate: -1000.0,
            max_coordinate: 1000.0,
            min_vertices: 8,
            max_vertices: 8,
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
struct MaterialProfile {
    materials_enabled: bool,
    min_materials: u32,
    max_materials: u32,
    nr_themes_materials: u32,
    generate_ambient_intensity: Option<bool>,
    generate_diffuse_color: Option<bool>,
    generate_emissive_color: Option<bool>,
    generate_specular_color: Option<bool>,
    generate_shininess: Option<bool>,
    generate_transparency: Option<bool>,
}

impl Default for MaterialProfile {
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

#[derive(Deserialize)]
#[serde(default)]
struct TextureProfile {
    textures_enabled: bool,
    min_textures: u32,
    max_textures: u32,
    nr_themes_textures: u32,
    max_vertices_texture: u32,
    texture_allow_none: bool,
}

impl Default for TextureProfile {
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

#[derive(Deserialize)]
#[serde(default)]
struct TemplateGenerationProfile {
    #[serde(alias = "use_templates")]
    enabled: bool,
    #[serde(alias = "min_templates")]
    min_count: u32,
    #[serde(alias = "max_templates")]
    max_count: u32,
}

impl Default for TemplateGenerationProfile {
    fn default() -> Self {
        Self {
            enabled: false,
            min_count: 1,
            max_count: 10,
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
#[allow(clippy::struct_excessive_bools)]
struct MetadataProfile {
    metadata_enabled: bool,
    metadata_geographical_extent: bool,
    metadata_identifier: bool,
    metadata_reference_date: bool,
    metadata_reference_system: bool,
    metadata_title: bool,
    metadata_point_of_contact: bool,
}

impl Default for MetadataProfile {
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

#[derive(Deserialize)]
#[serde(default)]
struct AttributeProfile {
    attributes_enabled: bool,
    min_attributes: u32,
    max_attributes: u32,
    attributes_max_depth: u8,
    attributes_random_keys: bool,
    attributes_random_values: bool,
}

impl Default for AttributeProfile {
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

#[derive(Deserialize)]
#[serde(default)]
struct SemanticProfile {
    semantics_enabled: bool,
}

impl Default for SemanticProfile {
    fn default() -> Self {
        Self {
            semantics_enabled: true,
        }
    }
}

impl SyntheticProfile {
    fn into_config(self) -> CJFakeConfig {
        CJFakeConfig {
            cityobjects: self.cityobjects.into_config(),
            geometry: self.geometry.into_config(),
            vertices: self.vertices.into_config(),
            materials: self.materials.into_config(),
            textures: self.textures.into_config(),
            templates: self.templates.into_config(),
            metadata: self.metadata.into_config(),
            attributes: self.attributes.into_config(),
            semantics: self.semantics.into_config(),
            ..CJFakeConfig::default()
        }
    }
}

impl CityObjectProfile {
    fn into_config(self) -> CityObjectConfig {
        CityObjectConfig {
            allowed_types_cityobject: self.allowed_types_cityobject.map(|types| {
                types
                    .into_iter()
                    .map(|ty| {
                        CityObjectType::<OwnedStringStorage>::from_str(&ty).unwrap_or_else(|err| {
                            panic!("failed to parse city object type '{ty}': {err}")
                        })
                    })
                    .collect()
            }),
            min_cityobjects: self.min_cityobjects,
            max_cityobjects: self.max_cityobjects,
            cityobject_hierarchy: self.cityobject_hierarchy,
            min_children: self.min_children,
            max_children: self.max_children,
        }
    }
}

impl GeometryProfile {
    fn into_config(self) -> GeometryConfig {
        GeometryConfig {
            allowed_types_geometry: self.allowed_types_geometry.map(|types| {
                types
                    .into_iter()
                    .map(|ty| {
                        GeometryType::from_str(&ty).unwrap_or_else(|err| {
                            panic!("failed to parse geometry type '{ty}': {err}")
                        })
                    })
                    .collect()
            }),
            allowed_lods: self
                .allowed_lods
                .map(|lods| lods.into_iter().map(|lod| parse_lod(&lod)).collect()),
            min_members_multipoint: self.min_members_multipoint,
            max_members_multipoint: self.max_members_multipoint,
            min_members_multilinestring: self.min_members_multilinestring,
            max_members_multilinestring: self.max_members_multilinestring,
            min_members_multisurface: self.min_members_multisurface,
            max_members_multisurface: self.max_members_multisurface,
            min_members_solid: self.min_members_solid,
            max_members_solid: self.max_members_solid,
            min_members_multisolid: self.min_members_multisolid,
            max_members_multisolid: self.max_members_multisolid,
            min_members_compositesurface: self.min_members_compositesurface,
            max_members_compositesurface: self.max_members_compositesurface,
            min_members_compositesolid: self.min_members_compositesolid,
            max_members_compositesolid: self.max_members_compositesolid,
            min_members_cityobject_geometries: self.min_members_cityobject_geometries,
            max_members_cityobject_geometries: self.max_members_cityobject_geometries,
        }
    }
}

impl VertexProfile {
    fn into_config(self) -> VertexConfig {
        VertexConfig {
            min_coordinate: self.min_coordinate,
            max_coordinate: self.max_coordinate,
            min_vertices: self.min_vertices,
            max_vertices: self.max_vertices,
        }
    }
}

impl MaterialProfile {
    fn into_config(self) -> MaterialConfig {
        MaterialConfig {
            materials_enabled: self.materials_enabled,
            min_materials: self.min_materials,
            max_materials: self.max_materials,
            nr_themes_materials: self.nr_themes_materials,
            generate_ambient_intensity: self.generate_ambient_intensity,
            generate_diffuse_color: self.generate_diffuse_color,
            generate_emissive_color: self.generate_emissive_color,
            generate_specular_color: self.generate_specular_color,
            generate_shininess: self.generate_shininess,
            generate_transparency: self.generate_transparency,
        }
    }
}

impl TextureProfile {
    fn into_config(self) -> TextureConfig {
        TextureConfig {
            textures_enabled: self.textures_enabled,
            min_textures: self.min_textures,
            max_textures: self.max_textures,
            nr_themes_textures: self.nr_themes_textures,
            max_vertices_texture: self.max_vertices_texture,
            texture_allow_none: self.texture_allow_none,
        }
    }
}

impl TemplateGenerationProfile {
    fn into_config(self) -> TemplateConfig {
        TemplateConfig {
            use_templates: self.enabled,
            min_templates: self.min_count,
            max_templates: self.max_count,
        }
    }
}

impl MetadataProfile {
    fn into_config(self) -> MetadataConfig {
        MetadataConfig {
            metadata_enabled: self.metadata_enabled,
            metadata_geographical_extent: self.metadata_geographical_extent,
            metadata_identifier: self.metadata_identifier,
            metadata_reference_date: self.metadata_reference_date,
            metadata_reference_system: self.metadata_reference_system,
            metadata_title: self.metadata_title,
            metadata_point_of_contact: self.metadata_point_of_contact,
        }
    }
}

impl AttributeProfile {
    fn into_config(self) -> AttributeConfig {
        AttributeConfig {
            attributes_enabled: self.attributes_enabled,
            min_attributes: self.min_attributes,
            max_attributes: self.max_attributes,
            attributes_max_depth: self.attributes_max_depth,
            attributes_random_keys: self.attributes_random_keys,
            attributes_random_values: self.attributes_random_values,
        }
    }
}

impl SemanticProfile {
    fn into_config(self) -> SemanticConfig {
        SemanticConfig {
            semantics_enabled: self.semantics_enabled,
            allowed_types_semantic: None,
        }
    }
}

impl ManifestCase {
    fn into_case_spec(self) -> CaseSpec {
        let source = match self.kind {
            CaseKind::Real => {
                let real = self
                    .source
                    .unwrap_or_else(|| panic!("real case '{}' is missing a source path", self.id));
                CaseSource::Real { path: real.path }
            }
            CaseKind::Synthetic => {
                let profile_path = self.profile_path.unwrap_or_else(|| {
                    panic!("synthetic case '{}' is missing a profile path", self.id)
                });
                let seed = self
                    .seed
                    .unwrap_or_else(|| panic!("synthetic case '{}' is missing a seed", self.id));
                let profile = load_synthetic_profile(&profile_path);
                CaseSource::Synthetic {
                    config: profile.into_config(),
                    seed,
                }
            }
        };

        CaseSpec {
            name: self.id,
            description: self.description,
            borrowed: self.borrowed,
            source,
        }
    }
}

impl CaseSpec {
    pub(crate) fn prepare_read(&self) -> PreparedReadCase {
        let input_json = match &self.source {
            CaseSource::Real { path } => read_file(path),
            CaseSource::Synthetic { config, seed } => {
                generate_string(config.clone(), Some(*seed)).unwrap()
            }
        };

        PreparedReadCase {
            name: self.name.clone(),
            description: self.description.clone(),
            borrowed: self.borrowed,
            input_bytes: input_json.len() as u64,
            input_json,
        }
    }

    pub(crate) fn prepare_write(&self) -> PreparedWriteCase {
        let model = match &self.source {
            CaseSource::Real { path } => {
                let input_json = read_file(path);
                from_str_owned(&input_json).unwrap()
            }
            CaseSource::Synthetic { config, seed } => generate_model(config.clone(), Some(*seed)),
        };

        prepare_write_case(self, model)
    }
}

pub(crate) fn write_read_suite_metadata(prepared: &[PreparedReadCase]) {
    let metadata = SuiteMetadata {
        suite: "read".to_owned(),
        cases: prepared
            .iter()
            .map(|case| CaseMetadata {
                id: case.name.clone(),
                description: case.description.clone(),
                borrowed: case.borrowed,
                input_bytes: Some(case.input_bytes),
                benchmark_bytes: BTreeMap::new(),
            })
            .collect(),
    };
    write_suite_metadata("read", &metadata);
}

pub(crate) fn write_write_suite_metadata(prepared: &[PreparedWriteCase]) {
    let metadata = SuiteMetadata {
        suite: "write".to_owned(),
        cases: prepared
            .iter()
            .map(|case| CaseMetadata {
                id: case.name.clone(),
                description: case.description.clone(),
                borrowed: case.borrowed,
                input_bytes: None,
                benchmark_bytes: case.benchmark_bytes.clone(),
            })
            .collect(),
    };
    write_suite_metadata("write", &metadata);
}

fn write_suite_metadata(suite: &str, metadata: &SuiteMetadata) {
    let output = serde_json::to_string_pretty(metadata).unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("results")
        .join(format!("suite_metadata_{suite}.json"));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, output).unwrap();
}

pub(crate) fn read_cases() -> Vec<CaseSpec> {
    load_manifest()
        .cases
        .into_iter()
        .filter(|case| case.suites.contains(&SuiteKind::Read))
        .map(ManifestCase::into_case_spec)
        .collect()
}

pub(crate) fn write_cases() -> Vec<CaseSpec> {
    load_manifest()
        .cases
        .into_iter()
        .filter(|case| case.suites.contains(&SuiteKind::Write))
        .map(ManifestCase::into_case_spec)
        .collect()
}

fn load_manifest() -> Manifest {
    let path = repo_path(MANIFEST_PATH);
    let manifest = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read manifest {}: {err}", path.display()));
    serde_json::from_str(&manifest)
        .unwrap_or_else(|err| panic!("failed to parse manifest {}: {err}", path.display()))
}

fn load_synthetic_profile(path: &Path) -> SyntheticProfile {
    let path = repo_path(path);
    let profile = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read profile {}: {err}", path.display()));
    serde_json::from_str(&profile)
        .unwrap_or_else(|err| panic!("failed to parse profile {}: {err}", path.display()))
}

fn repo_path(path: impl AsRef<Path>) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path.as_ref())
}

pub(crate) fn real_data_dir() -> PathBuf {
    repo_path(REAL_DATA_DIR)
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(resolve_real_path(path)).unwrap()
}

fn resolve_real_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_path(path)
    }
}

fn parse_lod(value: &str) -> LoD {
    match value {
        "0" => LoD::LoD0,
        "0.0" => LoD::LoD0_0,
        "0.1" => LoD::LoD0_1,
        "0.2" => LoD::LoD0_2,
        "0.3" => LoD::LoD0_3,
        "1" => LoD::LoD1,
        "1.0" => LoD::LoD1_0,
        "1.1" => LoD::LoD1_1,
        "1.2" => LoD::LoD1_2,
        "1.3" => LoD::LoD1_3,
        "2" => LoD::LoD2,
        "2.0" => LoD::LoD2_0,
        "2.1" => LoD::LoD2_1,
        "2.2" => LoD::LoD2_2,
        "2.3" => LoD::LoD2_3,
        "3" => LoD::LoD3,
        "3.0" => LoD::LoD3_0,
        "3.1" => LoD::LoD3_1,
        "3.2" => LoD::LoD3_2,
        "3.3" => LoD::LoD3_3,
        other => panic!("failed to parse LoD '{other}' from synthetic profile"),
    }
}

fn prepare_write_case(case: &CaseSpec, model: OwnedCityModel) -> PreparedWriteCase {
    let canonical_value = serde_json::to_value(as_json(&model)).unwrap();
    let serde_json_output = serde_json::to_string(&canonical_value).unwrap();
    let serde_cityjson_output = to_string(&model).unwrap();
    let serde_cityjson_validated_output = to_string_validated(&model).unwrap();

    let benchmark_bytes = BTreeMap::from([
        (
            WRITE_BENCH_SERDE_CITYJSON_AS_JSON_TO_VALUE.to_owned(),
            serde_json_output.len() as u64,
        ),
        (
            WRITE_BENCH_SERDE_CITYJSON_TO_STRING.to_owned(),
            serde_cityjson_output.len() as u64,
        ),
        (
            WRITE_BENCH_SERDE_CITYJSON_TO_STRING_VALIDATED.to_owned(),
            serde_cityjson_validated_output.len() as u64,
        ),
        (
            WRITE_BENCH_SERDE_JSON_TO_STRING.to_owned(),
            serde_json_output.len() as u64,
        ),
    ]);

    PreparedWriteCase {
        name: case.name.clone(),
        description: case.description.clone(),
        borrowed: case.borrowed,
        model,
        canonical_value,
        benchmark_bytes,
    }
}

impl PreparedWriteCase {
    pub(crate) fn benchmark_bytes(&self, bench_id: &str) -> u64 {
        *self
            .benchmark_bytes
            .get(bench_id)
            .unwrap_or_else(|| panic!("missing benchmark byte count for '{bench_id}'"))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_profiles_load_for_both_suites() {
        assert_eq!(super::read_cases().len(), 8);
        assert_eq!(super::write_cases().len(), 9);
    }

    #[test]
    fn write_baseline_is_canonicalized_from_the_typed_model() {
        let case = super::CaseSpec {
            name: "synthetic".to_owned(),
            description: "synthetic".to_owned(),
            borrowed: true,
            source: super::CaseSource::Synthetic {
                config: cjfake::cli::CJFakeConfig::default(),
                seed: 7,
            },
        };
        let prepared = super::prepare_write_case(
            &case,
            cjfake::generate_model(cjfake::cli::CJFakeConfig::default(), Some(7)),
        );

        let serde_json_output = serde_json::to_string(&prepared.canonical_value).unwrap();
        let serde_cityjson_output = super::to_string(&prepared.model).unwrap();

        assert_eq!(serde_json_output, serde_cityjson_output);
        assert_eq!(
            prepared.benchmark_bytes(super::WRITE_BENCH_SERDE_JSON_TO_STRING),
            serde_json_output.len() as u64
        );
        assert_eq!(
            prepared.benchmark_bytes(super::WRITE_BENCH_SERDE_CITYJSON_TO_STRING),
            serde_cityjson_output.len() as u64
        );
    }
}
