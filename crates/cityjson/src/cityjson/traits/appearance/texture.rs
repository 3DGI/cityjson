use crate::cityjson::core::appearance::{ImageType, RGBA, TextureType, WrapMode};
use crate::resources::storage::StringStorage;

/// Defines the interface for texture objects in CityJSON.
///
/// This trait provides methods for accessing and manipulating texture properties,
/// including image source, type, wrap mode, and other visual characteristics. Textures
/// are used to define detailed surface appearances that can be mapped onto geometry in 3D city models.
///
/// # Type Parameters
///
/// * `SS`: String storage type used for the image path and other string properties
///
/// # Examples
///
/// ```rust
/// use cityjson::cityjson::core::appearance::{ImageType, TextureType, WrapMode, RGBA};
/// use cityjson::cityjson::traits::appearance::TextureTrait;
/// use cityjson::resources::storage::{StringStorage, OwnedStringStorage};
///
/// // Define a texture implementation
/// struct MyTexture<SS: StringStorage> {
///     image: SS::String,
///     image_type: ImageType,
///     wrap_mode: Option<WrapMode>,
///     texture_type: Option<TextureType>,
///     border_color: Option<RGBA>,
/// }
///
/// // impl<SS: StringStorage> TextureTrait<SS> for MyTexture<SS> {
/// //    // Implementation of the trait methods
/// //    // ...
/// // }
///
/// // Create a new texture with OwnedStringStorage
/// let image_path = "textures/facade.jpg".to_string();
/// let mut texture = MyTexture::<OwnedStringStorage> {
///     image: image_path,
///     image_type: ImageType::Jpg,
///     wrap_mode: Some(WrapMode::Wrap),
///     texture_type: Some(TextureType::Specific),
///     border_color: Some([0.0, 0.0, 0.0, 1.0]),
/// };
/// ```
pub trait TextureTrait<SS: StringStorage>: PartialEq {
    /// Creates a new texture with the given image source and type.
    ///
    /// # Parameters
    ///
    /// * `image` - The path or URI to the texture image
    /// * `image_type` - The file format of the image
    ///
    /// # Returns
    ///
    /// A new texture object with the specified image source and type
    fn new(image: SS::String, image_type: ImageType) -> Self;

    /// Returns a reference to the image type.
    ///
    /// The image type identifies the file format of the texture image.
    ///
    /// # Returns
    ///
    /// A reference to the image type enumeration value
    fn image_type(&self) -> &ImageType;

    /// Sets the image type.
    ///
    /// # Parameters
    ///
    /// * `image_type` - The new file format for the texture image
    fn set_image_type(&mut self, image_type: ImageType);

    /// Returns a reference to the image path or URI.
    ///
    /// # Returns
    ///
    /// A reference to the image path or URI
    fn image(&self) -> &SS::String;

    /// Sets the image path or URI.
    ///
    /// # Parameters
    ///
    /// * `image` - The new path or URI for the texture image
    fn set_image(&mut self, image: SS::String);

    /// Returns the wrap mode if it exists.
    ///
    /// Wrap mode determines how texture coordinates outside the (0,1) range are handled.
    ///
    /// # Returns
    ///
    /// An `Option` containing the wrap mode,
    /// or `None` if not specified
    fn wrap_mode(&self) -> Option<WrapMode>;

    /// Sets the wrap mode.
    ///
    /// # Parameters
    ///
    /// * `wrap_mode` - The new wrap mode or `None` to unset
    fn set_wrap_mode(&mut self, wrap_mode: Option<WrapMode>);

    /// Returns the texture type if it exists.
    ///
    /// Texture type indicates how the texture should be applied to surfaces.
    ///
    /// # Returns
    ///
    /// An `Option` containing the texture type,
    /// or `None` if not specified
    fn texture_type(&self) -> Option<TextureType>;

    /// Sets the texture type.
    ///
    /// # Parameters
    ///
    /// * `texture_type` - The new texture type or `None` to unset
    fn set_texture_type(&mut self, texture_type: Option<TextureType>);

    /// Returns the border color if it exists.
    ///
    /// Border color is used for texture coordinates outside the texture image
    /// when the wrap mode is set to Border.
    ///
    /// # Returns
    ///
    /// An `Option` containing the RGBA border color,
    /// or `None` if not specified
    fn border_color(&self) -> Option<RGBA>;

    /// Sets the border color.
    ///
    /// # Parameters
    ///
    /// * `border_color` - The new RGBA border color or `None` to unset
    fn set_border_color(&mut self, border_color: Option<RGBA>);
}
