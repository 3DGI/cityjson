#![cfg(feature = "serialize")]

use crate::cli::CJFakeConfig;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

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

pub fn load_manifest(
    path: impl AsRef<Path>,
) -> Result<GenerationManifest, Box<dyn std::error::Error>> {
    let path = path.as_ref();
    let json = fs::read_to_string(path)?;
    let manifest = serde_json::from_str(&json)?;
    Ok(manifest)
}
