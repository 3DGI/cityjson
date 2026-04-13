use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use cityjson_lib::{Error, Result};
use ignore::WalkBuilder;
use serde_json::Value;

use crate::BBox;

pub const GET_WORKLOAD_COUNT: usize = 1_000;
pub const QUERY_BATCH_COUNT: usize = 10;
pub const QUERY_TILE_SAMPLE_SIZE: usize = 128;
pub const WORKLOAD_SHUFFLE_SEED: u64 = 0x6a09_e667_f3bc_c909;

#[derive(Clone, Debug)]
pub struct FeatureRecord {
    pub id: String,
    pub tile: PathBuf,
    pub bbox: BBox,
}

#[derive(Clone, Debug)]
pub struct RealisticWorkload {
    pub get_ids: Vec<String>,
    pub query_bboxes: Vec<BBox>,
}

/// Builds a deterministic read workload from a prepared feature-files corpus.
///
/// # Errors
///
/// Returns an error if the corpus cannot be scanned or does not contain enough
/// features and tiles to build the configured workload sizes.
pub fn build_realistic_workload(layout_root: &Path) -> Result<RealisticWorkload> {
    let feature_records = collect_feature_records(layout_root)?;
    Ok(RealisticWorkload {
        get_ids: build_get_workload(&feature_records)?,
        query_bboxes: build_query_workload(&feature_records)?,
    })
}

/// Collects feature metadata from a prepared feature-files corpus.
///
/// # Errors
///
/// Returns an error if the corpus cannot be scanned or a feature's metadata is
/// missing or inconsistent.
pub fn collect_feature_records(layout_root: &Path) -> Result<Vec<FeatureRecord>> {
    let feature_root = layout_root.join("features");
    let mut metadata_files = Vec::new();
    let mut feature_files = Vec::new();

    for entry in WalkBuilder::new(layout_root)
        .hidden(false)
        .follow_links(true)
        .build()
    {
        let entry = entry.map_err(|error| Error::Import(error.to_string()))?;
        if !entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
        {
            continue;
        }
        let path = entry.path().to_path_buf();
        if path.file_name().and_then(|name| name.to_str()) == Some("metadata.json") {
            metadata_files.push(path);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }
        if fs::metadata(&path)
            .map(|meta| meta.len() == 0)
            .unwrap_or(true)
        {
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
        let parent = metadata_path
            .parent()
            .unwrap_or(&feature_root)
            .to_path_buf();
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
        let metadata_path = resolve_feature_metadata_path(layout_root, &path, &metadata_by_dir)
            .ok_or_else(|| {
                Error::Import(format!(
                    "no ancestor metadata file found for feature {}",
                    path.display()
                ))
            })?;
        let metadata = metadata_cache.get(&metadata_path).ok_or_else(|| {
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

/// Builds the `get` benchmark workload from collected feature records.
///
/// # Errors
///
/// Returns an error if the corpus does not contain enough feature ids to
/// satisfy the configured workload size.
pub fn build_get_workload(feature_records: &[FeatureRecord]) -> Result<Vec<String>> {
    let mut ids_by_tile: BTreeMap<&PathBuf, Vec<&String>> = BTreeMap::new();
    for record in feature_records {
        ids_by_tile
            .entry(&record.tile)
            .or_default()
            .push(&record.id);
    }

    let mut ids = Vec::new();
    let mut selected_ids = BTreeSet::new();
    for (tile, tile_ids) in ids_by_tile {
        let mut tile_ids = tile_ids.into_iter().cloned().collect::<Vec<_>>();
        tile_ids.sort();
        seeded_shuffle(
            &mut tile_ids,
            WORKLOAD_SHUFFLE_SEED ^ stable_path_seed(tile.as_path()),
        );
        let selected = tile_ids
            .first()
            .cloned()
            .ok_or_else(|| Error::Import("tile grouping unexpectedly yielded no ids".into()))?;
        selected_ids.insert(selected.clone());
        ids.push(selected);
    }

    let mut remaining_ids = feature_records
        .iter()
        .map(|record| record.id.clone())
        .filter(|id| !selected_ids.contains(id))
        .collect::<Vec<_>>();
    remaining_ids.sort();
    seeded_shuffle(&mut remaining_ids, WORKLOAD_SHUFFLE_SEED);
    ids.extend(remaining_ids);

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

/// Builds the `query` benchmark workload from collected feature records.
///
/// # Errors
///
/// Returns an error if the corpus does not contain enough tiles or a tile
/// produces no bounding box when sampled.
pub fn build_query_workload(feature_records: &[FeatureRecord]) -> Result<Vec<BBox>> {
    let mut by_tile: BTreeMap<PathBuf, Vec<&FeatureRecord>> = BTreeMap::new();
    for record in feature_records {
        by_tile.entry(record.tile.clone()).or_default().push(record);
    }

    let mut tiles = by_tile.into_iter().collect::<Vec<_>>();
    if tiles.len() < QUERY_BATCH_COUNT {
        return Err(Error::Import(format!(
            "benchmark corpus only yielded {} tiles, expected at least {}",
            tiles.len(),
            QUERY_BATCH_COUNT
        )));
    }

    seeded_shuffle(&mut tiles, WORKLOAD_SHUFFLE_SEED ^ 0x9e37_79b9_7f4a_7c15);

    let mut bboxes = Vec::with_capacity(tiles.len());
    for (tile, records) in tiles {
        let mut selected_records = records;
        seeded_shuffle(
            &mut selected_records,
            WORKLOAD_SHUFFLE_SEED ^ 0xbf58_476d_1ce4_e5b9 ^ stable_path_seed(tile.as_path()),
        );

        let bbox = selected_records
            .into_iter()
            .take(QUERY_TILE_SAMPLE_SIZE)
            .map(|record| record.bbox)
            .reduce(union_bbox)
            .ok_or_else(|| Error::Import("query tile selection produced no bbox".into()))?;
        bboxes.push(bbox);
    }

    Ok(bboxes)
}

#[must_use]
pub fn union_bbox(a: BBox, b: BBox) -> BBox {
    BBox {
        min_x: a.min_x.min(b.min_x),
        max_x: a.max_x.max(b.max_x),
        min_y: a.min_y.min(b.min_y),
        max_y: a.max_y.max(b.max_y),
    }
}

/// Shuffles items with a deterministic seed.
///
/// # Panics
///
/// Panics if the generated shuffle index cannot be represented as `usize` on
/// the current target.
pub fn seeded_shuffle<T>(items: &mut [T], seed: u64) {
    if items.len() < 2 {
        return;
    }

    let mut state = seed;
    for index in (1..items.len()).rev() {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = state;
        value ^= value >> 30;
        value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value ^= value >> 27;
        value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^= value >> 31;
        let modulus = u64::try_from(index + 1).expect("shuffle index should fit in u64");
        let swap_with =
            usize::try_from(value % modulus).expect("shuffle index should fit in usize");
        items.swap(index, swap_with);
    }
}

fn stable_path_seed(path: &Path) -> u64 {
    path.to_string_lossy().bytes().fold(0u64, |acc, byte| {
        acc.wrapping_mul(131).wrapping_add(u64::from(byte))
    })
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

fn feature_id_from_value(value: &Value, context: &str) -> Result<String> {
    value
        .get("id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| Error::Import(format!("{context} is missing a string id")))
}
