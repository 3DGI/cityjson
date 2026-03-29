use std::collections::BTreeMap;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use cjindex::{BBox, CityIndex, StorageLayout};
use cjlib::{CityModel, Error, Result};
use criterion::{BatchSize, Criterion};
use serde_json::Value;
use walkdir::WalkDir;

#[allow(dead_code)]
#[path = "../tests/common/data_prep.rs"]
mod data_prep;

const WORKLOAD_FEATURE_COUNT: usize = 1_000;

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
    let feature_id = fixtures.get_feature_id.as_str();
    let query_bbox = fixtures.query_bbox;
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
            let model = populated_index
                .get(black_box(feature_id))
                .expect("get should succeed")
                .expect("feature should exist");
            black_box(model);
        });
    });

    c.bench_function(&format!("{label}_query"), |b| {
        b.iter(|| {
            let models = populated_index
                .query(black_box(&query_bbox))
                .expect("query should succeed");
            black_box(models);
        });
    });

    c.bench_function(&format!("{label}_query_iter"), |b| {
        b.iter(|| {
            let models = populated_index
                .query_iter(black_box(&query_bbox))
                .expect("query_iter should build")
                .collect::<std::result::Result<Vec<_>, _>>()
                .expect("query_iter should succeed");
            black_box(models);
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
    get_feature_id: String,
    query_bbox: BBox,
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
    let selected_files = selected_feature_files(&datasets.feature_files, WORKLOAD_FEATURE_COUNT)?;
    let feature_ids = selected_files
        .iter()
        .map(|path| feature_id_from_file(path))
        .collect::<Result<Vec<_>>>()?;
    let get_feature_id = feature_ids.first().cloned().ok_or_else(|| {
        Error::Import("benchmark workload must contain at least one feature".into())
    })?;
    let query_bbox = build_query_bbox(&datasets.feature_files, &feature_ids)?;

    Ok(BenchFixtures {
        datasets,
        get_feature_id,
        query_bbox,
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

fn build_query_bbox(root: &Path, feature_ids: &[String]) -> Result<BBox> {
    let index = build_index(LayoutKind::FeatureFiles, root);
    let mut bbox = None;

    for id in feature_ids {
        let model = index
            .get(id)?
            .ok_or_else(|| Error::Import(format!("feature {id} should be indexed")))?;
        let model_bbox = bbox_for_model(&model)?;
        bbox = Some(match bbox {
            Some(current) => union_bbox(current, model_bbox),
            None => model_bbox,
        });
    }

    bbox.ok_or_else(|| Error::Import("benchmark subset produced no bbox".into()))
}

fn union_bbox(a: BBox, b: BBox) -> BBox {
    BBox {
        min_x: a.min_x.min(b.min_x),
        max_x: a.max_x.max(b.max_x),
        min_y: a.min_y.min(b.min_y),
        max_y: a.max_y.max(b.max_y),
    }
}

fn bbox_for_model(model: &CityModel) -> Result<BBox> {
    let value: Value = serde_json::from_str(&cjlib::json::to_string(model)?)?;
    let vertices = value
        .get("vertices")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Import("model JSON is missing vertices".into()))?;
    let transform = value
        .get("transform")
        .and_then(Value::as_object)
        .ok_or_else(|| Error::Import("model JSON is missing transform".into()))?;
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
            "could not compute a finite bbox from the model".into(),
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

fn selected_feature_files(layout_root: &Path, limit: usize) -> Result<Vec<PathBuf>> {
    let feature_root = layout_root.join("features");
    let mut by_tile: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();

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
        let tile = path
            .strip_prefix(&feature_root)
            .map_err(|_| {
                Error::Import("feature path is outside the prepared feature-files root".into())
            })?
            .parent()
            .ok_or_else(|| Error::Import("feature file is missing a tile directory".into()))?
            .to_path_buf();
        by_tile.entry(tile).or_default().push(path);
    }

    for files in by_tile.values() {
        if files.len() >= limit {
            return Ok(files.iter().take(limit).cloned().collect());
        }
    }

    let mut selected = Vec::with_capacity(limit);
    for files in by_tile.into_values() {
        for path in files {
            selected.push(path);
            if selected.len() == limit {
                return Ok(selected);
            }
        }
    }

    Err(Error::Import(format!(
        "prepared feature-files root only yielded {} readable features, expected at least {limit}",
        selected.len()
    )))
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

fn feature_id_from_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|error| Error::Import(error.to_string()))?;
    let value: Value = serde_json::from_slice(&bytes)?;
    feature_id_from_value(&value, &format!("feature file {}", path.display()))
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
