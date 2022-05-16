#![allow(dead_code, unused_variables)]

#[derive(Debug)]
struct Geometry {
    boundary: Vec<usize>,
}

type Vertices = Vec<[f64; 2]>;

#[derive(Debug)]
struct Model {
    geometries: Vec<Geometry>,
    vertices: Vertices,
}

impl Model {
    fn drop_geometry(&mut self, i: usize) {
        if self.geometries.len() - 1 < i {
            println!("geometry index out of bounds")
        } else {
            let geom_removed = self.geometries.remove(i);
            let mut vtx_to_keep: Vec<usize> = Vec::new();
            // Need to iterate each Geometry, each boundary...
            for g in &self.geometries {
                for v in &g.boundary {
                    if geom_removed.boundary.contains(v) {
                        vtx_to_keep.push(v.clone())
                    }
                }
            }
            for v in &geom_removed.boundary {
                // Uh oh, this is not going to work, since Vec::remove() shifts all remaining
                // elements to the left, which messes up the vertex-indices in all the remaining
                // Geometries. In a naive approach for solving this would require iterating over
                // boundaries and shifting the vertex-indices for each removed vertex, which is
                // crazy.
                if !vtx_to_keep.contains(v) {
                    self.vertices.remove(*v);
                }
            }
        }
    }
}

fn main() {
    let vertices = vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0], [4.0, 4.0]];
    let g1 = Geometry {
        boundary: vec![0, 1, 2],
    };
    let g2 = Geometry {
        boundary: vec![2, 3, 4],
    };
    let mut model = Model {
        geometries: vec![g1, g2],
        vertices: vertices,
    };
    println!("{:#?}", model);

    model.drop_geometry(0);

    println!("{:#?}", model);

    model.drop_geometry(1);
}
