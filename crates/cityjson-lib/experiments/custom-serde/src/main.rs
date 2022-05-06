#![allow(dead_code, unused_variables, unused_must_use)]

use std::collections::HashMap;
use std::fmt;

use serde::de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

type Point = [f64; 2];

#[derive(Debug)]
struct TargetGeometry {
    type_geom: String,
    boundary: Vec<Point>,
}

struct TargetStruct {
    version: String,
    geometries: HashMap<String, TargetGeometry>,
}

#[derive(Deserialize, Debug)]
struct SourceGeometry {
    #[serde(rename = "type")]
    type_geom: String,
    boundary: Vec<usize>,
}

#[derive(Deserialize)]
struct Transform {
    scale: [f64; 2],
}

#[derive(Deserialize)]
struct SourceVertices {
    version: String,
    #[serde(skip)]
    geometries: HashMap<String, SourceGeometry>,
    transform: Transform,
    vertices: Vec<[i32; 2]>,
}

struct SourceGeometryMap<'a>(&'a mut HashMap<String, TargetGeometry>, &'a SourceVertices);

impl<'de, 'a> DeserializeSeed<'de> for SourceGeometryMap<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SourceGeometryMapVisitor<'a>(
            &'a mut HashMap<String, TargetGeometry>,
            &'a SourceVertices,
        );

        impl<'de, 'a> Visitor<'de> for SourceGeometryMapVisitor<'a> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a Geometry object")
            }

            fn visit_map<A>(self, mut map: A) -> Result<(), A::Error>
            where
                A: MapAccess<'de>,
            {
                while let Some((coid, co)) = map.next_entry::<String, SourceGeometry>()? {
                    println!("parsing Geometry {} in SourceGeometryMapVisitor", coid);
                    let mut target_boundary: Vec<Point> = Vec::with_capacity(co.boundary.len());
                    for vertex in co.boundary {
                        let true_point: Point = [
                            self.1.vertices[vertex][0] as f64 * self.1.transform.scale[0],
                            self.1.vertices[vertex][1] as f64 * self.1.transform.scale[1],
                        ];
                        target_boundary.push(true_point);
                    }
                    self.0.insert(
                        coid,
                        TargetGeometry {
                            type_geom: co.type_geom,
                            boundary: vec![[1.0, 2.0], [1.0, 2.0]],
                        },
                    );
                }
                Ok(())
            }
        }
        deserializer.deserialize_map(SourceGeometryMapVisitor(self.0, self.1))
    }
}

struct TargetStructMap<'a>(&'a mut TargetStruct, &'a SourceVertices);

impl<'de, 'a> DeserializeSeed<'de> for TargetStructMap<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TargetStructVisitor<'a>(&'a mut TargetStruct, &'a SourceVertices);

        impl<'de, 'a> Visitor<'de> for TargetStructVisitor<'a> {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a valid file")
            }

            fn visit_map<A>(self, mut map: A) -> Result<(), A::Error>
            where
                A: MapAccess<'de>,
            {
                while let Some(key) = map.next_key::<String>()? {
                    println!("{:#?}", key);
                    if key == "geometries" {
                        println!("--> yaaay!");
                        let a = map
                            .next_value_seed(SourceGeometryMap(&mut self.0.geometries, &self.1))?;
                        self.0.geometries.shrink_to_fit();
                    } else {
                        println!("--> do nothing");
                        let ignore_value: IgnoredAny = map.next_value::<IgnoredAny>()?;
                    }
                }
                Ok(())
            }
        }

        self.0.version = "from TargetStructMap".to_string();
        deserializer.deserialize_map(TargetStructVisitor(self.0, self.1));
        Ok(())
    }
}

fn main() {
    // This is a simplified version of my data. The complete version contains more properties, but
    // they not relevant to this problem.
    // The important part is that,
    //  - a geometry object has a "boundary",
    //  - a "boundary" is defined by an array of vertex-indices,
    //  - a vertex-index is the index of a point in the "vertices" array.
    // Additionally,
    //  - a point is an array of two coordinates,
    //  - a coordinate is stored in the JSON file as an integer and we can obtain the original
    //      coordinate value by multiplying the coordinate-integer by the scaling factor from the
    //      "transform" object.
    static DATA: &str = r##"
        {
            "version": "1",
            "geometries": {
                "id1": {"type": "Polygon", "boundary": [0,1,2,3]},
                "id2": {"type": "Polygon", "boundary": [0,1,2]}
            },
            "transform": {"scale": [0.001,0.001]},
            "vertices": [[1000,1000],[2000,1000],[2000,2000],[2000,1000]]
        }
        "##;

    // -- First pass
    // In the first pass, deserialize the 'vertices' and other properties that are required for
    // deserializing the "geometries". But skip the "geometries" now, because they need the the
    // "vertices" for parsing their "boundary".
    let vertices: SourceVertices = serde_json::from_str(DATA).unwrap();
    println!("Done first pass.");

    // -- Second pass
    // In the second pass through the data, deserialize the "geometries" by using the values from
    // the "vertices" of a SourceVertices.
    let mut cm = TargetStruct {
        version: "".to_string(),
        geometries: HashMap::new(),
    };
    let tsm = TargetStructMap(&mut cm, &vertices);
    let mut deserializer = serde_json::Deserializer::from_str(DATA);
    tsm.deserialize(&mut deserializer);
    println!("geometries from second pass:\n{:#?}", cm.geometries);
    println!("Done second pass.");
}
