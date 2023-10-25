#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use std::mem;

use datasize::{data_size, DataSize};

#[derive(DataSize)]
enum SemanticSurface {
    RoofSurface,
    GroundSurface,
    WallSurface,
}

#[derive(DataSize)]
struct Material {
    name: String,
    ambient_intensity: Option<f32>,
    diffuse_color: Option<[f32; 3]>,
    emissive_color: Option<[f32; 3]>,
    specular_color: Option<[f32; 3]>,
    shininess: Option<f32>,
    transparency: Option<f32>,
    is_smooth: Option<bool>,
}

#[derive(DataSize)]
struct Texture {
    image: String,
}

type Vertices = Vec<[f64; 3]>;

type Point = [f64; 3];
type LineString = Vec<Point>;

#[derive(DataSize)]
struct Surface {
    boundaries: Vec<LineString>,
    semantics: Option<SemanticSurface>,
    material: Option<Material>,
    texture: Option<Texture>,
}

type Shell = Vec<Surface>;

#[derive(DataSize)]
enum Geometry {
    MultiPoint {
        lod: String,
        boundaries: Vec<Point>,
    },
    MultiLineString {
        lod: String,
        boundaries: Vec<LineString>,
    },
    MultiSurface {
        lod: String,
        boundaries: Vec<Surface>,
    },
    CompositeSurface {
        lod: String,
        boundaries: Vec<Surface>,
    },
    Solid {
        lod: String,
        boundaries: Vec<Shell>,
    },
    MultiSolid {
        lod: String,
        boundaries: Vec<Geometry>, // This is not good here. I want to constrain this to a Solid.
    },
    CompositeSolid {
        lod: String,
        boundaries: Vec<Geometry>,
    },
}

#[derive(DataSize)]
struct CityObject {
    cotype: String,
    geometry: Vec<Geometry>,
}

#[derive(DataSize)]
struct CityModel {
    cmtype: String,
    version: String,
    cityobjects: HashMap<String, CityObject>,
}

// Different Point representations
type PointArrayAlias = [f64; 3];
struct PointTupleStruct(f64, f64, f64);
struct PointTupleStructArray([f64; 3]);
struct PointNamedFieldsStruct {
    x: f64,
    y: f64,
    z: f64,
}

fn main() {
    let type_sizes = format!(
        "Type sizes in bytes:\n\
    usize: {usize}\n\
    i32: {i32}\n\
    i64: {i64}\n\
    f32: {f32}\n\
    f64: {f64}\n\
    bool: {bool}\n\
    [f64; 3]: {a64}\n\
    ",
        usize = mem::size_of::<usize>(),
        i32 = mem::size_of::<i32>(),
        i64 = mem::size_of::<i64>(),
        f32 = mem::size_of::<f32>(),
        f64 = mem::size_of::<f64>(),
        bool = mem::size_of::<bool>(),
        a64 = mem::size_of::<[f64; 3]>()
    );

    print!("{}", type_sizes);

    let semsurf: Option<SemanticSurface> = None;

    let srf: Surface = Surface {
        boundaries: Vec::new(),
        semantics: semsurf,
        material: None,
        texture: None,
    };

    let solid: Shell = vec![srf];

    let geom: Geometry = Geometry::Solid {
        lod: "".to_string(),
        boundaries: vec![solid],
    };

    let co: CityObject = CityObject {
        cotype: "".to_string(),
        geometry: vec![geom],
    };

    let cm: CityModel = CityModel {
        cmtype: "".to_string(),
        version: "".to_string(),
        cityobjects: HashMap::from([("id1".to_string(), co)]),
    };

    println!("CityModel empty size: {}", data_size(&cm));

    println!("---Different point representations---");
    let pointarrayalias: PointArrayAlias = [123.0, 456.0, 678.0];
    let pointtuplestruct = PointTupleStruct(123.0, 456.0, 678.0);
    let pointtuplestructarray = PointTupleStructArray([123.0, 456.0, 678.0]);
    let pointnamedfieldsstruct = PointNamedFieldsStruct {
        x: 123.0,
        y: 456.0,
        z: 678.0,
    };

    println!(
        "Size of PointArrayAlias: {}",
        mem::size_of::<PointArrayAlias>()
    );
    println!(
        "Size of PointTupleStruct: {}",
        mem::size_of::<PointTupleStruct>()
    );
    println!(
        "Size of PointTupleStructArray: {}",
        mem::size_of::<PointTupleStructArray>()
    );
    println!(
        "Size of PointNamedFieldsStruct: {}",
        mem::size_of::<PointNamedFieldsStruct>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        main()
    }
}
