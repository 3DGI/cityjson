use crate::relational::RelationalAccess;
use crate::v2_0::OwnedCityModel;

/// Scalar summary over an owned city model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelSummary {
    pub cityobject_count: u32,
    pub geometry_count: u32,
    pub geometry_template_count: u32,
    pub vertex_count: u32,
    pub template_vertex_count: u32,
    pub uv_vertex_count: u32,
    pub semantic_count: u32,
    pub material_count: u32,
    pub texture_count: u32,
    pub symbol_count: u32,
    pub has_metadata: bool,
    pub has_transform: bool,
}

#[must_use]
pub fn summary(model: &OwnedCityModel) -> ModelSummary {
    let relational = model.relational();
    let raw = relational.raw();
    let symbol_count = u32::try_from(relational.snapshot().symbols().len()).unwrap_or(u32::MAX);

    ModelSummary {
        cityobject_count: u32::try_from(relational.cityobjects().len()).unwrap_or(u32::MAX),
        geometry_count: u32::try_from(raw.geometries().len()).unwrap_or(u32::MAX),
        geometry_template_count: u32::try_from(model.geometry_template_count()).unwrap_or(u32::MAX),
        vertex_count: u32::try_from(raw.vertices().len()).unwrap_or(u32::MAX),
        template_vertex_count: u32::try_from(raw.template_vertices().len()).unwrap_or(u32::MAX),
        uv_vertex_count: u32::try_from(raw.uv_coordinates().len()).unwrap_or(u32::MAX),
        semantic_count: u32::try_from(raw.semantics().len()).unwrap_or(u32::MAX),
        material_count: u32::try_from(raw.materials().len()).unwrap_or(u32::MAX),
        texture_count: u32::try_from(raw.textures().len()).unwrap_or(u32::MAX),
        symbol_count,
        has_metadata: model.metadata().is_some(),
        has_transform: model.transform().is_some(),
    }
}
