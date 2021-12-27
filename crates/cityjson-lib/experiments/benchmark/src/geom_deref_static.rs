pub mod geom_static {
    use std::collections::HashMap;

    #[derive(Debug)]
    pub enum SemanticSurface {
        RoofSurface,
        GroundSurface,
        WallSurface,
    }

    pub struct Material {
        name: String,
        ambient_intensity: Option<f32>,
        diffuse_color: Option<[f32; 3]>,
        emissive_color: Option<[f32; 3]>,
        specular_color: Option<[f32; 3]>,
        shininess: Option<f32>,
        transparency: Option<f32>,
        is_smooth: Option<bool>,
    }

    pub struct Texture {
        image: String,
    }

    struct Semantics {
        surfaces: Vec<SemanticSurface>,
        values: Vec<Vec<usize>>,
    }

    pub type Vertices = Vec<[f64; 3]>;

    type Point = [f64; 3];
    type LineString = Vec<Point>;

    pub struct Surface {
        pub boundaries: Vec<LineString>,
        pub semantics: Option<SemanticSurface>,
        pub material: Option<Material>,
        pub texture: Option<Texture>,
    }

    type Shell = Vec<Surface>;

    enum GeomStructSeparate {
        Surface(Surface),
    }

    // Named fields in the variant so that we attach the data directly to the variant
    enum GeomStructEmbed {
        Surface {
            boundaries: Vec<LineString>,
            semantics: Option<SemanticSurface>,
            material: Option<Material>,
            texture: Option<Texture>,
        },
    }

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
            boundaries: Vec<crate::Geometry>, // This is not good here. I want to constrain this to a Solid.
        },
        CompositeSolid {
            lod: String,
            boundaries: Vec<crate::Geometry>,
        },
    }

    pub struct CityObject {
        pub cotype: String,
        pub geometry: Vec<Geometry>,
    }

    pub struct CityModel {
        pub cmtype: String,
        pub version: String,
        pub cityobjects: HashMap<String, CityObject>,
    }

    fn main() {
        let mut new_cos: HashMap<String, CityObject> = HashMap::new();
        let mut new_geoms: Vec<Geometry> = Vec::new();

        println!(
            "type size of a single Point is {}",
            std::mem::size_of_val(&[1.0, 2.0, 3.0])
        );
        println!(
            "value size of a single Point [1.0, 2.0, 3.0] is {}",
            std::mem::size_of_val(&[1.0, 2.0, 3.0])
        );
        let mp = Geometry::MultiPoint {
            lod: "1.2".to_string(),
            boundaries: vec![[1.0, 2.0, 3.0], [1.0, 2.0, 3.0]],
        };
        println!(
            "type size of Geometry::MultiPoint is {}",
            std::mem::size_of_val(&mp)
        );
        match mp {
            Geometry::MultiPoint { lod, boundaries } => {
                println!("MultiPoint lod is {}", lod)
            }
            Geometry::MultiLineString { .. } => {}
            Geometry::MultiSurface { .. } => {}
            Geometry::CompositeSurface { .. } => {}
            Geometry::Solid { .. } => {}
            Geometry::MultiSolid { .. } => {}
            Geometry::CompositeSolid { .. } => {}
        }

        let g = GeomStructSeparate::Surface(Surface {
            boundaries: vec![vec![[1.0, 2.0, 3.0], [1.0, 2.0, 3.0]]],
            semantics: None,
            material: None,
            texture: None,
        });

        // This seems to me a bit more ergonomic compared to GeomStructEmbed, because I don't
        // need to list all the data members
        if let GeomStructSeparate::Surface(gsimp) = g {
            println!(
                "is Surface struct and has boundaries {:?}",
                gsimp.boundaries
            )
        }

        let ge = GeomStructEmbed::Surface {
            boundaries: vec![vec![[1.0, 2.0, 3.0], [1.0, 2.0, 3.0]]],
            semantics: None,
            material: Some(Material {
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
        if let GeomStructEmbed::Surface {
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
        if let GeomStructEmbed::Surface { material, .. } = &ge {
            {
                println!(
                    "is Surface struct and has material {:?}",
                    material.as_ref().expect("there is no material").name
                )
            }
        }

        // Let's see how does it look for a complete CityModel
        new_geoms.push(Geometry::Solid {
            lod: "2.2".to_string(),
            boundaries: vec![vec![
                Surface {
                    boundaries: vec![vec![
                        [1.0, 2.0, 3.0],
                        [1.0, 2.0, 3.0],
                        [1.0, 2.0, 3.0],
                        [1.0, 2.0, 3.0],
                    ]],
                    semantics: Some(SemanticSurface::GroundSurface),
                    material: None,
                    texture: None,
                },
                Surface {
                    boundaries: vec![vec![
                        [1.0, 2.0, 3.0],
                        [1.0, 2.0, 3.0],
                        [1.0, 2.0, 3.0],
                        [1.0, 2.0, 3.0],
                    ]],
                    semantics: Some(SemanticSurface::WallSurface),
                    material: None,
                    texture: None,
                },
            ]],
        });
        new_geoms.push(Geometry::MultiSurface {
            lod: "1".to_string(),
            boundaries: vec![Surface {
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
                match geom {
                    Geometry::Solid { boundaries, .. } => {
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
                    Geometry::MultiSurface { boundaries, .. } => {
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
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_main() {
            main()
        }
    }
}
