//! Estimate the stack and heap size of serde_cityjson's structs that are holding value. This
//! module only used for performance optimization during the development of the library. The data
//! size estimation has a significant runtime overhead, so don't enable the corresponding "datasize"
//! feature unless you need it.
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use datasize::{data_size, DataSize};
use serde::{Deserialize, Serialize};
use crate::attributes::Attributes;
use crate::v1_1::*;

/// Returns the Cargo target directory, possibly calling `cargo metadata` to
/// figure it out.
///
///
/// # Licence notice
/// Copyright 2014 Jorge Aparicio.
/// Function copied from: https://github.com/bheisler/criterion.rs/blob/master/src/lib.rs.
/// No changes were made to the the function.
fn cargo_target_directory() -> Option<PathBuf> {
    #[derive(Deserialize)]
    struct Metadata {
        target_directory: PathBuf,
    }

    env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            let output = Command::new(env::var_os("CARGO")?)
                .args(["metadata", "--format-version", "1"])
                .output()
                .ok()?;
            let metadata: Metadata = serde_json::from_slice(&output.stdout).ok()?;
            Some(metadata.target_directory)
        })
}

pub struct SerdeCityJSONDataSize {
    output_directory: PathBuf,
}

impl SerdeCityJSONDataSize {
    pub fn new(output_directory: Option<PathBuf>) -> Self {
        Self {
            output_directory: output_directory.unwrap_or_else(|| match cargo_target_directory() {
                None => PathBuf::from("target/serde_cityjson_datasize"),
                Some(path) => path.join("serde_cityjson_datasize"),
            }),
        }
    }
    pub fn run<P: AsRef<Path>>(
        &self,
        group_id: &str,
        benchmark_id: &str,
        path: P,
    ) -> Result<(), String> {
        println!("Running datasize benchmark {}/{}", group_id, benchmark_id);
        let record = DataSizeRecord::compute_from_file(path.as_ref()).map_err(|e| e.to_string())?;
        let filename = "datasizes.json";
        let bench_dir_new = self
            .output_directory
            .join(group_id)
            .join(benchmark_id)
            .join("new");
        let bench_filename_new = bench_dir_new.join(filename);
        let bench_dir_base = self
            .output_directory
            .join(group_id)
            .join(benchmark_id)
            .join("base");
        let bench_filename_base = bench_dir_base.join(filename);
        let mut base_created: bool = false;
        if bench_dir_new.exists() {
            fs::create_dir_all(&bench_dir_base).map_err(|e| e.to_string())?;
            fs::copy(&bench_filename_new, &bench_filename_base).map_err(|e| e.to_string())?;
            base_created = true;
        }
        fs::create_dir_all(&bench_dir_new).map_err(|e| e.to_string())?;
        record
            .save(&bench_filename_new)
            .map_err(|e| e.to_string())?;
        if base_created {
            Self::compare_runs(bench_filename_base, bench_filename_new)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn compare_runs<P: AsRef<Path>>(base_path: P, new_path: P) -> Result<(), String> {
        let base_file = File::open(base_path.as_ref()).map_err(|e| e.to_string())?;
        let new_file = File::open(new_path.as_ref()).map_err(|e| e.to_string())?;
        let base: DataSizeRecord =
            serde_json::from_reader(&base_file).map_err(|e| e.to_string())?;
        let new: DataSizeRecord = serde_json::from_reader(&new_file).map_err(|e| e.to_string())?;
        let new_size_percent =
            new.serde_cityjson_total as f64 / base.serde_cityjson_total as f64 * 100.0;
        let new_size_mb = new.serde_cityjson_total as f64 * 1e-6;
        let base_size_mb = base.serde_cityjson_total as f64 * 1e-6;

        let new_size_percent_json = new.serde_cityjson_total as f64 / base.json as f64 * 100.0;
        let json_mb = new.json as f64 * 1e-6;
        println!("\tNew serde_cityjson data size is:\n\t\t{new_size_mb:.2} MB/{base_size_mb:.2} MB, {new_size_percent:.3}% of the previous run\n\t\t{new_size_percent_json:.3}% of the JSON string ({json_mb:.2} MB)");
        Ok(())
    }
}

/// Stores one record of the measured datasizes.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DataSizeRecord {
    /// Size of the JSON String in memory
    json: usize,
    /// Size of the serde_json::Value
    serde_value_total: usize,
    /// Size of the serde_cityjson::CityModel
    serde_cityjson_total: usize,
    /// Detailed size of the serde_cityjson::CityModel
    serde_cityjson_citymodel: CityModelDataSize,
}

impl DataSizeRecord {
    /// Compute the size of the JSON string, serde_json::Value and
    /// serde_cityjson::CityModel from a CityJSON file.
    ///
    /// # Panics
    /// The function will panic if anything goes wrong.
    pub fn compute_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut file = File::open(path.as_ref()).map_err(|e| e.to_string())?;
        let mut cityjson_json = String::new();
        file.read_to_string(&mut cityjson_json)
            .map_err(|e| e.to_string())?;
        let cm_serde_value = CityModelSerdeValue {
            inner: serde_json::from_str(&cityjson_json).map_err(|e| e.to_string())?,
        };
        let cm: CityModel = serde_json::from_str(&cityjson_json).unwrap();
        let cm_size = CityModelDataSize::compute_from(&cm);
        Ok(DataSizeRecord {
            json: total_heap_stack_size(&cityjson_json),
            serde_value_total: total_heap_stack_size(&cm_serde_value),
            serde_cityjson_total: total_heap_stack_size(&cm),
            serde_cityjson_citymodel: cm_size,
        })
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let file_out = File::create(path.as_ref()).map_err(|e| e.to_string())?;
        serde_json::to_writer(file_out, &self).map_err(|e| e.to_string())
    }
}

/// Stores the size of [CityModel] and its members. The members `CityModelDataSize` hold the size in
/// bytes of their corresponding [CityModel] member.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CityModelDataSize {
    id: usize,
    type_cm: usize,
    version: usize,
    transform: usize,
    cityobjects: CityObjectsDataSize,
    vertices: usize,
    metadata: usize,
    appearance: usize,
    geometry_templates: usize,
    extra: usize,
    extensions: usize,
}

impl CityModelDataSize {
    /// Calculate the total size of a [CityModel] in memory (heap + stack).
    pub fn compute_from(cm: &CityModel) -> Self {
        let mut cm_size = CityModelDataSize {
            ..Default::default()
        };

        cm_size.id = total_heap_stack_size(&cm.id);
        cm_size.type_cm = total_heap_stack_size(&cm.type_cm);
        cm_size.version = total_heap_stack_size(&cm.version);
        cm_size.transform = total_heap_stack_size(&cm.transform);
        cm_size.vertices = total_heap_stack_size(&cm.vertices);
        cm_size.metadata = total_heap_stack_size(&cm.metadata);
        cm_size.appearance = total_heap_stack_size(&cm.appearance);
        cm_size.geometry_templates = total_heap_stack_size(&cm.geometry_templates);
        cm_size.extra = total_heap_stack_size(&cm.extra.as_ref());
        cm_size.extensions = total_heap_stack_size(&cm.extensions);

        let mut co_size = CityObjectsDataSize {
            ..Default::default()
        };
        let mut geometries_size: HashMap<LoD, GeometryDataSize> = HashMap::new();
        for (coid, co) in cm.cityobjects.iter() {
            co_size.count += 1;
            co_size.total_coid += total_heap_stack_size(coid);
            if let Some(ref geometry) = co.geometry {
                for geom in geometry {
                    co_size.count_geometry += 1;
                    co_size.total_geometry += total_heap_stack_size(geom);
                    let lod = geom.lod.unwrap();
                    if geometries_size.contains_key(&lod) {
                        let geometry_size = geometries_size.get_mut(&lod).unwrap();
                        geometry_size.count += 1;
                        geometry_size.total += total_heap_stack_size(geom);
                        geometry_size.add_geometry(geom);
                    } else {
                        geometries_size.insert(
                            lod,
                            GeometryDataSize {
                                lod,
                                count: 1,
                                total: total_heap_stack_size(geom),
                                boundaries: total_heap_stack_size(&geom.boundaries),
                                semantics: total_heap_stack_size(&geom.semantics),
                                texture: total_heap_stack_size(&geom.texture),
                                material: total_heap_stack_size(&geom.material),
                            },
                        );
                    }
                }
            }
            if let Some(ref attributes) = co.attributes {

                if let Some(a) = attributes.as_borrowed().unwrap().as_object() {
                    co_size.count_attributes += a.len();
                }
            }
            co_size.total_attributes +=
                sizeof_attributes_option(&co.attributes) + size_of_val(&co.attributes);
            if co.geographical_extent.is_some() {
                co_size.count_geographical_extent += 1;
            }
            co_size.total_geographical_extent += total_heap_stack_size(&co.geographical_extent);
            if let Some(ref children) = co.children {
                co_size.count_children += children.len();
            }
            co_size.total_children_id += total_heap_stack_size(&co.children);
            if let Some(ref parents) = co.parents {
                co_size.count_parents += parents.len();
            }
            co_size.total_parents_id += total_heap_stack_size(&co.parents);
        }
        co_size.geometries = geometries_size.values().cloned().collect();
        cm_size.cityobjects = co_size;
        cm_size
    }
}

impl Display for CityModelDataSize {
    /// Prints a hierarchy of members, including the amount of heap memory used by them.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

/// Stores the data size of all CityObjects.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CityObjectsDataSize {
    /// CityObject count
    count: usize,
    /// Size of all CityObject IDs
    total_coid: usize,
    /// Count of all geometries
    count_geometry: usize,
    /// Size of all geometries of all CityObjects
    total_geometry: usize,
    /// Vec of Geometry sizes, one per Geometry of the CityObjects. For instance if the CityObjects contain four LoDs, this Vec has four GeometryDataSize.
    geometries: Vec<GeometryDataSize>,
    /// Count of all attributes in all CityObjects
    count_attributes: usize,
    /// Size of all attributes
    total_attributes: usize,
    /// Count of all geographical_extent in all CityObjects
    count_geographical_extent: usize,
    /// Size of all geographical_extent
    total_geographical_extent: usize,
    /// Count of all children IDs in all CityObjects
    count_children: usize,
    /// Size of all children IDs
    total_children_id: usize,
    /// Count of all parents IDs in all CityObjects
    count_parents: usize,
    /// Size of all parents IDs
    total_parents_id: usize,
}

/// Stores the data size of one geometric representation, e.g. all Geometry objects with the same LoD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryDataSize {
    /// Count of geometries of this LoD
    count: usize,
    /// Size of all geometries of this LoD
    total: usize,
    lod: LoD,
    /// Size of all boundaries of this LoD
    boundaries: usize,
    /// Size of all semantics of this LoD
    semantics: usize,
    /// Size of all materials of this LoD
    material: usize,
    /// Size of all textures of this LoD
    texture: usize,
}

impl Default for GeometryDataSize {
    fn default() -> Self {
        Self {
            count: 0,
            total: 0,
            lod: LoD::LoD0,
            boundaries: 0,
            semantics: 0,
            material: 0,
            texture: 0,
        }
    }
}

impl GeometryDataSize {
    /// Compute the size of a Geometry and add the values.
    pub(crate) fn add_geometry(&mut self, geom: &Geometry) {
        self.boundaries += total_heap_stack_size(&geom.boundaries);
        self.semantics += total_heap_stack_size(&geom.semantics);
        self.texture += total_heap_stack_size(&geom.texture);
        self.material += total_heap_stack_size(&geom.material);
    }
}

/// Calculate the total heap and stack size of a variable.
pub fn total_heap_stack_size<T: DataSize>(data: &T) -> usize {
    data_size(data) + size_of_val(data)
}

/// Compute the heap size of the optional Attributes that use serde_json_borrow::Value.
pub(crate) fn sizeof_attributes_option(a: &Option<Attributes>) -> usize {
    if let Some(ref attributes) = a {
        attributes
            .as_borrowed()
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .map(|(_, v)| {
                size_of::<&str>()
                    + sizeof_serde_borrow_value(v)
                    + size_of::<usize>() * 3
            })
            .sum()
    } else {
        0
    }
}

/// Compute the heap size of the optional Attributes that use serde_json::Value.
#[allow(dead_code)]
pub(crate) fn sizeof_attributes_cloned_option(a: &Option<serde_json::Value>) -> usize {
    if let Some(ref attributes) = a {
        if let Some(map) = attributes.as_object() {
            map.iter()
                .map(|(k, v)| {
                    size_of::<String>()
                        + k.capacity()
                        + sizeof_serde_value(v)
                        + size_of::<usize>() * 3
                })
                .sum()
        } else {
            0
        }
    } else {
        0
    }
}

/// Compute the heap size of a serde_json::Value.
///
/// From https://stackoverflow.com/a/76456111
pub(crate) fn sizeof_serde_value(v: &serde_json::Value) -> usize {
    size_of::<serde_json::Value>()
        + match v {
            serde_json::Value::Null => 0,
            serde_json::Value::Bool(_) => 0,
            serde_json::Value::Number(_) => 0, // Incorrect if arbitrary_precision is enabled. oh well
            serde_json::Value::String(s) => s.capacity(),
            serde_json::Value::Array(a) => a.iter().map(sizeof_serde_value).sum(),
            serde_json::Value::Object(o) => o
                .iter()
                .map(|(k, v)| {
                    size_of::<String>()
                        + k.capacity()
                        + sizeof_serde_value(v)
                        + size_of::<usize>() * 3 // As a crude approximation, I pretend each map entry has 3 words of overhead
                })
                .sum(),
        }
}

/// Compute the heap size of a serde_json_borrow::Value.
pub(crate) fn sizeof_serde_borrow_value(v: &serde_json_borrow::Value) -> usize {
    size_of::<serde_json::Value>()
        + match v {
            serde_json_borrow::Value::Null => 0,
            serde_json_borrow::Value::Bool(_) => 0,
            serde_json_borrow::Value::Number(_) => 0, // Incorrect if arbitrary_precision is enabled. oh well
            serde_json_borrow::Value::Str(_) => size_of::<Cow<str>>(),
            serde_json_borrow::Value::Array(a) => a.iter().map(sizeof_serde_borrow_value).sum(),
            serde_json_borrow::Value::Object(o) => o
                .iter()
                .map(|(_, v)| {
                    size_of::<&str>()
                        + sizeof_serde_borrow_value(v)
                        + size_of::<usize>() * 3 // As a crude approximation, I pretend each map entry has 3 words of overhead
                })
                .sum(),
        }
}

/// Wrapper type over a serde_json::Value that DataSize can be implemented for the inner Value.
#[derive(DataSize)]
pub struct CityModelSerdeValue {
    #[data_size(with = sizeof_serde_value)]
    pub inner: serde_json::Value,
}
