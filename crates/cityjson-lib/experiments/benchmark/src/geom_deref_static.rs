#![feature(type_name_of_val)]
use std::collections::HashMap;
use std::ops::Mul;

struct SemanticSurface {
    semtype: String,
}

struct Semantics {
    surfaces: Vec<SemanticSurface>,
    values: Vec<Vec<usize>>,
}

type Vertices = Vec<[f64; 3]>;

type Point = [f64; 3];
type MultiPoint = Vec<Point>;
type Ring = Vec<Point>;
type Surface = Vec<Ring>;
type Shell = Vec<Surface>;
type MultiSurface = Vec<Surface>;
type Solid = Vec<Shell>;

enum Geometry {
    Point(Point),
    MultiPoint(MultiPoint),
    Solid(Solid),
    MultiSurface(MultiSurface),
}

#[repr(u8)]
enum Geometry_u8 {
    Point(Point),
    MultiPoint(MultiPoint),
    Solid(Solid),
    MultiSurface(MultiSurface),
}

enum Geometry_p {
    Point(Point),
}

struct CityObject {
    cotype: String,
    geometry: Vec<Geometry>,
}

struct CityModel {
    cmtype: String,
    version: String,
    cityobjects: HashMap<String, CityObject>,
}

fn main() {
    let mut new_cos: HashMap<String, CityObject> = HashMap::new();
    let mut new_geoms: Vec<Geometry> = Vec::new();

    println!(
        "size of Point is {}",
        std::mem::size_of_val(&[1.0, 2.0, 3.0])
    );
    println!(
        "size of Geometry::Point is {}",
        std::mem::size_of_val(&Geometry::Point([1.0, 2.0, 3.0]))
    );
    println!(
        "size of Geometry_u8::Point is {}",
        std::mem::size_of_val(&Geometry_u8::Point([1.0, 2.0, 3.0]))
    );
    println!(
        "size of Geometry_p::Point is {}",
        std::mem::size_of_val(&Geometry_p::Point([1.0, 2.0, 3.0]))
    );
    let mut s: Solid = vec![vec![vec![vec![
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
    ]]]];
    s.shrink_to_fit();
    println!("size of Solid is {}", std::mem::size_of_val(&s));
    println!(
        "size of Geometry::Solid is {}",
        std::mem::size_of_val(&Geometry::Solid(s))
    );

    let solid: Solid = vec![vec![vec![vec![
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
    ]]]];
    let msrf: MultiSurface = vec![vec![vec![
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
        [1.0, 2.0, 3.0],
    ]]];
    new_geoms.push(Geometry::Solid(solid));
    new_geoms.push(Geometry::MultiSurface(msrf));

    let new_co = CityObject {
        cotype: "Building".to_string(),
        geometry: new_geoms,
    };
    new_cos.insert("id-1".to_string(), new_co);

    let cm = CityModel {
        cmtype: "CityJSON".to_string(),
        version: "1.1".to_string(),
        cityobjects: new_cos,
    };

    for (coid, co) in cm.cityobjects {
        for geom in co.geometry {
            println!("type of geom {}", std::any::type_name_of_val(&geom));
            match geom {
                Geometry::Solid(solidgeom) => {
                    println!("This is a Solid");
                    for shell in solidgeom {
                        for surface in shell {
                            for ring in surface {
                                for point in ring {
                                    println!("{:?}", point)
                                }
                            }
                        }
                    }
                }
                Geometry::MultiSurface(msrfgeom) => {
                    println!("This is a MultiSurface");
                    for surface in msrfgeom {
                        for ring in surface {
                            for point in ring {
                                println!("{:?}", point)
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        main()
    }
}
