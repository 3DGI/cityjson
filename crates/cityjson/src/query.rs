use crate::relational::RelationalAccess;
use crate::v2_0::OwnedCityModel;

/// Cheap scalar summary over an owned city model.
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

    ModelSummary {
        cityobject_count: u32::try_from(relational.cityobjects().len()).unwrap_or(u32::MAX),
        geometry_count: u32::try_from(relational.geometries().len()).unwrap_or(u32::MAX),
        geometry_template_count: u32::try_from(relational.geometry_templates().len())
            .unwrap_or(u32::MAX),
        vertex_count: u32::try_from(relational.vertices().len()).unwrap_or(u32::MAX),
        template_vertex_count: u32::try_from(relational.template_vertices().len())
            .unwrap_or(u32::MAX),
        uv_vertex_count: u32::try_from(relational.uv_vertices().len()).unwrap_or(u32::MAX),
        semantic_count: u32::try_from(relational.semantics().len()).unwrap_or(u32::MAX),
        material_count: u32::try_from(relational.materials().len()).unwrap_or(u32::MAX),
        texture_count: u32::try_from(relational.textures().len()).unwrap_or(u32::MAX),
        symbol_count: u32::try_from(relational.symbols().len()).unwrap_or(u32::MAX),
        has_metadata: relational.metadata().is_some(),
        has_transform: relational.transform().is_some(),
    }
}
