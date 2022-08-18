use datasize::{data_size, DataSize};
use std::rc::Rc;

#[derive(DataSize, PartialEq, Eq)]
enum Semantic {
    RoofSurface,
    WallSurface,
}

impl Semantic {
    fn print_id(&self) {
        match self {
            RoofSurface => {
                println!("RoofSurface")
            }
            WallSurface => {
                println!("WallSurface")
            }
        }
    }
}

#[derive(DataSize)]
struct Material {
    name: String,
}

#[derive(DataSize)]
struct Texture {
    name: String,
}

#[derive(DataSize)]
struct Surface {
    id: i32,
    semantic: Rc<Semantic>,
    material: Rc<Material>,
    texture: Rc<Texture>,
}

#[derive(DataSize)]
struct SurfaceId {
    id: i32,
}

struct CityJSON {
    boundaries: Vec<Surface>,
    semantics: Vec<Rc<Semantic>>,
}

fn main() {
    let mut cm = CityJSON {
        boundaries: Vec::new(),
        semantics: Vec::new(),
    };
    // here sem1 owns the Rc<Semantic> value
    let sem1 = Rc::new(Semantic::WallSurface);
    if !cm.semantics.contains(&sem1) {
        // sem1 gives up ownership and the value is moved into the vector
        cm.semantics.push(sem1);
    }
    // Therefore, here i cannot access the value in sem1 anymore.
    // sem1.print_id();
    // Practically, sem1 is in an initialized variable without a value at this stage. Equal to 'let sem1';

    // An Rc<Semantic> automatically dereferences to a Semantic with the Deref trait, so one can
    // call Semantic's methods on a Rc<Semantic> value.
    cm.semantics[0].print_id();
    // In fact, i could directly move the value into the container, instead of creating an
    // intermediary variable. However, I need to check if a semantic is already in the container
    // before adding it to the container, and the only way I know how to do this is to initialize
    // the semantic into an intermediary variable first.

    let semantic1: Rc<Semantic> = Rc::new(Semantic::WallSurface);

    let texture1: Rc<Texture> = Rc::new(Texture {
        name: "texture 1".to_string(),
    });
    let material1: Rc<Material> = Rc::new(Material {
        name: "material 1".to_string(),
    });

    let srf1 = Surface {
        id: 1,
        semantic: Rc::clone(&semantic1),
        material: Rc::clone(&material1),
        texture: Rc::clone(&texture1),
    };

    let srf2 = Surface {
        id: 2,
        semantic: Rc::clone(&semantic1),
        material: Rc::clone(&material1),
        texture: Rc::clone(&texture1),
    };

    let srf_id = SurfaceId { id: 1 };

    println!(
        "estimated heap allocation of Surface.semantic: {}",
        data_size(&srf1.semantic) as f32
    );

    println!(
        "estimated heap allocation of Surface 2: {}",
        data_size(&srf2) as f32
    );

    println!(
        "estimated heap allocation of SurfaceId: {}",
        data_size(&srf_id) as f32
    );
}
