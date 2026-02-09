use crate::prelude::StringStorage;
use std::fmt;
use std::marker::PhantomData;

/// A collection of `CityJSON` Extensions.
///
/// This type provides functionality to manage multiple extensions in a `CityJSON` model.
/// It ensures that extension names are unique (replacing duplicates), and offers methods
/// to add, remove, and query extensions by name.
///
/// # Example
///
/// ```
/// use cityjson::prelude::*;
///
/// // Create a collection of extensions
/// let mut extensions = ExtensionsCore::<OwnedStringStorage, ExtensionCore<OwnedStringStorage>>::new();
///
/// // Add a noise extension to the collection
/// let noise_ext = ExtensionCore::new(
///     "noise".to_string(),
///     "https://example.com/noise-extension/1.0".to_string(),
///     "1.0".to_string()
/// );
/// extensions.add(noise_ext);
///
/// // Retrieve an extension by name
/// let found = extensions.get("noise");
/// assert!(found.is_some());
/// ```
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ExtensionsCore<SS: StringStorage, E> {
    inner: Vec<E>,
    _marker: PhantomData<SS>,
}

// Trait to define the interface for extension items
pub trait ExtensionItem<SS: StringStorage> {
    fn name(&self) -> &SS::String;
}

impl<SS: StringStorage, E: ExtensionItem<SS>> ExtensionsCore<SS, E> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            _marker: Default::default(),
        }
    }

    pub fn add(&mut self, extension: E) -> &mut Self {
        if let Some(pos) = self.inner.iter().position(|e| e.name() == extension.name()) {
            self.inner[pos] = extension;
        } else {
            self.inner.push(extension);
        }
        self
    }

    pub fn remove(&mut self, name: SS::String) -> bool {
        if let Some(pos) = self.inner.iter().position(|e| e.name() == &name) {
            self.inner.remove(pos);
            true
        } else {
            false
        }
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&E> {
        self.inner.iter().find(|e| e.name().as_ref() == name)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

// Allow consuming iteration
impl<SS: StringStorage, E: ExtensionItem<SS>> IntoIterator for ExtensionsCore<SS, E> {
    type Item = E;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

// Allow iterating by reference
impl<'a, SS: StringStorage, E: ExtensionItem<SS>> IntoIterator for &'a ExtensionsCore<SS, E> {
    type Item = &'a E;
    type IntoIter = std::slice::Iter<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

// Allow iterating by mutable reference
impl<'a, SS: StringStorage, E: ExtensionItem<SS>> IntoIterator for &'a mut ExtensionsCore<SS, E> {
    type Item = &'a mut E;
    type IntoIter = std::slice::IterMut<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl<SS: StringStorage, E: ExtensionItem<SS>> fmt::Display for ExtensionsCore<SS, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "available extensions: ")?;
        let mut iter = self.into_iter();
        if let Some(first) = iter.next() {
            write!(f, "{}", first.name())?;
            for ext in iter {
                write!(f, ", {}", ext.name())?;
            }
        }
        Ok(())
    }
}

/// Represents a `CityJSON` extension with a name, URL, and version.
///
/// Extensions in `CityJSON` allow for adding custom objects, attributes, and properties
/// to the standard `CityJSON` data model. Each extension must be defined with a unique name,
/// a URL where the extension schema is located, and a version identifier.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#using-an-extension-in-a-cityjson-file>
///
/// # Example
///
/// ```
/// use cityjson::prelude::*;
///
/// let noise_ext = ExtensionCore::<OwnedStringStorage>::new(
///     "noise".to_string(),
///     "https://example.com/noise-extension/1.0".to_string(),
///     "1.0".to_string()
/// );
///
/// assert_eq!(noise_ext.name().to_string(), "noise");
/// ```
#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ExtensionCore<SS: StringStorage> {
    name: SS::String,
    url: SS::String,
    version: SS::String,
}

impl<SS: StringStorage> ExtensionCore<SS> {
    /// Creates a new extension with the specified name, URL, and version.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique identifier for this extension
    /// * `url` - URL where the extension schema can be found
    /// * `version` - Version identifier of the extension
    pub fn new(name: SS::String, url: SS::String, version: SS::String) -> Self {
        Self { name, url, version }
    }

    /// Returns a reference to the extension name.
    pub fn name(&self) -> &SS::String {
        &self.name
    }

    /// Returns a reference to the extension schema URL.
    pub fn url(&self) -> &SS::String {
        &self.url
    }

    /// Returns a reference to the extension version.
    pub fn version(&self) -> &SS::String {
        &self.version
    }
}

impl<SS: StringStorage> ExtensionItem<SS> for ExtensionCore<SS> {
    fn name(&self) -> &SS::String {
        &self.name
    }
}

impl<SS: StringStorage> fmt::Display for ExtensionCore<SS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "name: {}, url: {}, version: {}",
            self.name, self.url, self.version
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::storage::OwnedStringStorage;

    #[test]
    fn test_extension() {
        // Create a new extension
        let ext = ExtensionCore::<OwnedStringStorage>::new(
            "noise".to_string(),
            "https://example.com/noise/1.0".to_string(),
            "1.0".to_string(),
        );

        // Test getters
        assert_eq!(ext.name(), &"noise".to_string());
        assert_eq!(ext.url(), &"https://example.com/noise/1.0".to_string());
        assert_eq!(ext.version(), &"1.0".to_string());

        // Test Display
        assert_eq!(
            format!("{}", ext),
            "name: noise, url: https://example.com/noise/1.0, version: 1.0"
        );
    }

    #[test]
    fn test_extensions_add_get() {
        let mut exts =
            ExtensionsCore::<OwnedStringStorage, ExtensionCore<OwnedStringStorage>>::new();

        // Add extension
        let ext1 = ExtensionCore::new(
            "noise".to_string(),
            "https://example.com/noise/1.0".to_string(),
            "1.0".to_string(),
        );
        exts.add(ext1.clone());

        // Test get
        let found = exts.get("noise").unwrap();
        assert_eq!(found, &ext1);

        // Test replace
        let ext2 = ExtensionCore::new(
            "noise".to_string(),
            "https://example.com/noise/2.0".to_string(),
            "2.0".to_string(),
        );
        exts.add(ext2.clone());

        assert_eq!(exts.len(), 1);
        assert_eq!(exts.get("noise").unwrap(), &ext2);
    }

    #[test]
    fn test_extensions_remove_empty() {
        let mut exts =
            ExtensionsCore::<OwnedStringStorage, ExtensionCore<OwnedStringStorage>>::new();

        // Add extension
        let ext = ExtensionCore::new(
            "noise".to_string(),
            "https://example.com/noise/1.0".to_string(),
            "1.0".to_string(),
        );
        exts.add(ext);

        // Remove non-existent extension
        assert!(!exts.remove("other".to_string()));
        assert_eq!(exts.len(), 1);
        assert!(!exts.is_empty());

        // Remove existing extension
        assert!(exts.remove("noise".to_string()));
        assert_eq!(exts.len(), 0);
        assert!(exts.is_empty());
    }

    #[test]
    fn test_extensions_iteration() {
        let mut exts =
            ExtensionsCore::<OwnedStringStorage, ExtensionCore<OwnedStringStorage>>::new();

        // Add extensions
        let ext1 = ExtensionCore::new(
            "noise".to_string(),
            "https://example.com/noise/1.0".to_string(),
            "1.0".to_string(),
        );
        let ext2 = ExtensionCore::new(
            "solar".to_string(),
            "https://example.com/solar/1.0".to_string(),
            "1.0".to_string(),
        );

        exts.add(ext1.clone());
        exts.add(ext2.clone());

        // Test reference iteration
        let mut count = 0;
        for ext in &exts {
            assert!(ext == &ext1 || ext == &ext2);
            count += 1;
        }
        assert_eq!(count, 2);

        // Test mutable iteration
        for _ in &mut exts {
            // Just testing we can get mutable references
        }

        // Test consuming iteration
        let mut names = Vec::new();
        for ext in exts {
            names.push(ext.name().clone());
        }
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"noise".to_string()));
        assert!(names.contains(&"solar".to_string()));
    }
}
