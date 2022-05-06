#![allow(dead_code, unused_variables, unused_must_use)]

use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

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

// Custom Visitor for streaming the entries of the "geometries" object and parsing them.
// Adapted from: https://github.com/serde-rs/json/issues/160#issuecomment-841344394
fn for_each<'de, D, K, V, F>(deserializer: D, f: F) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
    F: FnMut(K, V),
{
    struct GeometryVisitor<K, V, F>(F, PhantomData<K>, PhantomData<V>);

    impl<'de, K, V, F> Visitor<'de> for GeometryVisitor<K, V, F>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
        F: FnMut(K, V),
    {
        type Value = ();

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a geometry object")
        }

        fn visit_map<A>(mut self, mut seq: A) -> Result<(), A::Error>
        where
            A: MapAccess<'de>,
        {
            while let Some((coid, value)) = seq.next_entry::<K, V>()? {
                self.0(coid, value)
            }
            Ok(())
        }
    }
    let visitor = GeometryVisitor(f, PhantomData, PhantomData);
    deserializer.deserialize_map(visitor)
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
                write!(formatter, "a CityObject")
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

// Custom Visitor for doing the second pass over the data and getting to the entries of the
// "geometries" object.
struct TargetStructVisitor;

impl<'de> Visitor<'de> for TargetStructVisitor {
    type Value = HashMap<String, TargetGeometry>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a valid file")
    }

    fn visit_map<A>(self, mut map: A) -> Result<HashMap<String, TargetGeometry>, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut geometries: HashMap<String, TargetGeometry> = HashMap::new();
        let source_vertices = SourceVertices {
            version: "".to_string(),
            geometries: Default::default(),
            transform: Transform {
                scale: [0.001, 0.001],
            },
            vertices: vec![[1000, 1000], [2000, 1000], [2000, 2000], [2000, 1000]],
        };
        while let Some(key) = map.next_key::<String>()? {
            println!("{:#?}", key);
            if key == "geometries" {
                println!("--> yaaay!");
                // I think the parsing of the 'geometries' object should happen here, like this:
                // for each {geometry ID : geometry object} entry in the 'geometries' object of the JSON data:
                //      1. deserialize the geometry object into a Geometry, by parsing its 'boundary' with using the previously deserialized SourceVertices
                //      2. add the newly created Geometry to the 'geometries' of the TargetStruct
                //      3. drop the {geometry ID : geometry object} entry from memory
                //
                // Essentially, I think need to incorporate the 'for_each' Visitor with its own
                // deserializer here.
                let a =
                    map.next_value_seed(SourceGeometryMap(&mut geometries, &source_vertices))?;
                geometries.shrink_to_fit();
            } else {
                println!("--> do nothing");
                let ignore_value: IgnoredAny = map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(geometries)
    }
}

impl<'de> Deserialize<'de> for TargetStruct {
    fn deserialize<D>(deserializer: D) -> Result<TargetStruct, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(TargetStruct {
            version: "".to_string(),
            geometries: deserializer.deserialize_map(TargetStructVisitor).unwrap(),
        })
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
    let cm: TargetStruct = serde_json::from_str(DATA).unwrap();
    println!("geometries from second pass:\n{:#?}", cm.geometries);
    println!("Done second pass.");

    // -- The streaming part
    // This part would happen in the second pass, but I include it here, because I don't know
    // how to integrate it into the Visitor of the TargetStruct.
    // Once I get into the the "geometries" object, I have the entries like below:
    static DATA_GEOMETRIES: &str = r##"
        {
            "id1": {"type": "Polygon", "boundary": [0,1,2,3]},
            "id2": {"type": "Polygon", "boundary": [0,1,2]}
        }
        "##;

    let mut geometries: HashMap<String, TargetGeometry> = HashMap::new();

    // The deserializer passes the geometry objects one-by-one to the Visitor, which then calls the
    // closure that parses the geometry "boundary" and appends the TargetGeometry to the final
    // container. I think I need this setup with the closure, since closures can capture values
    // from their which allows me to access the previously deserialized vertices.
    let mut deserializer = serde_json::Deserializer::from_str(DATA_GEOMETRIES);
    for_each(&mut deserializer, |key: String, value: SourceGeometry| {
        println!("parsing Geometry {}", key);
        let mut target_boundary: Vec<Point> = Vec::with_capacity(value.boundary.len());
        for vertex in value.boundary {
            let true_point: Point = [
                vertices.vertices[vertex][0] as f64 * vertices.transform.scale[0],
                vertices.vertices[vertex][1] as f64 * vertices.transform.scale[1],
            ];
            target_boundary.push(true_point);
        }
        geometries.insert(
            key,
            TargetGeometry {
                type_geom: value.type_geom,
                boundary: target_boundary,
            },
        );
    });
    println!("geometries:\n{:#?}", geometries);
    println!("Done streaming.");
}
