//! Test implementation of a custom Deserialize into a CityJSON structure.
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

mod indexed {
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
            boundaries: IMultiSurface,
            semantics: Option<ISemantics>,
        },
        Solid {
            lod: String,
            boundaries: ISolid,
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

#[derive(Debug)]
pub enum SemanticSurface {
    RoofSurface,
    GroundSurface,
    WallSurface,
}

#[derive(Debug)]
pub struct Semantics {
    pub surfaces: Vec<SemanticSurface>,
    pub values: Vec<Vec<usize>>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Texture {
    image: String,
}

type Point = [f64; 3];
type LineString = Vec<Point>;

#[derive(Debug)]
pub struct Surface {
    pub boundaries: Vec<LineString>,
    pub semantics: Option<SemanticSurface>,
    pub material: Option<Material>,
    pub texture: Option<Texture>,
}

type Shell = Vec<Surface>;

#[derive(Debug)]
pub enum Geometry {
    MultiSurface {
        lod: String,
        boundaries: Vec<Surface>,
        semantics: Option<Semantics>,
    },
    Solid {
        lod: String,
        boundaries: Vec<Shell>,
        semantics: Option<Semantics>,
    },
}

#[derive(Debug)]
pub struct CityObject {
    pub type_: String,
    pub geometry: Vec<Geometry>,
}

#[derive(Debug)]
pub struct CityModel {
    pub type_: String,
    pub version: String,
    pub cityobjects: HashMap<String, CityObject>,
}

fn boundary_dereference(
    vertices: &indexed::Vertices,
    geom: &indexed::IGeometry,
) -> Option<Geometry> {
    match geom {
        indexed::IGeometry::Solid {
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
                    let mut semsurf: Option<SemanticSurface> = None;
                    if let Some(sem) = semantics {
                        let sem_i = &sem.values[shi][sui];
                        match sem.surfaces[*sem_i].semtype.as_str() {
                            "GroundSurface" => {
                                semsurf = Some(SemanticSurface::GroundSurface);
                            }
                            "WallSurface" => {
                                semsurf = Some(SemanticSurface::WallSurface);
                            }
                            "RoofSurface" => {
                                semsurf = Some(SemanticSurface::RoofSurface);
                            }
                            &_ => {
                                println!("Semantic Surface type not implemented")
                            }
                        }
                    }
                    new_shell.push(Surface {
                        boundaries: surface_bdry,
                        semantics: semsurf,
                        material: None,
                        texture: None,
                    });
                }
                new_solid_bdry.push(new_shell);
            }
            Some(Geometry::Solid {
                lod: lod.to_string(),
                boundaries: new_solid_bdry,
                semantics: None,
            })
        }
        indexed::IGeometry::MultiSurface {
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
                let mut semsurf: Option<SemanticSurface> = None;

                new_msrf_bdry.push(Surface {
                    boundaries: surface_bdry,
                    semantics: semsurf,
                    material: None,
                    texture: None,
                });
            }
            Some(Geometry::MultiSurface {
                lod: lod.to_string(),
                boundaries: new_msrf_bdry,
                semantics: None,
            })
        }
        _ => {
            println!("Geometry type not implemented");
            None
        }
    }
}

impl<'de> Deserialize<'de> for CityModel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let icm: indexed::ICityModel = indexed::ICityModel::deserialize(deserializer)?;
        let mut new_cos: HashMap<String, CityObject> = HashMap::new();
        for (coid, co) in icm.cityobjects {
            let mut new_geoms: Vec<Geometry> = Vec::new();
            for geom in co.geometry {
                let g = boundary_dereference(&icm.vertices, &geom);
                new_geoms.push(g.expect("Error in converting geometry"));
            }
            let new_co = CityObject {
                type_: co.cotype,
                geometry: new_geoms,
            };
            new_cos.insert(coid, new_co);
        }

        Ok(CityModel {
            type_: icm.cmtype,
            version: icm.version,
            cityobjects: Default::default(),
        })
    }
}

fn main() {
    let cj = r#"
        {
          "CityObjects":{
            "id-1":{
              "type": "Building",
              "geometry":[
                {
                  "boundaries":[[[[0,1,2,3]],[[4,5,6,7]],[[0,3,5,4]],[[3,2,6,5]],[[2,1,7,6]],[[1,0,4,7]]]],
                  "type":"Solid",
                  "lod":"1"
                },
                {
                  "boundaries":[[[0,1,2,3]],[[4,5,6,7]],[[0,3,5,4]],[[3,2,6,5]],[[2,1,7,6]],[[1,0,4,7]]],
                  "type":"MultiSurface",
                  "lod":"0"
                }
              ]
            }
          },
          "vertices":[[0,0,0],[0,1000,0],[1000,1000,0],[1000,0,0],[0,0,1000],[1000,0,1000],[1000,1000,1000],[0,1000,1000]]   
        }"#;

    // let cm: indexed::ICityModel = serde_json::from_str(cj).unwrap();

    let path_in =
        Path::new("/data/3D_basisvoorziening/32cz1_2020_volledig/32cz1_04_bench.city.json")
            .canonicalize()
            .expect("Could not find the INPUT file.");

    let file = File::open(path_in).expect("Couldn't read CityJSON file");
    let reader = BufReader::new(file);
    let cm: CityModel =
        serde_json::from_reader(reader).expect("Couldn't deserialize into CityModel");
}
