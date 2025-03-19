use crate::prelude::StringStorage;
use crate::traits::extension::{ExtensionTrait, ExtensionsTrait};
use std::fmt;

/// A collection of CityJSON Extensions.
///
/// This type provides functionality to manage multiple extensions in a CityJSON model.
/// It ensures that extension names are unique (replacing duplicates), and offers methods
/// to add, remove, and query extensions by name.
///
/// # Example
///
/// ```
/// use cityjson::prelude::*;
/// use cityjson::v1_1::*;
///
/// // Create a collection of extensions
/// let mut extensions = Extensions::<OwnedStringStorage>::new();
///
/// // Add a noise extension to the collection
/// let noise_ext = Extension::new(
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
pub struct Extensions<SS: StringStorage> {
    inner: Vec<Extension<SS>>,
}

impl<SS: StringStorage> ExtensionsTrait<SS, Extension<SS>> for Extensions<SS> {
    fn new() -> Self {
        Self { inner: Vec::new() }
    }

    fn add(&mut self, extension: Extension<SS>) -> &mut Self {
        if let Some(pos) = self.inner.iter().position(|e| e.name() == extension.name()) {
            self.inner[pos] = extension;
        } else {
            self.inner.push(extension);
        }
        self
    }

    fn remove(&mut self, name: SS::String) -> bool {
        if let Some(pos) = self.inner.iter().position(|e| e.name() == &name) {
            self.inner.remove(pos);
            true
        } else {
            false
        }
    }

    fn get(&self, name: &str) -> Option<&Extension<SS>> {
        self.inner.iter().find(|e| e.name().as_ref() == name)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

// Allow consuming iteration
impl<SS: StringStorage> IntoIterator for Extensions<SS> {
    type Item = Extension<SS>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

// Allow iterating by reference
impl<'a, SS: StringStorage> IntoIterator for &'a Extensions<SS> {
    type Item = &'a Extension<SS>;
    type IntoIter = std::slice::Iter<'a, Extension<SS>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

// Allow iterating by mutable reference
impl<'a, SS: StringStorage> IntoIterator for &'a mut Extensions<SS> {
    type Item = &'a mut Extension<SS>;
    type IntoIter = std::slice::IterMut<'a, Extension<SS>>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl<SS: StringStorage> fmt::Display for Extensions<SS> {
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

/// Represents a CityJSON extension with a name, URL, and version.
///
/// Extensions in CityJSON allow for adding custom objects, attributes, and properties
/// to the standard CityJSON data model. Each extension must be defined with a unique name,
/// a URL where the extension schema is located, and a version identifier.
///
/// Specs: <https://www.cityjson.org/specs/1.1.3/#using-an-extension-in-a-cityjson-file>
///
/// # Example
///
/// ```
/// use cityjson::prelude::*;
/// use cityjson::v1_1::*;
///
/// let noise_ext = Extension::<OwnedStringStorage>::new(
///     "noise".to_string(),
///     "https://example.com/noise-extension/1.0".to_string(),
///     "1.0".to_string()
/// );
///
/// assert_eq!(noise_ext.name().to_string(), "noise");
/// ```
#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Extension<SS: StringStorage> {
    name: SS::String,
    url: SS::String,
    version: SS::String,
}

impl<SS: StringStorage> ExtensionTrait<SS> for Extension<SS> {
    /// Creates a new extension with the specified name, URL, and version.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique identifier for this extension
    /// * `url` - URL where the extension schema can be found
    /// * `version` - Version identifier of the extension
    fn new(name: SS::String, url: SS::String, version: SS::String) -> Self {
        Self { name, url, version }
    }

    /// Returns a reference to the extension name.
    fn name(&self) -> &SS::String {
        &self.name
    }

    /// Returns a reference to the extension schema URL.
    fn url(&self) -> &SS::String {
        &self.url
    }

    /// Returns a reference to the extension version.
    fn version(&self) -> &SS::String {
        &self.version
    }
}

impl<SS: StringStorage> fmt::Display for Extension<SS> {
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
        let ext = Extension::<OwnedStringStorage>::new(
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
        let mut exts = Extensions::<OwnedStringStorage>::new();

        // Add extension
        let ext1 = Extension::new(
            "noise".to_string(),
            "https://example.com/noise/1.0".to_string(),
            "1.0".to_string(),
        );
        exts.add(ext1.clone());

        // Test get
        let found = exts.get("noise").unwrap();
        assert_eq!(found, &ext1);

        // Test replace
        let ext2 = Extension::new(
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
        let mut exts = Extensions::<OwnedStringStorage>::new();

        // Add extension
        let ext = Extension::new(
            "noise".to_string(),
            "https://example.com/noise/1.0".to_string(),
            "1.0".to_string(),
        );
        exts.add(ext);

        // Remove non-existent extension
        assert_eq!(exts.remove("other".to_string()), false);
        assert_eq!(exts.len(), 1);
        assert!(!exts.is_empty());

        // Remove existing extension
        assert_eq!(exts.remove("noise".to_string()), true);
        assert_eq!(exts.len(), 0);
        assert!(exts.is_empty());
    }

    #[test]
    fn test_extensions_iteration() {
        let mut exts = Extensions::<OwnedStringStorage>::new();

        // Add extensions
        let ext1 = Extension::new(
            "noise".to_string(),
            "https://example.com/noise/1.0".to_string(),
            "1.0".to_string(),
        );
        let ext2 = Extension::new(
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
