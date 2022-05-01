//! Dereference architecture with dynamic dispatch.
#![feature(type_name_of_val)]
use std::collections::HashMap;

struct SemanticSurface {
    semtype: String,
}

struct Semantics {
    surfaces: Vec<SemanticSurface>,
    values: Vec<Vec<usize>>,
}

type Vertices = Vec<[f64; 3]>;

type Point = [f64; 3];
type Ring = Vec<Point>;
type Surface = Vec<Ring>;
type Shell = Vec<Surface>;
type MultiSurface = Vec<Surface>;
type Solid = Vec<Shell>;

trait Geometry {
    fn get_type(&self) -> &str;
    fn get_type_std(&self) -> &str;
}
impl Geometry for Solid {
    fn get_type(&self) -> &str {
        "Solid"
    }
    fn get_type_std(&self) -> &str {
        std::any::type_name_of_val(self)
    }
}
impl Geometry for MultiSurface {
    fn get_type(&self) -> &str {
        "MultiSurface"
    }
    fn get_type_std(&self) -> &str {
        std::any::type_name_of_val(self)
    }
}

struct CityObject {
    cotype: String,
    geometry: Vec<Box<dyn Geometry>>,
}

struct CityModel {
    cmtype: String,
    version: String,
    cityobjects: HashMap<String, CityObject>,
}

fn main() {
    let mut new_cos: HashMap<String, CityObject> = HashMap::new();
    let mut new_geoms: Vec<Box<dyn Geometry>> = Vec::new();

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
    new_geoms.push(Box::new(solid));
    new_geoms.push(Box::new(msrf));

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
            println!("type of geom {}", std::any::type_name_of_val(&*geom));
            println!("hardcoded/manually set type: {}", &*geom.get_type());
            println!(
                "using std::any::type_name_of_val : {}",
                &*geom.get_type_std()
            );
        }
    }
}
