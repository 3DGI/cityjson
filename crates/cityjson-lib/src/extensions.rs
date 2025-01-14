use serde_cityjson::v1_1;
use std::collections::HashMap;

pub type ExtensionName = String;

#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Extensions(HashMap<ExtensionName, Extension>);

// Implement conversion from serde_cityjson types
impl From<v1_1::Extensions> for Extensions {
    fn from(ext: v1_1::Extensions) -> Self {
        Self(
            ext.into_iter()
                .map(|(k, v)| (k, Extension::from(v)))
                .collect(),
        )
    }
}

impl Extensions {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert<N: Into<ExtensionName>>(&mut self, name: N, extension: Extension) {
        self.0.insert(name.into(), extension);
    }

    pub fn remove(&mut self, name: &str) -> Option<Extension> {
        self.0.remove(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&Extension> {
        self.0.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Extension> {
        self.0.get_mut(name)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ExtensionName, &Extension)> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ExtensionName, &mut Extension)> {
        self.0.iter_mut()
    }
}

impl<'a> IntoIterator for &'a Extensions {
    type Item = (&'a ExtensionName, &'a Extension);
    type IntoIter = std::collections::hash_map::Iter<'a, ExtensionName, Extension>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Extensions {
    type Item = (&'a ExtensionName, &'a mut Extension);
    type IntoIter = std::collections::hash_map::IterMut<'a, ExtensionName, Extension>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Extension {
    url: String,
    version: String,
}

impl From<v1_1::Extension> for Extension {
    fn from(ext: v1_1::Extension) -> Self {
        Self {
            url: ext.url,
            version: ext.version,
        }
    }
}

impl Extension {
    pub fn new(url: String, version: String) -> Self {
        Self { url, version }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }
}
