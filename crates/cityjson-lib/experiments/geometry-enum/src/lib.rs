#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;

use serde::de::{
    DeserializeSeed, Deserializer, IgnoredAny, IntoDeserializer, MapAccess, SeqAccess, Visitor,
};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

#[derive(Debug, Default, Deserialize)]
pub struct Model {
    cityobjects: HashMap<String, CityObject>,
    vertices: Vec<[i64; 3]>,
}

#[derive(Debug, Default, Deserialize)]
pub struct CityObject {
    geometry: Vec<Geometry>,
}

#[derive(Debug)]
struct Geometry {
    type_geom: GeometryType,
    lod: String,
    boundaries: Boundary,
}

#[derive(Debug, Deserialize)]
enum GeometryType {
    MultiPoint,
    MultiLineString,
    MultiSurface,
    CompositeSurface,
    Solid,
    MultiSolid,
    CompositeSolid,
    GeometryInstance,
}

struct JsonRawValue<'a>(&'a RawValue);

struct GeometryVisitor;

impl<'de> Visitor<'de> for GeometryVisitor {
    type Value = Geometry;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a valid Geometry object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut geomtype_op: Option<GeometryType> = None;
        let mut boundaries_rw = serde_json::value::to_raw_value(&Boundary::default()).unwrap();
        let mut lod_rw = serde_json::value::to_raw_value(&String::new()).unwrap();

        while let Some(key) = map.next_key::<String>()? {
            if key == "type" {
                geomtype_op = Some(map.next_value::<GeometryType>()?);
            } else if key == "boundaries" {
                boundaries_rw = map.next_value::<Box<RawValue>>()?;
            } else if key == "lod" {
                lod_rw = map.next_value::<Box<RawValue>>()?;
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }

        if geomtype_op.is_none() {
            return Err(serde::de::Error::custom(
                "did not find the key 'type' in the Geometry",
            ));
        }
        let geomtype = geomtype_op.unwrap();

        let mut boundaries = Boundary::default();
        match geomtype {
            GeometryType::MultiPoint => {
                todo!()
            }
            GeometryType::MultiLineString => {
                todo!()
            }
            GeometryType::MultiSurface => {
                boundaries_rw
                    .deserialize_seq(ExtendSurfacesVisitor(&mut boundaries))
                    .map_err(serde::de::Error::custom)?;
            }
            GeometryType::CompositeSurface => {
                boundaries_rw
                    .deserialize_seq(ExtendSurfacesVisitor(&mut boundaries))
                    .map_err(serde::de::Error::custom)?;
            }
            GeometryType::Solid => {
                boundaries_rw
                    .deserialize_seq(ExtendShellsVisitor(&mut boundaries))
                    .map_err(serde::de::Error::custom)?;
            }
            GeometryType::MultiSolid => {
                todo!()
            }
            GeometryType::CompositeSolid => {
                todo!()
            }
            GeometryType::GeometryInstance => {
                todo!()
            }
        }
        let lod =
            String::deserialize(lod_rw.into_deserializer()).map_err(serde::de::Error::custom)?;

        Ok(Geometry {
            type_geom: geomtype,
            lod,
            boundaries,
        })
    }
}

struct GeometryTypeContainer(GeometryType);

impl<'de> Deserialize<'de> for Geometry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(GeometryVisitor)
    }
}

#[derive(Debug, Default, Serialize)]
struct Boundary {
    vertices: Vec<usize>,
    rings: Vec<usize>,
    surfaces: Vec<usize>,
    shells: Vec<usize>,
    solids: Vec<usize>,
}

struct ExtendVertices<'a>(&'a mut Boundary);

struct ExtendVerticesVisitor<'a>(&'a mut Boundary);

impl<'de, 'a> Visitor<'de> for ExtendVerticesVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an array of vertex indices")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if let Some(size_hint) = seq.size_hint() {
            self.0.vertices.reserve(size_hint);
        }

        while let Some(elem) = seq.next_element()? {
            self.0.vertices.push(elem);
        }

        Ok(())
    }
}

impl<'de, 'a> DeserializeSeed<'de> for ExtendVertices<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendVerticesVisitor(self.0))
    }
}

struct ExtendRings<'a>(&'a mut Boundary);

struct ExtendRingsVisitor<'a>(&'a mut Boundary);

impl<'de, 'a> Visitor<'de> for ExtendRingsVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a surface boundary array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.0.rings.push(self.0.vertices.len());
        while let Some(()) = seq.next_element_seed(ExtendVertices(self.0))? {
            self.0.rings.push(self.0.vertices.len());
        }
        if !self.0.rings.is_empty() {
            let last_idx = self.0.rings.len() - 1;
            self.0.rings.remove(last_idx);
        }
        Ok(())
    }
}

impl<'de, 'a> DeserializeSeed<'de> for ExtendRings<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendRingsVisitor(self.0))
    }
}

struct ExtendSurfaces<'a>(&'a mut Boundary);

struct ExtendSurfacesVisitor<'a>(&'a mut Boundary);

impl<'de, 'a> Visitor<'de> for ExtendSurfacesVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a multi-/compositesurface or shell boundary array"
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Add the start index of the first surface of the aggregate
        self.0.surfaces.push(self.0.rings.len());
        // Each iteration through this loop is one inner array.
        while let Some(()) = seq.next_element_seed(ExtendRings(self.0))? {
            self.0.surfaces.push(self.0.rings.len());
        }
        if !self.0.surfaces.is_empty() {
            let last_idx = self.0.surfaces.len() - 1;
            self.0.surfaces.remove(last_idx);
        }
        Ok(())
    }
}

impl<'de, 'a> DeserializeSeed<'de> for ExtendSurfaces<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendSurfacesVisitor(self.0))
    }
}

struct ExtendShells<'a>(&'a mut Boundary);

struct ExtendShellsVisitor<'a>(&'a mut Boundary);

impl<'de, 'a> Visitor<'de> for ExtendShellsVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a solid boundary array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Add the start index of the first shell of the aggregate
        self.0.shells.push(self.0.surfaces.len());
        // Each iteration through this loop is one inner array.
        while let Some(()) = seq.next_element_seed(ExtendSurfaces(self.0))? {
            self.0.shells.push(self.0.surfaces.len());
        }
        if !self.0.shells.is_empty() {
            let last_idx = self.0.shells.len() - 1;
            self.0.shells.remove(last_idx);
        }
        Ok(())
    }
}

impl<'de, 'a> DeserializeSeed<'de> for ExtendShells<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendShellsVisitor(self.0))
    }
}

impl<'de> Deserialize<'de> for Boundary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut boundary = Boundary::default();
        deserializer
            .deserialize_seq(ExtendShellsVisitor(&mut boundary))
            .map(|_| boundary)
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Read;

    use super::*;

    #[test]
    fn deser_all() {
        let mut file = File::open("benches/data/all.json").unwrap();
        let mut json_str = String::new();
        file.read_to_string(&mut json_str).unwrap();
        let ms: Model = serde_json::from_str(&json_str).unwrap();
        println!("done")
    }

    #[test]
    fn deserialize_solidboundary() {
        let solidboundary_json = r#"[]"#;
        let solidboundary: Boundary = serde_json::from_str(solidboundary_json).unwrap();
        assert!(
            solidboundary.shells.is_empty()
                && solidboundary.surfaces.is_empty()
                && solidboundary.rings.is_empty()
                && solidboundary.vertices.is_empty()
        );

        let solidboundary_json =
            r#"[[ [[0, 3, 2, 1]], [[4, 5, 6, 7]], [[0, 1, 5, 4]], [[1, 2, 6, 5]] ]]"#;
        let solidboundary: Boundary = serde_json::from_str(solidboundary_json).unwrap();
        assert_eq!(
            solidboundary.vertices,
            vec![0_usize, 3, 2, 1, 4, 5, 6, 7, 0, 1, 5, 4, 1, 2, 6, 5]
        );
        assert_eq!(solidboundary.rings, vec![0_usize, 4, 8, 12]);
        assert_eq!(solidboundary.surfaces, vec![0_usize, 1, 2, 3]);
        assert_eq!(solidboundary.shells, vec![0_usize]);

        let solidboundary_json =
            r#"[[ [[0, 3, 2, 1]], [[4, 5, 6, 7]], [[0, 1, 5, 4]], [[1, 2, 6, 5]] ], []]"#;
        let solidboundary: Boundary = serde_json::from_str(solidboundary_json).unwrap();
        assert_eq!(
            solidboundary.vertices,
            vec![0_usize, 3, 2, 1, 4, 5, 6, 7, 0, 1, 5, 4, 1, 2, 6, 5]
        );
        assert_eq!(solidboundary.rings, vec![0_usize, 4, 8, 12]);
        assert_eq!(solidboundary.surfaces, vec![0_usize, 1, 2, 3]);
        // Surface index 4 is out of bounds, which indicates and empty shell.
        assert_eq!(solidboundary.shells, vec![0_usize, 4]);

        let solidboundary_json = r#"[ [ [[0, 3, 2, 1], [4, 5, 6, 7]] ], [ [[0, 1, 5, 4]] ] ]"#;
        let solidboundary: Boundary = serde_json::from_str(solidboundary_json).unwrap();
        assert_eq!(
            solidboundary.vertices,
            vec![0_usize, 3, 2, 1, 4, 5, 6, 7, 0, 1, 5, 4]
        );
        assert_eq!(solidboundary.rings, vec![0_usize, 4, 8]);
        assert_eq!(solidboundary.surfaces, vec![0_usize, 2]);
        assert_eq!(solidboundary.shells, vec![0_usize, 1]);
    }

    #[test]
    fn deserialize_aggregatesurfaceboundary() {
        let multisurfaceboundary_json = r#"[]"#;
        let msrfbdry: Boundary = serde_json::from_str(multisurfaceboundary_json).unwrap();
        assert!(
            msrfbdry.surfaces.is_empty()
                && msrfbdry.rings.is_empty()
                && msrfbdry.vertices.is_empty()
        );

        let multisurfaceboundary_json = r#"[[[0, 3, 2, 1]]]"#;
        let msrfbdry: Boundary = serde_json::from_str(multisurfaceboundary_json).unwrap();
        assert_eq!(msrfbdry.vertices, vec![0_usize, 3, 2, 1]);
        assert_eq!(msrfbdry.rings, vec![0_usize]);

        let multisurfaceboundary_json = r#"[ [[0, 3, 2, 1], [4, 5, 6, 7]], [[0, 3, 2, 1]] ]"#;
        let msrfbdry: Boundary = serde_json::from_str(multisurfaceboundary_json).unwrap();
        assert_eq!(
            msrfbdry.vertices,
            vec![0_usize, 3, 2, 1, 4, 5, 6, 7, 0, 3, 2, 1]
        );
        assert_eq!(msrfbdry.rings, vec![0_usize, 4, 8]);
        assert_eq!(msrfbdry.surfaces, vec![0_usize, 2]);
    }

    #[test]
    fn deserialize_surfaceboundary() {
        let surfaceboundary_json = r#"[]"#;
        let surfaceboundary: Boundary = serde_json::from_str(surfaceboundary_json).unwrap();
        assert!(surfaceboundary.rings.is_empty() && surfaceboundary.vertices.is_empty());

        let surfaceboundary_json = r#"[[0, 3, 2, 1]]"#;
        let surfaceboundary: Boundary = serde_json::from_str(surfaceboundary_json).unwrap();
        assert_eq!(surfaceboundary.vertices, vec![0_usize, 3, 2, 1]);
        assert_eq!(surfaceboundary.rings, vec![0_usize]);

        let surfaceboundary_json = r#"[[0, 3, 2, 1], [4, 5, 6, 7]]"#;
        let surfaceboundary: Boundary = serde_json::from_str(surfaceboundary_json).unwrap();
        assert_eq!(surfaceboundary.vertices, vec![0_usize, 3, 2, 1, 4, 5, 6, 7]);
        assert_eq!(surfaceboundary.rings, vec![0_usize, 4]);

        let surfaceboundary_json = r#"[[0, 3, 2, 1], [4, 5, 6, 7], [4, 5, 6, 7]]"#;
        let surfaceboundary: Boundary = serde_json::from_str(surfaceboundary_json).unwrap();
        assert_eq!(
            surfaceboundary.vertices,
            vec![0_usize, 3, 2, 1, 4, 5, 6, 7, 4, 5, 6, 7]
        );
        assert_eq!(surfaceboundary.rings, vec![0_usize, 4, 8]);
    }
}
