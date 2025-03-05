use crate::prelude::StringStorage;

pub trait ExtensionsTrait<SS: StringStorage, Ext: ExtensionTrait<SS>> {
    /// Create a new empty Extensions collection.
    fn new() -> Self;
    /// Adds an extension to the collection.
    ///
    /// If an extension with the same name already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `extension` - The extension to add
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    fn add(&mut self, extension: Ext) -> &mut Self;
    /// Removes an extension by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the extension to remove
    ///
    /// # Returns
    ///
    /// `true` if the extension was found and removed, `false` otherwise
    fn remove(&mut self, name: SS::String) -> bool;
    /// Gets an extension by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the extension to retrieve
    ///
    /// # Returns
    ///
    /// Some reference to the extension if found, None otherwise
    fn get(&self, name: &str) -> Option<&Ext>;
    /// Returns the number of extensions in the collection.
    fn len(&self) -> usize;
    /// Returns true if the collection contains no extensions.
    fn is_empty(&self) -> bool;
}

pub trait ExtensionTrait<SS: StringStorage> {
    /// Creates a new extension with the specified name, URL, and version.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique identifier for this extension
    /// * `url` - URL where the extension schema can be found
    /// * `version` - Version identifier of the extension
    fn new(name: SS::String, url: SS::String, version: SS::String) -> Self;
    /// Returns a reference to the extension name.
    fn name(&self) -> &SS::String;
    /// Returns a reference to the extension schema URL.
    fn url(&self) -> &SS::String;
    /// Returns a reference to the extension version.
    fn version(&self) -> &SS::String;
}
