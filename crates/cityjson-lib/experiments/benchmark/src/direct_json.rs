//! Direct-JSON architecture.
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use ijson::IValue;
use serde::Serialize;
use serde_json::{json, Value};

fn parse_direct(path_in: PathBuf) -> ijson::IValue {
    let file = File::open(path_in).expect("Couldn't read CityJSON file");
    let reader = BufReader::new(file);
    let cm: ijson::IValue =
        serde_json::from_reader(reader).expect("Couldn't deserialize into ICityModel");
    return cm;
}

/// Parse a CityJSON file as-is, just by using serde_json's generic Value type.
/// Not using any CityJSON specific structure.
/// This is the quickest to get started, but requires lots of code later on in order to unwrap the
/// individual JSON members.
pub fn direct_deserialize(path_in: PathBuf) {
    let cm = parse_direct(path_in);
}

/// Get the boundary coordinates of each surface.
pub fn direct_geometry(path_in: PathBuf) {
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
            if geom["type"].as_string().unwrap() == "Solid" {
                for shell in geom["boundaries"].as_array().unwrap() {
                    for surface in shell.as_array().unwrap() {
                        for ring in surface.as_array().unwrap() {
                            for vtx_idx in ring.as_array().unwrap() {
                                let v = vertices[vtx_idx.to_u64().unwrap() as usize]
                                    .as_array()
                                    .unwrap();
                                let point: [f64; 3] = [
                                    v[0].to_f64().unwrap().clone(),
                                    v[1].to_f64().unwrap().clone(),
                                    v[2].to_f64().unwrap().clone(),
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
pub fn direct_semantics(path_in: PathBuf) {
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
                    if semsrf["type"].as_string().unwrap() == semantic_type {
                        si = Some(i);
                    }
                }
                if let Some(semsrf_idx) = si {
                    // Really need to be careful with the data types in the file
                    // println!("LoD: {}", geom["lod"].as_str().unwrap());
                    let shells = geom["boundaries"].as_array().unwrap();
                    if geom["type"].as_string().unwrap() == "Solid" {
                        for (shell_i, sem_shell) in geom["semantics"]["values"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .enumerate()
                        {
                            let shell: &IValue = &geom["boundaries"].as_array().unwrap()[shell_i];
                            for (srf_i, sem_surface) in
                                sem_shell.as_array().unwrap().iter().enumerate()
                            {
                                let surface: &IValue = &shell[&srf_i];
                                if sem_surface.to_i64().unwrap() == semsrf_idx as i64 {
                                    for ring in surface.as_array().unwrap() {
                                        for vtx_idx in ring.as_array().unwrap() {
                                            let v = vertices[vtx_idx.to_u64().unwrap() as usize]
                                                .as_array()
                                                .unwrap();
                                            let point: [f64; 3] = [
                                                v[0].to_f64().unwrap().clone(),
                                                v[1].to_f64().unwrap().clone(),
                                                v[2].to_f64().unwrap().clone(),
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
pub fn direct_create(path_in: PathBuf) {
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
}
