use std::collections::{BTreeMap, HashMap};

type Point = [f64; 3];
type LineString = Vec<Point>;

enum Geometry {
    Surface(LineString),
}

struct Material {
    name: String,
}

struct CityModel {
    boundary_components: Vec<Option<Geometry>>,
    material_components: Vec<Option<Material>>,
}

impl CityModel {
    fn new() -> Self {
        Self {
            boundary_components: Vec::new(),
            material_components: Vec::new(),
        }
    }

    fn new_entity(&mut self, boundary: Option<Geometry>, material: Option<Material>) {
        self.boundary_components.push(boundary);
        self.material_components.push(material);
    }
}

fn main() {
    let mut cm = CityModel::new();
    cm.new_entity(
        Some(Geometry::Surface(vec![[1.0, 1.0, 1.0], [2.0, 2.0, 2.0]])),
        Some(Material {
            name: "mat1".to_string(),
        }),
    );
    cm.new_entity(
        Some(Geometry::Surface(vec![[3.0, 1.0, 1.0], [2.0, 2.0, 2.0]])),
        Some(Material {
            name: "mat1".to_string(),
        }),
    );
    cm.new_entity(
        Some(Geometry::Surface(vec![[4.0, 1.0, 1.0], [2.0, 2.0, 2.0]])),
        None,
    );

    let zip = cm
        .boundary_components
        .iter()
        .zip(cm.material_components.iter());
    let with_boundary_and_material = zip.filter_map(
        |(boundary, material): (&Option<Geometry>, &Option<Material>)| {
            Some((boundary.as_ref()?, material.as_ref()?))
        },
    );

    for (boundary, material) in with_boundary_and_material {
        match boundary {
            Geometry::Surface(b) => {
                println!("{:#?}, {}", b, material.name)
            }
        }
    }
}
