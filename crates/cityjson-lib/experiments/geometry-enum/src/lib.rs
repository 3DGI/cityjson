#![allow(dead_code)]
use std::collections::HashMap;
use std::fmt;

use serde::de::{DeserializeSeed, Deserializer, SeqAccess, Visitor};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct ModelGeomEnum {
    cityobjects: HashMap<String, CityObjectGeomEnum>,
    vertices: Vec<[i64; 3]>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ModelGeomStruct {
    cityobjects: HashMap<String, CityObjectGeomStruct>,
    vertices: Vec<[i64; 3]>,
}

#[derive(Debug, Default, Deserialize)]
pub struct CityObjectGeomEnum {
    geometry: Vec<GeometryEnum>
}

#[derive(Debug, Default, Deserialize)]
pub struct CityObjectGeomStruct {
    geometry: Vec<GeometryStruct>
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum GeometryEnum {
    MultiSurface {
        boundaries: MultiSurfaceBoundary
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub struct GeometryStruct {
    #[serde(rename = "type")]
    type_geom: String,
    boundaries: MultiSurfaceBoundary
}

#[derive(Debug, Default)]
struct BoundaryContainer {
    vertices: Vec<usize>,
    rings: Vec<usize>,
    surfaces: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct MultiSurfaceBoundary {
    vertices: Vec<usize>,
    rings: Vec<usize>,
    surfaces: Vec<usize>,
}

struct ExtendVertices<'a>(&'a mut BoundaryContainer);

struct ExtendVerticesVisitor<'a>(&'a mut BoundaryContainer);

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

struct ExtendRings<'a>(&'a mut BoundaryContainer);

struct ExtendRingsVisitor<'a>(&'a mut BoundaryContainer);

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

struct ExtendSurfaces<'a>(&'a mut BoundaryContainer);

struct ExtendSurfacesVisitor<'a>(&'a mut BoundaryContainer);

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

impl<'de> Deserialize<'de> for MultiSurfaceBoundary {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut boundarycontainer = BoundaryContainer::default();
        deserializer
            .deserialize_seq(ExtendSurfacesVisitor(&mut boundarycontainer))
            .map(|_| Self {
                vertices: boundarycontainer.vertices,
                rings: boundarycontainer.rings,
                surfaces: boundarycontainer.surfaces,
            })
    }
}
