/// Container for vertex coordinates.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct Vertices(Vec<VertexCoordinate>);

/// 3D vertex coordinate
#[repr(C, align(32))]
#[derive(Clone, Debug)]
pub struct VertexCoordinate {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
}

impl VertexCoordinate {
    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }

    #[inline]
    pub fn y(&self) -> f64 {
        self.y
    }

    #[inline]
    pub fn z(&self) -> f64 {
        self.z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertices_container() {
        let vertices = Vertices(vec![
            VertexCoordinate {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            VertexCoordinate {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
        ]);

        assert_eq!(vertices.0.len(), 2);
    }
}
