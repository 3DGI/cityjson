use std::collections::{BTreeMap, HashMap};

type Point = [f64; 3];

enum Boundary {
    MultiPoint(Vec<Point>),
}

struct Semantic {
    name: String,
}

struct Geometry {
    boundary_components: Vec<Option<Boundary>>,
    semantic_components: Vec<Option<Semantic>>,
}

impl Geometry {
    fn new() -> Self {
        Self {
            boundary_components: Vec::new(),
            semantic_components: Vec::new(),
        }
    }

    fn new_entity(&mut self, boundary: Option<Boundary>, semantic: Option<Semantic>) {
        self.boundary_components.push(boundary);
        self.semantic_components.push(semantic);
    }
}

fn main() {
    let mut g = Geometry::new();
    g.new_entity(
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
    }
}
