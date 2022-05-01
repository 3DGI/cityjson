//! Dereference architecture.
use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::{json, Value};

pub mod geom_static {
    use serde::Serialize;
    use std::collections::HashMap;

    #[derive(Debug, Serialize)]
    pub enum SemanticSurface {
        RoofSurface,
        GroundSurface,
        WallSurface,
    }

    #[derive(Serialize)]
    pub struct Material {
        pub name: String,
        pub ambient_intensity: Option<f32>,
        pub diffuse_color: Option<[f32; 3]>,
        pub emissive_color: Option<[f32; 3]>,
        pub specular_color: Option<[f32; 3]>,
        pub shininess: Option<f32>,
        pub transparency: Option<f32>,
        pub is_smooth: Option<bool>,
    }

    #[derive(Serialize)]
    pub struct Texture {
        image: String,
    }

    pub type Vertices = Vec<[f64; 3]>;

    type Point = [f64; 3];
    type LineString = Vec<Point>;

    #[derive(Serialize)]
    pub struct Surface {
        pub boundaries: Vec<LineString>,
        pub semantics: Option<SemanticSurface>,
        pub material: Option<Material>,
        pub texture: Option<Texture>,
    }

    type Shell = Vec<Surface>;

    pub(crate) enum GeomStructSeparate {
        Surface(Surface),
    }

    // Named fields in the variant so that we attach the data directly to the variant
    pub(crate) enum GeomStructEmbed {
        Surface {
            boundaries: Vec<LineString>,
            semantics: Option<SemanticSurface>,
            material: Option<Material>,
            texture: Option<Texture>,
        },
    }

    #[derive(Serialize)]
    pub enum Geometry {
        MultiPoint {
            lod: String,
            boundaries: Vec<Point>,
        },
        MultiLineString {
            lod: String,
            boundaries: Vec<LineString>,
        },
        MultiSurface {
            lod: String,
            boundaries: Vec<Surface>,
        },
        CompositeSurface {
            lod: String,
            boundaries: Vec<Surface>,
        },
        Solid {
            lod: String,
            boundaries: Vec<Shell>,
        },
        MultiSolid {
            lod: String,
            boundaries: Vec<Geometry>, // This is not good here. I want to constrain this to a Solid.
        },
        CompositeSolid {
            lod: String,
            boundaries: Vec<Geometry>,
        },
    }

    pub struct CityObject {
        pub cotype: String,
        pub geometry: Vec<Geometry>,
    }

    #[derive(Serialize)]
    pub struct CityObjectSer {
        pub cotype: String,
        pub geometry: Vec<String>,
    }

    pub struct CityModel {
        pub cmtype: String,
        pub version: String,
        pub cityobjects: HashMap<String, CityObject>,
    }

    #[derive(Serialize)]
    pub struct CityModelSer {
        pub cmtype: String,
        pub version: String,
        pub cityobjects: HashMap<String, CityObjectSer>,
    }
}

mod deserialize {
    use serde::Deserialize;
    use std::collections::HashMap;

    // Deserialize into indexed CityJSON-like structures with serde
    #[derive(Deserialize)]
    pub struct SemanticSurface {
        #[serde(rename = "type")]
        pub semtype: String,
    }

    #[derive(Deserialize)]
    pub struct ISemantics {
        pub surfaces: Vec<SemanticSurface>,
        pub values: Vec<Vec<usize>>,
    }

    pub type Vertices = Vec<[f64; 3]>;

    // Indexed geometry
    pub type IVertex = usize;
    pub type IRing = Vec<IVertex>;
    pub type ISurface = Vec<IRing>;
    pub type IShell = Vec<ISurface>;
    pub type IMultiSurface = Vec<ISurface>;
    pub type ISolid = Vec<IShell>;

    #[derive(Deserialize)]
    #[serde(tag = "type")]
    pub enum IGeometry {
        MultiSurface {
            lod: String,
            boundaries: Vec<Vec<Vec<usize>>>,
            semantics: Option<ISemantics>,
        },
        Solid {
            lod: String,
            boundaries: Vec<Vec<Vec<Vec<usize>>>>,
            semantics: Option<ISemantics>,
        },
    }

    #[derive(Deserialize)]
    pub struct ICityObject {
        #[serde(rename = "type")]
        pub cotype: String,
        pub geometry: Vec<IGeometry>,
    }

    #[derive(Deserialize)]
    struct Transform {
        scale: [f64; 3],
        translate: [f64; 3],
    }

    #[derive(Deserialize)]
    pub struct ICityModel {
        #[serde(rename = "type")]
        pub cmtype: String,
        pub version: String,
        transform: Transform,
        #[serde(rename = "CityObjects")]
        pub cityobjects: HashMap<String, ICityObject>,
        pub vertices: Vertices,
    }
}

fn boundary_dereference(
    vertices: &geom_static::Vertices,
    geom: &deserialize::IGeometry,
) -> Option<geom_static::Geometry> {
    match geom {
        deserialize::IGeometry::Solid {
            lod,
            boundaries,
            semantics,
        } => {
            let mut new_solid_bdry = Vec::new();
            for (shi, shell) in boundaries.iter().enumerate() {
                let mut new_shell = Vec::new();
                for (sui, surface) in shell.iter().enumerate() {
                    let mut surface_bdry = Vec::new();
                    for ring in surface {
                        let mut new_ring = Vec::new();
                        for vtx_idx in ring {
                            let new_vertex: [f64; 3] = vertices[*vtx_idx];
                            new_ring.push(new_vertex);
                        }
                        surface_bdry.push(new_ring);
                    }
                    let mut semsurf: Option<geom_static::SemanticSurface> = None;
                    if let Some(sem) = semantics {
                        let sem_i = &sem.values[shi][sui];
                        match sem.surfaces[*sem_i].semtype.as_str() {
                            "GroundSurface" => {
                                semsurf = Some(geom_static::SemanticSurface::GroundSurface);
                            }
                            "WallSurface" => {
                                semsurf = Some(geom_static::SemanticSurface::WallSurface);
                            }
                            "RoofSurface" => {
                                semsurf = Some(geom_static::SemanticSurface::RoofSurface);
                            }
                            &_ => {
                                println!("Semantic Surface type not implemented")
                            }
                        }
                    }
                    new_shell.push(geom_static::Surface {
                        boundaries: surface_bdry,
                        semantics: semsurf,
                        material: None,
                        texture: None,
                    });
                }
                new_solid_bdry.push(new_shell);
            }
            Some(geom_static::Geometry::Solid {
                lod: lod.to_string(),
                boundaries: new_solid_bdry,
            })
        }
        deserialize::IGeometry::MultiSurface {
            lod,
            boundaries,
            semantics,
        } => {
            let mut new_msrf_bdry = Vec::new();
            for (sui, surface) in boundaries.iter().enumerate() {
                let mut surface_bdry = Vec::new();
                for ring in surface {
                    let mut new_ring = Vec::new();
                    for vtx_idx in ring {
                        let new_vertex: [f64; 3] = vertices[*vtx_idx];
                        new_ring.push(new_vertex);
                    }
                    surface_bdry.push(new_ring);
                }
                let mut semsurf: Option<geom_static::SemanticSurface> = None;
                //                 This needs can only be done after the IGeometry is implemented for MultiSurfaces too
                /*                if let Some(sem) = semantics {
                    let sem_i = &sem.values[shi][sui];
                    match &sem.surfaces[*sem_i].semtype {
                        String::from("GroundSurface") => {
                            semsurf = Some(geom_static::SemanticSurface::GroundSurface);
                        }
                        String::from("WallSurface") => {
                            semsurf = Some(geom_static::SemanticSurface::WallSurface);
                        }
                        String::from("RoofSurface") => {
                            semsurf = Some(geom_static::SemanticSurface::RoofSurface);
                        }
                    }
                }*/
                new_msrf_bdry.push(geom_static::Surface {
                    boundaries: surface_bdry,
                    semantics: semsurf,
                    material: None,
                    texture: None,
                });
            }
            Some(geom_static::Geometry::MultiSurface {
                lod: lod.to_string(),
                boundaries: new_msrf_bdry,
            })
        }
        _ => {
            println!("Geometry type not implemented");
            None
        }
    }
}

fn parse_dereferece(path_in: PathBuf) -> geom_static::CityModel {
    let file = File::open(path_in).expect("Couldn't read CityJSON file");
    let mmap = unsafe { memmap2::Mmap::map(&file) }.unwrap();
    let icm: deserialize::ICityModel =
        serde_json::from_slice(&mmap).expect("Couldn't deserialize into ICityModel");

    let mut new_cos: HashMap<String, geom_static::CityObject> = HashMap::new();
    for (coid, co) in icm.cityobjects {
        // println!("Processing CityObject {}", coid);
        let mut new_geoms: Vec<geom_static::Geometry> = Vec::new();
        for geom in co.geometry {
            let g = boundary_dereference(&icm.vertices, &geom);
            new_geoms.push(g.expect("Error in converting geometry"));
        }
        let new_co = geom_static::CityObject {
            cotype: co.cotype,
            geometry: new_geoms,
        };
        new_cos.insert(coid, new_co);
    }
    geom_static::CityModel {
        cmtype: icm.cmtype,
        version: icm.version,
        cityobjects: new_cos,
    }
}

pub fn deref_deserialize(path_in: PathBuf) {
    let cm = parse_dereferece(path_in);
}

pub fn deref_geometry(path_in: PathBuf) {
    let mut containter: Vec<[f64; 3]> = Vec::new();
    let cm = parse_dereferece(path_in);
    for (coid, co) in cm.cityobjects {
        for geom in co.geometry {
            match geom {
                geom_static::Geometry::MultiPoint { .. } => {}
                geom_static::Geometry::MultiLineString { .. } => {}
                geom_static::Geometry::MultiSurface { boundaries, .. } => {
                    for surface in boundaries {
                        for ring in surface.boundaries {
                            for vtx in ring {
                                containter.push(vtx);
                            }
                        }
                    }
                }
                geom_static::Geometry::CompositeSurface { .. } => {}
                geom_static::Geometry::Solid { boundaries, .. } => {
                    for shell in boundaries {
                        for surface in shell {
                            for ring in surface.boundaries {
                                for vtx in ring {
                                    containter.push(vtx);
                                }
                            }
                        }
                    }
                }
                geom_static::Geometry::MultiSolid { .. } => {}
                geom_static::Geometry::CompositeSolid { .. } => {}
            }
        }
    }
    println!(
        "In total there are {} points in the citymodel",
        containter.len()
    )
}

pub fn deref_semantics(path_in: PathBuf) {
    let semantic_type = "RoofSurface";
    println!("extracting the geometry of {}", semantic_type);
    let mut containter: Vec<[f64; 3]> = Vec::new();
    let cm = parse_dereferece(path_in);
    for (coid, co) in cm.cityobjects {
        for geom in co.geometry {
            match geom {
                geom_static::Geometry::MultiPoint { .. } => {}
                geom_static::Geometry::MultiLineString { .. } => {}
                geom_static::Geometry::MultiSurface { boundaries, .. } => {
                    for surface in boundaries {
                        if let Some(semsrf) = surface.semantics {
                            match semsrf {
                                geom_static::SemanticSurface::RoofSurface => {
                                    for ring in surface.boundaries {
                                        for vtx in ring {
                                            containter.push(vtx);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                geom_static::Geometry::CompositeSurface { .. } => {}
                geom_static::Geometry::Solid { boundaries, .. } => {
                    for shell in boundaries {
                        for surface in shell {
                            if let Some(semsrf) = surface.semantics {
                                match semsrf {
                                    geom_static::SemanticSurface::RoofSurface => {
                                        for ring in surface.boundaries {
                                            for vtx in ring {
                                                containter.push(vtx);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                geom_static::Geometry::MultiSolid { .. } => {}
                geom_static::Geometry::CompositeSolid { .. } => {}
            }
        }
    }
    println!(
        "In total there are {} points in the citymodel",
        containter.len()
    )
}

/// Creates an indexed boundary representation (as in CityJSON) from the dereferenced,
/// Simple Feature-like boundary.
fn index_boundaries(
    geometry: &geom_static::Geometry,
    vtx_lookup: &mut HashMap<String, usize>,
    vtx_idx: &mut usize,
) -> serde_json::Result<String> {
    match geometry {
        geom_static::Geometry::MultiSurface { boundaries, .. } => {
            let mut imsurface = Vec::new();
            for surface in boundaries {
                let mut isurface = Vec::new();
                for ring in &surface.boundaries {
                    let mut iring: Vec<usize> = Vec::new();
                    for vtx in ring {
                        let coord_str: String =
                            format!("{:.3} {:.3} {:.3}", vtx[0], vtx[1], vtx[2]);
                        match vtx_lookup.get(&coord_str) {
                            Some(existing_idx) => iring.push(existing_idx.clone()),
                            None => {
                                vtx_lookup.insert(coord_str, vtx_idx.clone());
                                iring.push(vtx_idx.clone());
                                *vtx_idx += 1;
                            }
                        }
                    }
                    isurface.push(iring);
                }
                imsurface.push(isurface);
            }
            serde_json::to_string(&imsurface)
        }
        geom_static::Geometry::Solid { boundaries, .. } => {
            let mut isolid = Vec::new();
            for shell in boundaries {
                let mut ishell = Vec::new();
                for surface in shell {
                    let mut isurface = Vec::new();
                    for ring in &surface.boundaries {
                        let mut iring: Vec<usize> = Vec::new();
                        for vtx in ring {
                            let coord_str: String =
                                format!("{:.3} {:.3} {:.3}", vtx[0], vtx[1], vtx[2]);
                            match vtx_lookup.get(&coord_str) {
                                Some(existing_idx) => iring.push(existing_idx.clone()),
                                None => {
                                    vtx_lookup.insert(coord_str, vtx_idx.clone());
                                    iring.push(vtx_idx.clone());
                                    *vtx_idx += 1;
                                }
                            }
                        }
                        isurface.push(iring);
                    }
                    ishell.push(isurface);
                }
                isolid.push(ishell);
            }
            serde_json::to_string(&isolid)
        }
        _ => serde_json::to_string("not implemented"),
    }
}

pub fn deref_create(path_in: PathBuf) {
    let mut vtx_lookup: HashMap<String, usize> = HashMap::new();
    let mut vtx_idx: usize = 0;
    let mut vertices: geom_static::Vertices = Vec::new();

    let g = geom_static::Geometry::Solid {
        lod: "1.2".to_string(),
        boundaries: vec![vec![geom_static::Surface {
            #[rustfmt::skip]
            boundaries: vec![vec![
                [557299.0, 477667.0, 10693.0], [55799.0, 477667.0, 10693.0], [57299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 1063.0],[557299.0, 47767.0, 10693.0], [55799.0, 477667.0, 10693.0], [55799.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 1693.0], [557299.0, 477667.0, 1069.0],[557299.0, 47767.0, 10693.0], [57299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0],[557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0], [557299.0, 477667.0, 10693.0]
            ]],
            semantics: Some(geom_static::SemanticSurface::RoofSurface),
            material: Some(geom_static::Material {
                name: "SomeMaterial".to_string(),
                ambient_intensity: Some(3.0),
                diffuse_color: None,
                emissive_color: None,
                specular_color: None,
                shininess: None,
                transparency: None,
                is_smooth: Some(true),
            }),
            texture: None,
        }]],
    };
    let res = index_boundaries(&g, &mut vtx_lookup, &mut vtx_idx);
    for vtx in vtx_lookup.keys() {
        let mut v: [f64; 3] = [0.0, 0.0, 0.0];
        for (i, c) in vtx.split_whitespace().enumerate() {
            v[i] = c.parse::<f64>().unwrap();
        }
        vertices.push(v);
    }
    let co = geom_static::CityObjectSer {
        cotype: "Building".to_string(),
        geometry: vec![res.unwrap()],
    };
    let mut cos: HashMap<String, geom_static::CityObjectSer> = HashMap::new();
    cos.insert("id-1".to_string(), co);
    let mut cityjson = json!({
      "type": "CityJSON",
      "version": "1.1",
      "CityObjects": cos,
      "vertices": vertices
    });

    println!("{}", cityjson.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_data() -> PathBuf {
        Path::new("../data/cluster_bench.city.json")
            .canonicalize()
            .expect("Could not find the INPUT file.")
    }

    #[test]
    fn test_deref_deserialize() {
        let path_in = get_data();
        deref_deserialize(path_in)
    }

    #[test]
    fn test_deref_geometry() {
        let path_in = get_data();
        deref_geometry(path_in)
    }
}
