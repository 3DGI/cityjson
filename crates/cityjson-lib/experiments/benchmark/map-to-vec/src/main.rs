//! Test how to deserialize a JSON Object into a Vec of tuples (key, value), and then use the Vec
//! as a stack.
#![allow(dead_code)]
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::json;
use std::fmt;
use std::marker::PhantomData;

#[derive(Deserialize, Debug)]
struct Part {
    value: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum Geometry {
    TypeA {
        attribute: String,
        boundary: Vec<usize>,
        other: Option<Part>,
    },
}

#[derive(Deserialize, Debug)]
struct ComplexObject {
    otype: String,
    value: Vec<Geometry>,
}

#[derive(Deserialize, Debug)]
struct MainData {
    #[serde(deserialize_with = "deserialize_complexobjects")]
    complexobjects: Vec<(String, ComplexObject)>,
}

fn deserialize_complexobjects<'de, D>(
    deserializer: D,
) -> Result<Vec<(String, ComplexObject)>, D::Error>
where
    D: Deserializer<'de>,
{
    struct MapVisitor(PhantomData<fn() -> Vec<(String, ComplexObject)>>);

    impl<'de> Visitor<'de> for MapVisitor {
        type Value = Vec<(String, ComplexObject)>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("the 'CityObjects' mapping of a CityJSON file")
        }

        fn visit_map<M>(self, mut data: M) -> Result<Vec<(String, ComplexObject)>, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut co_vec: Vec<(String, ComplexObject)> =
                Vec::with_capacity(data.size_hint().unwrap_or(0));
            while let Some((coid, co)) = data.next_entry()? {
                co_vec.push((coid, co));
            }
            co_vec.shrink_to_fit();
            Ok(co_vec)
        }
    }
    let visitor = MapVisitor(PhantomData);
    deserializer.deserialize_map(visitor)
}

fn main() {
    let j = json!({
        "complexobjects": {
            "id1": {
                "otype": "object type 1",
                "value": [
                    {
                        "type": "TypeA",
                        "attribute": "bla 1",
                        "boundary": [1,2,3],
                        "other": {
                            "value": "value 1"
                        }
                    }
                ]
            },
            "id2": {
                "otype": "object type 2",
                "value": [
                    {
                        "type": "TypeA",
                        "attribute": "bla 2",
                        "boundary": [4,5,6],
                        "other": {
                            "value": "value 2"
                        }
                    }
                ]
            }
        }
    });

    let mut maindata: MainData = serde_json::from_value(j).unwrap();
    println!("{:#?}", maindata);

    while let Some((coid, co)) = maindata.complexobjects.pop() {
        println!("ComplexObject key: {}", coid);
        println!("ComplexObject value: {:#?}", co);
    }

    println!(
        "remaining ComplexObjects: {}",
        maindata.complexobjects.len()
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
