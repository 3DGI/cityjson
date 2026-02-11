use crate::cityjson::core::geometry_struct::GeometryCore;
use crate::cityjson::core::vertex::{VertexIndex, VertexRef};
use crate::resources::handles::{MaterialRef, SemanticRef, TemplateGeometryRef, TextureRef};
use crate::resources::mapping::textures::TextureMapCore;
use crate::resources::mapping::{MaterialMap, SemanticMap, SemanticOrMaterialMap, TextureMap};
use crate::resources::pool::ResourceId32;
use crate::resources::storage::StringStorage;
use crate::v2_0::types::ThemeName;
use std::marker::PhantomData;
use std::ops::Index;

pub mod semantic;

#[derive(Clone, Debug)]
pub struct Geometry<VR: VertexRef, SS: StringStorage> {
    inner: GeometryCore<VR, ResourceId32, SS>,
}

#[derive(Clone, Copy, Debug)]
pub struct HandleOptionSlice<'a, H> {
    raw: &'a [Option<ResourceId32>],
    _marker: PhantomData<H>,
}

impl<'a, H> HandleOptionSlice<'a, H> {
    fn new(raw: &'a [Option<ResourceId32>]) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    #[inline]
    fn as_handle_slice(&self) -> &'a [Option<H>] {
        const {
            assert!(std::mem::size_of::<Option<H>>() == std::mem::size_of::<Option<ResourceId32>>());
            assert!(std::mem::align_of::<Option<H>>() == std::mem::align_of::<Option<ResourceId32>>());
        }

        // SAFETY: handle types are `#[repr(transparent)]` wrappers over `ResourceId32`.
        // Therefore `Option<Handle>` and `Option<ResourceId32>` have identical layout.
        unsafe { std::slice::from_raw_parts(self.raw.as_ptr().cast::<Option<H>>(), self.raw.len()) }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&'a Option<H>> {
        self.as_handle_slice().get(index)
    }

    pub fn iter(&self) -> std::slice::Iter<'a, Option<H>> {
        self.as_handle_slice().iter()
    }
}

impl<'a, H> Index<usize> for HandleOptionSlice<'a, H> {
    type Output = Option<H>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_handle_slice()[index]
    }
}

impl<'a, H: 'a> IntoIterator for HandleOptionSlice<'a, H> {
    type Item = &'a Option<H>;
    type IntoIter = std::slice::Iter<'a, Option<H>>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_handle_slice().iter()
    }
}

impl<'a, H: 'a> IntoIterator for &'_ HandleOptionSlice<'a, H> {
    type Item = &'a Option<H>;
    type IntoIter = std::slice::Iter<'a, Option<H>>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_handle_slice().iter()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SemanticMapView<'a, VR: VertexRef> {
    inner: &'a SemanticOrMaterialMap<VR, ResourceId32>,
}

impl<'a, VR: VertexRef> SemanticMapView<'a, VR> {
    #[must_use]
    pub fn points(&self) -> HandleOptionSlice<'a, SemanticRef> {
        HandleOptionSlice::new(self.inner.points())
    }

    #[must_use]
    pub fn linestrings(&self) -> HandleOptionSlice<'a, SemanticRef> {
        HandleOptionSlice::new(self.inner.linestrings())
    }

    #[must_use]
    pub fn surfaces(&self) -> HandleOptionSlice<'a, SemanticRef> {
        HandleOptionSlice::new(self.inner.surfaces())
    }

    #[must_use]
    pub fn shells(&self) -> &'a [VertexIndex<VR>] {
        self.inner.shells()
    }

    #[must_use]
    pub fn solids(&self) -> &'a [VertexIndex<VR>] {
        self.inner.solids()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MaterialMapView<'a, VR: VertexRef> {
    inner: &'a SemanticOrMaterialMap<VR, ResourceId32>,
}

impl<'a, VR: VertexRef> MaterialMapView<'a, VR> {
    #[must_use]
    pub fn points(&self) -> HandleOptionSlice<'a, MaterialRef> {
        HandleOptionSlice::new(self.inner.points())
    }

    #[must_use]
    pub fn linestrings(&self) -> HandleOptionSlice<'a, MaterialRef> {
        HandleOptionSlice::new(self.inner.linestrings())
    }

    #[must_use]
    pub fn surfaces(&self) -> HandleOptionSlice<'a, MaterialRef> {
        HandleOptionSlice::new(self.inner.surfaces())
    }

    #[must_use]
    pub fn shells(&self) -> &'a [VertexIndex<VR>] {
        self.inner.shells()
    }

    #[must_use]
    pub fn solids(&self) -> &'a [VertexIndex<VR>] {
        self.inner.solids()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MaterialThemesView<'a, VR: VertexRef, SS: StringStorage> {
    items: &'a [(SS::String, SemanticOrMaterialMap<VR, ResourceId32>)],
}

impl<'a, VR: VertexRef, SS: StringStorage> MaterialThemesView<'a, VR, SS> {
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'a SS::String, MaterialMapView<'a, VR>)> + 'a {
        self.items
            .iter()
            .map(|(theme, map)| (theme, MaterialMapView { inner: map }))
    }

    #[must_use]
    pub fn first(&self) -> Option<(&'a SS::String, MaterialMapView<'a, VR>)> {
        self.items
            .first()
            .map(|(theme, map)| (theme, MaterialMapView { inner: map }))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TextureMapView<'a, VR: VertexRef> {
    inner: &'a TextureMapCore<VR, ResourceId32>,
}

impl<'a, VR: VertexRef> TextureMapView<'a, VR> {
    #[must_use]
    pub fn vertices(&self) -> &'a [Option<VertexIndex<VR>>] {
        self.inner.vertices()
    }

    #[must_use]
    pub fn rings(&self) -> &'a [VertexIndex<VR>] {
        self.inner.rings()
    }

    #[must_use]
    pub fn ring_textures(&self) -> HandleOptionSlice<'a, TextureRef> {
        HandleOptionSlice::new(self.inner.ring_textures())
    }

    #[must_use]
    pub fn surfaces(&self) -> &'a [VertexIndex<VR>] {
        self.inner.surfaces()
    }

    #[must_use]
    pub fn shells(&self) -> &'a [VertexIndex<VR>] {
        self.inner.shells()
    }

    #[must_use]
    pub fn solids(&self) -> &'a [VertexIndex<VR>] {
        self.inner.solids()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TextureThemesView<'a, VR: VertexRef, SS: StringStorage> {
    items: &'a [(SS::String, TextureMapCore<VR, ResourceId32>)],
}

impl<'a, VR: VertexRef, SS: StringStorage> TextureThemesView<'a, VR, SS> {
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'a SS::String, TextureMapView<'a, VR>)> + 'a {
        self.items
            .iter()
            .map(|(theme, map)| (theme, TextureMapView { inner: map }))
    }

    #[must_use]
    pub fn first(&self) -> Option<(&'a SS::String, TextureMapView<'a, VR>)> {
        self.items
            .first()
            .map(|(theme, map)| (theme, TextureMapView { inner: map }))
    }
}

impl<VR: VertexRef, SS: StringStorage> Geometry<VR, SS> {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub(crate) fn new(
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

    pub fn semantics(&self) -> Option<SemanticMapView<'_, VR>> {
        self.inner.semantics().map(|inner| SemanticMapView { inner })
    }

    pub fn materials(&self) -> Option<MaterialThemesView<'_, VR, SS>> {
        self.inner
            .materials()
            .map(|items| MaterialThemesView { items })
    }

    pub fn textures(&self) -> Option<TextureThemesView<'_, VR, SS>> {
        self.inner
            .textures()
            .map(|items| TextureThemesView { items })
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
