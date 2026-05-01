//! Zero-copy view types for raw data access.

/// A type-safe view over a contiguous slice.
#[derive(Debug, Clone, Copy)]
pub struct RawSliceView<'a, T> {
    data: &'a [T],
}

impl<'a, T> RawSliceView<'a, T> {
    #[inline]
    pub fn new(data: &'a [T]) -> Self {
        Self { data }
    }

    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &'a [T] {
        self.data
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&'a T> {
        self.data.get(index)
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'a, T> {
        self.into_iter()
    }
}

impl<'a, T> IntoIterator for RawSliceView<'a, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a, T> IntoIterator for &'_ RawSliceView<'a, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

/// A raw view over a resource pool.
///
/// Includes free slots (`None`) and generation counters.
#[derive(Debug, Clone, Copy)]
pub struct RawPoolView<'a, T> {
    resources: &'a [Option<T>],
    generations: &'a [u16],
}

impl<'a, T> RawPoolView<'a, T> {
    #[inline]
    pub fn new(resources: &'a [Option<T>], generations: &'a [u16]) -> Self {
        Self {
            resources,
            generations,
        }
    }

    #[inline]
    #[must_use]
    pub fn resources(&self) -> &'a [Option<T>] {
        self.resources
    }

    #[inline]
    #[must_use]
    /// Generation counters for each slot.
    ///
    /// This is a low-level view intended for serializers and diagnostics.
    pub fn generations(&self) -> &'a [u16] {
        self.generations
    }

    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.resources.len()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.resources.iter().filter(|r| r.is_some()).count()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter_occupied(&self) -> impl Iterator<Item = (usize, &'a T)> {
        self.resources
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|value| (i, value)))
    }

    #[must_use]
    pub fn dense_index_remap(&self) -> DenseIndexRemap {
        DenseIndexRemap::from_occupied_indices(
            self.capacity(),
            self.iter_occupied().map(|(i, _)| i),
        )
    }
}

/// Remaps sparse stored slot indices to dense export indices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenseIndexRemap {
    dense_by_stored_index: Vec<Option<usize>>,
    dense_len: usize,
}

impl DenseIndexRemap {
    #[must_use]
    pub fn identity(len: usize) -> Self {
        Self {
            dense_by_stored_index: (0..len).map(Some).collect(),
            dense_len: len,
        }
    }

    #[must_use]
    pub fn from_occupied_indices(
        capacity: usize,
        occupied_indices: impl IntoIterator<Item = usize>,
    ) -> Self {
        let mut dense_by_stored_index = vec![None; capacity];
        let mut dense_len = 0;

        for stored_index in occupied_indices {
            if let Some(slot) = dense_by_stored_index.get_mut(stored_index) {
                *slot = Some(dense_len);
                dense_len += 1;
            }
        }

        Self {
            dense_by_stored_index,
            dense_len,
        }
    }

    #[must_use]
    pub fn get(&self, stored_index: usize) -> Option<usize> {
        self.dense_by_stored_index
            .get(stored_index)
            .copied()
            .flatten()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.dense_len
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.dense_len == 0
    }

    #[must_use]
    pub fn as_slice(&self) -> &[Option<usize>] {
        &self.dense_by_stored_index
    }
}

/// Columnar view of quantized coordinates.
#[derive(Debug, Clone, Copy)]
pub struct ColumnarCoordinates<'a> {
    pub x: &'a [i64],
    pub y: &'a [i64],
    pub z: &'a [i64],
}

/// Columnar view of real-world coordinates.
#[derive(Debug, Clone, Copy)]
pub struct ColumnarRealCoordinates<'a> {
    pub x: &'a [f64],
    pub y: &'a [f64],
    pub z: &'a [f64],
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CityModelType;
    use crate::v2_0::appearance::ImageType;
    use crate::v2_0::geometry::semantic::SemanticType;
    use crate::v2_0::{
        GeometryDraft, Material, OwnedCityModel, RingDraft, Semantic, SurfaceDraft, Texture,
    };

    #[test]
    fn dense_index_remap_compacts_sparse_slots() {
        let remap = DenseIndexRemap::from_occupied_indices(5, [1, 3, 4]);
        assert_eq!(remap.as_slice(), &[None, Some(0), None, Some(1), Some(2)]);
        assert_eq!(remap.len(), 3);
    }

    #[test]
    fn raw_export_remapping_is_stable_for_geometry_resources_and_uvs() {
        let mut model = OwnedCityModel::new(CityModelType::CityJSON);
        let stale_semantic = model
            .add_semantic(Semantic::new(SemanticType::RoofSurface))
            .unwrap();
        let stale_material = model
            .add_material(Material::new("stale".to_string()))
            .unwrap();
        let stale_texture = model
            .add_texture(Texture::new("stale.png".to_string(), ImageType::Png))
            .unwrap();

        let live_semantic = model
            .add_semantic(Semantic::new(SemanticType::WallSurface))
            .unwrap();
        let live_material = model
            .add_material(Material::new("live".to_string()))
            .unwrap();
        let live_texture = model
            .add_texture(Texture::new("live.png".to_string(), ImageType::Png))
            .unwrap();

        let geometry_handle = GeometryDraft::multi_surface(
            None,
            [SurfaceDraft::new(
                RingDraft::new([[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]).with_texture(
                    "theme".to_string(),
                    live_texture,
                    [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
                ),
                [],
            )
            .with_semantic(live_semantic)
            .with_material("theme".to_string(), live_material)],
        )
        .insert_into(&mut model)
        .unwrap();
        assert!(model.remove_semantic(stale_semantic).is_some());
        assert!(model.remove_material(stale_material).is_some());
        assert!(model.remove_texture(stale_texture).is_some());

        let raw = model.raw();
        let semantic_remap = raw.semantics().dense_index_remap();
        let material_remap = raw.materials().dense_index_remap();
        let texture_remap = raw.textures().dense_index_remap();
        let uv_remap = DenseIndexRemap::identity(raw.uv_coordinates().len());

        assert_eq!(
            semantic_remap.as_slice(),
            raw.semantics().dense_index_remap().as_slice()
        );
        assert_eq!(
            material_remap.as_slice(),
            raw.materials().dense_index_remap().as_slice()
        );
        assert_eq!(
            texture_remap.as_slice(),
            raw.textures().dense_index_remap().as_slice()
        );
        assert_eq!(
            uv_remap.as_slice(),
            DenseIndexRemap::identity(raw.uv_coordinates().len()).as_slice()
        );

        let geometry = model.get_geometry(geometry_handle).unwrap();
        let semantic_handle = geometry.semantics().unwrap().surfaces()[0].unwrap();
        let (_material_theme, material_map) = geometry.materials().unwrap().first().unwrap();
        let (_texture_theme, texture_map) = geometry.textures().unwrap().first().unwrap();

        assert_eq!(
            semantic_remap.get(semantic_handle.index() as usize),
            Some(0)
        );
        assert_eq!(
            material_remap.get(material_map.surfaces()[0].unwrap().index() as usize),
            Some(0)
        );
        assert_eq!(
            texture_remap.get(texture_map.ring_textures()[0].unwrap().index() as usize),
            Some(0)
        );

        let remapped_uvs: Vec<_> = texture_map
            .vertices()
            .iter()
            .map(|uv_ref| uv_remap.get(uv_ref.unwrap().to_usize()).unwrap())
            .collect();
        assert_eq!(remapped_uvs, vec![0, 1, 2]);
    }
}
