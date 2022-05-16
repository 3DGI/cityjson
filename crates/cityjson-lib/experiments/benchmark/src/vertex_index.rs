//! Vertex-index architecture.
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use memmap2::MmapOptions;
use serde::Deserialize;
use slotmap::{new_key_type, SecondaryMap, SlotMap};

// Deserialize into indexed CityJSON-like structures with serde
#[derive(Deserialize)]
struct SemanticSurface {
    #[serde(rename = "type")]
    semtype: String,
}

#[derive(Deserialize)]
struct Semantics {
    surfaces: Vec<SemanticSurface>,
    values: Vec<Vec<usize>>,
}

type Vertices = Vec<[f64; 3]>;

// Indexed geometry
type Vertex = usize;
type Ring = Vec<Vertex>;
type Surface = Vec<Ring>;
type Shell = Vec<Surface>;
type MultiSurface = Vec<Surface>;
type Solid = Vec<Shell>;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Geometry {
    MultiSurface {
        lod: String,
        boundaries: MultiSurface,
        semantics: Option<Semantics>,
    },
    Solid {
        lod: String,
        boundaries: Solid,
        semantics: Option<Semantics>,
    },
}

#[derive(Deserialize)]
struct CityObject {
    #[serde(rename = "type")]
    cotype: String,
    geometry: Vec<Geometry>,
}

#[derive(Deserialize, Copy, Clone)]
struct Transform {
    scale: [f64; 3],
    translate: [f64; 3],
}

#[derive(Deserialize)]
struct CityModel {
    #[serde(rename = "type")]
    cmtype: String,
    version: String,
    transform: Transform,
    #[serde(rename = "CityObjects")]
    cityobjects: HashMap<String, CityObject>,
    vertices: Vertices,
}

// SlotMap structs
new_key_type! { struct VertexKey; }

type RingSlot = Vec<VertexKey>;
type SurfaceSlot = Vec<RingSlot>;
type ShellSlot = Vec<SurfaceSlot>;
type MultiSurfaceSlot = Vec<SurfaceSlot>;
type SolidSlot = Vec<ShellSlot>;

enum GeometrySlot {
    MultiSurface {
        lod: String,
        boundaries: MultiSurfaceSlot,
        semantics: Option<Semantics>,
    },
    Solid {
        lod: String,
        boundaries: SolidSlot,
        semantics: Option<Semantics>,
    },
}

struct CityObjectSlot {
    cotype: String,
    geometry: Vec<GeometrySlot>,
}

struct CityModelSlot {
    cmtype: String,
    version: String,
    transform: Transform,
    cityobjects: HashMap<String, CityObjectSlot>,
    vertex_map: SlotMap<VertexKey, [f64; 3]>,
    vertex_geometries_map: SecondaryMap<VertexKey, Vec<(String, usize)>>,
}

fn parse_to_slotmap(cm: &CityModel) -> CityModelSlot {
    let mut vertex_map: SlotMap<VertexKey, [f64; 3]> = SlotMap::with_key();
    let mut vertex_index_map: HashMap<usize, VertexKey> = HashMap::new();

    // parse the vertices
    for (vidx, vtx) in cm.vertices.iter().enumerate() {
        let vtx_key = vertex_map.insert(*vtx);
        vertex_index_map.insert(vidx, vtx_key);
    }

    let mut model = CityModelSlot {
        cmtype: cm.cmtype.clone(),
        version: cm.version.clone(),
        transform: cm.transform.clone(),
        cityobjects: HashMap::new(),
        vertex_map,
        vertex_geometries_map: SecondaryMap::new(),
    };

    for (oi, obj) in &cm.cityobjects {
        let mut obj_new = CityObjectSlot {
            cotype: obj.cotype.clone(),
            geometry: Vec::new(),
        };
        for (gi, geom) in obj.geometry.iter().enumerate() {
            match geom {
                Geometry::MultiSurface {
                    lod,
                    boundaries,
                    semantics,
                } => {
                    let mut new_msrf_bdry = Vec::with_capacity(boundaries.len());
                    for (sui, surface) in boundaries.iter().enumerate() {
                        let mut surface_bdry = Vec::with_capacity(surface.len());
                        for ring in surface {
                            let mut new_ring = Vec::with_capacity(ring.len());
                            for vtx_idx in ring {
                                if let Some(vtx_key) = vertex_index_map.get(vtx_idx) {
                                    new_ring.push(*vtx_key);
                                    if model.vertex_geometries_map.contains_key(*vtx_key) {
                                        if let Some(obj_geom_vec) =
                                            model.vertex_geometries_map.get_mut(*vtx_key)
                                        {
                                            obj_geom_vec.push((oi.to_string(), gi));
                                        }
                                    } else {
                                        model
                                            .vertex_geometries_map
                                            .insert(*vtx_key, vec![(oi.to_string(), gi)]);
                                    }
                                }
                            }
                            surface_bdry.push(new_ring);
                        }
                        new_msrf_bdry.push(surface_bdry);
                    }
                    obj_new.geometry.push(GeometrySlot::MultiSurface {
                        lod: lod.to_string(),
                        boundaries: new_msrf_bdry,
                        semantics: None,
                    });
                }
                Geometry::Solid {
                    lod,
                    boundaries,
                    semantics,
                } => {
                    let mut new_solid_bdry = Vec::with_capacity(boundaries.len());
                    for (shi, shell) in boundaries.iter().enumerate() {
                        let mut new_shell = Vec::with_capacity(shell.len());
                        for (sui, surface) in shell.iter().enumerate() {
                            let mut surface_bdry = Vec::with_capacity(surface.len());
                            for ring in surface {
                                let mut new_ring = Vec::with_capacity(ring.len());
                                for vtx_idx in ring {
                                    if let Some(vtx_key) = vertex_index_map.get(vtx_idx) {
                                        new_ring.push(*vtx_key);
                                        if model.vertex_geometries_map.contains_key(*vtx_key) {
                                            if let Some(obj_geom_vec) =
                                                model.vertex_geometries_map.get_mut(*vtx_key)
                                            {
                                                obj_geom_vec.push((oi.to_string(), gi));
                                            }
                                        } else {
                                            model
                                                .vertex_geometries_map
                                                .insert(*vtx_key, vec![(oi.to_string(), gi)]);
                                        }
                                    }
                                }
                                surface_bdry.push(new_ring);
                            }
                        }
                        new_solid_bdry.push(new_shell);
                    }
                    obj_new.geometry.push(GeometrySlot::Solid {
                        lod: lod.to_string(),
                        boundaries: new_solid_bdry,
                        semantics: None,
                    });
                }
            }
        }
        model.cityobjects.insert(oi.to_string(), obj_new);
    }
    model
}

pub fn vindex_deserialize(path_in: PathBuf) {
    let mut file = File::open(path_in).expect("Couldn't open CityJSON file");
    let reader = BufReader::new(file);
    let cm: CityModel =
        serde_json::from_reader(reader).expect("Couldn't deserialize into CityModel");
    let cm_slot = parse_to_slotmap(&cm);
    drop(cm);
    println!("done par")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_data() -> PathBuf {
        Path::new("/home/balazs/Development/cjlib/experiments/data/3dbag_v210908_fd2cee53_5786_bench.city.json")
            .canonicalize()
            .expect("Could not find the INPUT file.")
    }

    #[test]
    fn test_vindex_deserialize() {
        let path_in = get_data();
        vindex_deserialize(path_in)
    }

    #[test]
    fn test_vindex_deserialize_debug() {
        let path_in = get_data();
        let mut file = File::open(path_in).expect("Couldn't open CityJSON file");
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect("Couldn't read CityJSON file contents");
        // let reader = BufReader::new(file);
        let cm: CityModel =
            serde_json::from_slice(&buffer[..]).expect("Couldn't deserialize into CityModel");
    }
}
