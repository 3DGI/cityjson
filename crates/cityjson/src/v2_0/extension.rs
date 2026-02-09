use crate::cityjson::core;
use crate::prelude::StringStorage;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Extensions<SS: StringStorage> {
    inner: core::extension::ExtensionsCore<SS, Extension<SS>>,
}

impl<SS: StringStorage> Extensions<SS> {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            inner: core::extension::ExtensionsCore::new(),
        }
    }

    pub fn add(&mut self, extension: Extension<SS>) -> &mut Self {
        self.inner.add(extension);
        self
    }

    pub fn remove(&mut self, name: SS::String) -> bool {
        self.inner.remove(name)
    }

    #[must_use] 
    pub fn get(&self, name: &str) -> Option<&Extension<SS>> {
        self.inner.get(name)
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

impl<SS: StringStorage> IntoIterator for Extensions<SS> {
    type Item = Extension<SS>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, SS: StringStorage> IntoIterator for &'a Extensions<SS> {
    type Item = &'a Extension<SS>;
    type IntoIter = std::slice::Iter<'a, Extension<SS>>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.inner).into_iter()
    }
}

impl<'a, SS: StringStorage> IntoIterator for &'a mut Extensions<SS> {
    type Item = &'a mut Extension<SS>;
    type IntoIter = std::slice::IterMut<'a, Extension<SS>>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.inner).into_iter()
    }
}

impl<SS: StringStorage> std::fmt::Display for Extensions<SS> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[repr(transparent)]
#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Extension<SS: StringStorage> {
    inner: core::extension::ExtensionCore<SS>,
}

impl<SS: StringStorage> Extension<SS> {
    pub fn new(name: SS::String, url: SS::String, version: SS::String) -> Self {
        Self {
            inner: core::extension::ExtensionCore::new(name, url, version),
        }
    }

    pub fn name(&self) -> &SS::String {
        self.inner.name()
    }

    pub fn url(&self) -> &SS::String {
        self.inner.url()
    }

    pub fn version(&self) -> &SS::String {
        self.inner.version()
    }
}

impl<SS: StringStorage> crate::cityjson::core::extension::ExtensionItem<SS> for Extension<SS> {
    fn name(&self) -> &SS::String {
        self.inner.name()
    }
}
