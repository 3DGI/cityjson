use std::fmt;

#[derive(Clone, Copy)]
pub struct Point {
    x: f32,
    y: f32,
    z: f32
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z )
    }
}

impl Point {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {x, y, z}
    }

    pub fn get_x(&self) -> f32 {
        self.x
    }

    pub fn z_greater(&self, m: f32) -> bool {
        self.z > m
    }
}

pub struct LineString {
    pub boundaries: Vec<Point>
}

impl LineString {
    pub fn new(boundaries: Vec<Point>) -> Self {
        Self { boundaries }
    }

    pub fn get_boundaries(&self) -> &Vec<Point> {
        &self.boundaries
    }

    pub fn len(&self) -> usize {
        self.boundaries.len()
    }
}

impl fmt::Display for LineString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();
        for p in &self.boundaries {
            s.push_str(&*p.to_string())
        }
        write!(f, "[{}]", s)
    }
}

#[cfg(test)]
mod tests {
    use crate::geometry::*;

    #[test]
    fn new_point(){
        let p = Point{ x: 1.0, y: 2.0, z: 3.0 };
        println!("point: {}", p);
    }

    #[test]
    fn new_linestring() {
        let l = LineString::new(
            vec![
                Point::new(0.0, 0.0, 1.0),
                Point::new(1.0, 0.0, 1.0),
                Point::new(1.0, 1.0, 1.0),
                Point::new(0.0, 1.0, 1.0),
            ]
        );
        // for p in &l.boundaries {
        //     println!("{}", p);
        // }
        println!("{}", l);
        println!("nr points: {}", l.len());
    }
}
