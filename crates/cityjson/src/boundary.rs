use crate::{Vertex, VertexCoordinate};

#[derive(Clone, Debug)]
pub struct Boundary<V: Vertex> {
    pub(crate) vertices: Vec<V>,
    pub(crate) rings: Vec<u32>,    // Indices into vertices
    pub(crate) surfaces: Vec<u32>, // Indices into rings
    pub(crate) shells: Vec<u32>,   // Indices into surfaces
    pub(crate) solids: Vec<u32>,   // Indices into shells
}

#[test]
fn test_boundary_creation() {
    let mut boundary = Boundary {
        vertices: vec![],
        rings: vec![],
        surfaces: vec![],
        shells: vec![],
        solids: vec![],
    };

    // Add vertices
    boundary.vertices.push(VertexCoordinate {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    });
    boundary.vertices.push(VertexCoordinate {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    });
    boundary.vertices.push(VertexCoordinate {
        x: 1.0,
        y: 1.0,
        z: 0.0,
    });

    // Add a ring (triangle)
    boundary.rings.push(0); // Start index of vertices

    assert_eq!(boundary.vertices.len(), 3);
    assert_eq!(boundary.rings.len(), 1);
}