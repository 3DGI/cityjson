use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use cjlib::{CityModel, Error, Result};
use globset::GlobMatcher;
use lru::LruCache;
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
            index: Index::open(index_path),
            backend,
        })
    }

    /// Rebuilds the index from the configured backend.
    ///
    /// # Errors
    ///
    /// Returns an error if backend scanning or index population fails.
    pub fn reindex(&mut self) -> Result<()> {
        let _ = self.backend.scan()?;
        todo!("index population is not scaffolded yet")
    }

    /// Returns a `CityJSON` feature by id.
    ///
    /// # Errors
    ///
    /// Returns an error if lookup fails.
    pub fn get(&self, _id: &str) -> Result<Option<CityModel>> {
        todo!("id lookup is not scaffolded yet")
    }

    /// Returns every feature intersecting the given bounding box.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub fn query(&self, _bbox: &BBox) -> Result<Vec<CityModel>> {
        todo!("bbox query is not scaffolded yet")
    }

    /// Returns an iterator over features intersecting the given bounding box.
    ///
    /// # Errors
    ///
    /// Returns an error if the iterator cannot be constructed.
    pub fn query_iter(&self, _bbox: &BBox) -> Result<impl Iterator<Item = Result<CityModel>> + '_> {
        Ok(std::iter::empty())
    }

    /// Returns cached metadata entries.
    ///
    /// # Errors
    ///
    /// Returns an error if metadata lookup fails.
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
    fn open(path: &Path) -> Self {
        let _ = path;
        Self {
            _conn: None,
            metadata_cache: HashMap::new(),
        }
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

    fn clear(&mut self) {
        self.metadata_cache.clear();
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

    fn load_shared_vertices(
        &self,
        source_path: &Path,
        base_document_bytes: &[u8],
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

        let vertices = Arc::new(parse_vertices_fragment(slice_range(
            base_document_bytes,
            offset,
            length,
            "shared vertices",
        )?)?);
        cache.put(source_path.to_path_buf(), Arc::clone(&vertices));
        Ok(vertices)
    }
}

impl StorageBackend for CityJsonBackend {
    fn scan(&self) -> Result<Vec<SourceScan>> {
        let _ = (&self.paths, &self.vertices_cache);
        todo!("CityJSON scanning is not scaffolded yet")
    }

    fn read_one(&self, loc: &FeatureLocation) -> Result<CityModel> {
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

        let base_document_bytes = fs::read(&loc.source_path)?;
        let object_fragment = slice_range(
            &base_document_bytes,
            loc.offset,
            loc.length,
            "CityObject entry",
        )?;
        let (object_id, object_value) = parse_cityobject_entry(object_fragment)?;
        let shared_vertices = self.load_shared_vertices(
            &loc.source_path,
            &base_document_bytes,
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
        let _ = (&self.root, &self.metadata_glob, &self.feature_glob);
        todo!("feature-tree scanning is not scaffolded yet")
    }

    fn read_one(&self, loc: &FeatureLocation) -> Result<CityModel> {
        let _ = loc;
        todo!("feature-file read is not scaffolded yet")
    }
}

fn slice_range<'a>(bytes: &'a [u8], offset: u64, length: u64, label: &str) -> Result<&'a [u8]> {
    let start = usize::try_from(offset)
        .map_err(|_| import_error(format!("{label} offset does not fit in memory")))?;
    let len = usize::try_from(length)
        .map_err(|_| import_error(format!("{label} length does not fit in memory")))?;
    let end = start
        .checked_add(len)
        .ok_or_else(|| import_error(format!("{label} range overflows")))?;
    bytes
        .get(start..end)
        .ok_or_else(|| import_error(format!("{label} range is outside the source document")))
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
            .read_one(&loc)
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
}
