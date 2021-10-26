pub mod geometry;

#[cfg(test)]
mod tests {
    use crate::geometry::Point;

    #[test]
    fn it_works() {
        let p = Point::new(1.0, 2.0, 3.0);
        println!("x: {}", p.get_x());
    }
}
