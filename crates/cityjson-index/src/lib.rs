use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io::{ErrorKind, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use cjlib::{CityModel, Error, Result};
use globset::GlobMatcher;
use ignore::WalkBuilder;
use lru::LruCache;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use serde_json::{Map, Number, Value};

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
    /// Opens an index for the given storage layout.
    ///
    /// # Errors
    ///
    /// Returns an error if the index backend cannot be created or the index
    /// store cannot be opened.
    pub fn open(layout: StorageLayout, index_path: &Path) -> Result<Self> {
        let backend: Box<dyn StorageBackend> = match layout {
            StorageLayout::Ndjson { paths } => Box::new(NdjsonBackend { paths }),
            StorageLayout::CityJson { paths } => Box::new(CityJsonBackend::new(paths)),
            StorageLayout::FeatureFiles {
                root,
                metadata_glob,
                feature_glob,
            } => Box::new(FeatureFilesBackend::new(
                root,
                metadata_glob.as_str(),
                feature_glob.as_str(),
            )),
        };

        Ok(Self {
            index: Index::open(index_path)?,
            backend,
        })
    }

    /// Rebuilds the index from the configured backend.
    ///
    /// # Errors
    ///
    /// Returns an error if backend scanning or index population fails.
    pub fn reindex(&mut self) -> Result<()> {
        let scans = self.backend.scan()?;
        self.index.rebuild(&scans)
    }

    /// Returns a `CityJSON` feature by id.
    ///
    /// # Errors
    ///
    /// Returns an error if lookup fails.
    pub fn get(&self, id: &str) -> Result<Option<CityModel>> {
        let Some(loc) = self.index.lookup_id(id)? else {
            return Ok(None);
        };
        let metadata = self.index.get_metadata(loc.source_id)?;
        self.backend.read_one(&loc, metadata).map(Some)
    }

    /// Returns every feature intersecting the given bounding box.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub fn query(&self, bbox: &BBox) -> Result<Vec<CityModel>> {
        self.index
            .lookup_bbox(bbox)?
            .into_iter()
            .map(|loc| {
                let metadata = self.index.get_metadata(loc.source_id)?;
                self.backend.read_one(&loc, metadata)
            })
            .collect()
    }

    /// Returns an iterator over features intersecting the given bounding box.
    ///
    /// # Errors
    ///
    /// Returns an error if the iterator cannot be constructed.
    pub fn query_iter(&self, bbox: &BBox) -> Result<impl Iterator<Item = Result<CityModel>> + '_> {
        let locations = self.index.lookup_bbox(bbox)?;
        Ok(locations.into_iter().map(move |loc| {
            let metadata = self.index.get_metadata(loc.source_id)?;
            self.backend.read_one(&loc, metadata)
        }))
    }

    /// Returns cached metadata entries.
    ///
    /// # Errors
    ///
    /// Returns an error if metadata lookup fails.
    pub fn metadata(&self) -> Result<Vec<Arc<Meta>>> {
        self.index.metadata()
    }
}

type Meta = serde_json::Value;

struct Index {
    conn: rusqlite::Connection,
    metadata_cache: Mutex<HashMap<i64, Arc<Meta>>>,
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
    path: PathBuf,
    offset: u64,
    length: u64,
    bbox: BBox,
}

impl Index {
    fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }

        let conn = sqlite_result(rusqlite::Connection::open(path))?;
        sqlite_result(conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS sources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                metadata TEXT NOT NULL,
                vertices_offset INTEGER,
                vertices_length INTEGER
            );

            CREATE TABLE IF NOT EXISTS features (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feature_id TEXT NOT NULL UNIQUE,
                source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
                path TEXT NOT NULL,
                offset INTEGER NOT NULL,
                length INTEGER NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS feature_bbox
            USING rtree(
                feature_rowid,
                min_x,
                max_x,
                min_y,
                max_y
            );

            CREATE TABLE IF NOT EXISTS bbox_map (
                feature_rowid INTEGER PRIMARY KEY,
                feature_id TEXT NOT NULL UNIQUE REFERENCES features(feature_id) ON DELETE CASCADE
            );
            "#,
        ))?;

        Ok(Self {
            conn,
            metadata_cache: Mutex::new(HashMap::new()),
        })
    }

    fn rebuild(&mut self, scans: &[SourceScan]) -> Result<()> {
        let tx = sqlite_result(self.conn.transaction())?;
        Self::clear_tables(&tx)?;

        let mut feature_entries = Vec::new();
        for scan in scans {
            let source_id = Self::insert_source_in_tx(
                &tx,
                scan.path.as_path(),
                &scan.metadata,
                scan.vertices_offset,
                scan.vertices_length,
            )?;
            for feature in &scan.features {
                feature_entries.push(FeatureIndexEntry {
                    id: feature.id.clone(),
                    source_id,
                    path: feature.path.clone(),
                    offset: feature.offset,
                    length: feature.length,
                    bbox: feature.bbox,
                });
            }
        }
        Self::insert_features_in_tx(&tx, &feature_entries)?;
        sqlite_result(tx.commit())?;

        self.metadata_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
        Ok(())
    }

    fn lookup_id(&self, id: &str) -> Result<Option<FeatureLocation>> {
        sqlite_result(
            self.conn
                .query_row(
                    r#"
                SELECT
                    s.id,
                    f.path,
                    f.offset,
                    f.length,
                    s.vertices_offset,
                    s.vertices_length
                FROM features AS f
                JOIN sources AS s ON s.id = f.source_id
                WHERE f.feature_id = ?1
                "#,
                    params![id],
                    |row| Self::feature_location_from_row(row),
                )
                .optional(),
        )
    }

    fn lookup_bbox(&self, bbox: &BBox) -> Result<Vec<FeatureLocation>> {
        let mut stmt = sqlite_result(self.conn.prepare(
            r#"
            SELECT DISTINCT
                s.id,
                f.path,
                f.offset,
                f.length,
                s.vertices_offset,
                s.vertices_length
            FROM feature_bbox AS fb
            JOIN bbox_map AS bm ON bm.feature_rowid = fb.feature_rowid
            JOIN features AS f ON f.feature_id = bm.feature_id
            JOIN sources AS s ON s.id = f.source_id
            WHERE fb.min_x <= ?2
              AND fb.max_x >= ?1
              AND fb.min_y <= ?4
              AND fb.max_y >= ?3
            ORDER BY bm.feature_id
            "#,
        ))?;
        let rows = sqlite_result(stmt.query_map(
            params![bbox.min_x, bbox.max_x, bbox.min_y, bbox.max_y],
            |row| Self::feature_location_from_row(row),
        ))?;
        sqlite_result(rows.collect())
    }

    fn get_metadata(&self, source_id: i64) -> Result<Arc<Meta>> {
        if let Some(metadata) = self
            .metadata_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&source_id)
            .cloned()
        {
            return Ok(metadata);
        }

        let metadata_json: String = sqlite_result(self.conn.query_row(
            "SELECT metadata FROM sources WHERE id = ?1",
            params![source_id],
            |row| row.get(0),
        ))?;
        let metadata: Meta = serde_json::from_str(&metadata_json)?;
        let metadata = Arc::new(metadata);

        self.metadata_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(source_id, Arc::clone(&metadata));

        Ok(metadata)
    }

    fn metadata(&self) -> Result<Vec<Arc<Meta>>> {
        let mut stmt = sqlite_result(self.conn.prepare("SELECT id FROM sources ORDER BY id"))?;
        let rows = sqlite_result(stmt.query_map([], |row| row.get::<_, i64>(0)))?;
        let source_ids = sqlite_result(rows.collect::<rusqlite::Result<Vec<_>>>())?;
        source_ids
            .into_iter()
            .map(|source_id| self.get_metadata(source_id))
            .collect()
    }

    fn clear_tables(tx: &rusqlite::Transaction<'_>) -> Result<()> {
        sqlite_result(tx.execute_batch(
            r#"
            DELETE FROM bbox_map;
            DELETE FROM feature_bbox;
            DELETE FROM features;
            DELETE FROM sources;
            "#,
        ))?;
        Ok(())
    }

    fn insert_source_in_tx(
        tx: &rusqlite::Transaction<'_>,
        path: &Path,
        meta: &Meta,
        vertices_offset: Option<u64>,
        vertices_length: Option<u64>,
    ) -> Result<i64> {
        let metadata_json = serde_json::to_string(meta)?;
        let vertices_offset = sqlite_result(vertices_offset.map(u64_to_i64).transpose())?;
        let vertices_length = sqlite_result(vertices_length.map(u64_to_i64).transpose())?;
        sqlite_result(tx.execute(
            r#"
            INSERT INTO sources (path, metadata, vertices_offset, vertices_length)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                path.to_string_lossy(),
                metadata_json,
                vertices_offset,
                vertices_length
            ],
        ))?;
        Ok(tx.last_insert_rowid())
    }

    fn insert_features_in_tx(
        tx: &rusqlite::Transaction<'_>,
        entries: &[FeatureIndexEntry],
    ) -> Result<()> {
        let mut feature_stmt = sqlite_result(tx.prepare(
            r#"
            INSERT INTO features (feature_id, source_id, path, offset, length)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        ))?;
        let mut bbox_stmt = sqlite_result(tx.prepare(
            r#"
            INSERT INTO feature_bbox (feature_rowid, min_x, max_x, min_y, max_y)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        ))?;
        let mut map_stmt = sqlite_result(tx.prepare(
            r#"
            INSERT INTO bbox_map (feature_rowid, feature_id)
            VALUES (?1, ?2)
            "#,
        ))?;
        for entry in entries {
            let offset = sqlite_result(u64_to_i64(entry.offset))?;
            let length = sqlite_result(u64_to_i64(entry.length))?;
            sqlite_result(feature_stmt.execute(params![
                &entry.id,
                entry.source_id,
                entry.path.to_string_lossy(),
                offset,
                length,
            ]))?;
            let feature_rowid = tx.last_insert_rowid();
            sqlite_result(bbox_stmt.execute(params![
                feature_rowid,
                entry.bbox.min_x,
                entry.bbox.max_x,
                entry.bbox.min_y,
                entry.bbox.max_y,
            ]))?;
            sqlite_result(map_stmt.execute(params![feature_rowid, &entry.id]))?;
        }

        Ok(())
    }

    fn feature_location_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<FeatureLocation> {
        let source_id = row.get::<_, i64>(0)?;
        let source_path = PathBuf::from(row.get::<_, String>(1)?);
        let offset = i64_to_u64(row.get::<_, i64>(2)?)?;
        let length = i64_to_u64(row.get::<_, i64>(3)?)?;
        let vertices_offset = match row.get::<_, Option<i64>>(4)? {
            Some(value) => Some(i64_to_u64(value)?),
            None => None,
        };
        let vertices_length = match row.get::<_, Option<i64>>(5)? {
            Some(value) => Some(i64_to_u64(value)?),
            None => None,
        };

        Ok(FeatureLocation {
            source_id,
            source_path,
            offset,
            length,
            vertices_offset,
            vertices_length,
        })
    }
}

trait StorageBackend: Send + Sync {
    fn scan(&self) -> Result<Vec<SourceScan>>;
    fn read_one(&self, loc: &FeatureLocation, metadata: Arc<Meta>) -> Result<CityModel>;
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
    path: PathBuf,
    offset: u64,
    length: u64,
    bbox: BBox,
}

struct SingleObjectFeatureParts {
    object_id: String,
    object_json: Box<RawValue>,
    vertices: Vec<[i64; 3]>,
}

struct NdjsonBackend {
    paths: Vec<PathBuf>,
}

impl StorageBackend for NdjsonBackend {
    fn scan(&self) -> Result<Vec<SourceScan>> {
        collect_layout_files(&self.paths, ".jsonl")?
            .into_iter()
            .map(|path| scan_ndjson_source(&path))
            .collect()
    }

    fn read_one(&self, loc: &FeatureLocation, metadata: Arc<Meta>) -> Result<CityModel> {
        let bytes = read_exact_range(&loc.source_path, loc.offset, loc.length)?;
        let metadata_bytes = serde_json::to_vec(metadata.as_ref())?;
        cjlib::json::from_feature_slice_with_base(&bytes, &metadata_bytes)
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

    fn load_shared_vertices(
        &self,
        source_path: &Path,
        source_file: &mut fs::File,
        offset: u64,
        length: u64,
    ) -> Result<Arc<Vec<[i64; 3]>>> {
        let mut cache = self
            .vertices_cache
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(vertices) = cache.get(source_path) {
            return Ok(Arc::clone(vertices));
        }

        let vertices_bytes = read_exact_range_from_file(source_file, source_path, offset, length)?;
        let vertices = Arc::new(parse_vertices_fragment(&vertices_bytes)?);
        cache.put(source_path.to_path_buf(), Arc::clone(&vertices));
        Ok(vertices)
    }
}

impl StorageBackend for CityJsonBackend {
    fn scan(&self) -> Result<Vec<SourceScan>> {
        let _ = &self.vertices_cache;
        collect_layout_files(&self.paths, ".city.json")?
            .into_iter()
            .map(|path| scan_cityjson_source(&path))
            .collect()
    }

    fn read_one(&self, loc: &FeatureLocation, metadata: Arc<Meta>) -> Result<CityModel> {
        let vertices_offset = loc.vertices_offset.ok_or_else(|| {
            Error::UnsupportedFeature(
                "regular CityJSON reads require an indexed shared vertices range".into(),
            )
        })?;
        let vertices_length = loc.vertices_length.ok_or_else(|| {
            Error::UnsupportedFeature(
                "regular CityJSON reads require an indexed shared vertices range".into(),
            )
        })?;

        let base_document_bytes = serde_json::to_vec(metadata.as_ref())?;
        let mut source_file = fs::File::open(&loc.source_path)?;
        let object_fragment =
            read_exact_range_from_file(&mut source_file, &loc.source_path, loc.offset, loc.length)?;
        let (object_id, object_value) = parse_cityobject_entry(&object_fragment)?;
        let shared_vertices = self.load_shared_vertices(
            &loc.source_path,
            &mut source_file,
            vertices_offset,
            vertices_length,
        )?;
        let feature_parts =
            build_single_object_feature_parts(&object_id, object_value, shared_vertices.as_ref())?;
        let cityobjects = [cjlib::json::FeatureObject {
            id: feature_parts.object_id.as_str(),
            object: feature_parts.object_json.as_ref(),
        }];
        let parts = cjlib::json::FeatureParts {
            id: feature_parts.object_id.as_str(),
            cityobjects: &cityobjects,
            vertices: &feature_parts.vertices,
        };

        cjlib::json::from_feature_parts_with_base(parts, &base_document_bytes)
    }
}

struct FeatureFilesBackend {
    root: PathBuf,
    metadata_glob: GlobMatcher,
    feature_glob: GlobMatcher,
}

impl FeatureFilesBackend {
    fn new(root: PathBuf, metadata_glob: &str, feature_glob: &str) -> Self {
        let metadata_glob = globset::Glob::new(metadata_glob)
            .expect("metadata glob must be valid")
            .compile_matcher();
        let feature_glob = globset::Glob::new(feature_glob)
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
        scan_feature_files_root(&self.root, &self.metadata_glob, &self.feature_glob)
    }

    fn read_one(&self, loc: &FeatureLocation, metadata: Arc<Meta>) -> Result<CityModel> {
        let feature_bytes = read_exact_range(&loc.source_path, loc.offset, loc.length)?;
        let metadata_bytes = serde_json::to_vec(metadata.as_ref())?;
        cjlib::json::from_feature_slice_with_base(&feature_bytes, &metadata_bytes)
    }
}

fn scan_feature_files_root(
    root: &Path,
    metadata_glob: &GlobMatcher,
    feature_glob: &GlobMatcher,
) -> Result<Vec<SourceScan>> {
    let mut metadata_files = Vec::new();
    let mut feature_files = Vec::new();

    for entry in WalkBuilder::new(root)
        .hidden(false)
        .follow_links(true)
        .build()
    {
        let entry = entry.map_err(|error| import_error(error.to_string()))?;
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        if entry
            .metadata()
            .map(|meta| meta.len() == 0)
            .unwrap_or(false)
        {
            continue;
        }
        let path = entry.into_path();
        let rel = path.strip_prefix(root).unwrap_or(path.as_path());
        if metadata_glob.is_match(rel) {
            metadata_files.push(path);
        } else if feature_glob.is_match(rel) {
            feature_files.push(path);
        }
    }

    metadata_files.sort();
    feature_files.sort();

    if metadata_files.is_empty() {
        return Err(import_error(format!(
            "feature-files root {} does not contain any metadata files",
            root.display()
        )));
    }

    let mut metadata_by_dir = BTreeMap::new();
    let mut sources = BTreeMap::new();

    for metadata_path in metadata_files {
        let metadata: Meta = read_json(&metadata_path)?;
        let parent = metadata_path.parent().unwrap_or(root).to_path_buf();
        metadata_by_dir.insert(parent, metadata_path.clone());
        sources.insert(
            metadata_path.clone(),
            SourceScan {
                path: metadata_path,
                metadata,
                vertices_offset: None,
                vertices_length: None,
                features: Vec::new(),
            },
        );
    }

    for feature_path in feature_files {
        let metadata_path = resolve_feature_metadata_path(root, &feature_path, &metadata_by_dir)
            .ok_or_else(|| {
                import_error(format!(
                    "no ancestor metadata file found for feature {}",
                    feature_path.display()
                ))
            })?;
        let source = sources.get_mut(&metadata_path).ok_or_else(|| {
            import_error(format!(
                "feature {} resolved to missing metadata source {}",
                feature_path.display(),
                metadata_path.display()
            ))
        })?;
        let feature: Value = read_json(&feature_path)?;
        let (id, bbox) = parse_feature_file_bbox(&feature, &source.metadata)?;
        let length = fs::metadata(&feature_path)?.len();
        source.features.push(ScannedFeature {
            id,
            path: feature_path.clone(),
            offset: 0,
            length,
            bbox,
        });
    }

    Ok(sources.into_values().collect())
}

fn resolve_feature_metadata_path(
    root: &Path,
    feature_path: &Path,
    metadata_by_dir: &BTreeMap<PathBuf, PathBuf>,
) -> Option<PathBuf> {
    let mut current = feature_path.parent();
    while let Some(dir) = current {
        if let Some(metadata_path) = metadata_by_dir.get(dir) {
            return Some(metadata_path.clone());
        }
        if dir == root {
            break;
        }
        current = dir.parent();
    }
    None
}

fn parse_feature_file_bbox(feature: &Value, metadata: &Meta) -> Result<(String, BBox)> {
    let id = feature_identifier(feature, "feature file")?;
    let vertices = feature
        .get("vertices")
        .cloned()
        .ok_or_else(|| import_error("feature file is missing vertices"))?;
    let vertices: Vec<[i64; 3]> = serde_json::from_value(vertices)?;

    let referenced_vertices = collect_feature_vertex_indices(feature, vertices.len())?;
    let (scale, translate) = parse_ndjson_transform(metadata)?;
    let bbox = bbox_from_vertices(&vertices, &referenced_vertices, scale, translate)?;
    Ok((id, bbox))
}

fn trim_fragment_delimiters(bytes: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = bytes.len();

    while start < end && (bytes[start].is_ascii_whitespace() || bytes[start] == b',') {
        start += 1;
    }
    while end > start && (bytes[end - 1].is_ascii_whitespace() || bytes[end - 1] == b',') {
        end -= 1;
    }

    &bytes[start..end]
}

fn parse_cityobject_entry(fragment: &[u8]) -> Result<(String, Value)> {
    let fragment = trim_fragment_delimiters(fragment);
    if fragment.is_empty() {
        return Err(import_error("CityObject entry fragment is empty"));
    }

    let mut wrapped = Vec::with_capacity(fragment.len() + 2);
    wrapped.push(b'{');
    wrapped.extend_from_slice(fragment);
    wrapped.push(b'}');

    let entry: Map<String, Value> = serde_json::from_slice(&wrapped)?;
    if entry.len() != 1 {
        return Err(import_error(
            "CityObject entry fragment must contain exactly one object entry",
        ));
    }

    let (object_id, object_value) = entry
        .into_iter()
        .next()
        .ok_or_else(|| import_error("CityObject entry fragment is empty"))?;
    if !object_value.is_object() {
        return Err(import_error("CityObject entry value must be a JSON object"));
    }

    Ok((object_id, object_value))
}

fn parse_vertices_fragment(fragment: &[u8]) -> Result<Vec<[i64; 3]>> {
    let fragment = trim_fragment_delimiters(fragment);
    if fragment.is_empty() {
        return Err(import_error("shared vertices fragment is empty"));
    }
    Ok(serde_json::from_slice(fragment)?)
}

fn build_single_object_feature_parts(
    object_id: &str,
    mut object_value: Value,
    shared_vertices: &[[i64; 3]],
) -> Result<SingleObjectFeatureParts> {
    filter_local_relationships(&mut object_value, object_id)?;

    let mut referenced_vertices = BTreeSet::new();
    if let Some(geometries) = object_value
        .as_object()
        .and_then(|object| object.get("geometry"))
        .and_then(Value::as_array)
    {
        for geometry in geometries {
            if let Some(boundaries) = geometry.get("boundaries") {
                collect_vertex_indices(boundaries, &mut referenced_vertices)?;
            }
        }
    }

    let local_vertices = build_local_vertices(shared_vertices, &referenced_vertices)?;
    let remap = referenced_vertices
        .iter()
        .enumerate()
        .map(|(new_index, old_index)| (*old_index, new_index))
        .collect::<HashMap<_, _>>();

    if let Some(geometries) = object_value
        .as_object_mut()
        .and_then(|object| object.get_mut("geometry"))
        .and_then(Value::as_array_mut)
    {
        for geometry in geometries {
            if let Some(boundaries) = geometry.get_mut("boundaries") {
                remap_vertex_indices(boundaries, &remap)?;
            }
        }
    }

    Ok(SingleObjectFeatureParts {
        object_id: object_id.to_owned(),
        object_json: RawValue::from_string(serde_json::to_string(&object_value)?)?,
        vertices: local_vertices,
    })
}

fn filter_local_relationships(object_value: &mut Value, object_id: &str) -> Result<()> {
    let object = object_value
        .as_object_mut()
        .ok_or_else(|| import_error("CityObject value must be a JSON object"))?;

    for key in ["children", "parents"] {
        let remove_key = match object.get_mut(key) {
            Some(value) => {
                let refs = value
                    .as_array_mut()
                    .ok_or_else(|| import_error(format!("{key} must be an array")))?;
                refs.retain(|entry| entry.as_str() == Some(object_id));
                refs.is_empty()
            }
            None => false,
        };

        if remove_key {
            object.remove(key);
        }
    }

    Ok(())
}

fn collect_vertex_indices(value: &Value, indices: &mut BTreeSet<usize>) -> Result<()> {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_vertex_indices(item, indices)?;
            }
            Ok(())
        }
        Value::Number(number) => {
            indices.insert(number_to_index(number)?);
            Ok(())
        }
        Value::Null => Ok(()),
        other => Err(import_error(format!(
            "boundary values must be arrays or non-negative integers, found {}",
            value_kind(other)
        ))),
    }
}

fn remap_vertex_indices(value: &mut Value, remap: &HashMap<usize, usize>) -> Result<()> {
    match value {
        Value::Array(items) => {
            for item in items {
                remap_vertex_indices(item, remap)?;
            }
            Ok(())
        }
        Value::Number(number) => {
            let old_index = number_to_index(number)?;
            let new_index = remap.get(&old_index).copied().ok_or_else(|| {
                import_error(format!(
                    "missing remap entry for referenced vertex index {old_index}"
                ))
            })?;
            *value =
                Value::Number(Number::from(u64::try_from(new_index).map_err(|_| {
                    import_error("localized vertex index does not fit in u64")
                })?));
            Ok(())
        }
        Value::Null => Ok(()),
        other => Err(import_error(format!(
            "boundary values must be arrays or non-negative integers, found {}",
            value_kind(other)
        ))),
    }
}

fn build_local_vertices(
    shared_vertices: &[[i64; 3]],
    referenced_vertices: &BTreeSet<usize>,
) -> Result<Vec<[i64; 3]>> {
    let mut vertices = Vec::with_capacity(referenced_vertices.len());

    for &index in referenced_vertices {
        let vertex = shared_vertices.get(index).copied().ok_or_else(|| {
            import_error(format!(
                "vertex index {index} is outside the shared vertices array"
            ))
        })?;
        vertices.push(vertex);
    }

    Ok(vertices)
}

fn number_to_index(number: &Number) -> Result<usize> {
    let index = number
        .as_u64()
        .ok_or_else(|| import_error("boundary vertex indices must be non-negative integers"))?;
    usize::try_from(index)
        .map_err(|_| import_error(format!("vertex index {index} does not fit in usize")))
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn import_error(message: impl Into<String>) -> Error {
    Error::Import(message.into())
}

fn read_exact_range(path: &Path, offset: u64, length: u64) -> Result<Vec<u8>> {
    let mut file = fs::File::open(path)
        .map_err(|error| import_error(format!("failed to open {}: {error}", path.display())))?;
    read_exact_range_from_file(&mut file, path, offset, length)
}

fn read_exact_range_from_file(
    file: &mut fs::File,
    path: &Path,
    offset: u64,
    length: u64,
) -> Result<Vec<u8>> {
    let length = usize::try_from(length).map_err(|_| {
        import_error(format!(
            "requested read of {length} bytes from {} exceeds the supported buffer size",
            path.display()
        ))
    })?;
    if length > isize::MAX as usize {
        return Err(import_error(format!(
            "requested read of {length} bytes from {} exceeds the supported buffer size",
            path.display()
        )));
    }

    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(|error| {
        import_error(format!(
            "failed to allocate buffer for {} bytes from {}: {error}",
            length,
            path.display()
        ))
    })?;
    bytes.resize(length, 0);

    file.seek(SeekFrom::Start(offset)).map_err(|error| {
        import_error(format!(
            "failed to seek to byte offset {offset} in {}: {error}",
            path.display()
        ))
    })?;
    file.read_exact(&mut bytes).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            import_error(format!(
                "short read while reading {length} bytes at offset {offset} from {}",
                path.display()
            ))
        } else {
            import_error(format!(
                "failed to read {length} bytes at offset {offset} from {}: {error}",
                path.display()
            ))
        }
    })?;

    Ok(bytes)
}

fn read_json(path: impl AsRef<Path>) -> Result<Value> {
    let bytes = fs::read(path.as_ref())?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn scan_ndjson_source(path: &Path) -> Result<SourceScan> {
    let bytes = fs::read(path)?;
    let line_spans = line_spans(&bytes);
    let Some((_, metadata_bytes)) = line_spans.first() else {
        return Err(import_error(format!(
            "NDJSON source {} is empty",
            path.display()
        )));
    };

    let metadata: Meta = serde_json::from_slice(metadata_bytes)?;
    let (scale, translate) = parse_ndjson_transform(&metadata)?;
    let mut features = Vec::new();

    for (offset, line_bytes) in line_spans.into_iter().skip(1) {
        if line_bytes.iter().all(|byte| byte.is_ascii_whitespace()) {
            continue;
        }

        let feature: Value = serde_json::from_slice(line_bytes)?;
        let (id, bbox) = parse_ndjson_feature_bbox(&feature, scale, translate)?;
        features.push(ScannedFeature {
            id,
            path: path.to_path_buf(),
            offset,
            length: u64::try_from(line_bytes.len())
                .map_err(|_| import_error("NDJSON feature line length does not fit in u64"))?,
            bbox,
        });
    }

    Ok(SourceScan {
        path: path.to_path_buf(),
        metadata,
        vertices_offset: None,
        vertices_length: None,
        features,
    })
}

fn collect_layout_files(paths: &[PathBuf], suffix: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for root in paths {
        if root.is_file() {
            if root.to_string_lossy().ends_with(suffix) {
                files.push(root.clone());
            }
            continue;
        }

        for entry in WalkBuilder::new(root)
            .hidden(false)
            .follow_links(true)
            .build()
        {
            let entry = entry.map_err(|error| import_error(error.to_string()))?;
            if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                continue;
            }
            let path = entry.into_path();
            if path.to_string_lossy().ends_with(suffix) {
                files.push(path);
            }
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

fn scan_cityjson_source(path: &Path) -> Result<SourceScan> {
    let bytes = fs::read(path)?;
    let document: Value = serde_json::from_slice(&bytes)?;
    let metadata = cityjson_base_metadata(&document)?;
    let (scale, translate) = parse_ndjson_transform(&metadata)?;

    let cityobjects = document
        .get("CityObjects")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            import_error(format!(
                "CityJSON source {} is missing CityObjects",
                path.display()
            ))
        })?;
    let vertices_value = document.get("vertices").ok_or_else(|| {
        import_error(format!(
            "CityJSON source {} is missing vertices",
            path.display()
        ))
    })?;
    let vertices: Vec<[i64; 3]> = serde_json::from_value(vertices_value.clone())?;
    let (vertices_offset, vertices_length) = top_level_value_range(&bytes, "vertices")?;
    let cityobject_ranges = cityobject_entry_ranges(&bytes)?
        .into_iter()
        .map(|(id, offset, length)| (id, (offset, length)))
        .collect::<HashMap<_, _>>();

    let mut features = Vec::with_capacity(cityobjects.len());
    for (id, _object) in cityobjects {
        let (offset, length) = cityobject_ranges.get(id).copied().ok_or_else(|| {
            import_error(format!(
                "CityObject fragment for {id} could not be located in {}",
                path.display()
            ))
        })?;
        let mut referenced_vertices = BTreeSet::new();
        let mut visited = BTreeSet::new();
        collect_cityjson_object_vertex_indices(
            id,
            cityobjects,
            &mut referenced_vertices,
            &mut visited,
        )?;
        if referenced_vertices.is_empty() {
            return Err(import_error(format!(
                "CityObject {id} in {} does not reference any vertices",
                path.display()
            )));
        }
        let bbox = bbox_from_vertices(&vertices, &referenced_vertices, scale, translate)?;
        features.push(ScannedFeature {
            id: id.clone(),
            path: path.to_path_buf(),
            offset,
            length,
            bbox,
        });
    }

    Ok(SourceScan {
        path: path.to_path_buf(),
        metadata,
        vertices_offset: Some(vertices_offset),
        vertices_length: Some(vertices_length),
        features,
    })
}

fn cityjson_base_metadata(document: &Value) -> Result<Meta> {
    let mut metadata = document.clone();
    let root = metadata
        .as_object_mut()
        .ok_or_else(|| import_error("CityJSON document root must be a JSON object"))?;
    root.insert("CityObjects".to_owned(), Value::Object(Map::new()));
    root.insert("vertices".to_owned(), Value::Array(Vec::new()));
    Ok(metadata)
}

fn collect_cityjson_object_vertex_indices(
    object_id: &str,
    cityobjects: &Map<String, Value>,
    indices: &mut BTreeSet<usize>,
    visited: &mut BTreeSet<String>,
) -> Result<()> {
    if !visited.insert(object_id.to_owned()) {
        return Ok(());
    }

    let object = cityobjects.get(object_id).ok_or_else(|| {
        import_error(format!(
            "CityJSON source is missing referenced CityObject {object_id}"
        ))
    })?;
    collect_object_vertex_indices(object, indices)?;

    if let Some(children) = object.get("children").and_then(Value::as_array) {
        for child in children {
            let Some(child_id) = child.as_str() else {
                return Err(import_error(
                    "CityObject children must be string identifiers",
                ));
            };
            if cityobjects.contains_key(child_id) {
                collect_cityjson_object_vertex_indices(child_id, cityobjects, indices, visited)?;
            }
        }
    }

    Ok(())
}

fn collect_object_vertex_indices(object: &Value, indices: &mut BTreeSet<usize>) -> Result<()> {
    if let Some(geometries) = object.get("geometry").and_then(Value::as_array) {
        for geometry in geometries {
            if let Some(boundaries) = geometry.get("boundaries") {
                collect_vertex_indices(boundaries, indices)?;
            }
        }
    }
    Ok(())
}

fn top_level_value_range(bytes: &[u8], key: &str) -> Result<(u64, u64)> {
    let key_start = find_json_key(bytes, key)
        .ok_or_else(|| import_error(format!("top-level key {key} could not be located")))?;
    let mut cursor = skip_json_whitespace(bytes, key_start + key.len() + 2);
    if bytes.get(cursor) != Some(&b':') {
        return Err(import_error(format!(
            "top-level key {key} is missing a value separator"
        )));
    }
    cursor = skip_json_whitespace(bytes, cursor + 1);
    let value_end = json_value_end(bytes, cursor)?;
    Ok((
        u64::try_from(cursor).map_err(|_| import_error("value offset does not fit in u64"))?,
        u64::try_from(value_end - cursor)
            .map_err(|_| import_error("value length does not fit in u64"))?,
    ))
}

fn cityobject_entry_ranges(bytes: &[u8]) -> Result<Vec<(String, u64, u64)>> {
    let key_start = find_json_key(bytes, "CityObjects")
        .ok_or_else(|| import_error("top-level key CityObjects could not be located"))?;
    let mut cursor = skip_json_whitespace(bytes, key_start + "\"CityObjects\"".len());
    if bytes.get(cursor) != Some(&b':') {
        return Err(import_error("CityObjects key is missing a value separator"));
    }
    cursor = skip_json_whitespace(bytes, cursor + 1);
    if bytes.get(cursor) != Some(&b'{') {
        return Err(import_error("CityObjects must be a JSON object"));
    }
    cursor += 1;

    let mut entries = Vec::new();
    loop {
        cursor = skip_json_whitespace(bytes, cursor);
        match bytes.get(cursor) {
            Some(b'}') => break,
            Some(b'"') => {
                let entry_start = cursor;
                let (id, after_key) = parse_json_string(bytes, cursor)?;
                cursor = skip_json_whitespace(bytes, after_key);
                if bytes.get(cursor) != Some(&b':') {
                    return Err(import_error(
                        "CityObject entry is missing a value separator",
                    ));
                }
                cursor = skip_json_whitespace(bytes, cursor + 1);
                let value_end = json_value_end(bytes, cursor)?;
                let offset = u64::try_from(entry_start)
                    .map_err(|_| import_error("CityObject entry offset does not fit in u64"))?;
                let length = u64::try_from(value_end - entry_start)
                    .map_err(|_| import_error("CityObject entry length does not fit in u64"))?;
                entries.push((id, offset, length));
                cursor = skip_json_whitespace(bytes, value_end);
                match bytes.get(cursor) {
                    Some(b',') => cursor += 1,
                    Some(b'}') => break,
                    _ => {
                        return Err(import_error(
                            "CityObjects entries must be separated by commas",
                        ));
                    }
                }
            }
            _ => return Err(import_error("unexpected token inside CityObjects object")),
        }
    }

    Ok(entries)
}

fn find_json_key(bytes: &[u8], key: &str) -> Option<usize> {
    let needle = format!("\"{key}\"");
    bytes
        .windows(needle.len())
        .position(|window| window == needle.as_bytes())
}

fn skip_json_whitespace(bytes: &[u8], mut index: usize) -> usize {
    while bytes
        .get(index)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        index += 1;
    }
    index
}

fn parse_json_string(bytes: &[u8], start: usize) -> Result<(String, usize)> {
    let mut index = start + 1;
    let mut escaped = false;

    while let Some(byte) = bytes.get(index) {
        if escaped {
            escaped = false;
        } else if *byte == b'\\' {
            escaped = true;
        } else if *byte == b'"' {
            let end = index + 1;
            return Ok((serde_json::from_slice(&bytes[start..end])?, end));
        }
        index += 1;
    }

    Err(import_error("unterminated JSON string"))
}

fn json_value_end(bytes: &[u8], start: usize) -> Result<usize> {
    match bytes.get(start) {
        Some(b'{') => nested_json_end(bytes, start, b'{', b'}'),
        Some(b'[') => nested_json_end(bytes, start, b'[', b']'),
        Some(b'"') => parse_json_string(bytes, start).map(|(_, end)| end),
        Some(_) => {
            let mut end = start;
            while let Some(byte) = bytes.get(end) {
                if byte.is_ascii_whitespace() || matches!(*byte, b',' | b'}' | b']') {
                    break;
                }
                end += 1;
            }
            Ok(end)
        }
        None => Err(import_error("unexpected end of JSON input")),
    }
}

fn nested_json_end(bytes: &[u8], start: usize, open: u8, close: u8) -> Result<usize> {
    let mut depth = 0usize;
    let mut index = start;
    let mut in_string = false;
    let mut escaped = false;

    while let Some(byte) = bytes.get(index) {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
        } else if *byte == b'"' {
            in_string = true;
        } else if *byte == open {
            depth += 1;
        } else if *byte == close {
            depth -= 1;
            if depth == 0 {
                return Ok(index + 1);
            }
        }
        index += 1;
    }

    Err(import_error("unterminated JSON value"))
}

fn parse_ndjson_transform(metadata: &Value) -> Result<([f64; 3], [f64; 3])> {
    let transform = metadata
        .get("transform")
        .and_then(Value::as_object)
        .ok_or_else(|| import_error("NDJSON metadata is missing transform"))?;

    let scale = parse_vector3_f64(transform, "scale")?;
    let translate = parse_vector3_f64(transform, "translate")?;
    Ok((scale, translate))
}

fn feature_identifier(feature: &Value, label: &str) -> Result<String> {
    if let Some(id) = feature.get("id").and_then(Value::as_str) {
        return Ok(id.to_owned());
    }

    let cityobjects = feature
        .get("CityObjects")
        .and_then(Value::as_object)
        .ok_or_else(|| import_error(format!("{label} is missing CityObjects")))?;
    if cityobjects.len() == 1 {
        return cityobjects
            .keys()
            .next()
            .cloned()
            .ok_or_else(|| import_error(format!("{label} is missing a CityObject")));
    }
    Err(import_error(format!(
        "{label} is missing a top-level id and contains multiple CityObjects"
    )))
}

fn collect_feature_vertex_indices(feature: &Value, vertex_count: usize) -> Result<BTreeSet<usize>> {
    let mut indices = BTreeSet::new();
    let cityobjects = feature
        .get("CityObjects")
        .and_then(Value::as_object)
        .ok_or_else(|| import_error("feature package is missing CityObjects"))?;

    for object in cityobjects.values() {
        collect_object_vertex_indices(object, &mut indices)?;
    }

    if indices.is_empty() {
        indices.extend(0..vertex_count);
    }

    Ok(indices)
}

fn parse_vector3_f64(object: &Map<String, Value>, key: &str) -> Result<[f64; 3]> {
    let array = object
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| import_error(format!("transform is missing {key}")))?;
    if array.len() != 3 {
        return Err(import_error(format!(
            "transform {key} must contain three values"
        )));
    }

    Ok([
        array[0]
            .as_f64()
            .ok_or_else(|| import_error(format!("transform {key}[0] must be numeric")))?,
        array[1]
            .as_f64()
            .ok_or_else(|| import_error(format!("transform {key}[1] must be numeric")))?,
        array[2]
            .as_f64()
            .ok_or_else(|| import_error(format!("transform {key}[2] must be numeric")))?,
    ])
}

fn parse_ndjson_feature_bbox(
    feature: &Value,
    scale: [f64; 3],
    translate: [f64; 3],
) -> Result<(String, BBox)> {
    let id = feature_identifier(feature, "NDJSON feature")?;
    let vertices = feature
        .get("vertices")
        .ok_or_else(|| import_error("NDJSON feature is missing vertices"))?;
    let vertices: Vec<[i64; 3]> = serde_json::from_value(vertices.clone())?;
    let referenced_vertices = collect_feature_vertex_indices(feature, vertices.len())?;
    let bbox = bbox_from_vertices(&vertices, &referenced_vertices, scale, translate)?;
    Ok((id, bbox))
}

fn bbox_from_vertices(
    vertices: &[[i64; 3]],
    referenced_vertices: &BTreeSet<usize>,
    scale: [f64; 3],
    translate: [f64; 3],
) -> Result<BBox> {
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for &index in referenced_vertices {
        let vertex = vertices.get(index).copied().ok_or_else(|| {
            import_error(format!(
                "vertex index {index} is outside the NDJSON feature vertex array"
            ))
        })?;
        let x = translate[0] + scale[0] * vertex[0] as f64;
        let y = translate[1] + scale[1] * vertex[1] as f64;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return Err(import_error("NDJSON feature bbox could not be computed"));
    }

    Ok(BBox {
        min_x,
        max_x,
        min_y,
        max_y,
    })
}

fn line_spans(bytes: &[u8]) -> Vec<(u64, &[u8])> {
    let mut spans = Vec::new();
    let mut offset = 0u64;

    for chunk in bytes.split_inclusive(|byte| *byte == b'\n') {
        spans.push((offset, trim_line_ending(chunk)));
        offset += u64::try_from(chunk.len()).expect("line chunk length fits in u64");
    }

    if bytes.is_empty() {
        spans.clear();
    }

    spans
}

fn trim_line_ending(bytes: &[u8]) -> &[u8] {
    let mut end = bytes.len();
    while end > 0 && (bytes[end - 1] == b'\n' || bytes[end - 1] == b'\r') {
        end -= 1;
    }
    &bytes[..end]
}

fn sqlite_result<T>(result: rusqlite::Result<T>) -> Result<T> {
    result.map_err(|value| Error::Import(value.to_string()))
}

fn u64_to_i64(value: u64) -> rusqlite::Result<i64> {
    i64::try_from(value).map_err(|_| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(import_error(format!(
            "value {value} does not fit in SQLite integer storage"
        ))))
    })
}

fn i64_to_u64(value: i64) -> rusqlite::Result<u64> {
    u64::try_from(value).map_err(|_| {
        rusqlite::Error::ToSqlConversionFailure(Box::new(import_error(format!(
            "value {value} is not representable as u64"
        ))))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn cityjson_read_one_localizes_vertices_and_preserves_base_root_members() {
        let selected_id = "building-1";
        let selected_object = serde_json::json!({
            "type": "Building",
            "children": ["building-1-part"],
            "geometry": [{
                "type": "MultiSurface",
                "lod": "0",
                "boundaries": [[[2, 7, 5]]]
            }]
        });
        let other_object = serde_json::json!({
            "type": "Building",
            "geometry": [{
                "type": "MultiSurface",
                "lod": "0",
                "boundaries": [[[0, 1, 3]]]
            }]
        });
        let vertices = serde_json::json!([
            [100, 0, 0],
            [101, 0, 0],
            [0, 0, 0],
            [102, 0, 0],
            [103, 0, 0],
            [2, 0, 0],
            [104, 0, 0],
            [1, 0, 0]
        ]);
        let document = serde_json::json!({
            "type": "CityJSON",
            "version": "2.0",
            "transform": {
                "scale": [0.5, 0.5, 0.5],
                "translate": [10.0, 20.0, 30.0]
            },
            "metadata": {
                "title": "unit-test-fixture"
            },
            "CityObjects": {
                selected_id: selected_object.clone(),
                "other-object": other_object
            },
            "vertices": vertices.clone()
        });
        let document_bytes = serde_json::to_vec(&document).expect("fixture JSON");
        let base_document = cityjson_base_metadata(&document).expect("base CityJSON metadata");
        let object_fragment = object_entry_fragment(selected_id, &selected_object);
        let vertices_fragment = serde_json::to_vec(&vertices).expect("vertices fragment");
        let loc = FeatureLocation {
            source_id: 0,
            source_path: write_temp_cityjson(&document_bytes),
            offset: find_subslice(&document_bytes, &object_fragment)
                .expect("selected object offset") as u64,
            length: object_fragment.len() as u64,
            vertices_offset: Some(
                find_subslice(&document_bytes, &vertices_fragment).expect("vertices offset") as u64,
            ),
            vertices_length: Some(vertices_fragment.len() as u64),
        };

        let backend = CityJsonBackend::new(vec![loc.source_path.clone()]);
        let model = backend
            .read_one(&loc, Arc::new(base_document))
            .expect("CityJSON read should succeed");
        let output: Value =
            serde_json::from_str(&cjlib::json::to_string(&model).expect("serialize result"))
                .expect("valid output JSON");

        let cityobjects = output["CityObjects"]
            .as_object()
            .expect("result CityObjects must be an object");
        assert_eq!(cityobjects.len(), 1);
        assert!(cityobjects.contains_key(selected_id));
        assert_eq!(output["transform"], document["transform"]);
        assert_eq!(output["metadata"], document["metadata"]);
        assert!(cityobjects[selected_id].get("children").is_none());
        assert_eq!(
            output["vertices"],
            serde_json::json!([[0, 0, 0], [2, 0, 0], [1, 0, 0]])
        );
        assert_eq!(
            cityobjects[selected_id]["geometry"][0]["boundaries"],
            serde_json::json!([[[0, 2, 1]]])
        );
    }

    #[test]
    fn feature_parts_builder_drops_dangling_parent_links() {
        let parts = build_single_object_feature_parts(
            "building-1-part",
            serde_json::json!({
                "type": "BuildingPart",
                "parents": ["building-1"],
                "geometry": [{
                    "type": "MultiSurface",
                    "lod": "0",
                    "boundaries": [[[5, 9, 7]]]
                }]
            }),
            &[
                [100, 0, 0],
                [101, 0, 0],
                [102, 0, 0],
                [103, 0, 0],
                [104, 0, 0],
                [0, 0, 0],
                [105, 0, 0],
                [2, 0, 0],
                [106, 0, 0],
                [1, 0, 0],
            ],
        )
        .expect("feature parts should build");
        let object: Value =
            serde_json::from_str(parts.object_json.get()).expect("valid object JSON");

        assert_eq!(parts.object_id, "building-1-part");
        assert!(object.get("parents").is_none());
        assert_eq!(parts.vertices, vec![[0, 0, 0], [2, 0, 0], [1, 0, 0]]);
        assert_eq!(
            object["geometry"][0]["boundaries"],
            serde_json::json!([[[0, 2, 1]]])
        );
    }

    #[test]
    fn ndjson_backend_scan_and_index_lookup_roundtrip() {
        let metadata = serde_json::json!({
            "type": "CityJSON",
            "version": "2.0",
            "transform": {
                "scale": [1.0, 1.0, 1.0],
                "translate": [0.0, 0.0, 0.0]
            }
        });
        let feature = serde_json::json!({
            "type": "CityJSONFeature",
            "id": "ndjson-test-feature",
            "CityObjects": {
                "ndjson-test-feature": {
                    "type": "Building",
                    "geometry": [{
                        "type": "MultiSurface",
                        "lod": "1.0",
                        "boundaries": [[[0, 1, 2]]]
                    }]
                }
            },
            "vertices": [
                [0, 0, 0],
                [1, 0, 0],
                [0, 1, 0]
            ]
        });
        let ndjson_path = write_temp_ndjson(&metadata, &feature);
        let backend = NdjsonBackend {
            paths: vec![ndjson_path.clone()],
        };
        let scans = backend.scan().expect("NDJSON scan should succeed");
        assert_eq!(scans.len(), 1);
        assert_eq!(scans[0].features.len(), 1);
        assert_eq!(scans[0].features[0].id, "ndjson-test-feature");

        let index_path = write_temp_index_path();
        let mut index = Index::open(&index_path).expect("SQLite index should open");
        index.rebuild(&scans).expect("NDJSON scan should index");

        let by_id = index
            .lookup_id("ndjson-test-feature")
            .expect("id lookup should succeed");
        assert!(
            by_id.is_some(),
            "indexed feature should be addressable by id"
        );

        let hits = index
            .lookup_bbox(&BBox {
                min_x: -1.0,
                max_x: 1.0,
                min_y: -1.0,
                max_y: 1.0,
            })
            .expect("bbox lookup should succeed");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].source_path, ndjson_path);
    }

    #[test]
    fn read_exact_range_reads_only_the_requested_span() {
        let path = write_temp_bytes(b"abcdefghij");

        let bytes = read_exact_range(&path, 3, 4).expect("range read should succeed");

        assert_eq!(bytes, b"defg");
    }

    #[test]
    fn read_exact_range_rejects_short_reads() {
        let path = write_temp_bytes(b"abc");

        let error = read_exact_range(&path, 2, 4).expect_err("range read should fail");

        assert!(error.to_string().contains("short read"));
    }

    #[test]
    fn read_exact_range_rejects_oversized_lengths() {
        let path = write_temp_bytes(b"abc");

        let error = read_exact_range(&path, 0, u64::MAX).expect_err("range read should fail");

        assert!(
            error
                .to_string()
                .contains("exceeds the supported buffer size")
        );
    }

    #[test]
    fn feature_files_metadata_resolution_prefers_nearest_ancestor() {
        let root = PathBuf::from("/data/root");
        let mut metadata_by_dir = BTreeMap::new();
        metadata_by_dir.insert(root.clone(), root.join("metadata.json"));
        metadata_by_dir.insert(
            root.join("features/8"),
            root.join("features/8/metadata.json"),
        );

        let feature_path = root.join("features/8/296/592/sample.city.jsonl");
        let resolved = resolve_feature_metadata_path(&root, &feature_path, &metadata_by_dir)
            .expect("metadata must resolve");

        assert_eq!(resolved, root.join("features/8/metadata.json"));
    }

    fn object_entry_fragment(object_id: &str, object: &Value) -> Vec<u8> {
        let mut map = Map::new();
        map.insert(object_id.to_owned(), object.clone());
        let serialized = serde_json::to_vec(&Value::Object(map)).expect("object entry");
        serialized[1..serialized.len() - 1].to_vec()
    }

    fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }

    fn write_temp_cityjson(bytes: &[u8]) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("cjindex-cityjson-read-one-{unique}.json"));
        fs::write(&path, bytes).expect("write temp cityjson");
        path
    }

    fn write_temp_ndjson(metadata: &Value, feature: &Value) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("cjindex-ndjson-{unique}.jsonl"));
        let contents = format!(
            "{}\n{}\n",
            serde_json::to_string(metadata).expect("metadata JSON"),
            serde_json::to_string(feature).expect("feature JSON")
        );
        fs::write(&path, contents).expect("write temp ndjson");
        path
    }

    fn write_temp_index_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("cjindex-ndjson-{unique}.sqlite"));
        if path.exists() {
            fs::remove_file(&path).expect("remove temp sqlite");
        }
        path
    }

    fn write_temp_bytes(bytes: &[u8]) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("cjindex-range-read-{unique}.bin"));
        fs::write(&path, bytes).expect("write temp bytes");
        path
    }
}
