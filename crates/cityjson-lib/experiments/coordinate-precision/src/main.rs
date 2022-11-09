#[cfg(all(feature = "single", feature = "double"))]
compile_error!("feature \"single\" and feature \"double\" cannot be enabled at the same time");

#[cfg(feature = "single")]
type Point = [f32; 3];
#[cfg(feature = "double")]
type Point = [f64; 3];

type Boundary = Vec<Point>;

fn main() {
    let mut b = Boundary::new();
    b.push([1.0, 1.0, 1.0]);
    for i in b {
        println!("{:?}", i);
    }
}
