#![allow(dead_code, unused_variables, unused_must_use)]
use std::collections::HashMap;
use std::fmt;

use serde::de::{Error, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::json;

#[derive(Deserialize)]
struct Geometry {
    type_geom: String,
    boundary: Vec<i32>,
}

struct TargetStruct {
    version: String,
    geometries: HashMap<String, Geometry>,
}

struct TargetStructVisitor;

impl<'de> Visitor<'de> for TargetStructVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a valid file")
    }

    fn visit_map<A>(self, mut map: A) -> Result<(), A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Ok(Some(key)) = map.next_key::<String>() {
            println!("{:#?}", key);
            if key == "geometries" {
                println!("--> yaaay!");
                // something here?
            } else {
                println!("--> do nothing")
            }
            let a: IgnoredAny = map.next_value::<IgnoredAny>()?;
        }
        Ok(())
    }

    fn visit_str<E>(self, v: &str) -> Result<(), E>
    where
        E: Error,
    {
        println!("str: {:#?}", v);
        Ok(())
    }
}

impl<'de> Deserialize<'de> for TargetStruct {
    fn deserialize<D>(deserializer: D) -> Result<TargetStruct, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(TargetStructVisitor);
        Ok(TargetStruct {
            version: "".to_string(),
            geometries: Default::default(),
        })
    }
}

fn main() {
    static DATA: &str = r##"
        {
            "version": "1",
            "geometries": {
                "id1": {"type_geom": "Polygon", "boundary": [1,2,3,4]},
                "id2": {"type_geom": "Polygon", "boundary": [1,2,4]}
            }
        }
        "##;

    // This fails with a "trailing characters" error, at the colon after "geometries"
    let cm: TargetStruct = serde_json::from_str(DATA).unwrap();

    let j = json!({
        "version": "1",
        "geometries": {
            "id1": {"type_geom": "Polygon", "boundary": [1,2,3,4]},
            "id2": {"type_geom": "Polygon", "boundary": [1,2,4]}
        }
    });
    // This succeeds
    let cm: TargetStruct = serde_json::from_value(j).unwrap();
    // This also succeeds
    let cm: serde_json::Value = serde_json::from_str(DATA).unwrap();
}
