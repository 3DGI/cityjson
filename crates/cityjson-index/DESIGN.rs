use std::path::{Path, PathBuf};
use std::sync::Arc;

use cityjson_rs::{
    BBox, CityModel, Metadata, Transform,
    ResourceId32, OwnedStringStorage,
};

type Model = CityModel<u32, ResourceId32, OwnedStringStorage>;
type Meta = Metadata<OwnedStringStorage>;

// ── Public API ────────────────────────────────────────────────────────

pub struct CityIndex {
    index: Index,
    backend: Box<dyn StorageBackend>,
}

pub enum StorageLayout {
    Ndjson {
        paths: Vec<PathBuf>,
    },
    CityJson {
        paths: Vec<PathBuf>,
    },
    FeatureFiles {
        root: PathBuf,
        metadata_glob: String,
        feature_glob: String,
    },
}

impl CityIndex {
    pub fn open(layout: StorageLayout, index_path: &Path) -> Result<Self> { .. }

    pub fn reindex(&mut self) -> Result<()> { .. }

    /// Returns a CityModel containing exactly one CityObject.
    pub fn get(&self, id: &str) -> Result<Option<Model>> {
        let loc = match self.index.lookup_id(id)? {
            Some(loc) => loc,
            None => return Ok(None),
        };
        Ok(Some(self.backend.read_one(&loc)?))
    }

    /// Returns one CityModel per matching feature.
    pub fn query(&self, bbox: &BBox) -> Result<Vec<Model>> {
        self.index
            .lookup_bbox(bbox)?
            .iter()
            .map(|loc| self.backend.read_one(loc))
            .collect()
    }

    /// Lazy variant.
    pub fn query_iter(
        &self,
        bbox: &BBox,
    ) -> Result<impl Iterator<Item = Result<Model>> + '_> {
        let locs = self.index.lookup_bbox(bbox)?;
        Ok(locs.into_iter().map(|loc| self.backend.read_one(&loc)))
    }

    /// All source metadata entries.
    pub fn metadata(&self) -> Result<Vec<Arc<Meta>>> { .. }
}

// ── Index ─────────────────────────────────────────────────────────────

/// ```sql
/// CREATE TABLE sources (
///     id              INTEGER PRIMARY KEY,
///     path            TEXT    NOT NULL UNIQUE,
///     meta_json       TEXT    NOT NULL,
///     vertices_offset INTEGER,
///     vertices_length INTEGER
/// );
///
/// CREATE TABLE features (
///     id        TEXT    PRIMARY KEY,
///     source_id INTEGER NOT NULL REFERENCES sources(id),
///     offset    INTEGER NOT NULL,
///     length    INTEGER NOT NULL
/// );
///
/// CREATE VIRTUAL TABLE feature_bbox USING rtree(
///     rowid,
///     min_x, max_x,
///     min_y, max_y
/// );
///
/// CREATE TABLE bbox_map (
///     rowid      INTEGER PRIMARY KEY,
///     feature_id TEXT    NOT NULL
/// );
/// ```
struct Index {
    conn: rusqlite::Connection,
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
    fn open(path: &Path) -> Result<Self> { .. }
    fn lookup_id(&self, id: &str) -> Result<Option<FeatureLocation>> { .. }
    fn lookup_bbox(&self, bbox: &BBox) -> Result<Vec<FeatureLocation>> { .. }
    fn insert_source(&mut self, path: &str, meta: &Meta) -> Result<i64> { .. }
    fn insert_features(&mut self, entries: &[FeatureIndexEntry]) -> Result<()> { .. }
    fn get_metadata(&self, source_id: i64) -> Result<Arc<Meta>> { .. }
    fn clear(&mut self) -> Result<()> { .. }
}

// ── Backend trait ─────────────────────────────────────────────────────

trait StorageBackend: Send + Sync {
    fn scan(&self) -> Result<Vec<SourceScan>>;

    /// Read one feature from disk. Returns a CityModel with one
    /// CityObject, its vertices, metadata, transform — everything.
    fn read_one(&self, loc: &FeatureLocation) -> Result<Model>;
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

// ── Backend: NDJSON ───────────────────────────────────────────────────

struct NdjsonBackend {
    paths: Vec<PathBuf>,
}

impl StorageBackend for NdjsonBackend {
    fn scan(&self) -> Result<Vec<SourceScan>> {
        // Per file:
        //   line 1 → Metadata
        //   lines 2..N → offset/length, partial parse for id + bbox
        ..
    }

    fn read_one(&self, loc: &FeatureLocation) -> Result<Model> {
        // pread the line → serde_json::from_slice::<Model>
        // A CityJSONFeature line is a valid CityModel with one object.
        // Attach metadata from index cache (CRS, transform).
        ..
    }
}

// ── Backend: CityJSON ─────────────────────────────────────────────────

struct CityJsonBackend {
    paths: Vec<PathBuf>,
    vertices_cache: Mutex<LruCache<PathBuf, Arc<Vec<[i64; 3]>>>>,
}

impl StorageBackend for CityJsonBackend {
    fn scan(&self) -> Result<Vec<SourceScan>> {
        // Per file:
        //   top-level → Metadata + transform
        //   "vertices" → record byte range
        //   "CityObjects" → per entry: offset/length, vertex refs → bbox
        ..
    }

    fn read_one(&self, loc: &FeatureLocation) -> Result<Model> {
        // 1. pread CityObject bytes at loc.offset
        // 2. pread or cache shared vertices
        // 3. Parse CityObject, collect its vertex indices
        // 4. Copy referenced vertices → local array, remap indices
        // 5. Assemble CityModel with one object, local vertices, metadata
        //
        // Only backend with nontrivial read logic.
        ..
    }
}

// ── Backend: FeatureFiles ─────────────────────────────────────────────

struct FeatureFilesBackend {
    root: PathBuf,
    metadata_glob: GlobMatcher,
    feature_glob: GlobMatcher,
}

impl StorageBackend for FeatureFilesBackend {
    fn scan(&self) -> Result<Vec<SourceScan>> {
        // Walk root (ignore::WalkBuilder, .gitignore-aware)
        // Collect + parse metadata files, sort by depth
        // Per feature file:
        //   resolve nearest ancestor metadata
        //   offset = 0, length = file size
        //   partial parse for id + bbox
        // Group by metadata into SourceScans
        ..
    }

    fn read_one(&self, loc: &FeatureLocation) -> Result<Model> {
        // fs::read whole file → serde_json::from_slice::<Model>
        // Attach metadata from index cache.
        ..
    }
}

// ── Cargo.toml ────────────────────────────────────────────────────────
//
// [package]
// name = "cjindex"
// version = "0.1.0"
// edition = "2021"
//
// [dependencies]
// cityjson-rs = { path = "../cityjson-rs" }
// rusqlite = { version = "0.31", features = ["bundled"] }
// serde = { version = "1", features = ["derive"] }
// serde_json = "1"
// globset = "0.4"
// ignore = "0.4"
// lru = "0.12"
// memmap2 = "0.9"
// thiserror = "2"
