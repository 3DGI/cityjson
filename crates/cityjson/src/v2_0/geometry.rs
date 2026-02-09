use crate::cityjson::core::geometry_struct::GeometryCore;
use crate::cityjson::core::vertex::VertexRef;
use crate::resources::handles::TemplateGeometryRef;
use crate::resources::mapping::{MaterialMap, SemanticMap, TextureMap};
use crate::resources::pool::ResourceId32;
use crate::resources::storage::StringStorage;
use crate::v2_0::types::ThemeName;

pub mod semantic;

#[derive(Clone, Debug)]
pub struct Geometry<VR: VertexRef, SS: StringStorage> {
    inner: GeometryCore<VR, ResourceId32, SS>,
}

impl<VR: VertexRef, SS: StringStorage> Geometry<VR, SS> {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        type_geometry: crate::cityjson::core::geometry::GeometryType,
        lod: Option<crate::cityjson::core::geometry::LoD>,
        boundaries: Option<crate::cityjson::core::boundary::Boundary<VR>>,
        semantics: Option<SemanticMap<VR>>,
        materials: Option<Vec<(ThemeName<SS>, MaterialMap<VR>)>>,
        textures: Option<Vec<(ThemeName<SS>, TextureMap<VR>)>>,
        instance_template: Option<TemplateGeometryRef>,
        instance_reference_point: Option<crate::cityjson::core::vertex::VertexIndex<VR>>,
        instance_transformation_matrix: Option<[f64; 16]>,
    ) -> Self {
        Self {
            inner: GeometryCore::new(
                type_geometry,
                lod,
                boundaries,
                semantics.map(|m| m.to_raw().clone()),
                materials.map(|items| {
                    items
                        .into_iter()
                        .map(|(theme, map)| (theme.into_inner(), map.to_raw().clone()))
                        .collect()
                }),
                textures.map(|items| {
                    items
                        .into_iter()
                        .map(|(theme, map)| (theme.into_inner(), map.to_raw().clone()))
                        .collect()
                }),
                instance_template
                    .map(super::super::resources::handles::TemplateGeometryRef::to_raw),
                instance_reference_point,
                instance_transformation_matrix,
            ),
        }
    }

    pub fn type_geometry(&self) -> &crate::cityjson::core::geometry::GeometryType {
        self.inner.type_geometry()
    }

    pub fn lod(&self) -> Option<&crate::cityjson::core::geometry::LoD> {
        self.inner.lod()
    }

    pub fn boundaries(&self) -> Option<&crate::cityjson::core::boundary::Boundary<VR>> {
        self.inner.boundaries()
    }

    pub fn semantics(&self) -> Option<SemanticMap<VR>> {
        self.inner
            .semantics()
            .map(|s| SemanticMap::from_raw(s.clone()))
    }

    pub fn materials(&self) -> Option<Vec<(ThemeName<SS>, MaterialMap<VR>)>>
    where
        SS::String: Clone,
    {
        self.inner.materials().map(|items| {
            items
                .iter()
                .map(|(theme, map)| {
                    (
                        ThemeName::new(theme.clone()),
                        MaterialMap::from_raw(map.clone()),
                    )
                })
                .collect()
        })
    }

    pub fn textures(&self) -> Option<Vec<(ThemeName<SS>, TextureMap<VR>)>>
    where
        SS::String: Clone,
    {
        self.inner.textures().map(|items| {
            items
                .iter()
                .map(|(theme, map)| {
                    (
                        ThemeName::new(theme.clone()),
                        TextureMap::from_raw(map.clone()),
                    )
                })
                .collect()
        })
    }

    pub fn instance_template(&self) -> Option<TemplateGeometryRef> {
        self.inner
            .instance_template()
            .copied()
            .map(TemplateGeometryRef::from_raw)
    }

    pub fn instance_reference_point(
        &self,
    ) -> Option<&crate::cityjson::core::vertex::VertexIndex<VR>> {
        self.inner.instance_reference_point()
    }

    pub fn instance_transformation_matrix(&self) -> Option<&[f64; 16]> {
        self.inner.instance_transformation_matrix()
    }
}

impl<VR: VertexRef, SS: StringStorage>
    crate::backend::default::geometry::GeometryConstructor<VR, ResourceId32, SS::String>
    for Geometry<VR, SS>
{
    #[allow(clippy::too_many_arguments)]
    fn new(
        type_geometry: crate::cityjson::core::geometry::GeometryType,
        lod: Option<crate::cityjson::core::geometry::LoD>,
        boundaries: Option<crate::cityjson::core::boundary::Boundary<VR>>,
        semantics: Option<crate::resources::mapping::SemanticOrMaterialMap<VR, ResourceId32>>,
        materials: Option<
            Vec<(
                SS::String,
                crate::resources::mapping::SemanticOrMaterialMap<VR, ResourceId32>,
            )>,
        >,
        textures: Option<
            Vec<(
                SS::String,
                crate::resources::mapping::textures::TextureMapCore<VR, ResourceId32>,
            )>,
        >,
        instance_template: Option<ResourceId32>,
        instance_reference_point: Option<crate::cityjson::core::vertex::VertexIndex<VR>>,
        instance_transformation_matrix: Option<[f64; 16]>,
    ) -> Self {
        Self {
            inner: GeometryCore::new(
                type_geometry,
                lod,
                boundaries,
                semantics,
                materials,
                textures,
                instance_template,
                instance_reference_point,
                instance_transformation_matrix,
            ),
        }
    }
}
