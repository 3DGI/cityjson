#![allow(unused, irrefutable_let_patterns)]
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use clap::{crate_version, App, Arg};
use serde::Serialize;
use serde_json::{json, Value};

fn parse_direct(path_in: PathBuf) -> serde_json::Value {
    let file = File::open(path_in).expect("Couldn't read CityJSON file");
    let reader = BufReader::new(file);
    let cm: Value = serde_json::from_reader(reader).expect("Couldn't deserialize into ICityModel");
    return cm;
}

/// Parse a CityJSON file as-is, just by using serde_json's generic Value type.
/// Not using any CityJSON specific structure.
/// This is the quickest to get started, but requires lots of code later on in order to unwrap the
/// individual JSON members.
fn direct_deserialize(path_in: PathBuf) {
    let cm = parse_direct(path_in);
}

/// Get the boundary coordinates of each surface.
fn direct_geometry(path_in: PathBuf) {
    let mut containter: Vec<[f64; 3]> = Vec::new();
    let cm = parse_direct(path_in);
    let cos = cm["CityObjects"].as_object().unwrap();
    let vertices = cm["vertices"].as_array().unwrap();
    for (coid, coval) in cos {
        // println!("Processing CityObject {}", coid);
        let geometry = coval
            .as_object()
            .unwrap()
            .get("geometry")
            .unwrap()
            .as_array()
            .unwrap();
        for geom in geometry {
            // Really need to be careful with the data types in the file
            // println!("LoD: {}", geom["lod"].as_str().unwrap());
            if geom["type"].as_str().unwrap() == "Solid" {
                for shell in geom["boundaries"].as_array().unwrap() {
                    for surface in shell.as_array().unwrap() {
                        for ring in surface.as_array().unwrap() {
                            for vtx_idx in ring.as_array().unwrap() {
                                let v = vertices[vtx_idx.as_u64().unwrap() as usize]
                                    .as_array()
                                    .unwrap();
                                let point: [f64; 3] = [
                                    v[0].as_f64().unwrap().clone(),
                                    v[1].as_f64().unwrap().clone(),
                                    v[2].as_f64().unwrap().clone(),
                                ];
                                containter.push(point);
                            }
                        }
                    }
                }
            } else {
                println!("Not a Solid geometry")
            }
        }
    }
    println!(
        "In total there are {} points in the citymodel and {} vertices",
        containter.len(),
        vertices.len()
    )
}

/// Get the boundary coordinates of a given semantic surface
fn direct_semantics(path_in: PathBuf) {
    let semantic_type = "RoofSurface";
    println!("extracting the geometry of {}", semantic_type);
    let mut containter: Vec<[f64; 3]> = Vec::new();
    let cm = parse_direct(path_in);
    let cos = cm["CityObjects"].as_object().unwrap();
    let vertices = cm["vertices"].as_array().unwrap();
    for coval in cos.values() {
        let geometry = coval
            .as_object()
            .unwrap()
            .get("geometry")
            .unwrap()
            .as_array()
            .unwrap();
        for geom in geometry {
            if !geom["semantics"].is_null() {
                let mut si: Option<usize> = None;
                for (i, semsrf) in geom["semantics"]["surfaces"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                {
                    if semsrf["type"].as_str().unwrap() == semantic_type {
                        si = Some(i);
                    }
                }
                if let Some(semsrf_idx) = si {
                    // Really need to be careful with the data types in the file
                    // println!("LoD: {}", geom["lod"].as_str().unwrap());
                    let shells = geom["boundaries"].as_array().unwrap();
                    if geom["type"].as_str().unwrap() == "Solid" {
                        for (shell_i, sem_shell) in geom["semantics"]["values"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .enumerate()
                        {
                            let shell: &Value = &geom["boundaries"].as_array().unwrap()[shell_i];
                            for (srf_i, sem_surface) in
                                sem_shell.as_array().unwrap().iter().enumerate()
                            {
                                let surface: &Value = &shell[&srf_i];
                                if sem_surface.as_i64().unwrap() == semsrf_idx as i64 {
                                    for ring in surface.as_array().unwrap() {
                                        for vtx_idx in ring.as_array().unwrap() {
                                            let v = vertices[vtx_idx.as_u64().unwrap() as usize]
                                                .as_array()
                                                .unwrap();
                                            let point: [f64; 3] = [
                                                v[0].as_f64().unwrap().clone(),
                                                v[1].as_f64().unwrap().clone(),
                                                v[2].as_f64().unwrap().clone(),
                                            ];
                                            containter.push(point);
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        println!("Not a Solid geometry")
                    }
                } else {
                    println!(
                        "The requested semantic surface type, {}, is not found on the geometry.",
                        semantic_type
                    )
                }
            }
        }
    }
    println!(
        "In total there are {} points in the citymodel and {} vertices",
        containter.len(),
        vertices.len()
    )
}

/// Create a new city model from scratch
fn direct_create(path_in: PathBuf) {
    // CityJSON structures
    #[derive(Serialize)]
    struct SemanticSurface {
        #[serde(rename = "type")]
        semtype: String,
    }

    #[derive(Serialize)]
    struct Semantics {
        surfaces: Vec<SemanticSurface>,
        values: Vec<Vec<i32>>,
    }
    #[derive(Serialize)]
    struct Geometry {
        #[serde(rename = "type")]
        geomtype: String,
        lod: String,
        boundaries: Vec<Vec<[[i32; 3]; 1]>>,
        semantics: Semantics,
    }
    #[derive(Serialize)]
    struct CityObject {
        #[serde(rename = "type")]
        cotype: String,
        geometry: Geometry,
    }
    let mut cityobjects: HashMap<String, CityObject> = HashMap::new();

    // Input values
    let semantics = Semantics {
        surfaces: vec![
            SemanticSurface {
                semtype: "GroundSurface".to_owned(),
            },
            SemanticSurface {
                semtype: String::from("RoofSurface"),
            },
            SemanticSurface {
                semtype: String::from("WallSurface"),
            },
            SemanticSurface {
                semtype: String::from("WallSurface"),
            },
        ],
        values: vec![vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            2, 3, 2, 2, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 2, 2, 2, 3, 3, 2, 2, 3, 3, 2,
            2, 3, 3, 3, 2, 2, 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ]],
    };
    let geometry = Geometry {
        geomtype: String::from("Solid"),
        lod: String::from("2.2"),
        #[rustfmt::skip]
        boundaries: vec![vec![
            [[32, 33, 34]],[[35, 36, 37]],[[35, 38, 36]],[[34, 39, 40]],[[41, 42, 43]],[[44, 35, 42]],[[34, 33, 38]],[[42, 37, 43]],[[34, 38, 39]],[[35, 37, 42]],[[38, 33, 36]],[[45, 39, 46]],[[47, 48, 45]],[[40, 39, 45]],[[40, 48, 49]],[[48, 40, 45]],[[40, 49, 50]],[[51, 52, 53]],[[54, 50, 52]],[[40, 50, 54]],[[52, 50, 53]],[[55, 42, 56]],[[56, 42, 41]],[[57, 52, 58]],[[58, 52, 51]],[[58, 51, 53]],[[59, 58, 53]],[[59, 53, 50]],[[60, 59, 50]],[[61, 62, 63]],[[63, 62, 48]],[[48, 62, 49]],[[63, 48, 64]],[[64, 48, 47]],[[65, 44, 55]],[[55, 44, 42]],[[66, 45, 46]],[[67, 66, 46]],[[68, 40, 69]],[[69, 40, 54]],[[70, 38, 71]],[[72, 70, 71]],[[71, 38, 35]],[[71, 35, 65]],[[65, 35, 44]],[[73, 43, 37]],[[74, 73, 37]],[[75, 76, 77]],[[69, 54, 52]],[[57, 69, 52]],[[78, 36, 75]],[[75, 36, 33]],[[79, 74, 78]],[[80, 79, 78]],[[36, 74, 37]],[[78, 74, 36]],[[76, 75, 81]],[[81, 33, 32]],[[81, 75, 33]],[[80, 78, 82]],[[81, 32, 83]],[[83, 32, 34]],[[84, 85, 86]],[[87, 84, 86]],[[88, 89, 90]],[[83, 91, 92]],[[89, 88, 92]],[[91, 83, 68]],[[68, 83, 40]],[[40, 83, 34]],[[64, 47, 66]],[[93, 64, 66]],[[66, 47, 45]],[[93, 66, 94]],[[95, 93, 94]],[[95, 94, 96]],[[96, 94, 97]],[[67, 46, 39]],[[84, 67, 85]],[[85, 67, 39]],[[87, 86, 98]],[[99, 87, 98]],[[56, 41, 73]],[[73, 41, 43]],[[72, 71, 79]],[[79, 71, 74]],[[85, 39, 70]],[[70, 39, 38]],[[99, 98, 90]],[[100, 99, 90]],[[100, 90, 101]],[[60, 50, 49]],[[62, 60, 49]],[[96, 97, 61]],[[61, 97, 62]],[[73, 74, 55]],[[56, 73, 55]],[[65, 55, 71]],[[55, 74, 71]],[[80, 82, 79]],[[88, 85, 70]],[[88, 86, 85]],[[90, 86, 88]],[[98, 86, 90]],[[102, 88, 70]],[[70, 72, 82]],[[102, 70, 82]],[[72, 79, 82]],[[77, 78, 75]],[[102, 82, 77]],[[77, 82, 78]],[[81, 83, 77]],[[102, 92, 88]],[[77, 83, 92]],[[76, 81, 77]],[[77, 92, 102]],[[101, 90, 89]],[[68, 101, 92]],[[92, 101, 89]],[[68, 92, 91]],[[96, 61, 63]],[[93, 63, 64]],[[95, 96, 63]],[[95, 63, 93]],[[84, 87, 94]],[[62, 97, 60]],[[69, 97, 68]],[[58, 59, 57]],[[60, 69, 57]],[[67, 94, 66]],[[84, 94, 67]],[[87, 99, 94]],[[99, 101, 94]],[[100, 101, 99]],[[101, 97, 94]],[[68, 97, 101]],[[69, 60, 97]],[[59, 60, 57]],
        ]],
        semantics,
    };
    let co1 = CityObject {
        cotype: String::from("Building"),
        geometry: geometry,
    };
    #[rustfmt::skip]
    let vertices: Vec<[i32; 3]> = vec![
        [557299, 477667, 10693],[555320, 476313, 10693],[557152, 477874, 10693],[549876, 475091, 10693],[552052, 474077, 10693],[551065, 473402, 10693],[553522, 479989, 10693],[555453, 480421, 10693],[548712, 476744, 10693],[557507, 481984, 10693],[555656, 480576, 10693],[557176, 482455, 10693],[557199, 477916, 10693],[551065, 473402, 15843],[552052, 474077, 16835],[555453, 480421, 14227],[555656, 480576, 13908],[557507, 481984, 18856],[557176, 482455, 19352],[553522, 479989, 14390],[548712, 476744, 14396],[557299, 477667, 14248],[557152, 477874, 14254],[555656, 480576, 18755],[554986, 480392, 18951],[554986, 480392, 14732],[555320, 476313, 17254],[557199, 477916, 14176],[553522, 479989, 19378],[549876, 475091, 15824],[552052, 474077, 17275],[550262, 474985, 16037],[557176, 482455, 10702],[556304, 481867, 10702],[555243, 485207, 10702],[552562, 482469, 10702],[554777, 480838, 10702],[553123, 479723, 10702],[552685, 482551, 10702],[551741, 484021, 10702],[554620, 486095, 10702],[547299, 478859, 10702],[547432, 478948, 10702],[548709, 476748, 10702],[547376, 479031, 10702],[551049, 487955, 10702],[549766, 487097, 10702],[551661, 488364, 10702],[551706, 488298, 10702],[552266, 488672, 10702],[552521, 488842, 10702],[552501, 488925, 10702],[552518, 488900, 10702],[552476, 488909, 10702],[552608, 488960, 10702],[547432, 478948, 13393],[547299, 478859, 13391],[552518, 488900, 16862],[552501, 488925, 16833],[552476, 488909, 16833],[552521, 488842, 16912],[552266, 488672, 18502],[552266, 488672, 16920],[551706, 488298, 18498],[551661, 488364, 18473],[547376, 479031, 13394],[551049, 487955, 16878],[549766, 487097, 16919],[554620, 486095, 20296],[552608, 488960, 16860],[552685, 482551, 19295],[552562, 482469, 13488],[552562, 482469, 19205],[548709, 476748, 13360],[553123, 479723, 13440],[556304, 481867, 19374],[556304, 481867, 19538],[556017, 482028, 19619],[554777, 480838, 19377],[553123, 479723, 18550],[554777, 480838, 19775],[557176, 482455, 18887],[554563, 481386, 19854],[555243, 485207, 18793],[551741, 484021, 20512],[551741, 484021, 19318],[552306, 484680, 19832],[552306, 484680, 20259],[553002, 484553, 20143],[553002, 484553, 19705],[552789, 484847, 20135],[555243, 485207, 19010],[554502, 484991, 19240],[551049, 487955, 18468],[552028, 486453, 18640],[552028, 486453, 19015],[552860, 487063, 19008],[552860, 487063, 18568],[552725, 484937, 20133],[552725, 484937, 20266],[552789, 484847, 20374],[552914, 485052, 20269],[554801, 482057, 20205],[556914, 492127, 10855],[556991, 492013, 10855],[553341, 489486, 10855],[552608, 488960, 10855],[553358, 489461, 10855],[554667, 486028, 10855],[558168, 490735, 10855],[557156, 492129, 10855],[558565, 490189, 10855],[562123, 485287, 10855],[559869, 483783, 10855],[561528, 481381, 10855],[562573, 479378, 10855],[561302, 481197, 10855],[565101, 481187, 10855],[555841, 484357, 10855],[557507, 481984, 10855],[557350, 482208, 10855],[557350, 482208, 14303],[557507, 481984, 14298],[556914, 492127, 17304],[553341, 489486, 17107],[559869, 483783, 14296],[561528, 481381, 14244],[561302, 481197, 14244],[562573, 479378, 14204],[565101, 481187, 14201],[555841, 484357, 17153],[557350, 482208, 17152],[560090, 487434, 19603],[560090, 487434, 17105],[561720, 485027, 17105],[561720, 485027, 19610],[552608, 488960, 17056],[554667, 486028, 20208],[553358, 489461, 17134],[556991, 492013, 17424],[557156, 492129, 17438],[555841, 484357, 17593],[558565, 490189, 18626],[558168, 490735, 18955],[558565, 490189, 19550],[558376, 489967, 18595],[558376, 489967, 19595],[555519, 486558, 20323],[555519, 486558, 18105],[554774, 488035, 18899],[562123, 485287, 19531],[562123, 485287, 14292],[561720, 485027, 14293],[556418, 484775, 17147],[556418, 484775, 17740],
    ];
    cityobjects.insert(String::from("id-1"), co1);

    let mut cityjson = json!({
      "type": "CityJSON",
      "version": "1.1",
      "transform": {
        "scale": [0.001, 0.001,0.001],
        "translate": [84386.96100146485, 446948.33899072267,-10.659793853759766]
      },
      "CityObjects": cityobjects,
      "vertices": vertices
    });
    println!("{}", cityjson.to_string());
}

// Dereference -----------------

mod geom_deref_static;
use crate::geom_deref_static::geom_static;

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
        pub(crate) cotype: String,
        pub(crate) geometry: Vec<IGeometry>,
    }

    #[derive(Deserialize)]
    struct Transform {
        scale: [f64; 3],
        translate: [f64; 3],
    }

    #[derive(Deserialize)]
    pub(crate) struct ICityModel {
        #[serde(rename = "type")]
        pub(crate) cmtype: String,
        pub(crate) version: String,
        transform: Transform,
        #[serde(rename = "CityObjects")]
        pub(crate) cityobjects: HashMap<String, ICityObject>,
        pub(crate) vertices: Vertices,
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
    let reader = BufReader::new(file);
    let icm: deserialize::ICityModel =
        serde_json::from_reader(reader).expect("Couldn't deserialize into ICityModel");

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

fn deref_deserialize(path_in: PathBuf) {
    let cm = parse_dereferece(path_in);
}

fn deref_geometry(path_in: PathBuf) {
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

fn deref_semantics(path_in: PathBuf) {
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

fn deref_create(path_in: PathBuf) {
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

// CLI -------------------------

static USECASES: [&str; 5] = [
    "deserialize",
    "serialize",
    "geometry",
    "semantics",
    "create",
];

enum Architectures {
    DirectJson,
    VertexIndex,
    Dereference,
}

impl Architectures {
    fn run_usecase(&self, case: &str, path_in: PathBuf) {
        match case {
            "deserialize" => self.deserialize(path_in),
            "serialize" => self.serialize(path_in),
            "geometry" => self.geometry(path_in),
            "semantics" => self.semantics(path_in),
            "create" => self.create(path_in),
            _ => {}
        }
    }
    fn deserialize(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => direct_deserialize(path_in),
            Architectures::VertexIndex => {
                println!("Not implemented")
            }
            Architectures::Dereference => deref_deserialize(path_in),
        }
    }
    fn serialize(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => {
                println!("Not implemented")
            }
            Architectures::VertexIndex => {
                println!("Not implemented")
            }
            Architectures::Dereference => {
                println!("Not implemented")
            }
        }
    }
    fn geometry(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => direct_geometry(path_in),
            Architectures::VertexIndex => {
                println!("Not implemented")
            }
            Architectures::Dereference => deref_geometry(path_in),
        }
    }
    fn semantics(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => direct_semantics(path_in),
            Architectures::VertexIndex => {
                println!("Not implemented")
            }
            Architectures::Dereference => deref_semantics(path_in),
        }
    }
    fn create(&self, path_in: PathBuf) {
        match self {
            Architectures::DirectJson => direct_create(path_in),
            Architectures::VertexIndex => {
                println!("Not implemented")
            }
            Architectures::Dereference => deref_create(path_in),
        }
    }
}

fn main() {
    let dispatch_architecture = HashMap::from([
        ("direct-json", Architectures::DirectJson),
        ("vertex-index", Architectures::VertexIndex),
        ("dereference", Architectures::Dereference),
    ]);
    let archs: Vec<&str> = dispatch_architecture.keys().cloned().collect();

    let app = App::new("benchmark")
        .about("Benchmark the potential cjlib architectures")
        .version(crate_version!())
        .arg(
            Arg::with_name("ARCH")
                .short("a")
                .long("architecture")
                .required(true)
                .help("The cjlib architecture")
                .takes_value(true)
                .possible_values(&archs),
        )
        .arg(
            Arg::with_name("CASE")
                .short("c")
                .long("case")
                .required(true)
                .help("The use case")
                .takes_value(true)
                .possible_values(&USECASES),
        )
        .arg(
            Arg::with_name("INPUT")
                .required(true)
                .help("CityJSON file to benchmark."),
        );
    let matches = app.get_matches();

    let path_in = Path::new(matches.value_of("INPUT").unwrap())
        .canonicalize()
        .expect("Could not find the INPUT file.");

    let arch = dispatch_architecture
        .get(&matches.value_of("ARCH").unwrap())
        .unwrap();
    arch.run_usecase(matches.value_of("CASE").unwrap(), path_in)
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
    fn test_direct_geometry() {
        let path_in = get_data();
        direct_geometry(path_in)
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
