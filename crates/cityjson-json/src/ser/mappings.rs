use std::collections::{HashMap, HashSet, VecDeque};

use cityjson::resources::handles::{SemanticHandle, TextureHandle};
use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{
    boundary::Boundary,
    geometry::{SemanticMapView, TextureMapView},
    CityModel, Geometry, GeometryType, Semantic, SemanticType, VertexRef,
};
use serde::ser::{Error as _, SerializeMap, SerializeSeq};
use serde::Serialize;

use crate::errors::Error;
use crate::ser::attributes::serialize_attributes_entries;
use crate::ser::context::WriteContext;
use crate::ser::geometry::{
    ring_range_for_surface, shell_range_for_solid, surface_range_for_shell,
};

pub(crate) fn has_semantics<VR, SS>(geometry: &Geometry<VR, SS>) -> bool
where
    VR: VertexRef,
    SS: StringStorage,
{
    geometry.semantics().is_some_and(|semantics| {
        !collect_referenced_semantic_handles(geometry, semantics).is_empty()
    })
}

pub(crate) fn has_materials<VR, SS>(geometry: &Geometry<VR, SS>) -> bool
where
    VR: VertexRef,
    SS: StringStorage,
{
    geometry.materials().is_some()
}

pub(crate) fn has_textures<VR, SS>(geometry: &Geometry<VR, SS>) -> bool
where
    VR: VertexRef,
    SS: StringStorage,
{
    geometry.textures().is_some()
}

pub(crate) struct SemanticsSerializer<'a, VR, SS>
where
    VR: VertexRef,
    SS: StringStorage,
{
    pub(crate) model: &'a CityModel<VR, SS>,
    pub(crate) geometry: &'a Geometry<VR, SS>,
}

impl<VR, SS> Serialize for SemanticsSerializer<'_, VR, SS>
where
    VR: VertexRef,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let semantics = self.geometry.semantics().ok_or_else(|| {
            S::Error::custom(Error::InvalidValue("missing geometry semantics".to_owned()))
        })?;
        let handles = collect_geometry_semantic_handles(self.model, self.geometry, semantics);
        if handles.is_empty() {
            let map = serializer.serialize_map(Some(0))?;
            return map.end();
        }

        let handle_to_local = handles
            .iter()
            .copied()
            .enumerate()
            .map(|(index, handle)| (handle, index))
            .collect::<HashMap<_, _>>();

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(
            "surfaces",
            &SemanticSurfacesSerializer {
                model: self.model,
                handles: &handles,
                handle_to_local: &handle_to_local,
            },
        )?;
        map.serialize_entry(
            "values",
            &SemanticValuesSerializer {
                geometry: self.geometry,
                handle_to_local: &handle_to_local,
            },
        )?;
        map.end()
    }
}

pub(crate) struct MaterialsSerializer<'a, VR, SS>
where
    VR: VertexRef,
    SS: StringStorage,
{
    pub(crate) geometry: &'a Geometry<VR, SS>,
    pub(crate) context: &'a WriteContext,
}

impl<VR, SS> Serialize for MaterialsSerializer<'_, VR, SS>
where
    VR: VertexRef,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let materials = self.geometry.materials().ok_or_else(|| {
            S::Error::custom(Error::InvalidValue("missing geometry materials".to_owned()))
        })?;
        let boundary = self.geometry.boundaries().ok_or_else(|| {
            S::Error::custom(Error::InvalidValue(format!(
                "geometry '{}' is missing boundaries",
                self.geometry.type_geometry()
            )))
        })?;

        let mut map = serializer.serialize_map(Some(materials.len()))?;
        for (theme, assignments) in materials.iter() {
            let surfaces = assignments
                .surfaces()
                .iter()
                .map(|material| {
                    (*material)
                        .and_then(|handle| self.context.material_indices.get(&handle).copied())
                })
                .collect::<Vec<_>>();

            if is_uniform_non_null(&surfaces) {
                map.serialize_entry(
                    theme.as_ref(),
                    &UniformMaterialSerializer(surfaces.first().copied().flatten()),
                )?;
            } else {
                map.serialize_entry(
                    theme.as_ref(),
                    &MaterialValuesSerializer {
                        boundary,
                        geometry_type: *self.geometry.type_geometry(),
                        assignments: &surfaces,
                    },
                )?;
            }
        }
        map.end()
    }
}

pub(crate) struct TexturesSerializer<'a, VR, SS>
where
    VR: VertexRef,
    SS: StringStorage,
{
    pub(crate) geometry: &'a Geometry<VR, SS>,
    pub(crate) context: &'a WriteContext,
}

impl<VR, SS> Serialize for TexturesSerializer<'_, VR, SS>
where
    VR: VertexRef,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let textures = self.geometry.textures().ok_or_else(|| {
            S::Error::custom(Error::InvalidValue("missing geometry textures".to_owned()))
        })?;
        let boundary = self.geometry.boundaries().ok_or_else(|| {
            S::Error::custom(Error::InvalidValue(format!(
                "geometry '{}' is missing boundaries",
                self.geometry.type_geometry()
            )))
        })?;

        let mut map = serializer.serialize_map(Some(textures.len()))?;
        for (theme, texture_map) in textures.iter() {
            map.serialize_entry(
                theme.as_ref(),
                &TextureThemeSerializer {
                    boundary,
                    geometry_type: *self.geometry.type_geometry(),
                    texture_map,
                    dense_indices: &self.context.texture_indices,
                },
            )?;
        }
        map.end()
    }
}

struct SemanticSurfacesSerializer<'a, VR, SS>
where
    VR: VertexRef,
    SS: StringStorage,
{
    model: &'a CityModel<VR, SS>,
    handles: &'a [SemanticHandle],
    handle_to_local: &'a HashMap<SemanticHandle, usize>,
}

impl<VR, SS> Serialize for SemanticSurfacesSerializer<'_, VR, SS>
where
    VR: VertexRef,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.handles.len()))?;
        for handle in self.handles {
            let semantic = self.model.get_semantic(*handle).ok_or_else(|| {
                S::Error::custom(Error::InvalidValue(format!(
                    "missing semantic for handle {handle}"
                )))
            })?;
            seq.serialize_element(&SemanticSerializer {
                semantic,
                handle_to_local: self.handle_to_local,
            })?;
        }
        seq.end()
    }
}

struct SemanticSerializer<'a, SS>
where
    SS: StringStorage,
{
    semantic: &'a Semantic<SS>,
    handle_to_local: &'a HashMap<SemanticHandle, usize>,
}

impl<SS> Serialize for SemanticSerializer<'_, SS>
where
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", semantic_type_to_str(self.semantic.type_semantic()))?;
        if let Some(children) = self.semantic.children() {
            let local_children = children
                .iter()
                .filter_map(|handle| self.handle_to_local.get(handle).copied())
                .collect::<Vec<_>>();
            if !local_children.is_empty() {
                map.serialize_entry("children", &local_children)?;
            }
        }
        if let Some(parent) = self.semantic.parent() {
            if let Some(index) = self.handle_to_local.get(&parent).copied() {
                map.serialize_entry("parent", &index)?;
            }
        }
        if let Some(attributes) = self.semantic.attributes() {
            serialize_attributes_entries(&mut map, attributes)?;
        }
        map.end()
    }
}

struct SemanticValuesSerializer<'a, VR, SS>
where
    VR: VertexRef,
    SS: StringStorage,
{
    geometry: &'a Geometry<VR, SS>,
    handle_to_local: &'a HashMap<SemanticHandle, usize>,
}

impl<VR, SS> Serialize for SemanticValuesSerializer<'_, VR, SS>
where
    VR: VertexRef,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let semantics = self.geometry.semantics().ok_or_else(|| {
            S::Error::custom(Error::InvalidValue("missing geometry semantics".to_owned()))
        })?;
        match self.geometry.type_geometry() {
            GeometryType::MultiPoint => FlatHandleSerializer {
                values: semantics.points().iter().collect(),
                handle_to_local: self.handle_to_local,
            }
            .serialize(serializer),
            GeometryType::MultiLineString => FlatHandleSerializer {
                values: semantics.linestrings().iter().collect(),
                handle_to_local: self.handle_to_local,
            }
            .serialize(serializer),
            GeometryType::MultiSurface | GeometryType::CompositeSurface => FlatHandleSerializer {
                values: semantics.surfaces().iter().collect(),
                handle_to_local: self.handle_to_local,
            }
            .serialize(serializer),
            GeometryType::Solid | GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                let boundary = self.geometry.boundaries().ok_or_else(|| {
                    S::Error::custom(Error::InvalidValue(format!(
                        "geometry '{}' is missing boundaries",
                        self.geometry.type_geometry()
                    )))
                })?;
                let assignments = semantics
                    .surfaces()
                    .iter()
                    .map(|handle| {
                        handle
                            .as_ref()
                            .and_then(|handle| self.handle_to_local.get(handle).copied())
                    })
                    .collect::<Vec<_>>();
                NestedOptionalIndexSerializer {
                    boundary,
                    geometry_type: *self.geometry.type_geometry(),
                    assignments: &assignments,
                }
                .serialize(serializer)
            }
            _ => Err(S::Error::custom(Error::InvalidValue(format!(
                "geometry semantics export is not supported for geometry type '{}'",
                self.geometry.type_geometry()
            )))),
        }
    }
}

struct FlatHandleSerializer<'a> {
    values: Vec<&'a Option<SemanticHandle>>,
    handle_to_local: &'a HashMap<SemanticHandle, usize>,
}

impl Serialize for FlatHandleSerializer<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.values.len()))?;
        for handle in &self.values {
            seq.serialize_element(&OptionalIndex(
                handle
                    .as_ref()
                    .and_then(|handle| self.handle_to_local.get(handle).copied()),
            ))?;
        }
        seq.end()
    }
}

struct UniformMaterialSerializer(Option<usize>);

impl Serialize for UniformMaterialSerializer {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("value", &OptionalIndex(self.0))?;
        map.end()
    }
}

struct MaterialValuesSerializer<'a, VR>
where
    VR: VertexRef,
{
    boundary: &'a Boundary<VR>,
    geometry_type: GeometryType,
    assignments: &'a [Option<usize>],
}

impl<VR> Serialize for MaterialValuesSerializer<'_, VR>
where
    VR: VertexRef,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(
            "values",
            &NestedOptionalIndexSerializer {
                boundary: self.boundary,
                geometry_type: self.geometry_type,
                assignments: self.assignments,
            },
        )?;
        map.end()
    }
}

struct NestedOptionalIndexSerializer<'a, VR>
where
    VR: VertexRef,
{
    boundary: &'a Boundary<VR>,
    geometry_type: GeometryType,
    assignments: &'a [Option<usize>],
}

impl<VR> Serialize for NestedOptionalIndexSerializer<'_, VR>
where
    VR: VertexRef,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.geometry_type {
            GeometryType::Solid => {
                let mut seq = serializer.serialize_seq(Some(self.boundary.shells().len()))?;
                for shell_index in 0..self.boundary.shells().len() {
                    let (start, end) = surface_range_for_shell(self.boundary, shell_index);
                    seq.serialize_element(&OptionalIndexSlice(&self.assignments[start..end]))?;
                }
                seq.end()
            }
            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                let mut seq = serializer.serialize_seq(Some(self.boundary.solids().len()))?;
                for solid_index in 0..self.boundary.solids().len() {
                    let (shell_start, shell_end) =
                        shell_range_for_solid(self.boundary, solid_index);
                    seq.serialize_element(&ShellAssignmentsSerializer {
                        boundary: self.boundary,
                        assignments: self.assignments,
                        shell_start,
                        shell_end,
                    })?;
                }
                seq.end()
            }
            _ => serialize_optional_index_seq(serializer, self.assignments),
        }
    }
}

struct OptionalIndexSlice<'a>(&'a [Option<usize>]);

impl Serialize for OptionalIndexSlice<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for value in self.0 {
            seq.serialize_element(&OptionalIndex(*value))?;
        }
        seq.end()
    }
}

struct ShellAssignmentsSerializer<'a, VR>
where
    VR: VertexRef,
{
    boundary: &'a Boundary<VR>,
    assignments: &'a [Option<usize>],
    shell_start: usize,
    shell_end: usize,
}

impl<VR> Serialize for ShellAssignmentsSerializer<'_, VR>
where
    VR: VertexRef,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq =
            serializer.serialize_seq(Some(self.shell_end.saturating_sub(self.shell_start)))?;
        for shell_index in self.shell_start..self.shell_end {
            let (surface_start, surface_end) = surface_range_for_shell(self.boundary, shell_index);
            seq.serialize_element(&OptionalIndexSlice(
                &self.assignments[surface_start..surface_end],
            ))?;
        }
        seq.end()
    }
}

struct TextureThemeSerializer<'a, VR>
where
    VR: VertexRef,
{
    boundary: &'a Boundary<VR>,
    geometry_type: GeometryType,
    texture_map: TextureMapView<'a, VR>,
    dense_indices: &'a HashMap<TextureHandle, usize>,
}

impl<VR> Serialize for TextureThemeSerializer<'_, VR>
where
    VR: VertexRef,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(
            "values",
            &TextureValuesSerializer {
                boundary: self.boundary,
                geometry_type: self.geometry_type,
                texture_map: self.texture_map,
                dense_indices: self.dense_indices,
            },
        )?;
        map.end()
    }
}

struct TextureValuesSerializer<'a, VR>
where
    VR: VertexRef,
{
    boundary: &'a Boundary<VR>,
    geometry_type: GeometryType,
    texture_map: TextureMapView<'a, VR>,
    dense_indices: &'a HashMap<TextureHandle, usize>,
}

impl<VR> Serialize for TextureValuesSerializer<'_, VR>
where
    VR: VertexRef,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.geometry_type {
            GeometryType::MultiSurface | GeometryType::CompositeSurface => {
                let mut seq = serializer.serialize_seq(Some(self.boundary.surfaces().len()))?;
                for surface_index in 0..self.boundary.surfaces().len() {
                    let (ring_start, ring_end) =
                        ring_range_for_surface(self.boundary, surface_index);
                    seq.serialize_element(&TextureRingRangeSerializer {
                        texture_map: self.texture_map,
                        dense_indices: self.dense_indices,
                        ring_start,
                        ring_end,
                    })?;
                }
                seq.end()
            }
            GeometryType::Solid => {
                let mut seq = serializer.serialize_seq(Some(self.boundary.shells().len()))?;
                for shell_index in 0..self.boundary.shells().len() {
                    let (surface_start, surface_end) =
                        surface_range_for_shell(self.boundary, shell_index);
                    seq.serialize_element(&TextureSurfaceRangeSerializer {
                        boundary: self.boundary,
                        texture_map: self.texture_map,
                        dense_indices: self.dense_indices,
                        surface_start,
                        surface_end,
                    })?;
                }
                seq.end()
            }
            GeometryType::MultiSolid | GeometryType::CompositeSolid => {
                let mut seq = serializer.serialize_seq(Some(self.boundary.solids().len()))?;
                for solid_index in 0..self.boundary.solids().len() {
                    let (shell_start, shell_end) =
                        shell_range_for_solid(self.boundary, solid_index);
                    seq.serialize_element(&TextureShellRangeSerializer {
                        boundary: self.boundary,
                        texture_map: self.texture_map,
                        dense_indices: self.dense_indices,
                        shell_start,
                        shell_end,
                    })?;
                }
                seq.end()
            }
            _ => Err(S::Error::custom(Error::InvalidValue(format!(
                "geometry texture export is not supported for geometry type '{}'",
                self.geometry_type
            )))),
        }
    }
}

struct TextureShellRangeSerializer<'a, VR>
where
    VR: VertexRef,
{
    boundary: &'a Boundary<VR>,
    texture_map: TextureMapView<'a, VR>,
    dense_indices: &'a HashMap<TextureHandle, usize>,
    shell_start: usize,
    shell_end: usize,
}

impl<VR> Serialize for TextureShellRangeSerializer<'_, VR>
where
    VR: VertexRef,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq =
            serializer.serialize_seq(Some(self.shell_end.saturating_sub(self.shell_start)))?;
        for shell_index in self.shell_start..self.shell_end {
            let (surface_start, surface_end) = surface_range_for_shell(self.boundary, shell_index);
            seq.serialize_element(&TextureSurfaceRangeSerializer {
                boundary: self.boundary,
                texture_map: self.texture_map,
                dense_indices: self.dense_indices,
                surface_start,
                surface_end,
            })?;
        }
        seq.end()
    }
}

struct TextureSurfaceRangeSerializer<'a, VR>
where
    VR: VertexRef,
{
    boundary: &'a Boundary<VR>,
    texture_map: TextureMapView<'a, VR>,
    dense_indices: &'a HashMap<TextureHandle, usize>,
    surface_start: usize,
    surface_end: usize,
}

impl<VR> Serialize for TextureSurfaceRangeSerializer<'_, VR>
where
    VR: VertexRef,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq =
            serializer.serialize_seq(Some(self.surface_end.saturating_sub(self.surface_start)))?;
        for surface_index in self.surface_start..self.surface_end {
            let (ring_start, ring_end) = ring_range_for_surface(self.boundary, surface_index);
            seq.serialize_element(&TextureRingRangeSerializer {
                texture_map: self.texture_map,
                dense_indices: self.dense_indices,
                ring_start,
                ring_end,
            })?;
        }
        seq.end()
    }
}

struct TextureRingRangeSerializer<'a, VR>
where
    VR: VertexRef,
{
    texture_map: TextureMapView<'a, VR>,
    dense_indices: &'a HashMap<TextureHandle, usize>,
    ring_start: usize,
    ring_end: usize,
}

impl<VR> Serialize for TextureRingRangeSerializer<'_, VR>
where
    VR: VertexRef,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq =
            serializer.serialize_seq(Some(self.ring_end.saturating_sub(self.ring_start)))?;
        for ring_index in self.ring_start..self.ring_end {
            seq.serialize_element(&TextureRingSerializer {
                texture_map: self.texture_map,
                dense_indices: self.dense_indices,
                ring_index,
            })?;
        }
        seq.end()
    }
}

struct TextureRingSerializer<'a, VR>
where
    VR: VertexRef,
{
    texture_map: TextureMapView<'a, VR>,
    dense_indices: &'a HashMap<TextureHandle, usize>,
    ring_index: usize,
}

impl<VR> Serialize for TextureRingSerializer<'_, VR>
where
    VR: VertexRef,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let texture = self
            .texture_map
            .ring_textures()
            .get(self.ring_index)
            .copied()
            .flatten()
            .and_then(|handle| self.dense_indices.get(&handle).copied());
        let Some(texture_index) = texture else {
            let mut seq = serializer.serialize_seq(Some(1))?;
            seq.serialize_element(&OptionalIndex(None))?;
            return seq.end();
        };

        let vertex_start = self
            .texture_map
            .rings()
            .get(self.ring_index)
            .map_or(0, cityjson::v2_0::VertexIndex::to_usize);
        let vertex_end = self.texture_map.rings().get(self.ring_index + 1).map_or(
            self.texture_map.vertices().len(),
            cityjson::v2_0::VertexIndex::to_usize,
        );

        let mut seq =
            serializer.serialize_seq(Some(vertex_end.saturating_sub(vertex_start) + 1))?;
        seq.serialize_element(&texture_index)?;
        for uv_index in &self.texture_map.vertices()[vertex_start..vertex_end] {
            seq.serialize_element(&OptionalIndex(uv_index.map(|uv_index| uv_index.to_usize())))?;
        }
        seq.end()
    }
}

struct OptionalIndex(Option<usize>);

impl Serialize for OptionalIndex {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            Some(value) => serializer.serialize_u64(value as u64),
            None => serializer.serialize_none(),
        }
    }
}

fn collect_referenced_semantic_handles<VR, SS>(
    geometry: &Geometry<VR, SS>,
    semantics: SemanticMapView<'_, VR>,
) -> Vec<SemanticHandle>
where
    VR: VertexRef,
    SS: StringStorage,
{
    match geometry.type_geometry() {
        GeometryType::MultiPoint => semantics.points().iter().flatten().copied().collect(),
        GeometryType::MultiLineString => {
            semantics.linestrings().iter().flatten().copied().collect()
        }
        _ => semantics.surfaces().iter().flatten().copied().collect(),
    }
}

fn collect_geometry_semantic_handles<VR, SS>(
    model: &CityModel<VR, SS>,
    geometry: &Geometry<VR, SS>,
    semantics: SemanticMapView<'_, VR>,
) -> Vec<SemanticHandle>
where
    VR: VertexRef,
    SS: StringStorage,
{
    let mut ordered = Vec::new();
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();

    let push_handle = |handle: SemanticHandle,
                       ordered: &mut Vec<SemanticHandle>,
                       seen: &mut HashSet<SemanticHandle>,
                       queue: &mut VecDeque<SemanticHandle>| {
        if seen.insert(handle) {
            ordered.push(handle);
            queue.push_back(handle);
        }
    };

    match geometry.type_geometry() {
        GeometryType::MultiPoint => {
            for handle in semantics.points().iter().flatten() {
                push_handle(*handle, &mut ordered, &mut seen, &mut queue);
            }
        }
        GeometryType::MultiLineString => {
            for handle in semantics.linestrings().iter().flatten() {
                push_handle(*handle, &mut ordered, &mut seen, &mut queue);
            }
        }
        _ => {
            for handle in semantics.surfaces().iter().flatten() {
                push_handle(*handle, &mut ordered, &mut seen, &mut queue);
            }
        }
    }

    while let Some(handle) = queue.pop_front() {
        let Some(semantic) = model.get_semantic(handle) else {
            continue;
        };
        if let Some(parent) = semantic.parent() {
            push_handle(parent, &mut ordered, &mut seen, &mut queue);
        }
        if let Some(children) = semantic.children() {
            for &child in children {
                push_handle(child, &mut ordered, &mut seen, &mut queue);
            }
        }
    }

    ordered
}

fn semantic_type_to_str<SS>(semantic_type: &SemanticType<SS>) -> &str
where
    SS: StringStorage,
{
    match semantic_type {
        SemanticType::RoofSurface => "RoofSurface",
        SemanticType::GroundSurface => "GroundSurface",
        SemanticType::WallSurface => "WallSurface",
        SemanticType::ClosureSurface => "ClosureSurface",
        SemanticType::OuterCeilingSurface => "OuterCeilingSurface",
        SemanticType::OuterFloorSurface => "OuterFloorSurface",
        SemanticType::Window => "Window",
        SemanticType::Door => "Door",
        SemanticType::InteriorWallSurface => "InteriorWallSurface",
        SemanticType::CeilingSurface => "CeilingSurface",
        SemanticType::FloorSurface => "FloorSurface",
        SemanticType::WaterSurface => "WaterSurface",
        SemanticType::WaterGroundSurface => "WaterGroundSurface",
        SemanticType::WaterClosureSurface => "WaterClosureSurface",
        SemanticType::TrafficArea => "TrafficArea",
        SemanticType::AuxiliaryTrafficArea => "AuxiliaryTrafficArea",
        SemanticType::TransportationMarking => "TransportationMarking",
        SemanticType::TransportationHole => "TransportationHole",
        SemanticType::Extension(value) => value.as_ref(),
        _ => "Default",
    }
}

fn is_uniform_non_null(values: &[Option<usize>]) -> bool {
    let Some(first) = values.first().copied().flatten() else {
        return false;
    };
    values.iter().all(|value| *value == Some(first))
}

fn serialize_optional_index_seq<S>(
    serializer: S,
    values: &[Option<usize>],
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(values.len()))?;
    for value in values {
        seq.serialize_element(&OptionalIndex(*value))?;
    }
    seq.end()
}
