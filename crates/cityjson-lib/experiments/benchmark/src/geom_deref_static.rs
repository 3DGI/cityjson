//! Dereference architecture with static dispatch.
use crate::geom_static::{Geometry, Surface};
use std::collections::HashMap;

pub mod geom_static {
    use serde::Serialize;
    use std::collections::HashMap;

    #[derive(Debug, Serialize)]
    pub enum SemanticSurface {
        RoofSurface,
        GroundSurface,
        WallSurface,
    }

    #[derive(Serialize)]
    pub struct Material {
        pub name: String,
        pub ambient_intensity: Option<f32>,
        pub diffuse_color: Option<[f32; 3]>,
        pub emissive_color: Option<[f32; 3]>,
        pub specular_color: Option<[f32; 3]>,
        pub shininess: Option<f32>,
        pub transparency: Option<f32>,
        pub is_smooth: Option<bool>,
    }

    #[derive(Serialize)]
    pub struct Texture {
        image: String,
    }

    pub type Vertices = Vec<[f64; 3]>;

    type Point = [f64; 3];
    type LineString = Vec<Point>;

    #[derive(Serialize)]
    pub struct Surface {
        pub boundaries: Vec<LineString>,
        pub semantics: Option<SemanticSurface>,
        pub material: Option<Material>,
        pub texture: Option<Texture>,
    }

    type Shell = Vec<Surface>;

    pub(crate) enum GeomStructSeparate {
        Surface(Surface),
    }

    // Named fields in the variant so that we attach the data directly to the variant
    pub(crate) enum GeomStructEmbed {
        Surface {
            boundaries: Vec<LineString>,
            semantics: Option<SemanticSurface>,
            material: Option<Material>,
            texture: Option<Texture>,
        },
    }

    #[derive(Serialize)]
    pub enum Geometry {
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

    pub struct CityObject {
        pub cotype: String,
        pub geometry: Vec<Geometry>,
    }

    #[derive(Serialize)]
    pub struct CityObjectSer {
        pub cotype: String,
        pub geometry: Vec<String>,
    }

    pub struct CityModel {
        pub cmtype: String,
        pub version: String,
        pub cityobjects: HashMap<String, CityObject>,
    }

    #[derive(Serialize)]
    pub struct CityModelSer {
        pub cmtype: String,
        pub version: String,
        pub cityobjects: HashMap<String, CityObjectSer>,
    }
}

fn main() {
    let mut new_cos: HashMap<String, geom_static::CityObject> = HashMap::new();
    let mut new_geoms: Vec<geom_static::Geometry> = Vec::new();

    println!(
        "type size of a single Point is {}",
        std::mem::size_of_val(&[1.0, 2.0, 3.0])
    );
    println!(
        "value size of a single Point [1.0, 2.0, 3.0] is {}",
        std::mem::size_of_val(&[1.0, 2.0, 3.0])
    );
    let mp = geom_static::Geometry::MultiPoint {
        lod: "1.2".to_string(),
        boundaries: vec![[1.0, 2.0, 3.0], [1.0, 2.0, 3.0]],
    };
    println!(
        "type size of Geometry::MultiPoint is {}",
        std::mem::size_of_val(&mp)
    );
    match mp {
        geom_static::Geometry::MultiPoint { lod, boundaries } => {
            println!("MultiPoint lod is {}", lod)
        }
        geom_static::Geometry::MultiLineString { .. } => {}
        geom_static::Geometry::MultiSurface { .. } => {}
        geom_static::Geometry::CompositeSurface { .. } => {}
        geom_static::Geometry::Solid { .. } => {}
        geom_static::Geometry::MultiSolid { .. } => {}
        geom_static::Geometry::CompositeSolid { .. } => {}
    }

    let g = geom_static::GeomStructSeparate::Surface(geom_static::Surface {
        boundaries: vec![vec![[1.0, 2.0, 3.0], [1.0, 2.0, 3.0]]],
        semantics: None,
        material: None,
        texture: None,
    });

    // This seems to me a bit more ergonomic compared to GeomStructEmbed, because I don't
    // need to list all the data members
    if let geom_static::GeomStructSeparate::Surface(gsimp) = g {
        println!(
            "is Surface struct and has boundaries {:?}",
            gsimp.boundaries
        )
    }

    let ge = geom_static::GeomStructEmbed::Surface {
        boundaries: vec![vec![[1.0, 2.0, 3.0], [1.0, 2.0, 3.0]]],
        semantics: None,
        material: Some(geom_static::Material {
            name: "someMaterial".to_string(),
            ambient_intensity: None,
            diffuse_color: None,
            emissive_color: None,
            specular_color: None,
            shininess: None,
            transparency: None,
            is_smooth: None,
        }),
        texture: None,
    };
    // Compared to GeomStrucSeparate, here I have to know all the data members and list them
    if let geom_static::GeomStructEmbed::Surface {
        boundaries,
        semantics,
        material,
        texture,
    } = &ge
    {
        println!("is Surface struct and has boundaries {:?}", boundaries)
    }
    // See https://doc.rust-lang.org/book/ch18-03-pattern-syntax.html#ignoring-remaining-parts-of-a-value-with-
    // But I think this is going to be fine with this partial matching
    if let geom_static::GeomStructEmbed::Surface { material, .. } = &ge {
        {
            println!(
                "is Surface struct and has material {:?}",
                material.as_ref().expect("there is no material").name
            )
        }
    }

    // Let's see how does it look for a complete CityModel
    new_geoms.push(geom_static::Geometry::Solid {
        lod: "2.2".to_string(),
        boundaries: vec![vec![
            geom_static::Surface {
                #[rustfmt::skip]
                boundaries: vec![vec![[1.0, 2.0, 3.0],[1.0, 2.0, 3.0],[1.0, 2.0, 3.0],[1.0, 2.0, 3.0],]],
                semantics: Some(geom_static::SemanticSurface::GroundSurface),
                material: None,
                texture: None,
            },
            geom_static::Surface {
                #[rustfmt::skip]
                boundaries: vec![vec![[1.0, 2.0, 3.0],[1.0, 2.0, 3.0],[1.0, 2.0, 3.0],[1.0, 2.0, 3.0],]],
                semantics: Some(geom_static::SemanticSurface::WallSurface),
                material: None,
                texture: None,
            },
        ]],
    });
    new_geoms.push(geom_static::Geometry::MultiSurface {
        lod: "1".to_string(),
        boundaries: vec![geom_static::Surface {
            boundaries: vec![vec![
                [1.0, 2.0, 3.0],
                [1.0, 2.0, 3.0],
                [1.0, 2.0, 3.0],
                [1.0, 2.0, 3.0],
            ]],
            semantics: None,
            material: None,
            texture: None,
        }],
    });

    let new_co = geom_static::CityObject {
        cotype: "Building".to_string(),
        geometry: new_geoms,
    };
    new_cos.insert("id-1".to_string(), new_co);

    let cm = geom_static::CityModel {
        cmtype: "CityJSON".to_string(),
        version: "1.1".to_string(),
        cityobjects: new_cos,
    };

    for (coid, co) in cm.cityobjects {
        for geom in co.geometry {
            match geom {
                geom_static::Geometry::Solid { boundaries, .. } => {
                    println!("This is a Solid");
                    for shell in boundaries {
                        for surface in shell {
                            println!("This surface is a {:?}", surface.semantics.unwrap());
                            for ring in surface.boundaries {
                                for point in ring {
                                    println!("{:?}", point)
                                }
                            }
                        }
                    }
                }
                geom_static::Geometry::MultiSurface { boundaries, .. } => {
                    println!("This is a MultiSurface");
                    for surface in boundaries {
                        for ring in surface.boundaries {
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

    // Serializing the CityJSON structures
    println!(
        "SemanticSurface::RoofSurface in JSON: {}",
        serde_json::to_string(&geom_static::SemanticSurface::RoofSurface)
            .unwrap()
            .to_string()
    );

    println!(
        "Material {{
            name: SomeMaterial.to_string(),
            ambient_intensity: Some(3.0),
            diffuse_color: None,
            emissive_color: None,
            specular_color: None,
            shininess: None,
            transparency: None,
            is_smooth: Some(true)
        }} in JSON: {}",
        serde_json::to_string(&geom_static::Material {
            name: "SomeMaterial".to_string(),
            ambient_intensity: Some(3.0),
            diffuse_color: None,
            emissive_color: None,
            specular_color: None,
            shininess: None,
            transparency: None,
            is_smooth: Some(true)
        })
        .unwrap()
        .to_string()
    );

    let g = geom_static::Geometry::Solid {
        lod: "1.2".to_string(),
        boundaries: vec![vec![geom_static::Surface {
            boundaries: vec![vec![
                [1.0, 2.0, 3.0],
                [1.0, 2.0, 3.0],
                [1.0, 2.0, 3.0],
                [1.0, 2.0, 3.0],
            ]],
            semantics: Some(geom_static::SemanticSurface::RoofSurface),
            material: Some(geom_static::Material {
                name: "SomeMaterial".to_string(),
                ambient_intensity: Some(3.0),
                diffuse_color: None,
                emissive_color: None,
                specular_color: None,
                shininess: None,
                transparency: None,
                is_smooth: Some(true),
            }),
            texture: None,
        }]],
    };
    println!(
        "Solid Geometry in JSON: {}",
        serde_json::to_string(&g).unwrap().to_string()
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
