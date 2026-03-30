use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::copy;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use cjlib::{Error, Result};
use flate2::read::GzDecoder;
use flatgeobuf::{FallibleStreamingIterator, FeatureProperties, FgbReader};
use ignore::WalkBuilder;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

const FEATURE_DIR: &str = "features";

pub const DEFAULT_INPUT_ROOT: &str = "/home/balazs/Data/3DBAG_3dtiles_test/input";
pub const DEFAULT_OUTPUT_ROOT: &str = "/home/balazs/Data/3DBAG_3dtiles_test/cjindex";
pub const DEFAULT_TILE_INDEX_URL: &str = "https://data.3dbag.nl/v20250903/tile_index.fgb";
pub const DEFAULT_TARGET_CITYOBJECTS: usize = 270_000;
pub const DEFAULT_TARGET_CITYOBJECTS_MIN: usize = 265_000;
pub const DEFAULT_TARGET_CITYOBJECTS_MAX: usize = 275_000;
pub const DEFAULT_STAGING_DIR_NAME: &str = ".prep-staging";

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PreparedDatasets {
    pub feature_files: PathBuf,
    pub cityjson: PathBuf,
    pub ndjson: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BenchmarkDataManifest {
    pub tile_index_url: String,
    pub tile_index_sha256: String,
    pub target_cityobjects: usize,
    pub accepted_cityobjects_min: usize,
    pub accepted_cityobjects_max: usize,
    pub total_cityobjects: usize,
    pub total_features: usize,
    pub selected_tiles: Vec<BenchmarkTileManifest>,
    pub cjseq_version: String,
    pub cjval_version: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BenchmarkTileManifest {
    pub tile_id: String,
    pub download_url: String,
    pub cityobject_count: usize,
    pub feature_package_count: usize,
    pub cityjson_sha256: String,
    pub ndjson_sha256: String,
    pub feature_file_count: usize,
}

#[derive(Debug, Clone)]
struct TileIndexRecord {
    tile_id: String,
    download_url: String,
}

/// Prepares the reproducible 3DBAG benchmark corpus.
///
/// # Errors
///
/// Returns an error if the tile index cannot be read, a tile download fails, or
/// any generated layout does not validate.
#[allow(clippy::too_many_lines)]
pub fn prepare_3dbag_benchmark_datasets(output_root: &Path) -> Result<PreparedDatasets> {
    let staging_root = output_root.join(DEFAULT_STAGING_DIR_NAME);
    let cityjson_root = staging_root.join("cityjson");
    let ndjson_root = staging_root.join("ndjson");
    let feature_files_root = staging_root.join("feature-files");

    if staging_root.exists() {
        fs::remove_dir_all(&staging_root)?;
    }
    fs::create_dir_all(&cityjson_root)?;
    fs::create_dir_all(&ndjson_root)?;
    fs::create_dir_all(feature_files_root.join("features"))?;

    let http_client = reqwest::blocking::Client::builder()
        .user_agent("cjindex/3dbag-benchmark-prep")
        .build()
        .map_err(|error| import_error(error.to_string()))?;

    let tile_index_path = staging_root.join("tile_index.fgb");
    download_file(&http_client, DEFAULT_TILE_INDEX_URL, &tile_index_path)?;
    let tile_index_sha256 = sha256_file(&tile_index_path)?;
    let tile_records = read_tile_index_records(&tile_index_path)?;

    if tile_records.is_empty() {
        return Err(import_error(
            "tile index did not contain any downloadable tiles",
        ));
    }

    let mut total_cityobjects = 0usize;
    let mut total_features = 0usize;
    let mut manifest_tiles = Vec::new();
    let root_metadata_path = feature_files_root.join("metadata.json");
    let mut root_metadata_written = false;

    for tile in tile_records {
        let cityjson_path = cityjson_output_path(&cityjson_root, &tile.tile_id);
        if let Some(parent) = cityjson_path.parent() {
            fs::create_dir_all(parent)?;
        }
        download_file(&http_client, &tile.download_url, &cityjson_path)?;
        validate_cjval(&cityjson_path)?;

        let cityobject_count = count_cityobjects(&cityjson_path)?;
        let cityjson_sha256 = sha256_file(&cityjson_path)?;

        let seq_output = run_cjseq_cat(&cityjson_path)?;
        let seq_lines = split_seq_lines(&seq_output);
        if seq_lines.is_empty() {
            return Err(import_error(format!(
                "cjseq cat produced no records for {}",
                cityjson_path.display()
            )));
        }

        let metadata_value: Value =
            serde_json::from_slice(seq_lines[0].as_bytes()).map_err(|error| {
                import_error(format!(
                    "invalid CityJSONSeq metadata for {}: {error}",
                    cityjson_path.display()
                ))
            })?;
        let metadata_bytes =
            serde_json::to_vec(&metadata_value).map_err(|error| import_error(error.to_string()))?;

        if !root_metadata_written {
            write_json(&root_metadata_path, &metadata_value)?;
            root_metadata_written = true;
        }

        let tile_feature_dir = feature_files_root
            .join("features")
            .join(tile_path(&tile.tile_id));
        fs::create_dir_all(&tile_feature_dir)?;
        write_json(&tile_feature_dir.join("metadata.json"), &metadata_value)?;

        let mut feature_file_count = 0usize;
        for line in seq_lines.iter().skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            let feature_value: Value =
                serde_json::from_slice(line.as_bytes()).map_err(|error| {
                    import_error(format!(
                        "invalid CityJSONSeq feature for {}: {error}",
                        cityjson_path.display()
                    ))
                })?;
            let feature_id = feature_id_from_value(&feature_value, "CityJSONSeq feature")?;
            let feature_path = tile_feature_dir.join(format!("{feature_id}.city.jsonl"));
            write_bytes(&feature_path, line.as_bytes())?;
            validate_feature_file(&feature_path, &metadata_bytes)?;
            feature_file_count += 1;
        }

        let ndjson_path = ndjson_output_path(&ndjson_root, &tile.tile_id);
        if let Some(parent) = ndjson_path.parent() {
            fs::create_dir_all(parent)?;
        }
        write_bytes(&ndjson_path, seq_output.as_bytes())?;
        validate_cjval(&ndjson_path)?;
        let ndjson_sha256 = sha256_file(&ndjson_path)?;

        total_cityobjects = total_cityobjects.saturating_add(cityobject_count);
        total_features = total_features.saturating_add(feature_file_count);
        manifest_tiles.push(BenchmarkTileManifest {
            tile_id: tile.tile_id,
            download_url: tile.download_url,
            cityobject_count,
            feature_package_count: seq_lines.len().saturating_sub(1),
            cityjson_sha256,
            ndjson_sha256,
            feature_file_count,
        });

        if total_cityobjects >= DEFAULT_TARGET_CITYOBJECTS_MAX {
            break;
        }
        if total_cityobjects >= DEFAULT_TARGET_CITYOBJECTS {
            break;
        }
    }

    if !(DEFAULT_TARGET_CITYOBJECTS_MIN..=DEFAULT_TARGET_CITYOBJECTS_MAX)
        .contains(&total_cityobjects)
    {
        return Err(import_error(format!(
            "prepared corpus has {total_cityobjects} cityobjects, expected between {DEFAULT_TARGET_CITYOBJECTS_MIN} and {DEFAULT_TARGET_CITYOBJECTS_MAX}"
        )));
    }

    let manifest = BenchmarkDataManifest {
        tile_index_url: DEFAULT_TILE_INDEX_URL.to_string(),
        tile_index_sha256,
        target_cityobjects: DEFAULT_TARGET_CITYOBJECTS,
        accepted_cityobjects_min: DEFAULT_TARGET_CITYOBJECTS_MIN,
        accepted_cityobjects_max: DEFAULT_TARGET_CITYOBJECTS_MAX,
        total_cityobjects,
        total_features,
        selected_tiles: manifest_tiles,
        cjseq_version: tool_version("cjseq")?,
        cjval_version: tool_version("cjval")?,
    };
    write_json(
        &staging_root.join("manifest.json"),
        &serde_json::to_value(&manifest).map_err(|error| import_error(error.to_string()))?,
    )?;

    promote_staging_layout(output_root, &staging_root)?;

    Ok(PreparedDatasets {
        feature_files: output_root.join("feature-files"),
        cityjson: output_root.join("cityjson"),
        ndjson: output_root.join("ndjson"),
    })
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

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    fs::write(path, bytes)?;
    Ok(())
}

fn validate_cjval(path: &Path) -> Result<()> {
    let bytes = fs::read(path)?;
    let mut child = Command::new("cjval")
        .arg("-q")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write as _;
        stdin.write_all(&bytes)?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(import_error(format!(
            "cjval failed for {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

fn validate_feature_file(path: &Path, metadata_bytes: &[u8]) -> Result<()> {
    let bytes = fs::read(path)?;
    let _feature = cjlib::json::staged::from_feature_slice_with_base(&bytes, metadata_bytes)?;
    Ok(())
}

fn run_cjseq_cat(cityjson_path: &Path) -> Result<String> {
    let output = Command::new("cjseq")
        .args(["cat", "-o", "lexicographical"])
        .arg(cityjson_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(import_error(format!(
            "cjseq cat failed for {}: {}",
            cityjson_path.display(),
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    String::from_utf8(output.stdout).map_err(|error| import_error(error.to_string()))
}

fn split_seq_lines(seq_output: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for line in seq_output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        lines.push(line.to_owned());
    }
    lines
}

fn read_tile_index_records(index_path: &Path) -> Result<Vec<TileIndexRecord>> {
    let mut file = File::open(index_path)?;
    let reader = FgbReader::open(&mut file).map_err(|error| import_error(error.to_string()))?;
    let mut features = reader
        .select_all()
        .map_err(|error| import_error(error.to_string()))?;
    let mut records = Vec::new();

    while let Some(feature) = features
        .next()
        .map_err(|error| import_error(error.to_string()))?
    {
        let tile_id = feature_property_string(feature, &["tile_id", "tileid", "id"])?;
        let download_url =
            feature_property_string(feature, &["cj_download", "cityjson_download", "download"])?;
        records.push(TileIndexRecord {
            tile_id,
            download_url,
        });
    }

    records.sort_by(|left, right| left.tile_id.cmp(&right.tile_id));
    Ok(records)
}

fn feature_property_string(feature: &flatgeobuf::FgbFeature, keys: &[&str]) -> Result<String> {
    for key in keys {
        if let Ok(value) = feature.property(key) {
            return Ok(value);
        }
    }

    Err(import_error(format!(
        "tile index feature is missing one of these properties: {}",
        keys.join(", ")
    )))
}

fn download_file(client: &reqwest::blocking::Client, url: &str, path: &Path) -> Result<()> {
    let response = client
        .get(url)
        .send()
        .map_err(|error| import_error(error.to_string()))?;
    if !response.status().is_success() {
        return Err(import_error(format!(
            "download failed for {url}: {}",
            response.status()
        )));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = response
        .bytes()
        .map_err(|error| import_error(error.to_string()))?;
    if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut decoder = GzDecoder::new(bytes.as_ref());
        let mut file = File::create(path)?;
        copy(&mut decoder, &mut file)?;
    } else {
        fs::write(path, bytes.as_ref())?;
    }
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn count_cityobjects(path: &Path) -> Result<usize> {
    let document = read_json(path)?;
    let cityobjects = document
        .get("CityObjects")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            import_error(format!(
                "CityJSON file {} is missing CityObjects",
                path.display()
            ))
        })?;
    Ok(cityobjects.len())
}

fn feature_id_from_value(value: &Value, context: &str) -> Result<String> {
    value
        .get("id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| import_error(format!("{context} is missing a string id")))
}

fn cityjson_output_path(root: &Path, tile_id: &str) -> PathBuf {
    root.join(tile_path(tile_id)).with_extension("city.json")
}

fn ndjson_output_path(root: &Path, tile_id: &str) -> PathBuf {
    root.join(tile_path(tile_id)).with_extension("city.jsonl")
}

fn tile_path(tile_id: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for component in tile_id.split('/') {
        path.push(component);
    }
    path
}

fn tool_version(tool: &str) -> Result<String> {
    let output = Command::new(tool).arg("--version").output()?;
    if !output.status.success() {
        return Err(import_error(format!(
            "{tool} --version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout)
        .map(|text| text.trim().to_owned())
        .map_err(|error| import_error(error.to_string()))
}

fn promote_staging_layout(output_root: &Path, staging_root: &Path) -> Result<()> {
    let final_feature_root = output_root.join("feature-files");
    let final_cityjson_root = output_root.join("cityjson");
    let final_ndjson_root = output_root.join("ndjson");
    let final_manifest = output_root.join("manifest.json");

    for path in [
        &final_feature_root,
        &final_cityjson_root,
        &final_ndjson_root,
        &final_manifest,
    ] {
        if path.exists() {
            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
    }

    fs::create_dir_all(output_root)?;
    fs::rename(staging_root.join("feature-files"), &final_feature_root)?;
    fs::rename(staging_root.join("cityjson"), &final_cityjson_root)?;
    fs::rename(staging_root.join("ndjson"), &final_ndjson_root)?;
    fs::rename(staging_root.join("manifest.json"), &final_manifest)?;
    fs::remove_dir_all(staging_root)?;
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
