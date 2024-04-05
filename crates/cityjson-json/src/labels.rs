use std::fmt;

use serde::de::{DeserializeSeed, Deserializer, SeqAccess, Visitor};
use serde::ser::{Error, SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};

use crate::boundary::{BoundaryCounter, BoundaryType};

/// Stores the Semantic and Material indices of a Boundary.
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq, Deserialize)]
pub struct LabelIndex {
    pub(crate) points: Vec<OptionalIndex>,
    pub(crate) linestrings: Vec<OptionalIndex>,
    /// Points to a Surface in the "surfaces" array of the Geometry Semantics
    pub(crate) surfaces: Vec<OptionalIndex>,
    pub(crate) shells: Vec<usize>,
    pub(crate) solids: Vec<usize>,
}

impl Serialize for LabelIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.check_type() {
            BoundaryType::MultiOrCompositeSolid => {
                let mut nested_json = serializer.serialize_seq(Some(self.solids.len()))?;
                let mut counter = BoundaryCounter::default();
                for shells_start_i in &self.solids {
                    let shells_len = self.shells.len();
                    let shells_end_i = self
                        .solids
                        .get(counter.next_solid_i())
                        .unwrap_or(&shells_len);
                    if let Some(shells) = self.shells.get(*shells_start_i..*shells_end_i) {
                        let mut solid = NestedSolidSemanticsValues::with_capacity(shells.len());
                        for surfaces_start_i in shells {
                            let surfaces_len = self.surfaces.len();
                            let surfaces_end_i = self
                                .shells
                                .get(counter.next_shell_i())
                                .unwrap_or(&surfaces_len);
                            if let Some(surfaces) =
                                self.surfaces.get(*surfaces_start_i..*surfaces_end_i)
                            {
                                let mut shell =
                                    NestedShellSemanticsValues::with_capacity(surfaces.len());
                                for op_idx in surfaces {
                                    shell.push(*op_idx);
                                }
                                solid.push(shell);
                            }
                        }
                        nested_json.serialize_element(&solid)?;
                    }
                }
                nested_json.end()
            }
            BoundaryType::Solid => {
                let mut nested_json = serializer.serialize_seq(Some(self.shells.len()))?;
                let mut counter = BoundaryCounter::default();
                // For the semantics.values of a Solid, we need a two-level deep array
                for surfaces_start_i in &self.shells {
                    let surfaces_len = self.surfaces.len();
                    let surfaces_end_i = self
                        .shells
                        .get(counter.next_shell_i())
                        .unwrap_or(&surfaces_len);
                    if let Some(surfaces) = self.surfaces.get(*surfaces_start_i..*surfaces_end_i) {
                        let mut shell = NestedShellSemanticsValues::with_capacity(surfaces.len());
                        for op_idx in surfaces {
                            shell.push(*op_idx);
                        }
                        nested_json.serialize_element(&shell)?;
                    }
                }
                nested_json.end()
            }
            BoundaryType::MultiOrCompositeSurface => {
                let mut nested_json = serializer.serialize_seq(Some(self.surfaces.len()))?;
                for member in &self.surfaces {
                    nested_json.serialize_element(member)?;
                }
                nested_json.end()
            }
            BoundaryType::MultiLineString => {
                let mut nested_json = serializer.serialize_seq(Some(self.linestrings.len()))?;
                for member in &self.linestrings {
                    nested_json.serialize_element(member)?;
                }
                nested_json.end()
            }
            BoundaryType::MultiPoint => {
                let mut nested_json = serializer.serialize_seq(Some(self.points.len()))?;
                for member in &self.points {
                    nested_json.serialize_element(member)?;
                }
                nested_json.end()
            }
            BoundaryType::None => Err(Error::custom("cannot serialize an empty LabelIndex")),
        }
    }
}

impl LabelIndex {
    /// Hint what [crate::boundary::BoundaryType] does the LabelIndex belong to.
    pub fn check_type(&self) -> BoundaryType {
        if !self.solids.is_empty() {
            BoundaryType::MultiOrCompositeSolid
        } else if !self.shells.is_empty() {
            BoundaryType::Solid
        } else if !self.surfaces.is_empty() {
            BoundaryType::MultiOrCompositeSurface
        } else if !self.linestrings.is_empty() {
            BoundaryType::MultiLineString
        } else if !self.points.is_empty() {
            BoundaryType::MultiPoint
        } else {
            BoundaryType::None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Visitor implementations
struct ExtendSurfaces<'a>(&'a mut LabelIndex);
pub(crate) struct ExtendSurfacesVisitor<'a>(pub(crate) &'a mut LabelIndex);

impl<'de, 'a> Visitor<'de> for ExtendSurfacesVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "an array of Surface indices")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if let Some(size_hint) = seq.size_hint() {
            self.0.surfaces.reserve(size_hint);
        }

        while let Some(elem) = seq.next_element()? {
            self.0.surfaces.push(elem);
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

struct ExtendShells<'a>(&'a mut LabelIndex);
pub(crate) struct ExtendShellsVisitor<'a>(pub(crate) &'a mut LabelIndex);
impl<'de, 'a> Visitor<'de> for ExtendShellsVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a two level deep semantics.values array of a Solid"
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Add the start index of the first ring of the surface.
        self.0.shells.push(self.0.surfaces.len());
        // Each iteration through this loop is one ring.
        while let Some(()) = seq.next_element_seed(ExtendSurfaces(self.0))? {
            self.0.shells.push(self.0.surfaces.len());
        }
        // The last shell index needs to be removed, because that is surfaces.len()
        // after the last iteration.
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

struct ExtendSolids<'a>(&'a mut LabelIndex);
pub(crate) struct ExtendSolidsVisitor<'a>(pub(crate) &'a mut LabelIndex);

impl<'de, 'a> Visitor<'de> for ExtendSolidsVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a three level deep semantics.values array of a Multi-/CompositeSolid"
        )
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Add the start index of the first surface of the aggregate
        self.0.solids.push(self.0.shells.len());
        // Each iteration through this loop is one inner array.
        while let Some(()) = seq.next_element_seed(ExtendShells(self.0))? {
            self.0.solids.push(self.0.shells.len());
        }
        if !self.0.solids.is_empty() {
            let last_idx = self.0.solids.len() - 1;
            self.0.solids.remove(last_idx);
        }
        Ok(())
    }
}

impl<'de, 'a> DeserializeSeed<'de> for ExtendSolids<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendSolidsVisitor(self.0))
    }
}
////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NestedMultiSolidSemanticsValues = Vec<Vec<Vec<OptionalIndex>>>;
pub type NestedSolidSemanticsValues = Vec<Vec<OptionalIndex>>;
pub type NestedShellSemanticsValues = Vec<OptionalIndex>;

// TODO: this can easily be u8, couz I don't expect to have more than 255 different Semantic object
//  on a single geometry...but if the shitty code does not dedup the Semantic objects then I could
//  have a problem, because there will be as many Semantic objects as geometry primitives.
pub type OptionalIndex = Option<usize>;
