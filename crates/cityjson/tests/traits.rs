use cjgeometry::vertex::{Coordinate, Index, Vertex, VertexCoordinate, VertexIndex};

#[test]
fn test_vertex_trait_implementations() {
    fn assert_vertex_reference<T: Vertex>() {}
    fn assert_coordinate3d<T: Coordinate>() {}
    fn assert_index_reference<T: Index>() {}

    assert_vertex_reference::<VertexCoordinate>();
    assert_vertex_reference::<VertexIndex>();

    assert_coordinate3d::<VertexCoordinate>();
    assert_index_reference::<VertexIndex>();
}
