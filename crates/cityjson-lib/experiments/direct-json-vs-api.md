# Why use an API instead of traversing the JSON directly?

One of the advantages of CityJSON that its simplified data model and json encoding makes it straightforward to work with the files in common progrmming languages.
One of the most common geo-programming languages, Python, even includes a json parsing library as part of its standard libraries. 
JavaScript naturally works with json.
Systems programming languages, such as C++ and Rust, have tried-and-tested json libraries available.

In fact, in our CityJSON presentations we always include a slide that shows how easy is to get started with CityJSON in Python.

```python
import json

with ("cluster.city.json").resolve().open("r") as fo:
    cm = json.load(fo)

for co_id, co in cm["CityObjects"].items():
    print(f"Found CityObject {co_id} of type {co['type']}")
```

Its equivalent in Rust is similarly simple. 
It requires the [serde_json](https://github.com/serde-rs/json) crate which is not part of the standard libraries.

```rust
use serde_json::Value;

fn main() -> Result<(), serde_json::Error> {
    let path = "cluster.city.json";
    let str_dataset = std::fs::read_to_string(&path).expect("Couldn't read CityJSON file");
    let j: Value = serde_json::from_str(&str_dataset)?;
    let cos = j.get("CityObjects").unwrap().as_object().unwrap();
    for coid in cos.keys() {
        println!("CityObject {} is of type {}", coid, j["CityObjects"][coid]["type"])
    }
    Ok(())
}
```

There are three approaches in consideration.
1. The **direct-json**. This approach simply deserializes the json string with a json library (eg. `serde_json` or Python's standard `json`) and operates on the deserialized json values as-is.
2. The **vertex-index**. This approach adds a layer of abstraction over the deserialized json values by providing an API that helps with common operations. It retains the global/local vertex list and the  in-memory geometry structures index into this vertex list. This approach is commonly applied in applications that need to store and access meshes efficiently, such as rendering and game engines.
3. The **dereference**. This approach adds a layer of abstraction over the deserialized json values by providing an API that help with common operations. It replaces the vertex indices in the geometry boundaries by their coordinates. This is approach is widely used in GIS applications and is similar to the Simple Features specification.

## Evaluating the direct approach

The evaluation below is based on with writing CityJSON-based libraries.
For standalone applications different aspects need to be considered.
Is that true though?
Because standalone applications are either built from a library or the direct approach...

### Advantages

**Computationally efficient** \
There is no abstraction over the deserialized json, apart from what is minimally required by the json library.

**Follows the CityJSON file one-to-one** \
Is this an advantage though?

### Disadvantages

**Duplication of effort** \
need to write and rewrite the same accessors and error handling

**Error prone** \
Easy to make mistakes in the CityJSON schema.
The [documentation of `serde_json`](https://docs.serde.rs/serde_json/#operating-on-untyped-json-values) says: 

> *"The `Value` representation is sufficient for very basic tasks but can be tedious to work with for anything more significant. Error handling is verbose to implement correctly, for example imagine trying to detect the presence of unrecognized fields in the input data. The compiler is powerless to help you when you make a mistake, for example imagine typoing `v["name"]` as `v["nmae"]` in one of the dozens of places it is used in your code."*

**Only suitable for reading files, not so much for interacting with models** \
the global vertex list gets in the way

**Problematic to reuse**  \
an example for this is cjio, where the functions require that everything is stored as a single json object, so I have an application that generates a citymodel, regardless of cityjson, then in order to reuse the functions from cjio it first need to write the whole citymodel into a json string (or python dictionary of the same schema), so that a cjio function can work with it

## Dereferencing the geometry boundaries

### Heterogenous collections

The primitive collections, such as `Array` and `Vec` only store items of the same type.
However, in order to store the Geometries (Solid, MultiSurface etc.) of a CityObject in a single collection, we need to be able to store heterogenous collections.
There are two solutions that come to mind.
Both of them require some level of dynamic dispatch.

#### Vector of pointers

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

#### Enums

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

### References
- https://stackoverflow.com/a/27957213
- https://stackoverflow.com/a/51936494 and then I don't need Boxed Traits, which is great.
- https://stackoverflow.com/questions/27324821/why-does-an-enum-require-extra-memory-size
- https://stackoverflow.com/questions/65903095/static-vs-dynamic-heterogeneous-collection-in-rust
- https://stackoverflow.com/questions/48327964/store-a-collection-of-heterogeneous-types-with-generic-type-parameters-in-rust
- https://stackoverflow.com/questions/40411045/is-it-possible-to-have-a-heterogeneous-vector-of-types-that-implement-eq

## A middle-ground – An architecture based on a vertex buffer

Maybe a global vertex list is not such a bad idea within the application either.
What advantages would it bring?

1) reduced space
2) topology? but that doesn't come by default either

Although, it would complicate things.
Possibly, overcomplicate things.
For instance, what if I erase a vertex?
If I cannot erase a vertex, then when can I remove the vertices from the memory?
How do CAD software do this? Or 3D graphics software like Blender? Or games?

A [SlotMap](https://docs.rs/slotmap/1.0.6/slotmap/) could be very useful for building a vertex buffer based library.

Okay, game engines use an *indexed triangle list*, also called a *vertex buffer* (DirectX)
or *vertex array* (OpenGL), just as OBJ and CityJSON[1].
Games often store quite a lot of metadata with each vertex, so repeating this data in a triangle list wastes memory.
It also wastes GPU bandwidth, because a duplicated vertex will be transformed and lit multiple times.

*"So in a 3D rendered world, everything seen will start as a collection of vertices and texture maps. They are collated into memory buffers that link together -- a __vertex buffer__ contains the information about the vertices; an __index buffer__ tells us how the vertices connect to form shapes; a __resource buffer__ contains the textures and portions of memory set aside to be used later in the rendering process; a __command buffer__ the list of instructions of what to do with it all."*[3]

Blender uses a non-manifold boundary representation called *BMesh*[2].
This it uses a global vertex list too.

## [Benchmarking different architectures](https://github.com/balazsdukai/cjlib/blob/master/experiments/benchmarking.md)

# References

+ [1]: Gregory, J. (2018). Game engine architecture. Taylor and Francis, CRC Press.
+ [2]: Blender. (2020). The BMesh Structure. https://wiki.blender.org/wiki/Source/Modeling/BMesh/Design. Accessed on 2021-11-15.
+ [3]: Evanson, N. (2019). 3D Game Rendering. https://www.techspot.com/article/1851-3d-game-rendering-explained/. Accessed on 2021-11-15.

