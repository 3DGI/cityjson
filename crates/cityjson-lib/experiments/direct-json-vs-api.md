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
easy to forget parts

**Only suitable for reading files, not so much for interacting with models** \
the global vertex list gets in the way

**Problematic to reuse**  \
an example for this is cjio, where the functions require that everything is stored as a single json object, so I have an application that generates a citymodel, regardless of cityjson, then in order to reuse the functions from cjio it first need to write the whole citymodel into a json string (or python dictionary of the same schema), so that a cjio function can work with it


