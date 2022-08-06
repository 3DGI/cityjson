use std::collections::{BTreeMap, HashMap};

// But this is still not optimal, because let's say we have 10 Geometries, each of them referencing
// the same location (the same Point), but each of them assigns a different meaning to it
// (semantic). With the current setup, we need to duplicate the coordinates 10 times, just that each
// can store the semantic of the respective Geometry.
struct Point {
    x: f64,
    y: f64,
    z: f64,
    semantic: Option<u16>,
}

struct LineString {
    start: u32,
    end: u32,
    semantic: Option<u16>,
}

struct Surface {
    boundary: Vec<LineString>,
    semantic: Option<u16>,
    material: Option<u16>,
    texture: Option<u16>,
}

enum Boundary {
    MultiPoint(Vec<Point>),
    MultiLineString(Vec<LineString>),
}

enum Semantic {
    TransportationHole,
    TransportationMarking,
}

enum LoD {
    LoD0,
    LoD1,
    LoD2_2,
}

struct Geometry {
    lod: Option<LoD>,
    boundary: Option<Boundary>,
}

impl Geometry {
    fn new() -> Self {
        Self {
            lod: None,
            boundary: None,
        }
    }

    /*    fn new_entity(&mut self, boundary: Option<Boundary>, semantic: Option<Semantic>) {
        self.boundary_components.push(boundary);
        self.semantic_components.push(semantic);
    }*/
}

fn main() {
    let mut g = Geometry::new();
    /*    g.new_entity(
        Some(Boundary::MultiPoint(vec![[1.0, 1.0, 1.0], [2.0, 2.0, 2.0]])),
        Some(Semantic {
            name: "TransportationMarking".to_string(),
        }),
    );
    g.new_entity(
        Some(Boundary::MultiPoint(vec![[3.0, 1.0, 1.0], [2.0, 2.0, 2.0]])),
        Some(Semantic {
            name: "TransportationHole".to_string(),
        }),
    );
    g.new_entity(
        Some(Boundary::MultiPoint(vec![[4.0, 1.0, 1.0], [2.0, 2.0, 2.0]])),
        None,
    );

    let zip = g
        .boundary_components
        .iter()
        .zip(g.semantic_components.iter());
    let with_boundary_and_semantic = zip.filter_map(
        |(boundary, semantic): (&Option<Boundary>, &Option<Semantic>)| {
            Some((boundary.as_ref()?, semantic.as_ref()?))
        },
    );

    for (boundary, semantic) in with_boundary_and_semantic {
        match boundary {
            Boundary::MultiPoint(b) => {
                println!("{:#?}, {}", b, semantic.name)
            }
        }
    }*/
}
