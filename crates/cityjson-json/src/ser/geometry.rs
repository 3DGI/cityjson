use cityjson_types::resources::handles::GeometryHandle;
use cityjson_types::resources::storage::StringStorage;
use cityjson_types::v2_0::{Boundary, CityModel, Geometry, GeometryType, VertexRef};
use serde::Serialize;
use serde::ser::{Error as _, SerializeMap, SerializeSeq};

use crate::errors::{Error, Result};
use crate::ser::context::WriteContext;
use crate::ser::mappings::{
    MaterialsSerializer, SemanticsSerializer, TexturesSerializer, has_materials, has_semantics,
    has_textures,
};

pub(crate) struct GeometriesSerializer<'a, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    pub(crate) model: &'a CityModel<VR, SS>,
    pub(crate) handles: &'a [GeometryHandle],
    pub(crate) context: &'a WriteContext,
}

impl<VR, SS> Serialize for GeometriesSerializer<'_, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.handles.len()))?;
        for handle in self.handles {
            let geometry = self.model.get_geometry(*handle).ok_or_else(|| {
                S::Error::custom(Error::InvalidValue(format!(
                    "missing geometry for handle {handle}"
                )))
            })?;
            seq.serialize_element(&GeometrySerializer {
                model: self.model,
                geometry,
                context: self.context,
            })?;
        }
        seq.end()
    }
}

pub(crate) struct GeometrySerializer<'a, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    pub(crate) model: &'a CityModel<VR, SS>,
    pub(crate) geometry: &'a Geometry<VR, SS>,
    pub(crate) context: &'a WriteContext,
}

impl<VR, SS> GeometrySerializer<'_, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    pub(crate) fn validate(&self) -> Result<()> {
        if let Some(instance) = self.geometry.instance() {
            if !self
                .context
                .template_indices
                .contains_key(&instance.template())
            {
                return Err(Error::InvalidValue(format!(
                    "missing dense template index for template {}",
                    instance.template()
                )));
            }
            return Ok(());
        }

        if self.geometry.boundaries().is_none() {
            return Err(Error::InvalidValue(format!(
                "geometry '{}' is missing boundaries",
                self.geometry.type_geometry()
            )));
        }

        match self.geometry.type_geometry() {
            GeometryType::MultiPoint
            | GeometryType::MultiLineString
            | GeometryType::MultiSurface
            | GeometryType::CompositeSurface
            | GeometryType::Solid
            | GeometryType::MultiSolid
            | GeometryType::CompositeSolid
            | GeometryType::GeometryInstance => Ok(()),
            _ => Err(Error::InvalidValue(format!(
                "unsupported geometry type '{}'",
                self.geometry.type_geometry()
            ))),
        }
    }
}

impl<VR, SS> Serialize for GeometrySerializer<'_, VR, SS>
where
    VR: VertexRef + serde::Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.validate().map_err(S::Error::custom)?;

        if let Some(instance) = self.geometry.instance() {
            let template_index = self
                .context
                .template_indices
                .get(&instance.template())
                .copied()
                .ok_or_else(|| {
                    S::Error::custom(Error::InvalidValue(format!(
                        "missing dense template index for template {}",
                        instance.template()
                    )))
                })?;
            let mut map = serializer.serialize_map(Some(4))?;
            map.serialize_entry("type", "GeometryInstance")?;
            map.serialize_entry("template", &template_index)?;
            map.serialize_entry(
                "boundaries",
                &InstanceBoundarySerializer(instance.reference_point().value()),
            )?;
            map.serialize_entry(
                "transformationMatrix",
                &MatrixSerializer(instance.transformation().into_array()),
            )?;
            return map.end();
        }

        let boundary = self.geometry.boundaries().expect("validated above");
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", &self.geometry.type_geometry().to_string())?;
        if let Some(lod) = self.geometry.lod() {
            map.serialize_entry("lod", &lod.to_string())?;
        }
        map.serialize_entry(
            "boundaries",
            &BoundarySerializer {
                boundary,
                geometry_type: *self.geometry.type_geometry(),
            },
        )?;
        if has_semantics(self.geometry) {
            map.serialize_entry(
                "semantics",
                &SemanticsSerializer {
                    model: self.model,
                    geometry: self.geometry,
                    context: self.context,
                },
            )?;
        }
        if has_materials(self.geometry) {
            map.serialize_entry(
                "material",
                &MaterialsSerializer {
                    geometry: self.geometry,
                    context: self.context,
                },
            )?;
        }
        if has_textures(self.geometry) {
            map.serialize_entry(
                "texture",
                &TexturesSerializer {
                    geometry: self.geometry,
                    context: self.context,
                },
            )?;
        }
        map.end()
    }
}

struct BoundarySerializer<'a, VR>
where
    VR: VertexRef + serde::Serialize,
{
    boundary: &'a Boundary<VR>,
    geometry_type: GeometryType,
}

impl<VR> Serialize for BoundarySerializer<'_, VR>
where
    VR: VertexRef + serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.geometry_type {
            GeometryType::MultiPoint => {
                VertexSliceSerializer(self.boundary.vertices()).serialize(serializer)
            }
            GeometryType::MultiLineString => RingRangeSerializer {
                boundary: self.boundary,
                start: 0,
                end: self.boundary.rings().len(),
            }
            .serialize(serializer),
            GeometryType::MultiSurface | GeometryType::CompositeSurface => SurfaceRangeSerializer {
                boundary: self.boundary,
                start: 0,
                end: self.boundary.surfaces().len(),
            }
            .serialize(serializer),
            GeometryType::Solid => ShellRangeSerializer {
                boundary: self.boundary,
                start: 0,
                end: self.boundary.shells().len(),
            }
            .serialize(serializer),
            GeometryType::MultiSolid | GeometryType::CompositeSolid => SolidRangeSerializer {
                boundary: self.boundary,
                start: 0,
                end: self.boundary.solids().len(),
            }
            .serialize(serializer),
            GeometryType::GeometryInstance => unreachable!("handled separately"),
            _ => Err(S::Error::custom(Error::InvalidValue(format!(
                "unsupported geometry type '{}'",
                self.geometry_type
            )))),
        }
    }
}

struct VertexSliceSerializer<'a, VR>(&'a [cityjson_types::v2_0::VertexIndex<VR>])
where
    VR: VertexRef + serde::Serialize;

impl<VR> Serialize for VertexSliceSerializer<'_, VR>
where
    VR: VertexRef + serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for vertex in self.0 {
            seq.serialize_element(&vertex.value())?;
        }
        seq.end()
    }
}

struct RingRangeSerializer<'a, VR>
where
    VR: VertexRef + serde::Serialize,
{
    boundary: &'a Boundary<VR>,
    start: usize,
    end: usize,
}

impl<VR> Serialize for RingRangeSerializer<'_, VR>
where
    VR: VertexRef + serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.end.saturating_sub(self.start)))?;
        for ring_index in self.start..self.end {
            let vertex_start = self.boundary.rings()[ring_index].to_usize();
            let vertex_end = self.boundary.rings().get(ring_index + 1).map_or(
                self.boundary.vertices().len(),
                cityjson_types::v2_0::VertexIndex::to_usize,
            );
            seq.serialize_element(&VertexSliceSerializer(
                &self.boundary.vertices()[vertex_start..vertex_end],
            ))?;
        }
        seq.end()
    }
}

struct SurfaceRangeSerializer<'a, VR>
where
    VR: VertexRef + serde::Serialize,
{
    boundary: &'a Boundary<VR>,
    start: usize,
    end: usize,
}

impl<VR> Serialize for SurfaceRangeSerializer<'_, VR>
where
    VR: VertexRef + serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.end.saturating_sub(self.start)))?;
        for surface_index in self.start..self.end {
            let (ring_start, ring_end) = ring_range_for_surface(self.boundary, surface_index);
            seq.serialize_element(&RingRangeSerializer {
                boundary: self.boundary,
                start: ring_start,
                end: ring_end,
            })?;
        }
        seq.end()
    }
}

struct ShellRangeSerializer<'a, VR>
where
    VR: VertexRef + serde::Serialize,
{
    boundary: &'a Boundary<VR>,
    start: usize,
    end: usize,
}

impl<VR> Serialize for ShellRangeSerializer<'_, VR>
where
    VR: VertexRef + serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.end.saturating_sub(self.start)))?;
        for shell_index in self.start..self.end {
            let (surface_start, surface_end) = surface_range_for_shell(self.boundary, shell_index);
            seq.serialize_element(&SurfaceRangeSerializer {
                boundary: self.boundary,
                start: surface_start,
                end: surface_end,
            })?;
        }
        seq.end()
    }
}

struct SolidRangeSerializer<'a, VR>
where
    VR: VertexRef + serde::Serialize,
{
    boundary: &'a Boundary<VR>,
    start: usize,
    end: usize,
}

impl<VR> Serialize for SolidRangeSerializer<'_, VR>
where
    VR: VertexRef + serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.end.saturating_sub(self.start)))?;
        for solid_index in self.start..self.end {
            let (shell_start, shell_end) = shell_range_for_solid(self.boundary, solid_index);
            seq.serialize_element(&ShellRangeSerializer {
                boundary: self.boundary,
                start: shell_start,
                end: shell_end,
            })?;
        }
        seq.end()
    }
}

struct InstanceBoundarySerializer<VR>(VR)
where
    VR: VertexRef + serde::Serialize;

impl<VR> Serialize for InstanceBoundarySerializer<VR>
where
    VR: VertexRef + serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(1))?;
        seq.serialize_element(&self.0)?;
        seq.end()
    }
}

struct MatrixSerializer([f64; 16]);

impl Serialize for MatrixSerializer {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(16))?;
        for value in self.0 {
            seq.serialize_element(&value)?;
        }
        seq.end()
    }
}

pub(crate) fn ring_range_for_surface<VR>(
    boundary: &Boundary<VR>,
    surface_index: usize,
) -> (usize, usize)
where
    VR: VertexRef,
{
    let start = boundary.surfaces()[surface_index].to_usize();
    let end = boundary.surfaces().get(surface_index + 1).map_or(
        boundary.rings().len(),
        cityjson_types::v2_0::VertexIndex::to_usize,
    );
    (start, end)
}

pub(crate) fn surface_range_for_shell<VR>(
    boundary: &Boundary<VR>,
    shell_index: usize,
) -> (usize, usize)
where
    VR: VertexRef,
{
    let start = boundary.shells()[shell_index].to_usize();
    let end = boundary.shells().get(shell_index + 1).map_or(
        boundary.surfaces().len(),
        cityjson_types::v2_0::VertexIndex::to_usize,
    );
    (start, end)
}

pub(crate) fn shell_range_for_solid<VR>(
    boundary: &Boundary<VR>,
    solid_index: usize,
) -> (usize, usize)
where
    VR: VertexRef,
{
    let start = boundary.solids()[solid_index].to_usize();
    let end = boundary.solids().get(solid_index + 1).map_or(
        boundary.shells().len(),
        cityjson_types::v2_0::VertexIndex::to_usize,
    );
    (start, end)
}
