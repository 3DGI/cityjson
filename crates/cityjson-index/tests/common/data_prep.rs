use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use cjlib::{Error, Result};
use ignore::WalkBuilder;
use serde_json::{Map, Value};

const FEATURE_DIR: &str = "features";

pub const DEFAULT_INPUT_ROOT: &str = "/home/balazs/Data/3DBAG_3dtiles_test/input";
pub const DEFAULT_OUTPUT_ROOT: &str = "/home/balazs/Data/3DBAG_3dtiles_test/cjindex";

#[derive(Debug, Clone)]
pub struct PreparedDatasets {
    pub feature_files: PathBuf,
    pub cityjson: PathBuf,
    pub ndjson: PathBuf,
}

/// Prepares the feature-files fixture tree.
///
/// # Errors
///
/// Returns an error if the output tree cannot be created or linked.
pub fn prepare_feature_files_only(input_root: &Path, output_root: &Path) -> Result<PathBuf> {
    if let Some(parent) = output_root.parent() {
        fs::create_dir_all(parent)?;
    }
    prepare_feature_files_fixture(input_root, output_root)?;
    Ok(output_root.to_path_buf())
}

/// Prepares the per-tile `CityJSON` fixture tree.
///
/// # Errors
///
/// Returns an error if the input metadata is invalid or a tile cannot be
/// written.
pub fn prepare_cityjson_only(input_root: &Path, output_root: &Path) -> Result<PathBuf> {
    if let Some(parent) = output_root.parent() {
        fs::create_dir_all(parent)?;
    }
    let source_metadata = read_json(input_root.join("metadata.json"))?;
    let transform = source_metadata
        .get("transform")
        .cloned()
        .ok_or_else(|| import_error("source metadata is missing transform"))?;
    let scale = transform
        .get("scale")
        .and_then(Value::as_array)
        .ok_or_else(|| import_error("source metadata transform is missing scale"))?
        .iter()
        .map(value_as_f64)
        .collect::<Result<Vec<_>>>()?;
    let translate = transform
        .get("translate")
        .and_then(Value::as_array)
        .ok_or_else(|| import_error("source metadata transform is missing translate"))?
        .iter()
        .map(value_as_f64)
        .collect::<Result<Vec<_>>>()?;
    if scale.len() != 3 || translate.len() != 3 {
        return Err(import_error(
            "source transform must contain three scale and translate values",
        ));
    }

    write_cityjson_tiles(
        input_root,
        output_root,
        &source_metadata,
        [scale[0], scale[1], scale[2]],
        [translate[0], translate[1], translate[2]],
    )?;
    Ok(output_root.to_path_buf())
}

/// Prepares the per-tile NDJSON fixture tree.
///
/// # Errors
///
/// Returns an error if the intermediate `CityJSON` or `NDJSON` conversion fails.
pub fn prepare_ndjson_only(input_root: &Path, output_root: &Path) -> Result<PathBuf> {
    if let Some(parent) = output_root.parent() {
        fs::create_dir_all(parent)?;
    }
    let cityjson_root = output_root.join("cityjson");
    let ndjson_root = output_root.join("ndjson");
    let _ = prepare_cityjson_only(input_root, &cityjson_root)?;
    write_ndjson_tiles(&cityjson_root, &ndjson_root)?;
    Ok(ndjson_root)
}

/// Prepares all three fixture trees.
///
/// # Errors
///
/// Returns an error if any of the fixture trees cannot be generated.
pub fn prepare_test_sets(input_root: &Path, output_root: &Path) -> Result<PreparedDatasets> {
    fs::create_dir_all(output_root)?;

    let feature_files_root = output_root.join("feature-files");
    let cityjson_root = output_root.join("cityjson");
    let ndjson_root = output_root.join("ndjson");

    prepare_feature_files_fixture(input_root, &feature_files_root)?;
    let _ = prepare_cityjson_only(input_root, &cityjson_root)?;
    write_ndjson_tiles(&cityjson_root, &ndjson_root)?;

    Ok(PreparedDatasets {
        feature_files: feature_files_root,
        cityjson: cityjson_root,
        ndjson: ndjson_root,
    })
}

fn prepare_feature_files_fixture(input_root: &Path, output_root: &Path) -> Result<()> {
    if output_root.exists() {
        if output_root.symlink_metadata()?.file_type().is_symlink() {
            fs::remove_file(output_root)?;
        } else {
            fs::remove_dir_all(output_root)?;
        }
    }

    symlink_dir(input_root, output_root)
}

fn write_cityjson_tiles(
    input_root: &Path,
    cityjson_root: &Path,
    source_metadata: &Value,
    source_scale: [f64; 3],
    source_translate: [f64; 3],
) -> Result<()> {
    let tiles = collect_tiles(input_root)?;
    if cityjson_root.exists() {
        fs::remove_dir_all(cityjson_root)?;
    }
    fs::create_dir_all(cityjson_root)?;

    for (tile_rel, files) in tiles {
        let tile_cityjson =
            build_cityjson_tile(source_metadata, &files, source_scale, source_translate)?;
        let cityjson_path = cityjson_root.join(&tile_rel).with_extension("city.json");
        if let Some(parent) = cityjson_path.parent() {
            fs::create_dir_all(parent)?;
        }
        write_json(&cityjson_path, &tile_cityjson)?;
    }

    Ok(())
}

fn write_ndjson_tiles(cityjson_root: &Path, ndjson_root: &Path) -> Result<()> {
    if ndjson_root.exists() {
        fs::remove_dir_all(ndjson_root)?;
    }
    fs::create_dir_all(ndjson_root)?;
    for entry in WalkBuilder::new(cityjson_root).hidden(false).build() {
        let entry = entry.map_err(|error| import_error(error.to_string()))?;
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        let cityjson_path = entry.into_path();
        if cityjson_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let rel = cityjson_path
            .strip_prefix(cityjson_root)
            .map_err(|_| import_error("cityjson tile path is outside the output root"))?;
        let file_stem = cityjson_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| import_error("cityjson tile path is missing a valid file stem"))?;
        let ndjson_path = ndjson_root
            .join(rel)
            .with_file_name(format!("{file_stem}.jsonl"));
        if let Some(parent) = ndjson_path.parent() {
            fs::create_dir_all(parent)?;
        }
        convert_cityjson_to_seq(&cityjson_path, &ndjson_path)?;
    }
    Ok(())
}

fn symlink_dir(target: &Path, link: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)?;
        return Ok(());
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(target, link)?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(import_error("symlinks are not supported on this platform"))
}

fn collect_tiles(input_root: &Path) -> Result<BTreeMap<PathBuf, Vec<PathBuf>>> {
    let feature_root = input_root.join(FEATURE_DIR);
    let mut tiles: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();

    for entry in WalkBuilder::new(&feature_root).hidden(false).build() {
        let entry = entry.map_err(|error| import_error(error.to_string()))?;
        let file_type = entry.file_type();
        if !file_type.is_some_and(|ft| ft.is_file()) {
            continue;
        }
        let path = entry.into_path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }
        if fs::metadata(&path)
            .map(|meta| meta.len() == 0)
            .unwrap_or(false)
        {
            continue;
        }
        let tile_rel = path
            .strip_prefix(&feature_root)
            .map_err(|_| import_error("feature path is outside the input root"))?
            .parent()
            .ok_or_else(|| import_error("feature file is missing a tile directory"))?
            .to_path_buf();
        tiles.entry(tile_rel).or_default().push(path);
    }

    for files in tiles.values_mut() {
        files.sort();
    }

    Ok(tiles)
}

#[allow(clippy::too_many_lines)]
fn build_cityjson_tile(
    source_metadata: &Value,
    files: &[PathBuf],
    source_scale: [f64; 3],
    source_translate: [f64; 3],
) -> Result<Value> {
    let mut parsed_features = Vec::with_capacity(files.len());
    let mut tile_min = [f64::INFINITY; 3];

    for file in files {
        let feature = read_json(file)?;
        let vertices = feature
            .get("vertices")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                import_error(format!(
                    "feature file {} is missing vertices",
                    file.display()
                ))
            })?;
        let mut abs_vertices = Vec::with_capacity(vertices.len());
        for vertex in vertices {
            let coords = vertex
                .as_array()
                .ok_or_else(|| import_error("feature vertex must be an array"))?;
            if coords.len() != 3 {
                return Err(import_error("feature vertex must have three coordinates"));
            }
            let mut abs = [0.0; 3];
            for axis in 0..3 {
                abs[axis] =
                    source_translate[axis] + source_scale[axis] * value_as_f64(&coords[axis])?;
                tile_min[axis] = tile_min[axis].min(abs[axis]);
            }
            abs_vertices.push(abs);
        }
        parsed_features.push((feature, abs_vertices));
    }

    let tile_translate = tile_min;
    let mut combined_vertices = Vec::new();
    let mut cityobjects = Map::new();

    for (feature, abs_vertices) in parsed_features {
        let feature_vertex_offset = combined_vertices.len();
        for vertex in abs_vertices {
            let coords = vertex
                .into_iter()
                .zip(tile_translate)
                .zip(source_scale)
                .map(|((abs, translate), scale)| {
                    let quantized = ((abs - translate) / scale).round();
                    if !quantized.is_finite() {
                        return Err(import_error(
                            "vertex quantization produced a non-finite value",
                        ));
                    }
                    let integer = format!("{quantized:.0}").parse::<i64>().map_err(|_| {
                        import_error("vertex quantization produced an out-of-range integer")
                    })?;
                    Ok(Value::Number(serde_json::Number::from(integer)))
                })
                .collect::<Result<Vec<_>>>()?;
            combined_vertices.push(Value::Array(coords));
        }

        let feature_objects = feature
            .get("CityObjects")
            .and_then(Value::as_object)
            .ok_or_else(|| import_error("feature file is missing CityObjects"))?;
        for (id, cityobject) in feature_objects {
            if cityobjects.contains_key(id) {
                return Err(import_error(format!(
                    "duplicate CityObject id in tile output: {id}"
                )));
            }
            let mut cityobject = cityobject.clone();
            remap_cityobject_boundaries(&mut cityobject, feature_vertex_offset)?;
            cityobjects.insert(id.clone(), cityobject);
        }
    }

    let mut root = source_metadata.clone();
    let root_map = root
        .as_object_mut()
        .ok_or_else(|| import_error("source metadata must be a JSON object"))?;
    root_map.insert("type".to_string(), Value::String("CityJSON".to_string()));
    root_map.insert("CityObjects".to_string(), Value::Object(cityobjects));
    root_map.insert("vertices".to_string(), Value::Array(combined_vertices));
    root_map.insert(
        "transform".to_string(),
        Value::Object(Map::from_iter([
            (
                "scale".to_string(),
                Value::Array(
                    source_scale
                        .into_iter()
                        .map(number_from_f64)
                        .collect::<Result<Vec<_>>>()?,
                ),
            ),
            (
                "translate".to_string(),
                Value::Array(
                    tile_translate
                        .into_iter()
                        .map(number_from_f64)
                        .collect::<Result<Vec<_>>>()?,
                ),
            ),
        ])),
    );

    Ok(root)
}

fn remap_cityobject_boundaries(value: &mut Value, vertex_offset: usize) -> Result<()> {
    match value {
        Value::Object(map) => {
            if let Some(boundaries) = map.get_mut("boundaries") {
                remap_boundaries(boundaries, vertex_offset)?;
            }
            for (key, nested) in map.iter_mut() {
                if key != "boundaries" {
                    remap_cityobject_boundaries(nested, vertex_offset)?;
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                remap_cityobject_boundaries(item, vertex_offset)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn remap_boundaries(value: &mut Value, vertex_offset: usize) -> Result<()> {
    match value {
        Value::Array(items) => {
            for item in items {
                remap_boundaries(item, vertex_offset)?;
            }
        }
        Value::Number(number) => {
            let index = number
                .as_u64()
                .ok_or_else(|| import_error("vertex index must be a non-negative integer"))?;
            *value = Value::Number(serde_json::Number::from(index + vertex_offset as u64));
        }
        _ => return Err(import_error("boundaries must be arrays of vertex indices")),
    }
    Ok(())
}

fn convert_cityjson_to_seq(cityjson_path: &Path, ndjson_path: &Path) -> Result<()> {
    let output = Command::new("cjseq")
        .args(["cat", "-o", "lexicographical"])
        .arg(cityjson_path)
        .output()?;

    if !output.status.success() {
        return Err(import_error(format!(
            "cjseq cat failed for {}: {}",
            cityjson_path.display(),
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    fs::write(ndjson_path, output.stdout)?;
    Ok(())
}

fn read_json(path: impl AsRef<Path>) -> Result<Value> {
    let bytes = fs::read(path.as_ref())?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(value).map_err(|error| Error::Import(error.to_string()))?;
    fs::write(path, bytes)?;
    Ok(())
}

fn value_as_f64(value: &Value) -> Result<f64> {
    value
        .as_f64()
        .ok_or_else(|| import_error("expected a floating-point number"))
}

fn number_from_f64(value: f64) -> Result<Value> {
    serde_json::Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| import_error("expected a finite floating-point number"))
}

fn import_error(message: impl Into<String>) -> Error {
    Error::Import(message.into())
}
