use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Geometry {
    MultiSurface {
        lod: String,
        boundaries: Vec<Vec<Vec<i64>>>,
    },
    Solid {
        lod: String,
        boundaries: Vec<Vec<Vec<Vec<i64>>>>,
    },
}

// #[derive(Deserialize, Debug)]
// #[serde(try_from = "IntermediateGeometry")]
// pub struct Geometry {
//     #[serde(rename = "type")]
//     pub type_: BoundaryType,
//     pub lod: String,
//     pub boundaries: Option<Boundary>,
// }

#[derive(Deserialize, Debug)]
pub struct CityObject {
    #[serde(rename = "type")]
    pub type_: String,
    pub geometry: Vec<Geometry>,
}

#[derive(Deserialize, Debug)]
pub struct CityModel {
    #[serde(rename = "CityObjects")]
    pub cityobjects: HashMap<String, CityObject>,
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

    let cm: CityModel = serde_json::from_str(cj).unwrap();
}
