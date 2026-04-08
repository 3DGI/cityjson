#![cfg(feature = "serialize")]

use crate::cli::CJFakeConfig;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const DEFAULT_SCHEMA_FILENAME: &str = "cjfake-manifest.schema.json";

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GenerationManifest {
    pub version: u32,
    pub purpose: Option<String>,
    pub cases: Vec<GenerationCase>,
}

impl Default for GenerationManifest {
    fn default() -> Self {
        Self {
            version: 1,
            purpose: None,
            cases: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerationCase {
    pub id: String,
    pub description: Option<String>,
    pub seed: Option<u64>,
    pub output: Option<PathBuf>,
    pub count: Option<usize>,
    #[serde(flatten)]
    pub config: CJFakeConfig,
}

impl Default for GenerationCase {
    fn default() -> Self {
        Self {
            id: String::new(),
            description: None,
            seed: None,
            output: None,
            count: None,
            config: CJFakeConfig::default(),
        }
    }
}

impl GenerationManifest {
    pub fn case(&self, id: &str) -> Option<&GenerationCase> {
        self.cases.iter().find(|case| case.id == id)
    }

    pub fn cases(&self) -> impl Iterator<Item = &GenerationCase> {
        self.cases.iter()
    }
}

pub fn default_schema_path(manifest_path: impl AsRef<Path>) -> PathBuf {
    let manifest_path = manifest_path.as_ref();
    manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(DEFAULT_SCHEMA_FILENAME)
}

pub fn validate_manifest(
    manifest_path: impl AsRef<Path>,
    schema_path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = manifest_path.as_ref();
    let schema_path = schema_path.as_ref();
    let manifest_json = fs::read_to_string(manifest_path)?;
    let schema_json = fs::read_to_string(schema_path)?;
    validate_manifest_json(manifest_path, schema_path, &manifest_json, &schema_json)
}

pub fn load_manifest(
    path: impl AsRef<Path>,
) -> Result<GenerationManifest, Box<dyn std::error::Error>> {
    let path = path.as_ref();
    let json = fs::read_to_string(path)?;
    let manifest = serde_json::from_str(&json)?;
    Ok(manifest)
}

pub fn load_manifest_validated(
    manifest_path: impl AsRef<Path>,
    schema_path: impl AsRef<Path>,
) -> Result<GenerationManifest, Box<dyn std::error::Error>> {
    let manifest_path = manifest_path.as_ref();
    let schema_path = schema_path.as_ref();
    let manifest_json = fs::read_to_string(manifest_path)?;
    let schema_json = fs::read_to_string(schema_path)?;
    validate_manifest_json(manifest_path, schema_path, &manifest_json, &schema_json)?;
    let manifest = serde_json::from_str(&manifest_json)?;
    Ok(manifest)
}

fn validate_manifest_json(
    manifest_path: &Path,
    schema_path: &Path,
    manifest_json: &str,
    schema_json: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let schema: Value = serde_json::from_str(schema_json)?;
    let manifest: Value = serde_json::from_str(manifest_json)?;
    let validator = match jsonschema::compile(&schema) {
        Ok(validator) => validator,
        Err(error) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "manifest schema {} is invalid: {}",
                    schema_path.display(),
                    error
                ),
            )
            .into());
        }
    };

    if let Err(errors) = validator.validate(&manifest) {
        let details = errors
            .map(|error| format!("{} at {}", error, error.instance_path))
            .collect::<Vec<_>>()
            .join("\n");
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "manifest {} failed validation against {}:\n{}",
                manifest_path.display(),
                schema_path.display(),
                details
            ),
        )
        .into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cjfake-{prefix}-{stamp}"));
        fs::create_dir_all(&dir).expect("failed to create temp dir");
        dir
    }

    #[test]
    fn validate_manifest_against_schema() {
        let dir = temp_dir("manifest-valid");
        let schema_path = dir.join("schema.json");
        let manifest_path = dir.join("manifest.json");

        fs::write(
            &schema_path,
            r#"{
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
                    "additionalProperties": false,
                    "required": ["id"],
                    "properties": {
                      "id": { "type": "string", "minLength": 1 }
                    }
                  }
                }
              }
            }"#,
        )
        .expect("failed to write schema");
        fs::write(&manifest_path, r#"{"version":1,"cases":[{"id":"case-a"}]}"#)
            .expect("failed to write manifest");

        validate_manifest(&manifest_path, &schema_path).expect("manifest should validate");
    }

    #[test]
    fn reject_manifest_with_unknown_field() {
        let dir = temp_dir("manifest-invalid");
        let schema_path = dir.join("schema.json");
        let manifest_path = dir.join("manifest.json");

        fs::write(
            &schema_path,
            r#"{
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
                    "additionalProperties": false,
                    "required": ["id"],
                    "properties": {
                      "id": { "type": "string", "minLength": 1 }
                    }
                  }
                }
              }
            }"#,
        )
        .expect("failed to write schema");
        fs::write(
            &manifest_path,
            r#"{"version":1,"cases":[{"id":"case-a","extra":true}]}"#,
        )
        .expect("failed to write manifest");

        let err = validate_manifest(&manifest_path, &schema_path)
            .expect_err("manifest should fail validation");
        let message = err.to_string();
        assert!(message.contains("failed validation"));
        assert!(message.contains("extra"));
    }

    #[test]
    fn load_manifest_deserializes_flattened_case_config() {
        let manifest: GenerationManifest = serde_json::from_str(
            r#"{
              "version": 1,
              "cases": [
                {
                  "id": "stress-geometry",
                  "seed": 2001,
                  "min_cityobjects": 16,
                  "max_cityobjects": 16,
                  "allowed_types_geometry": ["MultiSurface"],
                  "materials_enabled": false,
                  "textures_enabled": false,
                  "metadata_enabled": false,
                  "attributes_enabled": false,
                  "semantics_enabled": false
                }
              ]
            }"#,
        )
        .expect("manifest should deserialize");

        let case = manifest.case("stress-geometry").expect("case should exist");
        assert_eq!(case.seed, Some(2001));
        assert_eq!(case.config.cityobjects.min_cityobjects, 16);
        assert_eq!(case.config.cityobjects.max_cityobjects, 16);
        assert_eq!(
            case.config.geometry.allowed_types_geometry,
            Some(vec![cityjson::v2_0::GeometryType::MultiSurface])
        );
        assert!(!case.config.materials.materials_enabled);
        assert!(!case.config.textures.textures_enabled);
        assert!(!case.config.metadata.metadata_enabled);
        assert!(!case.config.attributes.attributes_enabled);
        assert!(!case.config.semantics.semantics_enabled);
    }
}
