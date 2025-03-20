use crate::cityjson::shared::appearance::RGB;
use crate::resources::storage::StringStorage;

/// Defines the interface for material objects in CityJSON.
///
/// This trait provides methods for accessing and manipulating material properties,
/// including name, colors, intensity values, and surface characteristics. Materials
/// are used to define the visual appearance of geometry surfaces in 3D city models.
///
/// # Type Parameters
///
/// * `SS`: String storage type used for the material name and other string properties
///
/// # Examples
///
/// ```rust
/// use cityjson::cityjson::appearance::{MaterialTrait, RGB};
/// use cityjson::resources::storage::{StringStorage, OwnedStringStorage};
///
/// // Define a material implementation
/// struct MyMaterial<SS: StringStorage> {
///     name: SS::String,
///     ambient_intensity: Option<f32>,
///     diffuse_color: Option<RGB>,
///     emissive_color: Option<RGB>,
///     specular_color: Option<RGB>,
///     shininess: Option<f32>,
///     transparency: Option<f32>,
///     is_smooth: Option<bool>,
/// }
///
/// // impl<SS: StringStorage> MaterialTrait<SS> for MyMaterial<SS> {
/// //    // Implementation of the trait methods
/// //    // ...
/// // }
///
/// // Create a new material with OwnedStringStorage
/// let material_name = "BuildingFacade".to_string();
/// let mut material = MyMaterial::<OwnedStringStorage> {
///     name: material_name,
///     ambient_intensity: Some(0.5),
///     diffuse_color: Some([0.8, 0.8, 0.8]),
///     emissive_color: None,
///     specular_color: Some([1.0, 1.0, 1.0]),
///     shininess: Some(0.2),
///     transparency: Some(0.0),
///     is_smooth: Some(true),
/// };
/// ```
pub trait MaterialTrait<SS: StringStorage> {
    /// Creates a new material with the given name.
    ///
    /// # Parameters
    ///
    /// * `name` - The name identifier for the material
    ///
    /// # Returns
    ///
    /// A new material object with the specified name
    fn new(name: SS::String) -> Self;

    /// Returns a reference to the material name.
    ///
    /// # Returns
    ///
    /// A reference to the material name
    fn name(&self) -> &SS::String;

    /// Sets the material name.
    ///
    /// # Parameters
    ///
    /// * `name` - The new name for the material
    fn set_name(&mut self, name: SS::String);

    /// Returns the ambient intensity if it exists.
    ///
    /// Ambient intensity affects how much ambient light the material reflects.
    /// Values typically range from 0.0 to 1.0.
    ///
    /// # Returns
    ///
    /// An `Option` containing the ambient intensity value,
    /// or `None` if not specified
    fn ambient_intensity(&self) -> Option<f32>;

    /// Sets the ambient intensity.
    ///
    /// # Parameters
    ///
    /// * `ambient_intensity` - The new ambient intensity value or `None` to unset
    fn set_ambient_intensity(&mut self, ambient_intensity: Option<f32>);

    /// Returns a reference to the diffuse color if it exists.
    ///
    /// Diffuse color represents the main color of the material under direct light.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the RGB diffuse color,
    /// or `None` if not specified
    fn diffuse_color(&self) -> Option<&RGB>;

    /// Sets the diffuse color.
    ///
    /// # Parameters
    ///
    /// * `diffuse_color` - The new diffuse color or `None` to unset
    fn set_diffuse_color(&mut self, diffuse_color: Option<RGB>);

    /// Returns a reference to the emissive color if it exists.
    ///
    /// Emissive color represents light emitted by the material itself.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the RGB emissive color,
    /// or `None` if not specified
    fn emissive_color(&self) -> Option<&RGB>;

    /// Sets the emissive color.
    ///
    /// # Parameters
    ///
    /// * `emissive_color` - The new emissive color or `None` to unset
    fn set_emissive_color(&mut self, emissive_color: Option<RGB>);

    /// Returns a reference to the specular color if it exists.
    ///
    /// Specular color represents the color of highlights on the material.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the RGB specular color,
    /// or `None` if not specified
    fn specular_color(&self) -> Option<&RGB>;

    /// Sets the specular color.
    ///
    /// # Parameters
    ///
    /// * `specular_color` - The new specular color or `None` to unset
    fn set_specular_color(&mut self, specular_color: Option<RGB>);

    /// Returns the shininess value if it exists.
    ///
    /// Shininess controls how focused the specular highlight is.
    /// Higher values create sharper highlights.
    ///
    /// # Returns
    ///
    /// An `Option` containing the shininess value,
    /// or `None` if not specified
    fn shininess(&self) -> Option<f32>;

    /// Sets the shininess value.
    ///
    /// # Parameters
    ///
    /// * `shininess` - The new shininess value or `None` to unset
    fn set_shininess(&mut self, shininess: Option<f32>);

    /// Returns the transparency value if it exists.
    ///
    /// Transparency controls how see-through the material is.
    /// Values typically range from 0.0 (opaque) to 1.0 (transparent).
    ///
    /// # Returns
    ///
    /// An `Option` containing the transparency value,
    /// or `None` if not specified
    fn transparency(&self) -> Option<f32>;

    /// Sets the transparency value.
    ///
    /// # Parameters
    ///
    /// * `transparency` - The new transparency value or `None` to unset
    fn set_transparency(&mut self, transparency: Option<f32>);

    /// Returns whether the material is smooth if specified.
    ///
    /// Smooth materials have interpolated normals across surfaces,
    /// while non-smooth materials have a faceted appearance.
    ///
    /// # Returns
    ///
    /// An `Option` containing a boolean indicating if the material is smooth,
    /// or `None` if not specified
    fn is_smooth(&self) -> Option<bool>;

    /// Sets whether the material is smooth.
    ///
    /// # Parameters
    ///
    /// * `is_smooth` - Whether the material should be smooth or `None` to unset
    fn set_is_smooth(&mut self, is_smooth: Option<bool>);
}
