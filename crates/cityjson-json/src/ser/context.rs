use std::collections::HashMap;

use cityjson_types::resources::handles::{
    CityObjectHandle, GeometryTemplateHandle, MaterialHandle, TextureHandle,
};
use cityjson_types::resources::storage::StringStorage;
use cityjson_types::v2_0::{CityModel, VertexRef};

pub(crate) struct WriteContext {
    pub(crate) id_by_handle: HashMap<CityObjectHandle, String>,
    pub(crate) template_indices: HashMap<GeometryTemplateHandle, usize>,
    pub(crate) material_indices: HashMap<MaterialHandle, usize>,
    pub(crate) texture_indices: HashMap<TextureHandle, usize>,
}

impl WriteContext {
    pub(crate) fn new<VR, SS>(model: &CityModel<VR, SS>) -> Self
    where
        VR: VertexRef,
        SS: StringStorage,
    {
        Self {
            id_by_handle: model
                .cityobjects()
                .iter()
                .map(|(handle, cityobject)| (handle, cityobject.id().to_owned()))
                .collect(),
            template_indices: model
                .iter_geometry_templates()
                .enumerate()
                .map(|(index, (handle, _))| (handle, index))
                .collect(),
            material_indices: model
                .iter_materials()
                .enumerate()
                .map(|(index, (handle, _))| (handle, index))
                .collect(),
            texture_indices: model
                .iter_textures()
                .enumerate()
                .map(|(index, (handle, _))| (handle, index))
                .collect(),
        }
    }
}
