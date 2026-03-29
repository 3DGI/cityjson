use std::collections::BTreeMap;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use cjindex::{BBox, CityIndex, StorageLayout};
use cjlib::{Error, Result};
use criterion::{BatchSize, Criterion};
use serde_json::Value;
use walkdir::WalkDir;

#[allow(dead_code)]
#[path = "../tests/common/data_prep.rs"]
mod data_prep;

const GET_WORKLOAD_COUNT: usize = 1_000;
const QUERY_WORKLOAD_COUNT: usize = 10;
const QUERY_TILE_MIN_FEATURES: usize = 100;
const QUERY_TILE_SAMPLE_SIZE: usize = 128;
const WORKLOAD_SHUFFLE_SEED: u64 = 0x6a09e667f3bcc909;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum LayoutKind {
    FeatureFiles,
    CityJson,
    Ndjson,
}

pub fn bench_layout(c: &mut Criterion, kind: LayoutKind) {
    let fixtures = fixtures();
    let layout_root = fixtures.layout_root(kind).to_path_buf();
    let populated_index = build_index(kind, &layout_root);
    let label = kind.label();

    c.bench_function(&format!("{label}_reindex"), |b| {
        let reindex_root = layout_root.clone();
        b.iter_batched_ref(
            || empty_index(kind, &reindex_root),
            |index| {
                index.reindex().expect("reindex should succeed");
                black_box(index.metadata().expect("metadata should load"));
            },
            BatchSize::LargeInput,
        );
    });

    c.bench_function(&format!("{label}_get"), |b| {
        b.iter(|| {
            for feature_id in fixtures.get_ids.iter() {
                let model = populated_index
                    .get(black_box(feature_id.as_str()))
                    .expect("get should succeed")
                    .expect("feature should exist");
                black_box(model);
            }
        });
    });

    c.bench_function(&format!("{label}_query"), |b| {
        b.iter(|| {
            for bbox in fixtures.query_bboxes.iter() {
                let models = populated_index
                    .query(black_box(bbox))
                    .expect("query should succeed");
                black_box(models);
            }
        });
    });

    c.bench_function(&format!("{label}_query_iter"), |b| {
        b.iter(|| {
            for bbox in fixtures.query_bboxes.iter() {
                let models = populated_index
                    .query_iter(black_box(bbox))
                    .expect("query_iter should build")
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .expect("query_iter should succeed");
                black_box(models);
            }
        });
    });

    c.bench_function(&format!("{label}_metadata"), |b| {
        b.iter(|| {
            let metadata = populated_index.metadata().expect("metadata should succeed");
            black_box(metadata);
        });
    });
}

impl LayoutKind {
    fn label(self) -> &'static str {
        match self {
            Self::FeatureFiles => "feature_files",
            Self::CityJson => "cityjson",
            Self::Ndjson => "ndjson",
        }
    }

    fn storage_layout(self, root: &Path) -> StorageLayout {
        match self {
            Self::FeatureFiles => StorageLayout::FeatureFiles {
                root: root.to_path_buf(),
                metadata_glob: "**/metadata.json".to_owned(),
                feature_glob: "**/*.city.jsonl".to_owned(),
            },
            Self::CityJson => StorageLayout::CityJson {
                paths: vec![root.to_path_buf()],
            },
            Self::Ndjson => StorageLayout::Ndjson {
                paths: vec![root.to_path_buf()],
            },
        }
    }
}

struct BenchFixtures {
    datasets: data_prep::PreparedDatasets,
    get_ids: Vec<String>,
    query_bboxes: Vec<BBox>,
}

impl BenchFixtures {
    fn layout_root(&self, kind: LayoutKind) -> &Path {
        match kind {
            LayoutKind::FeatureFiles => self.datasets.feature_files.as_path(),
            LayoutKind::CityJson => self.datasets.cityjson.as_path(),
            LayoutKind::Ndjson => self.datasets.ndjson.as_path(),
        }
    }
}

fn fixtures() -> &'static BenchFixtures {
    static FIXTURES: OnceLock<BenchFixtures> = OnceLock::new();
    FIXTURES.get_or_init(|| prepare_bench_fixtures().expect("benchmark fixtures should prepare"))
}

fn prepare_bench_fixtures() -> Result<BenchFixtures> {
    let datasets = prepared_datasets()?;
    let feature_records = collect_feature_records(&datasets.feature_files)?;
    let get_ids = build_get_workload(&feature_records)?;
    let query_bboxes = build_query_workload(&feature_records)?;

    validate_workloads(&datasets, &get_ids, &query_bboxes)?;

    Ok(BenchFixtures {
        datasets,
        get_ids,
        query_bboxes,
    })
}

fn prepared_datasets() -> Result<data_prep::PreparedDatasets> {
    let output_root = Path::new(data_prep::DEFAULT_OUTPUT_ROOT);
    let feature_files_root = output_root.join("feature-files");
    let cityjson_root = output_root.join("cityjson");
    let ndjson_root = output_root.join("ndjson");

    if feature_files_root.exists() && cityjson_root.exists() && ndjson_root.exists() {
        return Ok(data_prep::PreparedDatasets {
            feature_files: feature_files_root,
            cityjson: cityjson_root,
            ndjson: ndjson_root,
        });
    }

    data_prep::prepare_test_sets(Path::new(data_prep::DEFAULT_INPUT_ROOT), output_root)
}

fn build_index(kind: LayoutKind, root: &Path) -> CityIndex {
    let index_path = unique_temp_file(&format!("cjindex-bench-{}-build", kind.label()), "sqlite");
    let mut index = CityIndex::open(kind.storage_layout(root), &index_path)
        .expect("benchmark index should open");
    index.reindex().expect("benchmark index should reindex");
    index
}

fn empty_index(kind: LayoutKind, root: &Path) -> CityIndex {
    let index_path = unique_temp_file(&format!("cjindex-bench-{}-empty", kind.label()), "sqlite");
    CityIndex::open(kind.storage_layout(root), &index_path).expect("benchmark index should open")
}

fn validate_workloads(
    datasets: &data_prep::PreparedDatasets,
    get_ids: &[String],
    query_bboxes: &[BBox],
) -> Result<()> {
    let layouts = [
        (
            LayoutKind::FeatureFiles,
            datasets.feature_files.as_path().to_path_buf(),
        ),
        (
            LayoutKind::CityJson,
            datasets.cityjson.as_path().to_path_buf(),
        ),
        (LayoutKind::Ndjson, datasets.ndjson.as_path().to_path_buf()),
    ];

    for (kind, root) in layouts {
        let index = build_index(kind, &root);

        for id in get_ids {
            let model = index
                .get(id)?
                .ok_or_else(|| Error::Import(format!("feature {id} should be indexed")))?;
            black_box(model);
        }

        for bbox in query_bboxes {
            let query_hits = index.query(bbox)?;
            if query_hits.is_empty() {
                return Err(Error::Import(format!(
                    "query workload bbox produced no hits for {}",
                    kind.label()
                )));
            }

            let iter_hits = index
                .query_iter(bbox)?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            if iter_hits.is_empty() {
                return Err(Error::Import(format!(
                    "query_iter workload bbox produced no hits for {}",
                    kind.label()
                )));
            }
        }
    }

    Ok(())
}

fn build_get_workload(feature_records: &[FeatureRecord]) -> Result<Vec<String>> {
    let mut ids = feature_records
        .iter()
        .map(|record| record.id.clone())
        .collect::<Vec<_>>();
    ids.sort();
    seeded_shuffle(&mut ids, WORKLOAD_SHUFFLE_SEED);

    if ids.len() < GET_WORKLOAD_COUNT {
        return Err(Error::Import(format!(
            "benchmark corpus only yielded {} feature ids, expected at least {}",
            ids.len(),
            GET_WORKLOAD_COUNT
        )));
    }

    ids.truncate(GET_WORKLOAD_COUNT);
    Ok(ids)
}

fn build_query_workload(feature_records: &[FeatureRecord]) -> Result<Vec<BBox>> {
    let mut by_tile: BTreeMap<PathBuf, Vec<&FeatureRecord>> = BTreeMap::new();
    for record in feature_records {
        by_tile.entry(record.tile.clone()).or_default().push(record);
    }

    let mut qualifying_tiles = by_tile
        .into_iter()
        .filter(|(_, records)| records.len() >= QUERY_TILE_MIN_FEATURES)
        .collect::<Vec<_>>();
    seeded_shuffle(
        &mut qualifying_tiles,
        WORKLOAD_SHUFFLE_SEED ^ 0x9e3779b97f4a7c15,
    );

    let mut bboxes = Vec::with_capacity(QUERY_WORKLOAD_COUNT);
    for (_tile, records) in qualifying_tiles.into_iter().take(QUERY_WORKLOAD_COUNT) {
        let mut selected_records = records;
        seeded_shuffle(
            &mut selected_records,
            WORKLOAD_SHUFFLE_SEED ^ 0xbf58476d1ce4e5b9,
        );

        let bbox = selected_records
            .into_iter()
            .take(QUERY_TILE_SAMPLE_SIZE)
            .map(|record| record.bbox)
            .reduce(union_bbox)
            .ok_or_else(|| Error::Import("query tile selection produced no bbox".into()))?;
        bboxes.push(bbox);
    }

    if bboxes.len() < QUERY_WORKLOAD_COUNT {
        return Err(Error::Import(format!(
            "benchmark corpus only yielded {} qualifying tiles, expected {}",
            bboxes.len(),
            QUERY_WORKLOAD_COUNT
        )));
    }

    Ok(bboxes)
}

fn collect_feature_records(layout_root: &Path) -> Result<Vec<FeatureRecord>> {
    let feature_root = layout_root.join("features");
    let mut metadata_files = Vec::new();
    let mut feature_files = Vec::new();

    for entry in WalkDir::new(&feature_root).sort_by_file_name() {
        let entry = entry.map_err(|error| Error::Import(error.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.into_path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }
        if fs::metadata(&path)
            .map(|meta| meta.len() == 0)
            .unwrap_or(true)
        {
            continue;
        }

        if path.file_name().and_then(|name| name.to_str()) == Some("metadata.json") {
            metadata_files.push(path);
            continue;
        }

        feature_files.push(path);
    }

    metadata_files.sort();
    feature_files.sort();

    if metadata_files.is_empty() {
        return Err(Error::Import(format!(
            "prepared feature-files root {} did not yield any metadata files",
            feature_root.display()
        )));
    }

    let mut metadata_by_dir = BTreeMap::new();
    let mut metadata_cache = BTreeMap::new();
    for metadata_path in metadata_files {
        let parent = metadata_path.parent().unwrap_or(&feature_root).to_path_buf();
        metadata_by_dir.insert(parent, metadata_path.clone());
        metadata_cache.insert(metadata_path.clone(), read_json(&metadata_path)?);
    }

    let mut records = Vec::new();
    for path in feature_files {
        let tile = path
            .strip_prefix(&feature_root)
            .map_err(|_| {
                Error::Import("feature path is outside the prepared feature-files root".into())
            })?
            .parent()
            .ok_or_else(|| Error::Import("feature file is missing a tile directory".into()))?
            .to_path_buf();
        let metadata_path = resolve_feature_metadata_path(&feature_root, &path, &metadata_by_dir)
            .ok_or_else(|| {
                Error::Import(format!(
                    "no ancestor metadata file found for feature {}",
                    path.display()
                ))
            })?;
        let metadata = metadata_cache
            .get(&metadata_path)
            .ok_or_else(|| {
                Error::Import(format!(
                    "metadata {} was not cached for feature {}",
                    metadata_path.display(),
                    path.display()
                ))
            })?;
        let value: Value = read_json(&path)?;
        let id = feature_id_from_value(&value, &format!("feature file {}", path.display()))?;
        let bbox = feature_bbox(&value, metadata)?;
        records.push(FeatureRecord { id, tile, bbox });
    }

    if records.is_empty() {
        return Err(Error::Import(format!(
            "prepared feature-files root {} did not yield any features",
            feature_root.display()
        )));
    }

    Ok(records)
}

fn union_bbox(a: BBox, b: BBox) -> BBox {
    BBox {
        min_x: a.min_x.min(b.min_x),
        max_x: a.max_x.max(b.max_x),
        min_y: a.min_y.min(b.min_y),
        max_y: a.max_y.max(b.max_y),
    }
}

fn seeded_shuffle<T>(items: &mut [T], seed: u64) {
    if items.len() < 2 {
        return;
    }

    let mut state = seed;
    for index in (1..items.len()).rev() {
        state = state.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = state;
        value ^= value >> 30;
        value = value.wrapping_mul(0xbf58476d1ce4e5b9);
        value ^= value >> 27;
        value = value.wrapping_mul(0x94d049bb133111eb);
        value ^= value >> 31;
        let swap_with = (value as usize) % (index + 1);
        items.swap(index, swap_with);
    }
}

fn feature_bbox(feature: &Value, metadata: &Value) -> Result<BBox> {
    let vertices = feature
        .get("vertices")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Import("feature JSON is missing vertices".into()))?;
    let transform = metadata
        .get("transform")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::Import("feature metadata is missing transform".into()))?;
    let scale = parse_vector3_f64(transform, "scale")?;
    let translate = parse_vector3_f64(transform, "translate")?;

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for vertex in vertices {
        let coords = vertex
            .as_array()
            .ok_or_else(|| Error::Import("vertex must be an array".into()))?;
        if coords.len() != 3 {
            return Err(Error::Import("vertex must have three coordinates".into()));
        }

        let x = translate[0] + scale[0] * value_as_f64(&coords[0])?;
        let y = translate[1] + scale[1] * value_as_f64(&coords[1])?;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }

    if !min_x.is_finite() || !max_x.is_finite() || !min_y.is_finite() || !max_y.is_finite() {
        return Err(Error::Import(
            "could not compute a finite bbox from the feature".into(),
        ));
    }

    Ok(BBox {
        min_x,
        max_x,
        min_y,
        max_y,
    })
}

fn parse_vector3_f64(object: &serde_json::Map<String, Value>, key: &str) -> Result<[f64; 3]> {
    let array = object
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Import(format!("transform is missing {key}")))?;
    if array.len() != 3 {
        return Err(Error::Import(format!(
            "transform {key} must contain three values"
        )));
    }

    Ok([
        value_as_f64(&array[0])?,
        value_as_f64(&array[1])?,
        value_as_f64(&array[2])?,
    ])
}

fn value_as_f64(value: &Value) -> Result<f64> {
    value
        .as_f64()
        .ok_or_else(|| Error::Import("expected a numeric value".into()))
}

fn read_json(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).map_err(|error| Error::Import(error.to_string()))?;
    Ok(serde_json::from_slice(&bytes)?)
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

fn unique_temp_file(label: &str, suffix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after the unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("cjindex-{label}-{unique}.{suffix}"));
    if path.exists() {
        fs::remove_file(&path).expect("benchmark temp file should be removable");
    }
    path
}

fn feature_id_from_value(feature: &Value, label: &str) -> Result<String> {
    if let Some(id) = feature.get("id").and_then(Value::as_str) {
        return Ok(id.to_owned());
    }

    let cityobjects = feature
        .get("CityObjects")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::Import(format!("{label} is missing CityObjects")))?;
    if cityobjects.len() == 1 {
        return cityobjects
            .keys()
            .next()
            .cloned()
            .ok_or_else(|| Error::Import(format!("{label} is missing a CityObject")));
    }

    Err(Error::Import(format!(
        "{label} is missing a top-level id and contains multiple CityObjects"
    )))
}

struct FeatureRecord {
    id: String,
    tile: PathBuf,
    bbox: BBox,
}
