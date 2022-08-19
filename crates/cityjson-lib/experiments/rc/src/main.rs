use datasize::{data_size, DataSize};
use std::fmt::format;
use std::rc::Rc;

#[derive(DataSize, PartialEq, Eq, Debug)]
enum Semantic {
    RoofSurface,
    WallSurface,
}

impl Semantic {
    fn print_id(&self) {
        match self {
            Self::RoofSurface => {
                println!("RoofSurface");
            }
            Self::WallSurface => {
                println!("WallSurface");
            }
        }
    }
}

#[derive(DataSize, PartialEq, Eq, Debug)]
struct Material {
    name: String,
}

#[derive(DataSize, PartialEq, Eq, Debug)]
struct Texture {
    name: String,
}

#[derive(DataSize, Debug)]
struct Surface {
    id: i32,
}

#[derive(DataSize)]
enum Geometry {
    MultiSurface {
        boundaries: Vec<Surface>,
        semantics_values: Option<Vec<Option<Rc<Semantic>>>>,
        textures_values: Option<Vec<Option<Rc<Texture>>>>,
        materials_values: Option<Vec<Option<Rc<Material>>>>,
    },
}

#[derive(DataSize)]
struct CityJSON {
    geometries: Vec<Geometry>,
    semantics: Vec<Rc<Semantic>>,
    textures: Vec<Rc<Texture>>,
    materials: Vec<Rc<Material>>,
}

fn main() {
    let mut cm = CityJSON {
        geometries: Vec::new(),
        semantics: Vec::new(),
        textures: Vec::new(),
        materials: Vec::new(),
    };

    // Init an empty geometry. It must have 'boundaries', other members are optional.
    let mut geom = Geometry::MultiSurface {
        boundaries: vec![],
        semantics_values: None,
        textures_values: None,
        materials_values: None,
    };

    // ----- First Surface -----
    // Here sem1 owns the Rc<Semantic> value
    let sem1 = Rc::new(Semantic::WallSurface);
    if !cm.semantics.contains(&sem1) {
        // sem1 gives up ownership and the value is moved into the vector
        cm.semantics.push(sem1);
    }
    // Therefore, here i cannot access the value in sem1 anymore.
    // sem1.print_id();
    // Practically, sem1 is in an initialized variable without a value at this stage.
    // Equal to 'let sem1';

    // An Rc<Semantic> automatically dereferences to a Semantic with the Deref trait, so one can
    // call Semantic's methods on a Rc<Semantic> value.
    cm.semantics[0].print_id();
    // In fact, i could directly move the value into the container, instead of creating an
    // intermediary variable. However, I need to check if a semantic is already in the container
    // before adding it to the container, and the only way I know how to do this is to initialize
    // the semantic into an intermediary variable first.
    match &mut geom {
        Geometry::MultiSurface {
            semantics_values, ..
        } => {
            if let Some(ref mut sv) = semantics_values {
                sv.push(Some(cm.semantics[0].clone()));
            } else {
                *semantics_values = Some(vec![Some(cm.semantics[0].clone())]);
            }
        }
    }

    let _tex: Rc<Texture> = Rc::new(Texture {
        name: "texture 1".to_string(),
    });
    // When a new semantic/texture/material is created, I need to check if it already exists in the
    // container. Thus we don't allow duplicates, which helps to keep memory use in check on the
    // cost of slower Geometry creation.
    // In fact, this is O(n) to the number of textures in the container, since for each addition we
    // have to loop through the container and test for equality. However, I expect a low number of
    // different semantics/textures/materials in the whole citymodel, so I think this won't become
    // an issue.
    let mut _ti: usize;
    if let Some(tidx) = &cm.textures.iter().position(|r| r == &_tex) {
        _ti = tidx.clone();
    } else {
        cm.textures.push(_tex);
        _ti = cm.textures.len() - 1;
    }
    // Get a mutable reference to the value in the variable, because otherwise the 'match'
    // consumes the value.
    match &mut geom {
        Geometry::MultiSurface {
            textures_values, ..
        } => {
            if let Some(ref mut tv) = textures_values {
                tv.push(Some(cm.textures[_ti].clone()));
            } else {
                *textures_values = Some(vec![Some(cm.textures[_ti].clone())]);
            }
        }
    }

    // Add the boundary of one Surface
    match &mut geom {
        Geometry::MultiSurface { boundaries, .. } => {
            boundaries.push(Surface { id: 1 });
        }
    }

    // ----- Second Surface -----
    // semantics
    let _sem: Rc<Semantic> = Rc::new(Semantic::RoofSurface);
    let mut _si: usize;
    if let Some(sidx) = &cm.semantics.iter().position(|r| r == &_sem) {
        _si = sidx.clone();
    } else {
        cm.semantics.push(_sem);
        _si = cm.semantics.len() - 1;
    }
    match &mut geom {
        Geometry::MultiSurface {
            semantics_values, ..
        } => {
            if let Some(ref mut sv) = semantics_values {
                sv.push(Some(cm.semantics[_si].clone()));
            } else {
                *semantics_values = Some(vec![Some(cm.semantics[_si].clone())]);
            }
        }
    }
    // textures
    let _tex: Rc<Texture> = Rc::new(Texture {
        name: "texture 1".to_string(),
    });
    let mut _ti: usize;
    if let Some(tidx) = &cm.textures.iter().position(|r| r == &_tex) {
        _ti = tidx.clone();
    } else {
        cm.textures.push(_tex);
        _ti = cm.textures.len() - 1;
    }
    match &mut geom {
        Geometry::MultiSurface {
            textures_values, ..
        } => {
            if let Some(ref mut tv) = textures_values {
                tv.push(Some(cm.textures[_ti].clone()));
            } else {
                *textures_values = Some(vec![Some(cm.textures[_ti].clone())]);
            }
        }
    }
    // boundary
    match &mut geom {
        Geometry::MultiSurface { boundaries, .. } => {
            boundaries.push(Surface { id: 2 });
        }
    }

    // Testing operation on the Geometry
    match &geom {
        Geometry::MultiSurface {
            boundaries,
            semantics_values,
            textures_values,
            materials_values,
        } => {
            for (i, srf) in boundaries.iter().enumerate() {
                // semantic
                let sem: String;
                if let Some(_s) = semantics_values {
                    if let Some(_ss) = &_s[i] {
                        sem = format!("{:?}", _ss);
                    } else {
                        sem = "None".to_string();
                    }
                } else {
                    sem = "None".to_string();
                }
                // texture
                let tex: String;
                if let Some(_s) = textures_values {
                    if let Some(_ss) = &_s[i] {
                        tex = format!("{:?}", _ss);
                    } else {
                        tex = "None".to_string();
                    }
                } else {
                    tex = "None".to_string();
                }
                // material
                let mat: String;
                if let Some(_s) = materials_values {
                    if let Some(_ss) = &_s[i] {
                        mat = format!("{:?}", _ss);
                    } else {
                        mat = "None".to_string();
                    }
                } else {
                    mat = "None".to_string();
                }
                println!(
                    "Surface {}={:?}, semantic={}, texture={}, material={}",
                    i, srf, sem, tex, mat
                )
            }
        }
    }

    for t in &cm.textures {
        println!("texture in cm.textures: {:?}", t);
    }

    match &geom {
        Geometry::MultiSurface {
            boundaries,
            semantics_values,
            textures_values,
            materials_values,
        } => {
            println!(
                "estimated heap allocation of geom.boundaries: {}",
                data_size(boundaries) as f32
            );
            println!(
                "estimated heap allocation of geom.semantics: {}",
                data_size(semantics_values) as f32
            );
            println!(
                "estimated heap allocation of geom.textures: {}",
                data_size(textures_values) as f32
            );
            println!(
                "estimated heap allocation of geom.materials: {}",
                data_size(materials_values) as f32
            );
        }
    }

    println!(
        "estimated heap allocation of Geometry: {}",
        data_size(&geom) as f32
    );

    println!(
        "estimated heap allocation of CityModel: {}",
        data_size(&cm) as f32
    );
}
