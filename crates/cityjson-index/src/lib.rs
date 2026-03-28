use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use cjlib::{CityModel, Result};
use globset::GlobMatcher;
use lru::LruCache;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BBox {
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

pub struct CityIndex {
    index: Index,
    backend: Box<dyn StorageBackend>,
}

pub enum StorageLayout {
    Ndjson { paths: Vec<PathBuf> },
    CityJson { paths: Vec<PathBuf> },
    FeatureFiles {
        root: PathBuf,
        metadata_glob: String,
        feature_glob: String,
    },
}

impl CityIndex {
    pub fn open(layout: StorageLayout, index_path: &Path) -> Result<Self> {
        let backend: Box<dyn StorageBackend> = match layout {
            StorageLayout::Ndjson { paths } => Box::new(NdjsonBackend { paths }),
            StorageLayout::CityJson { paths } => Box::new(CityJsonBackend::new(paths)),
            StorageLayout::FeatureFiles {
                root,
                metadata_glob,
                feature_glob,
            } => Box::new(FeatureFilesBackend::new(root, metadata_glob, feature_glob)),
        };

        Ok(Self {
            index: Index::open(index_path)?,
            backend,
        })
    }

    pub fn reindex(&mut self) -> Result<()> {
        let _ = self.backend.scan()?;
        todo!("index population is not scaffolded yet")
    }

    pub fn get(&self, _id: &str) -> Result<Option<CityModel>> {
        todo!("id lookup is not scaffolded yet")
    }

    pub fn query(&self, _bbox: &BBox) -> Result<Vec<CityModel>> {
        todo!("bbox query is not scaffolded yet")
    }

    pub fn query_iter(&self, _bbox: &BBox) -> Result<impl Iterator<Item = Result<CityModel>> + '_> {
        Ok(std::iter::empty())
    }

    pub fn metadata(&self) -> Result<Vec<Arc<Meta>>> {
        todo!("metadata lookup is not scaffolded yet")
    }
}

type Meta = serde_json::Value;

struct Index {
    _conn: Option<rusqlite::Connection>,
    metadata_cache: HashMap<i64, Arc<Meta>>,
}

struct FeatureLocation {
    source_id: i64,
    source_path: PathBuf,
    offset: u64,
    length: u64,
    vertices_offset: Option<u64>,
    vertices_length: Option<u64>,
}

struct FeatureIndexEntry {
    id: String,
    source_id: i64,
    offset: u64,
    length: u64,
    bbox: BBox,
}

impl Index {
    fn open(path: &Path) -> Result<Self> {
        let _ = path;
        Ok(Self {
            _conn: None,
            metadata_cache: HashMap::new(),
        })
    }

    fn lookup_id(&self, _id: &str) -> Result<Option<FeatureLocation>> {
        todo!("id lookup is not scaffolded yet")
    }

    fn lookup_bbox(&self, _bbox: &BBox) -> Result<Vec<FeatureLocation>> {
        todo!("bbox lookup is not scaffolded yet")
    }

    fn insert_source(&mut self, _path: &str, _meta: &Meta) -> Result<i64> {
        todo!("source insertion is not scaffolded yet")
    }

    fn insert_features(&mut self, _entries: &[FeatureIndexEntry]) -> Result<()> {
        todo!("feature insertion is not scaffolded yet")
    }

    fn get_metadata(&self, _source_id: i64) -> Result<Arc<Meta>> {
        todo!("metadata cache lookup is not scaffolded yet")
    }

    fn clear(&mut self) -> Result<()> {
        self.metadata_cache.clear();
        Ok(())
    }
}

trait StorageBackend: Send + Sync {
    fn scan(&self) -> Result<Vec<SourceScan>>;
    fn read_one(&self, loc: &FeatureLocation) -> Result<CityModel>;
}

struct SourceScan {
    path: PathBuf,
    metadata: Meta,
    vertices_offset: Option<u64>,
    vertices_length: Option<u64>,
    features: Vec<ScannedFeature>,
}

struct ScannedFeature {
    id: String,
    offset: u64,
    length: u64,
    bbox: BBox,
}

struct NdjsonBackend {
    paths: Vec<PathBuf>,
}

impl StorageBackend for NdjsonBackend {
    fn scan(&self) -> Result<Vec<SourceScan>> {
        let _ = &self.paths;
        todo!("NDJSON scanning is not scaffolded yet")
    }

    fn read_one(&self, loc: &FeatureLocation) -> Result<CityModel> {
        let _ = loc;
        todo!("NDJSON read is not scaffolded yet")
    }
}

struct CityJsonBackend {
    paths: Vec<PathBuf>,
    vertices_cache: Mutex<LruCache<PathBuf, Arc<Vec<[i64; 3]>>>>,
}

impl CityJsonBackend {
    fn new(paths: Vec<PathBuf>) -> Self {
        Self {
            paths,
            vertices_cache: Mutex::new(LruCache::unbounded()),
        }
    }
}

impl StorageBackend for CityJsonBackend {
    fn scan(&self) -> Result<Vec<SourceScan>> {
        let _ = (&self.paths, &self.vertices_cache);
        todo!("CityJSON scanning is not scaffolded yet")
    }

    fn read_one(&self, loc: &FeatureLocation) -> Result<CityModel> {
        let _ = loc;
        todo!("CityJSON read is not scaffolded yet")
    }
}

struct FeatureFilesBackend {
    root: PathBuf,
    metadata_glob: GlobMatcher,
    feature_glob: GlobMatcher,
}

impl FeatureFilesBackend {
    fn new(root: PathBuf, metadata_glob: String, feature_glob: String) -> Self {
        let metadata_glob = globset::Glob::new(&metadata_glob)
            .expect("metadata glob must be valid")
            .compile_matcher();
        let feature_glob = globset::Glob::new(&feature_glob)
            .expect("feature glob must be valid")
            .compile_matcher();
        Self {
            root,
            metadata_glob,
            feature_glob,
        }
    }
}

impl StorageBackend for FeatureFilesBackend {
    fn scan(&self) -> Result<Vec<SourceScan>> {
        let _ = (&self.root, &self.metadata_glob, &self.feature_glob);
        todo!("feature-tree scanning is not scaffolded yet")
    }

    fn read_one(&self, loc: &FeatureLocation) -> Result<CityModel> {
        let _ = loc;
        todo!("feature-file read is not scaffolded yet")
    }
}
