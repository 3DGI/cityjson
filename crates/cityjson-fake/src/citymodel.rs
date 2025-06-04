use cityjson::prelude::{Attributes, CityModelTrait, ResourceRef, StringStorage, VertexRef};
use cityjson::v2_0::CityModel;
use rand::prelude::SmallRng;
use rand::{thread_rng, SeedableRng};
use cityjson::CityModelType;
use crate::attribute::AttributesBuilder;
use crate::cli::CJFakeConfig;
use crate::material::MaterialBuilder;
use crate::metadata::MetadataBuilder;
use crate::prelude::{TextureBuilder, VerticesBuilder};

/// Builder for creating CityJSON models with fake data.
///
/// The builder provides methods to configure and generate different aspects of a CityJSON model,
/// such as vertices, cityobjects, materials, textures, etc. The generated data is valid according
/// to the CityJSON specification, though the geometric values are random.
///
/// # Examples
///
/// ```rust
/// use cjfake::prelude::*;
///
/// // Create a basic CityJSON model with default settings
/// let model: CityModel<u32, ResourceId32, OwnedStringStorage> = CityModelBuilder::default().build();
///
/// // Create a customized model
/// let config = CJFakeConfig::default();
/// let model: CityModel<u32, ResourceId32, OwnedStringStorage> = CityModelBuilder::new(config, None)
///     .metadata(None)
///     .vertices(None)
///     .materials(None)
///     .textures(None)
///     .attributes(None)
///     .cityobjects()
///     .build();
/// ```
pub struct CityModelBuilder<VR: VertexRef, RR: ResourceRef, SS: StringStorage> {
    model: CityModel<VR, RR, SS>,
    rng: SmallRng,
    config: CJFakeConfig,
    themes_material: Vec<String>,
    themes_texture: Vec<String>,
    attributes_cityobject: Option<Attributes<SS, RR>>,
    attributes_semantic: Option<Attributes<SS, RR>>,
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> From<CityModelBuilder<VR, RR, SS>>
    for CityModel<VR, RR, SS>
{
    fn from(val: CityModelBuilder<VR, RR, SS>) -> Self {
        val.build()
    }
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> Default for CityModelBuilder<VR, RR, SS> {
    fn default() -> Self {
        CityModelBuilder::new(CJFakeConfig::default(), None)
            .metadata(None)
            .vertices(None)
            .materials(None)
            .textures(None)
            .attributes(None)
            .cityobjects()
    }
}

impl<VR: VertexRef, RR: ResourceRef, SS: StringStorage> CityModelBuilder<VR, RR, SS> {
    /// Creates a new CityModelBuilder with the given configuration and optional random seed.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration options for generating fake data
    /// * `seed` - Optional seed for random number generation. If None, uses thread RNG
    ///
    /// # Returns
    ///
    /// A new CityModelBuilder instance
    #[must_use]
    pub fn new(config: CJFakeConfig, seed: Option<u64>) -> Self {
        let rng = if let Some(state) = seed {
            SmallRng::seed_from_u64(state)
        } else {
            SmallRng::from_rng(thread_rng()).expect("SmallRng should be seeded from thread_rng()")
        };
        Self {
            model: CityModel::new(CityModelType::CityJSON),
            rng,
            config,
            themes_material: Vec::new(),
            themes_texture: Vec::new(),
            attributes_cityobject: None,
            attributes_semantic: None,
        }
    }

    /// Generates attributes for both CityObjects and semantic surfaces.
    ///
    /// Creates random but valid attribute values for use in CityObjects and semantic surface elements.
    ///
    /// # Returns
    ///
    /// Self with attributes added
    pub fn attributes(mut self, attributes_builder: Option<AttributesBuilder>) -> Self {
        todo!()
    }

    /// Generates CityObjects for the model.
    ///
    /// This method will:
    /// - Generate 1 CityObject if `nr_cityobjects` in config is None
    /// - Otherwise generate the number of CityObjects within the provided range
    /// - If `cityobject_hierarchy` is true and generating one object, an additional 2nd-level object may be created
    /// - If generating multiple objects with hierarchy, the total number of 1st and 2nd level objects will be in range
    /// - Automatically generates vertices if not already present
    ///
    /// # Returns
    ///
    /// Self with CityObjects added
    pub fn cityobjects(mut self) -> Self {
        todo!()
    }

    /// Adds materials to the model using an optional MaterialBuilder.
    ///
    /// If no builder is provided, generates default materials. The number of materials generated
    /// is controlled by the configuration.
    ///
    /// # Arguments
    ///
    /// * `material_builder` - Optional MaterialBuilder to customize material generation
    ///
    /// # Returns
    ///
    /// Self with materials added
    pub fn materials(mut self, material_builder: Option<MaterialBuilder>) -> Self {
        todo!()
    }

    /// Adds metadata to the model using an optional MetadataBuilder.
    ///
    /// If no builder is provided, generates default metadata with fake values for all fields.
    ///
    /// # Arguments
    ///
    /// * `metadata_builder` - Optional MetadataBuilder to customize metadata generation
    ///
    /// # Returns
    ///
    /// Self with metadata added
    pub fn metadata(mut self, metadata_builder: Option<MetadataBuilder>) -> Self {
        todo!()
    }

    /// Adds textures to the model using an optional TextureBuilder.
    ///
    /// If no builder is provided, generates default textures. The number of textures and vertices
    /// is controlled by the configuration.
    ///
    /// # Arguments
    ///
    /// * `texture_builder` - Optional TextureBuilder to customize texture generation
    ///
    /// # Returns
    ///
    /// Self with textures added
    pub fn textures(mut self, texture_builder: Option<TextureBuilder>) -> Self {
        todo!()
    }

    /// Generates vertices for the model if not already present.
    ///
    /// The number and range of vertex coordinates is controlled by the configuration.
    ///
    /// # Returns
    ///
    /// Self with vertices added
    pub fn vertices(mut self, vertices_builder: Option<VerticesBuilder>) -> Self {
        todo!()
    }

    /// Builds the final CityJSON model.
    ///
    /// Handles any unused vertices by either:
    /// - Appending them to an existing geometry if possible
    /// - Removing them if only GeometryInstance objects exist
    /// - Removing vertices entirely if no geometries were generated
    ///
    /// # Returns
    ///
    /// The complete CityJSON model
    pub fn build(mut self) -> CityModel<VR, RR, SS> {
        todo!()
    }

    /// Builds the model and converts it to a JSON string.
    ///
    /// # Returns
    ///
    /// Result containing the JSON string or a serialization error
    #[allow(dead_code)]
    pub fn build_string(self) -> serde_json::Result<String> {
        // serde_json::to_string::<CityModel>(&self.into())
        todo!()
    }

    /// Builds the model and converts it to a byte vector.
    ///
    /// # Returns
    ///
    /// Result containing the byte vector or a serialization error
    #[allow(dead_code)]
    pub fn build_vec(self) -> serde_json::Result<Vec<u8>> {
        // serde_json::to_vec::<CityModel>(&self.into())
        todo!()
    }
}