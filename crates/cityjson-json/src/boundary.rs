//! CityJSON Geometry objects.
//!
//! # Boundary representations
//! Internally `serde_cityjson` uses a different a different boundary representation than what is
//! defined in the
//! [CityJSON specification](https://www.cityjson.org/specs/1.1.3/#arrays-to-represent-boundaries),
//! and each boundary type is represented by the [Boundary] type.
//!
//! The CityJSON-like nested array representations are defined as the [BoundaryNestedMultiPoint],
//! `BoundaryNested*` type aliases. However, they are only included in `serde_cityjson` for
//! convenient conversion between the nested and the `serde_cityjson` internal boundary
//! representations, and otherwise not used by `serde_cityjson`.
//!
//! Do not rely on the `BoundaryNested*` types when using `serde_cityjson`, use [Boundary] instead.
use std::fmt;

#[cfg(feature = "datasize")]
use datasize::DataSize;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use serde::de::{Deserializer, DeserializeSeed, SeqAccess, Visitor};
use serde::ser::{Error, Serializer, SerializeSeq};

use crate::errors;

/// A generic geometry Boundary that can represent every type of boundary. The Boundary itself
/// does not "know" what type it is. Some boundary types are ambiguous in CityJSON, for example a
/// `MultiSurface`, `CompositeSurface` and `Shell` each have the same representation.
/// The exact boundary type is defined by the [crate::v1_1::GeometryType]
/// of the parent [crate::v1_1::Geometry]. Therefore, in most cases a Boundary should only be used in conjunction
/// with its parent Geometry.
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct Boundary {
    vertices: Vec<VertexIndex>,
    rings: Vec<usize>,
    surfaces: Vec<usize>,
    shells: Vec<usize>,
    solids: Vec<usize>,
}

#[derive(Copy, Clone, Debug, Display, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub enum BoundaryType {
    MultiOrCompositeSolid,
    Solid,
    MultiOrCompositeSurface,
    MultiLineString,
    MultiPoint,
    /// Represents an empty Boundary.
    #[default]
    None,
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

impl Serialize for Boundary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.check_type() {
            BoundaryType::MultiOrCompositeSolid => {
                let mut nested_json = serializer.serialize_seq(Some(self.solids.len()))?;
                let nested = self
                    .to_nested_multi_or_compositesolid()
                    .map_err(|e| Error::custom(e))?;
                for member in &nested {
                    nested_json.serialize_element(member)?;
                }
                nested_json.end()
            }
            BoundaryType::Solid => {
                let mut nested_json = serializer.serialize_seq(Some(self.shells.len()))?;
                let nested = self.to_nested_solid().map_err(|e| Error::custom(e))?;
                for member in &nested {
                    nested_json.serialize_element(member)?;
                }
                nested_json.end()
            }
            BoundaryType::MultiOrCompositeSurface => {
                let mut nested_json = serializer.serialize_seq(Some(self.surfaces.len()))?;
                let nested = self
                    .to_nested_multi_or_compositesurface()
                    .map_err(|e| Error::custom(e))?;
                for member in &nested {
                    nested_json.serialize_element(member)?;
                }
                nested_json.end()
            }
            BoundaryType::MultiLineString => {
                let mut nested_json = serializer.serialize_seq(Some(self.rings.len()))?;
                let nested = self
                    .to_nested_multilinestring()
                    .map_err(|e| Error::custom(e))?;
                for member in &nested {
                    nested_json.serialize_element(member)?;
                }
                nested_json.end()
            }
            BoundaryType::MultiPoint => {
                let mut nested_json = serializer.serialize_seq(Some(self.vertices.len()))?;
                let nested = self.to_nested_multipoint().map_err(|e| Error::custom(e))?;
                for member in &nested {
                    nested_json.serialize_element(member)?;
                }
                nested_json.end()
            }
            BoundaryType::None => Err(Error::custom(
                "cannot serialize an empty Boundary (BoundaryType::None)",
            )),
        }
    }
}

impl From<BoundaryNestedMultiPoint> for Boundary {
    fn from(value: BoundaryNestedMultiPoint) -> Self {
        if value.is_empty() {
            Self::default()
        } else {
            Self {
                vertices: value,
                ..Self::default()
            }
        }
    }
}

impl From<BoundaryNestedMultiLineString> for Boundary {
    fn from(value: BoundaryNestedMultiLineString) -> Self {
        if value.is_empty() {
            Self::default()
        } else {
            let mut vertices: Vec<usize> = Vec::new();
            let mut rings: Vec<usize> = Vec::with_capacity(value.len());
            let mut ring_start: usize = 0;
            for ring in &value {
                rings.push(ring_start);
                for vertex in ring {
                    vertices.push(*vertex);
                    ring_start += 1;
                }
            }
            Self {
                vertices,
                rings,
                ..Self::default()
            }
        }
    }
}

impl From<BoundaryNestedMultiOrCompositeSurface> for Boundary {
    fn from(value: BoundaryNestedMultiOrCompositeSurface) -> Self {
        todo!()
    }
}

impl From<BoundaryNestedSolid> for Boundary {
    fn from(value: BoundaryNestedSolid) -> Self {
        todo!()
    }
}

impl From<BoundaryNestedMultiOrCompositeSolid> for Boundary {
    fn from(value: BoundaryNestedMultiOrCompositeSolid) -> Self {
        todo!()
    }
}

impl Boundary {
    // Prefix conversion to nested types with `to_nested_` because,
    //  - it is an expensive conversion, since we need to iterate the boundaries and check the indices,
    //  - we stay at the same level of abstraction, just convert from one representation to another,
    //  - the conversion is fallible, since the Boundary might not contain the data for the target type,
    //  - we borrow the input and returned owned output.

    // TODO: add to_nested_<geom>_unchecked() methods that skip the boundary type check, because
    //  the boundary type is already checked in the Serialize implementation

    /// Convert to a nested MultiPoint boundary representation, if the Boundary can be interpreted
    /// as a MultiPoint boundary.
    pub fn to_nested_multipoint(&self) -> errors::Result<BoundaryNestedMultiPoint> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiPoint {
            Ok(self.vertices.clone())
        } else {
            Err(errors::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "MultiPoint".to_string(),
            ))
        }
    }

    /// Convert to a nested MultiLineString boundary representation, if the Boundary can be
    /// interpreted as a MultiLineString boundary.
    pub fn to_nested_multilinestring(&self) -> errors::Result<BoundaryNestedMultiLineString> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiLineString {
            let mut counter = BoundaryCounter::default();
            let mut ml = BoundaryNestedMultiLineString::with_capacity(self.rings.len());
            self.push_rings_to_surface(self.rings.as_slice(), &mut ml, &mut counter);
            Ok(ml)
        } else {
            Err(errors::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "MultiLineString".to_string(),
            ))
        }
    }

    /// Convert to a nested Multi- or CompositeSurface boundary representation, if the Boundary can be
    /// interpreted as an Multi- or CompositeSurface boundary.
    pub fn to_nested_multi_or_compositesurface(
        &self,
    ) -> errors::Result<BoundaryNestedMultiOrCompositeSurface> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiOrCompositeSurface {
            let mut counter = BoundaryCounter::default();
            let mut mcsurface =
                BoundaryNestedMultiOrCompositeSurface::with_capacity(self.surfaces.len());
            self.push_surfaces_to_multisurface(self.surfaces.as_slice(), &mut mcsurface, &mut counter);
            Ok(mcsurface)
        } else {
            Err(errors::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "MultiOrCompositeSurface".to_string(),
            ))
        }
    }

    /// Convert to a nested Solid boundary representation, if the Boundary can be
    /// interpreted as a Solid boundary.
    pub fn to_nested_solid(&self) -> errors::Result<BoundaryNestedSolid> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::Solid {
            let mut counter = BoundaryCounter::default();
            let mut solid = BoundaryNestedSolid::with_capacity(self.shells.len());
            self.push_shells_to_solid(self.shells.as_slice(), &mut solid, &mut counter);
            Ok(solid)
        } else {
            Err(errors::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "Solid".to_string(),
            ))
        }
    }

    /// Convert to a nested Multi- or CompositeSolid boundary representation, if the Boundary can be
    /// interpreted as an Multi- or CompositeSolid boundary.
    pub fn to_nested_multi_or_compositesolid(
        &self,
    ) -> errors::Result<BoundaryNestedMultiOrCompositeSolid> {
        let boundary_type = self.check_type();
        if boundary_type == BoundaryType::MultiOrCompositeSolid {
            let mut counter = BoundaryCounter::default();
            let mut mcsolid = BoundaryNestedMultiOrCompositeSolid::with_capacity(self.solids.len());
            for shells_start_i in &self.solids {
                let shells_len = self.shells.len();
                let shells_end_i = self.solids.get(counter.next_solid_i()).unwrap_or(&shells_len);
                if let Some(shells) = self.shells.get(*shells_start_i..*shells_end_i) {
                    let mut solid = BoundaryNestedSolid::with_capacity(shells.len());
                    self.push_shells_to_solid(shells, &mut solid, &mut counter);
                    mcsolid.push(solid);
                }
            }
            Ok(mcsolid)
        } else {
            Err(errors::Error::IncompatibleBoundary(
                boundary_type.to_string(),
                "MultiOrCompositeSolid".to_string(),
            ))
        }
    }

    fn push_shells_to_solid(&self, shells: &[usize], solid: &mut Vec<BoundaryNestedMultiOrCompositeSurface>, mut counter: &mut BoundaryCounter) {
        for surfaces_start_i in shells {
            let surfaces_len = self.surfaces.len();
            let surfaces_end_i = self.shells.get(counter.next_shell_i()).unwrap_or(&surfaces_len);
            if let Some(surfaces) = self.surfaces.get(*surfaces_start_i..*surfaces_end_i) {
                let mut mcsurface =
                    BoundaryNestedMultiOrCompositeSurface::with_capacity(surfaces.len());
                self.push_surfaces_to_multisurface(surfaces, &mut mcsurface, &mut counter);
                solid.push(mcsurface);
            }
        }
    }

    fn push_surfaces_to_multisurface(&self, surfaces: &[usize], mcsurface: &mut BoundaryNestedMultiOrCompositeSurface, mut counter: &mut BoundaryCounter) {
        for ring_start_i in surfaces {
            let rings_len = self.rings.len();
            let ring_end_i = self
                .surfaces
                .get(counter.next_surface_i())
                .unwrap_or(&rings_len);
            if let Some(rings) = self.rings.get(*ring_start_i..*ring_end_i) {
                let mut surface = BoundaryNestedMultiLineString::with_capacity(rings.len());
                self.push_rings_to_surface(rings, &mut surface, &mut counter);
                mcsurface.push(surface);
            }
        }
    }

    fn push_rings_to_surface(
        &self,
        rings: &[usize],
        surface: &mut BoundaryNestedMultiLineString,
        counter: &mut BoundaryCounter,
    ) {
        for vertices_start_i in rings {
            let vertices_len = self.vertices.len();
            let vertices_end_i = self.rings.get(counter.next_ring_i()).unwrap_or(&vertices_len);
            // At the last ring we are out of bounds of the rings vec with v_endi, so
            // we get all the remaining vertices.
            if let Some(vertices) = self.vertices.get(*vertices_start_i..*vertices_end_i) {
                surface.push(BoundaryNestedMultiPoint::from(vertices));
            }
        }
    }

    /// Hint what type of boundary is stored in the Boundary.
    pub fn check_type(&self) -> BoundaryType {
        if !self.solids.is_empty() {
            BoundaryType::MultiOrCompositeSolid
        } else if !self.shells.is_empty() {
            BoundaryType::Solid
        } else if !self.surfaces.is_empty() {
            BoundaryType::MultiOrCompositeSurface
        } else if !self.rings.is_empty() {
            BoundaryType::MultiLineString
        } else if !self.vertices.is_empty() {
            BoundaryType::MultiPoint
        } else {
            BoundaryType::None
        }
    }

    /// Verify that the internal representation of the boundary is consistent, that there are no
    /// dangling indices.
    pub fn is_consistent(&self) -> bool {
        todo!()
    }
}

#[derive(Default)]
pub(crate) struct BoundaryCounter {
    pub(crate) ring_i: usize,
    pub(crate) surface_i: usize,
    pub(crate) shell_i: usize,
    pub(crate) solid_i: usize,
}

impl BoundaryCounter {
    pub(crate) fn next_ring_i(&mut self) -> usize {
        self.ring_i += 1;
        self.ring_i
    }

    pub(crate) fn next_surface_i(&mut self) -> usize {
        self.surface_i += 1;
        self.surface_i
    }

    pub(crate) fn next_shell_i(&mut self) -> usize {
        self.shell_i += 1;
        self.shell_i
    }

    pub(crate) fn next_solid_i(&mut self) -> usize {
        self.solid_i += 1;
        self.solid_i
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Visitor implementations

// The `deserialize` method of `ExtendVertices` is traversing the inner arrays of the
// MultiPoint/LineString/Ring JSON input and appending each vertex index into an existing Vec.
struct ExtendVertices<'a>(&'a mut Boundary);
pub(crate) struct ExtendVerticesVisitor<'a>(pub(crate) &'a mut Boundary);

impl<'de, 'a> Visitor<'de> for ExtendVerticesVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
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
pub(crate) struct ExtendRingsVisitor<'a>(pub(crate) &'a mut Boundary);
impl<'de, 'a> Visitor<'de> for ExtendRingsVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a surface boundary array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Add the start index of the first ring of the surface.
        self.0.rings.push(self.0.vertices.len());
        // Each iteration through this loop is one ring.
        while let Some(()) = seq.next_element_seed(ExtendVertices(self.0))? {
            self.0.rings.push(self.0.vertices.len());
        }
        // The last ring index needs to be removed, because that is vertices.len()
        // after the last iteration.
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
pub(crate) struct ExtendSurfacesVisitor<'a>(pub(crate) &'a mut Boundary);

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
pub(crate) struct ExtendShellsVisitor<'a>(pub(crate) &'a mut Boundary);

impl<'de, 'a> Visitor<'de> for ExtendShellsVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a solid boundary array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Add the start index of the first surface of the aggregate
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

struct ExtendSolids<'a>(&'a mut Boundary);
pub(crate) struct ExtendSolidsVisitor<'a>(pub(crate) &'a mut Boundary);

impl<'de, 'a> Visitor<'de> for ExtendSolidsVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a multi- or compositesolid boundary array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Add the start index of the first shell of the aggregate
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
// Nested boundaries

/// The boundary of a `MultiSolid` or a `CompositeSolid`, represented as nested vectors.
/// Do not rely on this type, see the module documentation for details.
///
/// # Examples
/// ```
/// # use serde_cityjson::boundary::*;
/// # use serde_cityjson::errors;
/// # fn main() -> errors::Result<()> {
/// let aso_nested: BoundaryNestedMultiOrCompositeSolid = vec![vec![vec![vec![vec![0, 1, 2, 3]]]]];
/// let boundary = Boundary::from(aso_nested.clone());
/// let aso_nested_rev: BoundaryNestedMultiOrCompositeSolid = boundary.to_nested_multi_or_compositesolid()?;
/// assert_eq!(aso_nested, aso_nested_rev);
/// # Ok(())
/// # }
pub type BoundaryNestedMultiOrCompositeSolid = Vec<BoundaryNestedSolid>;

/// The boundary of a `Solid`, represented as nested vectors.
/// Do not rely on this type, see the module documentation for details.
///
/// # Examples
/// ```
/// # use serde_cityjson::boundary::*;
/// # use serde_cityjson::errors;
/// # fn main() -> errors::Result<()> {
/// let so_nested: BoundaryNestedSolid = vec![vec![vec![vec![0, 1, 2, 3]]]];
/// let boundary = Boundary::from(so_nested.clone());
/// let so_nested_rev: BoundaryNestedSolid = boundary.to_nested_solid()?;
/// assert_eq!(so_nested, so_nested_rev);
/// # Ok(())
/// # }
pub type BoundaryNestedSolid = Vec<BoundaryNestedMultiOrCompositeSurface>;

/// The boundary of a `MultiSurface`, `CompositeSurface` or `Shell` represented as nested vectors.
/// Do not rely on this type, see the module documentation for details.
///
/// # Examples
/// ```
/// # use serde_cityjson::boundary::*;
/// # use serde_cityjson::errors;
/// # fn main() -> errors::Result<()> {
/// let asrf_nested: BoundaryNestedMultiOrCompositeSurface = vec![vec![vec![0, 1, 2, 3]]];
/// let boundary = Boundary::from(asrf_nested.clone());
/// let asrf_nested_rev: BoundaryNestedMultiOrCompositeSurface = boundary.to_nested_multi_or_compositesurface()?;
/// assert_eq!(asrf_nested, asrf_nested_rev);
/// # Ok(())
/// # }
pub type BoundaryNestedMultiOrCompositeSurface = Vec<BoundaryNestedMultiLineString>;

/// The boundary of a `MultiLineString`, or `Surface` represented as nested vectors.
/// Do not rely on this type, see the module documentation for details.
///
/// # Examples
/// ```
/// # use serde_cityjson::boundary::*;
/// # use serde_cityjson::errors;
/// # fn main() -> errors::Result<()> {
/// let ml_nested: BoundaryNestedMultiLineString = vec![vec![0, 1, 2, 3]];
/// let boundary = Boundary::from(ml_nested.clone());
/// let ml_nested_rev: BoundaryNestedMultiLineString = boundary.to_nested_multilinestring()?;
/// assert_eq!(ml_nested, ml_nested_rev);
/// # Ok(())
/// # }
pub type BoundaryNestedMultiLineString = Vec<BoundaryNestedMultiPoint>;

/// The boundary of a `MultiPoint`, `LineString` or `Ring` represented as nested vectors.
/// Do not rely on this type, see the module documentation for details.
///
/// # Examples
/// ```
/// # use serde_cityjson::boundary::*;
/// # use serde_cityjson::errors;
/// # fn main() -> errors::Result<()> {
/// let mp_nested: BoundaryNestedMultiPoint = vec![0, 1, 2, 3];
/// let boundary = Boundary::from(mp_nested.clone());
/// let mp_nested_rev: BoundaryNestedMultiPoint = boundary.to_nested_multipoint()?;
/// assert_eq!(mp_nested, mp_nested_rev);
/// # Ok(())
/// # }
pub type BoundaryNestedMultiPoint = Vec<VertexIndex>;

/// Represents a vertex index.
pub type VertexIndex = usize; // TODO: u32/usize feature

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn from_multilinestring_empty_last() {
        let ml_nested: BoundaryNestedMultiLineString = vec![vec![0, 1, 2, 3], vec![]];
        let boundary = Boundary::from(ml_nested);
        assert_eq!(boundary.rings, vec![0_usize, 4])
    }

    #[test]
    fn from_multilinestring_empty_inner() {
        let ml_nested: BoundaryNestedMultiLineString =
            vec![vec![0, 1, 2, 3], vec![], vec![0, 1, 2, 3], vec![0, 1, 2, 3]];
        let boundary = Boundary::from(ml_nested);
        assert_eq!(boundary.rings, vec![0_usize, 4, 4, 8])
    }

    #[test]
    fn serialize_none() {
        let boundary = Boundary {
            ..Default::default()
        };
        let boundary_json_res = serde_json::to_string(&boundary);
        assert!(boundary_json_res.is_err());
    }

    #[test]
    fn serialize_multipoint() {
        let boundary = Boundary {
            vertices: vec![0_usize, 3, 2, 1],
            ..Default::default()
        };
        let boundary_json = serde_json::to_string(&boundary)
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(&boundary_json, "[0,3,2,1]");
    }

    #[test]
    fn serialize_multilinestring_basic() {
        let boundary = Boundary {
            vertices: vec![0_usize, 3, 2, 1, 4, 5, 6, 7, 8],
            rings: vec![0, 4, 7],
            ..Default::default()
        };
        let boundary_json = serde_json::to_string(&boundary)
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(&boundary_json, "[[0,3,2,1],[4,5,6],[7,8]]")
    }

    #[test]
    fn serialize_multilinestring_empty() {
        let boundary = Boundary {
            vertices: vec![0_usize, 3, 2, 1, 4, 5, 6, 7],
            rings: vec![0, 4, 4, 8],
            ..Default::default()
        };
        let boundary_json = serde_json::to_string(&boundary)
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(&boundary_json, "[[0,3,2,1],[],[4,5,6,7],[]]")
    }

    #[test]
    fn serialize_multi_or_compositesurface_inner_ring() {
        let boundary = Boundary {
            vertices: vec![
                0_usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                22,
            ],
            rings: vec![0_usize, 4, 8, 12, 16, 19],
            surfaces: vec![0_usize, 3, 4],
            ..Default::default()
        };
        let boundary_json = serde_json::to_string(&boundary)
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(
            &boundary_json,
            "[[[0,1,2,3],[4,5,6,7],[8,9,10,11]],[[12,13,14,15]],[[16,17,18],[19,20,21,22]]]"
        )
    }

    #[test]
    fn serialize_solid() {
        let boundary = Boundary {
            vertices: vec![
                0_usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                22,
            ],
            rings: vec![0_usize, 4, 8, 12, 16, 19],
            surfaces: vec![0_usize, 3, 4],
            shells: vec![0_usize, 2],
            ..Default::default()
        };
        let boundary_json = serde_json::to_string(&boundary)
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(
            &boundary_json,
            "[[[[0,1,2,3],[4,5,6,7],[8,9,10,11]],[[12,13,14,15]]],[[[16,17,18],[19,20,21,22]]]]"
        )
    }

    #[test]
    fn serialize_multi_or_compositesolid() {
        let boundary = Boundary {
            vertices: vec![
                0_usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                22, 23, 24, 25, 26, 27, 28,
            ],
            rings: vec![0_usize, 4, 8, 12, 16, 19, 23, 26],
            surfaces: vec![0_usize, 3, 4, 6, 7],
            shells: vec![0_usize, 2, 3],
            solids: vec![0_usize, 2],
        };
        let boundary_json = serde_json::to_string(&boundary)
            .map_err(|e| e.to_string())
            .unwrap();
        assert_eq!(&boundary_json, "[[[[[0,1,2,3],[4,5,6,7],[8,9,10,11]],[[12,13,14,15]]],[[[16,17,18],[19,20,21,22]]]],[[[[23,24,25]],[[26,27,28]]]]]")
    }

    #[test]
    fn deserialize_multi_or_compositesolid() {
        let mcsolidboundary_value = json!([
            [
                [
                    [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11]],
                    [[12, 13, 14, 15]]
                ],
                [[[16, 17, 18], [19, 20, 21, 22]]]
            ],
            [[[[23, 24, 25]], [[26, 27, 28]]]]
        ]);
        let mut mcsolidboundary = Boundary::default();
        mcsolidboundary_value
            .deserialize_seq(ExtendSolidsVisitor(&mut mcsolidboundary))
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();
        assert_eq!(
            mcsolidboundary.vertices,
            vec![
                0_usize, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
                22, 23, 24, 25, 26, 27, 28,
            ],
        );
        assert_eq!(
            mcsolidboundary.rings,
            vec![0_usize, 4, 8, 12, 16, 19, 23, 26]
        );
        assert_eq!(mcsolidboundary.surfaces, vec![0_usize, 3, 4, 6, 7]);
        assert_eq!(mcsolidboundary.shells, vec![0_usize, 2, 3]);
        assert_eq!(mcsolidboundary.solids, vec![0_usize, 2]);
    }

    #[test]
    fn deserialize_solidboundary_empty() {
        let solidboundary_value = json!([]);
        let mut solidboundary = Boundary::default();
        solidboundary_value
            .deserialize_seq(ExtendShellsVisitor(&mut solidboundary))
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();
        assert!(
            solidboundary.shells.is_empty()
                && solidboundary.surfaces.is_empty()
                && solidboundary.rings.is_empty()
                && solidboundary.vertices.is_empty()
        );
    }
    #[test]
    fn deserialize_solidboundary_basic() {
        let solidboundary_value = json!([[
            [[0, 3, 2, 1]],
            [[4, 5, 6, 7]],
            [[0, 1, 5, 4]],
            [[1, 2, 6, 5]]
        ]]);
        let mut solidboundary = Boundary::default();
        solidboundary_value
            .deserialize_seq(ExtendShellsVisitor(&mut solidboundary))
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();
        assert_eq!(
            solidboundary.vertices,
            vec![0_usize, 3, 2, 1, 4, 5, 6, 7, 0, 1, 5, 4, 1, 2, 6, 5]
        );
        assert_eq!(solidboundary.rings, vec![0_usize, 4, 8, 12]);
        assert_eq!(solidboundary.surfaces, vec![0_usize, 1, 2, 3]);
        assert_eq!(solidboundary.shells, vec![0_usize]);
    }

    #[test]
    fn deserialize_solidboundary_empty_shell() {
        let solidboundary_value = json!([
            [
                [[0, 3, 2, 1]],
                [[4, 5, 6, 7]],
                [[0, 1, 5, 4]],
                [[1, 2, 6, 5]]
            ],
            []
        ]);
        let mut solidboundary = Boundary::default();
        solidboundary_value
            .deserialize_seq(ExtendShellsVisitor(&mut solidboundary))
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();
        assert_eq!(
            solidboundary.vertices,
            vec![0_usize, 3, 2, 1, 4, 5, 6, 7, 0, 1, 5, 4, 1, 2, 6, 5]
        );
        assert_eq!(solidboundary.rings, vec![0_usize, 4, 8, 12]);
        assert_eq!(solidboundary.surfaces, vec![0_usize, 1, 2, 3]);
        // Surface index 4 is out of bounds, which indicates and empty shell.
        assert_eq!(solidboundary.shells, vec![0_usize, 4]);
    }
    #[test]
    fn deserialize_solidboundary_surface_inner_ring() {
        let solidboundary_value = json!([[[[0, 3, 2, 1], [4, 5, 6, 7]]], [[[0, 1, 5, 4]]]]);
        let mut solidboundary = Boundary::default();
        solidboundary_value
            .deserialize_seq(ExtendShellsVisitor(&mut solidboundary))
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();
        assert_eq!(
            solidboundary.vertices,
            vec![0_usize, 3, 2, 1, 4, 5, 6, 7, 0, 1, 5, 4]
        );
        assert_eq!(solidboundary.rings, vec![0_usize, 4, 8]);
        assert_eq!(solidboundary.surfaces, vec![0_usize, 2]);
        assert_eq!(solidboundary.shells, vec![0_usize, 1]);
    }

    #[test]
    fn deserialize_multi_or_compositesurfaceboundary_empty() {
        let multisurfaceboundary_value = json!([]);
        let mut multisurfaceboundary = Boundary::default();
        multisurfaceboundary_value
            .deserialize_seq(ExtendSurfacesVisitor(&mut multisurfaceboundary))
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();
        assert!(
            multisurfaceboundary.surfaces.is_empty()
                && multisurfaceboundary.rings.is_empty()
                && multisurfaceboundary.vertices.is_empty()
        );
    }
    #[test]
    fn deserialize_multi_or_compositesurfaceboundary_basic() {
        let multisurfaceboundary_value = json!([[[0, 3, 2, 1]]]);
        let mut multisurfaceboundary = Boundary::default();
        multisurfaceboundary_value
            .deserialize_seq(ExtendSurfacesVisitor(&mut multisurfaceboundary))
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();
        assert_eq!(multisurfaceboundary.vertices, vec![0_usize, 3, 2, 1]);
        assert_eq!(multisurfaceboundary.rings, vec![0_usize]);
    }
    #[test]
    fn deserialize_multi_or_compositesurfaceboundary_surface_inner_ring() {
        let multisurfaceboundary_value = json!([[[0, 3, 2, 1], [4, 5, 6, 7]], [[0, 3, 2, 1]]]);
        let mut multisurfaceboundary = Boundary::default();
        multisurfaceboundary_value
            .deserialize_seq(ExtendSurfacesVisitor(&mut multisurfaceboundary))
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();
        assert_eq!(
            multisurfaceboundary.vertices,
            vec![0_usize, 3, 2, 1, 4, 5, 6, 7, 0, 3, 2, 1]
        );
        assert_eq!(multisurfaceboundary.rings, vec![0_usize, 4, 8]);
        assert_eq!(multisurfaceboundary.surfaces, vec![0_usize, 2]);
    }

    #[test]
    fn deserialize_surfaceboundary_empty() {
        let surfaceboundary_value = json!([]);
        let mut surfaceboundary = Boundary::default();
        surfaceboundary_value
            .deserialize_seq(ExtendRingsVisitor(&mut surfaceboundary))
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();
        assert!(surfaceboundary.rings.is_empty() && surfaceboundary.vertices.is_empty());
    }
    #[test]
    fn deserialize_surfaceboundary_basic() {
        let surfaceboundary_value = json!([[0, 3, 2, 1]]);
        let mut surfaceboundary = Boundary::default();
        surfaceboundary_value
            .deserialize_seq(ExtendRingsVisitor(&mut surfaceboundary))
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();
        assert_eq!(surfaceboundary.vertices, vec![0_usize, 3, 2, 1]);
        assert_eq!(surfaceboundary.rings, vec![0_usize]);
    }
    #[test]
    fn deserialize_surfaceboundary_inner_ring() {
        let surfaceboundary_value = json!([[0, 3, 2, 1], [4, 5, 6, 7]]);
        let mut surfaceboundary = Boundary::default();
        surfaceboundary_value
            .deserialize_seq(ExtendRingsVisitor(&mut surfaceboundary))
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();
        assert_eq!(surfaceboundary.vertices, vec![0_usize, 3, 2, 1, 4, 5, 6, 7]);
        assert_eq!(surfaceboundary.rings, vec![0_usize, 4]);
    }
    #[test]
    fn deserialize_surfaceboundary_inner_ring_multiple() {
        let surfaceboundary_value = json!([[0, 3, 2, 1], [4, 5, 6, 7], [4, 5, 6, 7]]);
        let mut surfaceboundary = Boundary::default();
        surfaceboundary_value
            .deserialize_seq(ExtendRingsVisitor(&mut surfaceboundary))
            .map_err(|e: serde_json::Error| e.to_string())
            .unwrap();
        assert_eq!(
            surfaceboundary.vertices,
            vec![0_usize, 3, 2, 1, 4, 5, 6, 7, 4, 5, 6, 7]
        );
        assert_eq!(surfaceboundary.rings, vec![0_usize, 4, 8]);
    }
}
