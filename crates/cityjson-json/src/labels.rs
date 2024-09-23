use std::fmt;

use serde::{Deserialize, Serialize};
use serde::de::{Deserializer, DeserializeSeed, SeqAccess, Visitor};
use serde::ser::{Error, Serializer, SerializeSeq};
#[cfg(feature = "datasize")]
use datasize::DataSize;

use crate::boundary::{BoundaryCounter, BoundaryType};
use crate::indices::{LargeIndex, LargeIndexVec, OptionalLargeIndex};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Texture indices

/// Stores the Texture indices of a Boundary.
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq, Deserialize)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct TextureIndex {
    pub vertices: Vec<OptionalLargeIndex>,
    pub rings: LargeIndexVec,
    pub rings_textures: Vec<OptionalLargeIndex>,
    pub surfaces: LargeIndexVec,
    pub shells: LargeIndexVec,
    pub solids: LargeIndexVec,
}

impl Serialize for TextureIndex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.check_type() {
            BoundaryType::MultiOrCompositeSolid => {
                let mut nested_json = serializer.serialize_seq(Some(self.solids.len()))?;
                let mut counter = BoundaryCounter::default();
                for shells_start_i in &self.solids {
                    let shells_len = LargeIndex::try_from(self.shells.len()).unwrap();
                    let shells_end_i = self
                        .solids
                        .get(counter.next_solid_i())
                        .unwrap_or(&shells_len);
                    let s_usize = usize::try_from(*shells_start_i).unwrap();
                    let e_usize = usize::try_from(*shells_end_i).unwrap();
                    if let Some(shells) = self.shells.get(s_usize..e_usize) {
                        let mut solid = NestedSolidTextureValues::with_capacity(shells.len());
                        for surfaces_start_i in shells {
                            let surfaces_len = LargeIndex::try_from(self.surfaces.len()).unwrap();
                            let _nshi = counter.next_shell_i();
                            let _srf_e_i_op = self.shells.get(_nshi);
                            let surfaces_end_i = _srf_e_i_op.unwrap_or(&surfaces_len);
                            let s_usize = usize::try_from(*surfaces_start_i).unwrap();
                            let e_usize = usize::try_from(*surfaces_end_i).unwrap();
                            if let Some(surfaces) =
                                self.surfaces.get(s_usize..e_usize)
                            {
                                let mut shell =
                                    NestedShellTextureValues::with_capacity(surfaces.len());
                                for rings_start_i in surfaces {
                                    let rings_len = LargeIndex::try_from(self.rings.len()).unwrap();
                                    let rings_end_i = self
                                        .surfaces
                                        .get(counter.next_surface_i())
                                        .unwrap_or(&rings_len);
                                    let s_usize = usize::try_from(*rings_start_i).unwrap();
                                    let e_usize = usize::try_from(*rings_end_i).unwrap();
                                    if let Some(rings) =
                                        self.rings.get(s_usize..e_usize)
                                    {
                                        let mut surface =
                                            NestedSurfaceTextureValues::with_capacity(rings.len());
                                        for vertices_start_i in rings {
                                            let vertices_len = LargeIndex::try_from(self.vertices.len()).unwrap();
                                            if let Some(ring_texture) =
                                                self.rings_textures.get(counter.ring_i)
                                            {
                                                let vertices_end_i = self
                                                    .rings
                                                    .get(counter.next_ring_i())
                                                    .unwrap_or(&vertices_len);
                                                let s_usize = usize::try_from(*vertices_start_i).unwrap();
                                                let e_usize = usize::try_from(*vertices_end_i).unwrap();
                                                if let Some(vertices) = self
                                                    .vertices
                                                    .get(s_usize..e_usize)
                                                {
                                                    let ring = [
                                                        &[*ring_texture],
                                                        vertices,
                                                    ]
                                                    .concat();
                                                    surface.push(ring.iter().map(|v|v.map(|i| u32::from(&i))).collect());
                                                }
                                            }
                                        }
                                        shell.push(surface);
                                    }
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
                for surfaces_start_i in &self.shells {
                    let surfaces_len = LargeIndex::try_from(self.surfaces.len()).unwrap();
                    let surfaces_end_i = self
                        .shells
                        .get(counter.next_shell_i())
                        .unwrap_or(&surfaces_len);
                    let s_usize = usize::try_from(*surfaces_start_i).unwrap();
                    let e_usize = usize::try_from(*surfaces_end_i).unwrap();
                    if let Some(surfaces) = self.surfaces.get(s_usize..e_usize) {
                        let mut shell = NestedShellTextureValues::with_capacity(surfaces.len());
                        for rings_start_i in surfaces {
                            let rings_len = LargeIndex::try_from(self.rings.len()).unwrap();
                            let rings_end_i = self
                                .surfaces
                                .get(counter.next_surface_i())
                                .unwrap_or(&rings_len);
                            let s_usize = usize::try_from(*rings_start_i).unwrap();
                            let e_usize = usize::try_from(*rings_end_i).unwrap();
                            if let Some(rings) = self.rings.get(s_usize..e_usize) {
                                let mut surface =
                                    NestedSurfaceTextureValues::with_capacity(rings.len());
                                for vertices_start_i in rings {
                                    let vertices_len = LargeIndex::try_from(self.vertices.len()).unwrap();
                                    if let Some(ring_texture) =
                                        self.rings_textures.get(counter.ring_i)
                                    {
                                        let vertices_end_i = self
                                            .rings
                                            .get(counter.next_ring_i())
                                            .unwrap_or(&vertices_len);
                                        let s_usize = usize::try_from(*vertices_start_i).unwrap();
                                        let e_usize = usize::try_from(*vertices_end_i).unwrap();
                                        if let Some(vertices) =
                                            self.vertices.get(s_usize..e_usize)
                                        {
                                            let ring =
                                                [&[*ring_texture], vertices].concat();
                                            surface.push(ring.iter().map(|v|v.map(|i| u32::from(&i))).collect());
                                        }
                                    }
                                }
                                shell.push(surface);
                            }
                        }
                        nested_json.serialize_element(&shell)?;
                    }
                }
                nested_json.end()
            }
            BoundaryType::MultiOrCompositeSurface => {
                let mut nested_json = serializer.serialize_seq(Some(self.surfaces.len()))?;
                let mut counter = BoundaryCounter::default();
                for rings_start_i in &self.surfaces {
                    let rings_len = LargeIndex::try_from(self.rings.len()).unwrap();
                    let rings_end_i = self
                        .surfaces
                        .get(counter.next_surface_i())
                        .unwrap_or(&rings_len);
                    let s_usize = usize::try_from(*rings_start_i).unwrap();
                    let e_usize = usize::try_from(*rings_end_i).unwrap();
                    if let Some(rings) = self.rings.get(s_usize..e_usize) {
                        let mut surface = NestedSurfaceTextureValues::with_capacity(rings.len());
                        for vertices_start_i in rings {
                            let vertices_len = LargeIndex::try_from(self.vertices.len()).unwrap();
                            if let Some(ring_texture) = self.rings_textures.get(counter.ring_i) {
                                let vertices_end_i = self
                                    .rings
                                    .get(counter.next_ring_i())
                                    .unwrap_or(&vertices_len);
                                let s_usize = usize::try_from(*vertices_start_i).unwrap();
                                let e_usize = usize::try_from(*vertices_end_i).unwrap();
                                if let Some(vertices) =
                                    self.vertices.get(s_usize..e_usize)
                                {
                                    let ring =
                                        [&[*ring_texture], vertices].concat();
                                    surface.push(ring.iter().map(|v|v.map(|i| u32::from(&i))).collect());
                                }
                            }
                        }
                        nested_json.serialize_element(&surface)?;
                    }
                }
                nested_json.end()
            }
            _ => Err(Error::custom("cannot serialize an empty TextureIndex")),
        }
    }
}

impl TextureIndex {
    /// Hint what [crate::boundary::BoundaryType] does the TextureIndex belong to.
    pub fn check_type(&self) -> BoundaryType {
        if !self.solids.is_empty() {
            BoundaryType::MultiOrCompositeSolid
        } else if !self.shells.is_empty() {
            BoundaryType::Solid
        } else if !self.surfaces.is_empty() {
            BoundaryType::MultiOrCompositeSurface
        } else {
            BoundaryType::None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Visitor implementations for TextureIndex

struct ExtendTextureIndexVertices<'a>(&'a mut TextureIndex);
pub(crate) struct ExtendTextureIndexVerticesVisitor<'a>(pub(crate) &'a mut TextureIndex);

impl<'de, 'a> Visitor<'de> for ExtendTextureIndexVerticesVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "an array of optional vertex indices")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if let Some(size_hint) = seq.size_hint() {
            self.0.vertices.reserve(size_hint);
        }

        // First item in the ring is the index of the texture object
        if let Some(ring_texture_i) = seq.next_element()? {
            self.0.rings_textures.push(ring_texture_i);

            while let Some(elem) = seq.next_element()? {
                self.0.vertices.push(elem);
            }
        }

        Ok(())
    }
}

impl<'de, 'a> DeserializeSeed<'de> for ExtendTextureIndexVertices<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendTextureIndexVerticesVisitor(self.0))
    }
}

struct ExtendTextureIndexRings<'a>(&'a mut TextureIndex);
pub(crate) struct ExtendTextureIndexRingsVisitor<'a>(pub(crate) &'a mut TextureIndex);
impl<'de, 'a> Visitor<'de> for ExtendTextureIndexRingsVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a surface boundary array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Add the start index of the first ring of the surface.
        self.0.rings.push(LargeIndex::try_from(self.0.vertices.len()).unwrap());
        // Each iteration through this loop is one ring.
        while let Some(()) = seq.next_element_seed(ExtendTextureIndexVertices(self.0))? {
            self.0.rings.push(LargeIndex::try_from(self.0.vertices.len()).unwrap());
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
impl<'de, 'a> DeserializeSeed<'de> for ExtendTextureIndexRings<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendTextureIndexRingsVisitor(self.0))
    }
}

struct ExtendTextureIndexSurfaces<'a>(&'a mut TextureIndex);
pub(crate) struct ExtendTextureIndexSurfacesVisitor<'a>(pub(crate) &'a mut TextureIndex);

impl<'de, 'a> Visitor<'de> for ExtendTextureIndexSurfacesVisitor<'a> {
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
        self.0.surfaces.push(LargeIndex::try_from(self.0.rings.len()).unwrap());
        // Each iteration through this loop is one inner array.
        while let Some(()) = seq.next_element_seed(ExtendTextureIndexRings(self.0))? {
            self.0.surfaces.push(LargeIndex::try_from(self.0.rings.len()).unwrap());
        }
        if !self.0.surfaces.is_empty() {
            let last_idx = self.0.surfaces.len() - 1;
            self.0.surfaces.remove(last_idx);
        }
        Ok(())
    }
}

impl<'de, 'a> DeserializeSeed<'de> for ExtendTextureIndexSurfaces<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendTextureIndexSurfacesVisitor(self.0))
    }
}

struct ExtendTextureIndexShells<'a>(&'a mut TextureIndex);
pub(crate) struct ExtendTextureIndexShellsVisitor<'a>(pub(crate) &'a mut TextureIndex);

impl<'de, 'a> Visitor<'de> for ExtendTextureIndexShellsVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a solid boundary array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Add the start index of the first surface of the aggregate
        self.0.shells.push(LargeIndex::try_from(self.0.surfaces.len()).unwrap());
        // Each iteration through this loop is one inner array.
        while let Some(()) = seq.next_element_seed(ExtendTextureIndexSurfaces(self.0))? {
            self.0.shells.push(LargeIndex::try_from(self.0.surfaces.len()).unwrap());
        }
        if !self.0.shells.is_empty() {
            let last_idx = self.0.shells.len() - 1;
            self.0.shells.remove(last_idx);
        }
        Ok(())
    }
}

impl<'de, 'a> DeserializeSeed<'de> for ExtendTextureIndexShells<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendTextureIndexShellsVisitor(self.0))
    }
}

#[allow(dead_code)]
struct ExtendTextureIndexSolids<'a>(&'a mut TextureIndex);
pub(crate) struct ExtendTextureIndexSolidsVisitor<'a>(pub(crate) &'a mut TextureIndex);

impl<'de, 'a> Visitor<'de> for ExtendTextureIndexSolidsVisitor<'a> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a multi- or compositesolid boundary array")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Add the start index of the first shell of the aggregate
        self.0.solids.push(LargeIndex::try_from(self.0.shells.len()).unwrap());
        // Each iteration through this loop is one inner array.
        while let Some(()) = seq.next_element_seed(ExtendTextureIndexShells(self.0))? {
            self.0.solids.push(LargeIndex::try_from(self.0.shells.len()).unwrap());
        }
        if !self.0.solids.is_empty() {
            let last_idx = self.0.solids.len() - 1;
            self.0.solids.remove(last_idx);
        }
        Ok(())
    }
}

impl<'de, 'a> DeserializeSeed<'de> for ExtendTextureIndexSolids<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendTextureIndexSolidsVisitor(self.0))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Semantic and Material indices

/// Stores the Semantic and Material indices of a Boundary.
///
/// The arrays that store the Semantic or Material indices and point to the geometry
/// primitives have the same structure for semantics and materials. Both label the geometry
/// primitives with extra information, hence the name `LabelIndex`.
#[derive(Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq, Deserialize)]
#[cfg_attr(feature = "datasize", derive(DataSize))]
pub struct LabelIndex {
    /// Each item corresponds to the point with the same index in a MultiPoint boundary, the value
    /// of the item is the index of the Semantic or Material object.
    pub points: Vec<OptionalLargeIndex>,
    /// Each item corresponds to the linestring with the same index in a MultiLineString boundary,
    /// the value of the item is the index of the Semantic or Material object.
    pub linestrings: Vec<OptionalLargeIndex>,
    /// Each item corresponds to the surface with the same index, the value
    /// of the item is the index of the Semantic or Material object.
    pub surfaces: Vec<OptionalLargeIndex>,
    pub shells: LargeIndexVec,
    pub solids: LargeIndexVec,
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
                    let shells_len = LargeIndex::try_from(self.shells.len()).unwrap();
                    let shells_end_i = self
                        .solids
                        .get(counter.next_solid_i())
                        .unwrap_or(&shells_len);
                    let s_usize = usize::try_from(*shells_start_i).unwrap();
                    let e_usize = usize::try_from(*shells_end_i).unwrap();
                    if let Some(shells) = self.shells.get(s_usize..e_usize) {
                        let mut solid = NestedSolidSemanticsValues::with_capacity(shells.len());
                        for surfaces_start_i in shells {
                            let surfaces_len = LargeIndex::try_from(self.surfaces.len()).unwrap();
                            let surfaces_end_i = self
                                .shells
                                .get(counter.next_shell_i())
                                .unwrap_or(&surfaces_len);
                            let s_usize = usize::try_from(*surfaces_start_i).unwrap();
                            let e_usize = usize::try_from(*surfaces_end_i).unwrap();
                            if let Some(surfaces) =
                                self.surfaces.get(s_usize..e_usize)
                            {
                                let mut shell =
                                    NestedShellSemanticsValues::with_capacity(surfaces.len());
                                for op_idx in surfaces {
                                    shell.push(op_idx.map(|v| u32::from(&v)));
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
                    let surfaces_len = LargeIndex::try_from(self.surfaces.len()).unwrap();
                    let surfaces_end_i = self
                        .shells
                        .get(counter.next_shell_i())
                        .unwrap_or(&surfaces_len);
                    let s_usize = usize::try_from(*surfaces_start_i).unwrap();
                    let e_usize = usize::try_from(*surfaces_end_i).unwrap();
                    if let Some(surfaces) = self.surfaces.get(s_usize..e_usize) {
                        let mut shell = NestedShellSemanticsValues::with_capacity(surfaces.len());
                        for op_idx in surfaces {
                            shell.push(op_idx.map(|v| u32::from(&v)));
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
struct ExtendLabelIndexSurfaces<'a>(&'a mut LabelIndex);
pub(crate) struct ExtendLabelIndexSurfacesVisitor<'a>(pub(crate) &'a mut LabelIndex);

impl<'de, 'a> Visitor<'de> for ExtendLabelIndexSurfacesVisitor<'a> {
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

impl<'de, 'a> DeserializeSeed<'de> for ExtendLabelIndexSurfaces<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendLabelIndexSurfacesVisitor(self.0))
    }
}

struct ExtendLabelIndexShells<'a>(&'a mut LabelIndex);
pub(crate) struct ExtendLabelIndexShellsVisitor<'a>(pub(crate) &'a mut LabelIndex);
impl<'de, 'a> Visitor<'de> for ExtendLabelIndexShellsVisitor<'a> {
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
        self.0
            .shells
            .push(LargeIndex::try_from(self.0.surfaces.len()).unwrap());
        // Each iteration through this loop is one ring.
        while let Some(()) = seq.next_element_seed(ExtendLabelIndexSurfaces(self.0))? {
            self.0
                .shells
                .push(LargeIndex::try_from(self.0.surfaces.len()).unwrap());
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
impl<'de, 'a> DeserializeSeed<'de> for ExtendLabelIndexShells<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendLabelIndexShellsVisitor(self.0))
    }
}

#[allow(dead_code)]
struct ExtendLabelIndexSolids<'a>(&'a mut LabelIndex);
pub(crate) struct ExtendLabelIndexSolidsVisitor<'a>(pub(crate) &'a mut LabelIndex);

impl<'de, 'a> Visitor<'de> for ExtendLabelIndexSolidsVisitor<'a> {
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
        self.0
            .solids
            .push(LargeIndex::try_from(self.0.shells.len()).unwrap());
        // Each iteration through this loop is one inner array.
        while let Some(()) = seq.next_element_seed(ExtendLabelIndexShells(self.0))? {
            self.0
                .solids
                .push(LargeIndex::try_from(self.0.shells.len()).unwrap());
        }
        if !self.0.solids.is_empty() {
            let last_idx = self.0.solids.len() - 1;
            self.0.solids.remove(last_idx);
        }
        Ok(())
    }
}

impl<'de, 'a> DeserializeSeed<'de> for ExtendLabelIndexSolids<'a> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ExtendLabelIndexSolidsVisitor(self.0))
    }
}
////////////////////////////////////////////////////////////////////////////////////////////////////

// TODO: unify with semantics/material nested as
pub type NestedMultiSolidTextureValues = Vec<Vec<Vec<Vec<Vec<OptionalIndex>>>>>;
pub type NestedSolidTextureValues = Vec<Vec<Vec<Vec<OptionalIndex>>>>;
pub type NestedShellTextureValues = Vec<Vec<Vec<OptionalIndex>>>;
pub type NestedSurfaceTextureValues = Vec<Vec<OptionalIndex>>;
pub type NestedRingTextureValues = Vec<OptionalIndex>;

// TODO: these are used for the Materials too
pub type NestedMultiSolidSemanticsValues = Vec<Vec<Vec<OptionalIndex>>>;
pub type NestedSolidSemanticsValues = Vec<Vec<OptionalIndex>>;
pub type NestedShellSemanticsValues = Vec<OptionalIndex>;

// TODO: this can easily be u8, couz I don't expect to have more than 255 different Semantic object
//  on a single geometry...but if the shitty code does not dedup the Semantic objects then I could
//  have a problem, because there will be as many Semantic objects as geometry primitives.
pub type OptionalIndex = Option<u32>;
