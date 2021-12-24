# Heterogenous collections

The primitive collections, such as `Array` and `Vec` only store items of the same type.
However, in order to store the Geometries (Solid, MultiSurface etc.) of a CityObject in a single collection, we need to be able to store heterogenous collections.
There are two solutions that come to mind.
Both of them require some level of dynamic dispatch.

## Vector of pointers

The heterogenous collection is made up of pointers to trait objects.
Rust ensures at compile time that these trait objects implement the trait object's trait.
Consequently, we don't need to know all the possible types at compile time.

```rust
type MultiSurface = Vec<Surface>;
type Solid = Vec<Shell>;
trait Geometry {
    fn get_type(&self) -> &str;
}
impl Geometry for Solid {
    fn get_type(&self) -> &str {
        "Solid"
    }
}
impl Geometry for MultiSurface {
    fn get_type(&self) -> &str {
        "MultiSurface"
    }
}
struct CityObject {
    cotype: String,
    geometry: Vec<Box<dyn Geometry>>,
}
```

A pattern to access the geometry then is the following.

```rust
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
    }
}
```

However, traits only store associated functions and don't store data.
Therefore, in order to access the data members of *Solid*, we would need to implement getter and setter functions for each member.
In case of Geometries this is not the end of the world, since there are only a couple of members that might need to be set, *lod*, *semantics*, *appearances*.

Additionally, we actually know all the possible types that can go into the geometry collection of a CityObject.
So maybe we don't need this completely dynamic approach and so we could improve perfomrance a bit.

Run the executable `geom_deref_dyn` in the file `geom_deref_dyn.rs` for an example.

## Enums

Since I know all the possible Geometry types that can be stored in the CityObject's geometry collection, I can make use of Enums.

A potential disadvantage of the Enum approach is the memory footprint of an Enum.
The size of an Enum is determined by its largest variant.
However, this is not a big issue, since we have only 8 bytes of difference between a `Point` and a `Solid`, given the following definitions.
Where the size of a `Point` is 24 bytes, since it's an array with 3 members of `f64`.
The size of a `Solid` is 32 bytes.
Therefore the size of the `enum Geometry` is also 32 bytes.
Also, it is important to consider that the Enum's memory layout it optimal, because of the padding that can occur.

```rust
type Point = [f64; 3];
type Ring = Vec<Point>;
type Surface = Vec<Ring>;
type Shell = Vec<Surface>;
type Solid = Vec<Shell>;

enum Geometry {
    Point(Point),
    Solid(Solid),
}
```

Then the geometries can be accessed by matching on the enum.

```rust
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
```

Run the executable `geom_deref_static` in the file `geom_deref_static.rs` for an example.

## References
- https://stackoverflow.com/a/27957213
- https://stackoverflow.com/a/51936494 and then I don't need Boxed Traits, which is great.
- https://stackoverflow.com/questions/27324821/why-does-an-enum-require-extra-memory-size 
- https://stackoverflow.com/questions/65903095/static-vs-dynamic-heterogeneous-collection-in-rust
- https://stackoverflow.com/questions/48327964/store-a-collection-of-heterogeneous-types-with-generic-type-parameters-in-rust
- https://stackoverflow.com/questions/40411045/is-it-possible-to-have-a-heterogeneous-vector-of-types-that-implement-eq



