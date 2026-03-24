use std::fmt;
#[derive(Debug)]
enum Geometry {
    MultiPoint { boundaries: MultiPointBoundary },
    MultiLineString { boundaries: MultiLineStringBoundary },
}

#[derive(Debug, Clone)]
struct MultiLineStringBoundary(Vec<PointBoundary>);

#[derive(Debug, Clone)]
struct MultiPointBoundary(Vec<PointBoundary>);

#[derive(Clone, Debug, Default)]
struct PointBoundary {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PointBoundary {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        PointBoundary { x, y, z }
    }
}

impl fmt::Display for PointBoundary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} {} {})", self.x, self.y, self.z)
    }
}

impl From<&[f64; 3]> for PointBoundary {
    fn from(value: &[f64; 3]) -> Self {
        PointBoundary {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl From<[f64; 3]> for PointBoundary {
    fn from(value: [f64; 3]) -> Self {
        PointBoundary {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

trait Boundary {
    fn mean_point(&self) -> PointBoundary;
}
impl Boundary for MultiLineStringBoundary {
    fn mean_point(&self) -> PointBoundary {
        todo!()
    }
}
impl Boundary for MultiPointBoundary {
    fn mean_point(&self) -> PointBoundary {
        let sum_pt = self.0.iter().fold([0.0, 0.0, 0.0], |acc, pt| {
            [acc[0] + pt.x, acc[1] + pt.y, acc[2] + pt.z]
        });
        let len_f64 = self.0.len() as f64;
        PointBoundary::new(
            sum_pt[0] / len_f64,
            sum_pt[1] / len_f64,
            sum_pt[2] / len_f64,
        )
    }
}

fn main() {
    let multipointboundary = MultiPointBoundary(vec![
        PointBoundary::new(1.0, 2.0, 3.0),
        PointBoundary::new(4.0, 5.0, 6.0),
        PointBoundary::new(7.0, 8.0, 9.0),
    ]);
    let geom = Geometry::MultiPoint {
        boundaries: multipointboundary,
    };

    // Compute the mean coordinate of a Geometry
    let mean_point = match geom {
        Geometry::MultiPoint { boundaries, .. } => boundaries.mean_point(),
        Geometry::MultiLineString { boundaries, .. } => boundaries.mean_point(),
    };
    dbg!(mean_point);
}
