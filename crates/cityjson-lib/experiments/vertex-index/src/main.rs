#![allow(dead_code, unused_variables)]

use slotmap::{new_key_type, SecondaryMap, SlotMap};
use std::collections::HashMap;

type Vertices = Vec<[f64; 3]>;

#[derive(Debug)]
struct Geometry {
    boundary: Vec<usize>,
}

#[derive(Debug)]
struct Object {
    geometries: Vec<Geometry>,
}

#[derive(Debug)]
struct SGeometry {
    boundary: Vec<[f64; 3]>,
}

new_key_type! { struct VertexKey; }

#[derive(Debug)]
struct TGeometry {
    boundary: Vec<VertexKey>,
}

#[derive(Debug)]
struct TObject {
    geometries: Vec<TGeometry>,
}

#[derive(Debug)]
struct Model {
    objects: HashMap<String, TObject>,
    vertex_map: SlotMap<VertexKey, [f64; 3]>,
    vertex_geometries_map: SecondaryMap<VertexKey, Vec<(String, usize)>>,
}

impl Model {
    fn drop_geometry(&mut self, oi: &str, gi: usize) {
        if let Some(obj) = self.objects.get_mut(oi) {
            if obj.geometries.len() - 1 < gi {
                panic!("geometry index out of bounds")
            } else {
                let geom_remove = obj.geometries.remove(gi);
                for vtx_key in &geom_remove.boundary {
                    let mut drop_vtx = true;
                    if let Some(obj_vec) = self.vertex_geometries_map.get_mut(*vtx_key) {
                        let mut drop_vi: Option<usize> = None;
                        for (vi, (oi_, gi_)) in obj_vec.iter().enumerate() {
                            if oi_ != oi || *gi_ != gi {
                                drop_vtx = false;
                            } else if oi_ == oi && *gi_ == gi {
                                drop_vi = Some(vi);
                            }
                        }
                        if let Some(_dv) = drop_vi {
                            obj_vec.remove(_dv);
                        }
                    }
                    if drop_vtx {
                        self.vertex_map.remove(*vtx_key);
                    }
                }
            }
        } else {
            println!("object not present");
        }
    }

    fn add_geometry(&mut self, oi: &str, sgeom: SGeometry) {
        if let Some(obj) = self.objects.get_mut(oi) {
            let gi_new = obj.geometries.len();
            let mut new_geom = TGeometry {
                boundary: Vec::new(),
            };
            for point in &sgeom.boundary {
                // instead of simply inserting, I should check for coordinate equality with X precision
                let vtx_key = self.vertex_map.insert(*point);
                if let Some(obj_vec) = self.vertex_geometries_map.get_mut(vtx_key) {
                    obj_vec.push((oi.to_string(), gi_new));
                } else {
                    self.vertex_geometries_map
                        .insert(vtx_key, vec![(oi.to_string(), gi_new)]);
                }
                new_geom.boundary.push(vtx_key);
            }
            obj.geometries.push(new_geom);
        } else {
            println!("Object does not exist");
        }
    }
}

fn main() {
    // CityJSON data
    let vertices = vec![
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 1.0],
        [2.0, 2.0, 2.0],
        [3.0, 3.0, 3.0],
        [4.0, 4.0, 4.0],
    ];
    let g1 = Geometry {
        boundary: vec![0, 1, 2],
    };
    let o1 = Object {
        geometries: vec![g1],
    };
    let g2 = Geometry {
        boundary: vec![2, 3, 4],
    };
    let o2 = Object {
        geometries: vec![g2],
    };

    // cjlib model
    let mut vertex_map: SlotMap<VertexKey, [f64; 3]> = SlotMap::with_key();
    let mut vertex_index_map: HashMap<usize, VertexKey> = HashMap::new();

    // parse the vertices
    for (vidx, vtx) in vertices.iter().enumerate() {
        let vtx_key = vertex_map.insert(*vtx);
        vertex_index_map.insert(vidx, vtx_key);
    }
    drop(vertices);

    // parse the geometries
    let mut model = Model {
        objects: HashMap::new(),
        vertex_map,
        vertex_geometries_map: SecondaryMap::new(),
    };
    for (oi, obj) in [("id1", o1), ("id2", o2)] {
        let mut obj_new = TObject {
            geometries: Vec::new(),
        };
        for (gi, geom) in obj.geometries.iter().enumerate() {
            let mut boundary_new: Vec<VertexKey> = Vec::new();
            for vidx in &geom.boundary {
                if let Some(vtx_key) = vertex_index_map.get(vidx) {
                    boundary_new.push(*vtx_key);
                    if model.vertex_geometries_map.contains_key(*vtx_key) {
                        if let Some(obj_geom_vec) = model.vertex_geometries_map.get_mut(*vtx_key) {
                            obj_geom_vec.push((oi.to_string(), gi));
                        }
                    } else {
                        model
                            .vertex_geometries_map
                            .insert(*vtx_key, vec![(oi.to_string(), gi)]);
                    }
                } else {
                    // this is error, because it this point each vidx in a Geometry boundary
                    // should have a corresponding point (coordinates) in the vertices array
                    println!("error!!!");
                }
            }
            obj_new.geometries.push({
                TGeometry {
                    boundary: boundary_new,
                }
            })
        }
        model.objects.insert(oi.to_string(), obj_new);
    }

    println!("{:#?}", model);

    model.drop_geometry("id1", 0);

    println!("{:#?}", model);

    model.add_geometry(
        "id2",
        SGeometry {
            boundary: vec![[3.0, 3.0, 3.0], [4.0, 4.0, 4.0], [5.0, 5.0, 5.0]],
        },
    );

    println!("{:#?}", model);
}
