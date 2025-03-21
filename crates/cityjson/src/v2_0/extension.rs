use crate::cityjson::core;
use crate::prelude::{ExtensionTrait, ExtensionsTrait, StringStorage};
use std::fmt;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Extensions<SS: StringStorage> {
    inner: core::extension::ExtensionsCore<SS, Extension<SS>>,
}

impl<SS: StringStorage> ExtensionsTrait<SS, Extension<SS>> for Extensions<SS> {
    fn new() -> Self {
        Self {
            inner: core::extension::ExtensionsCore::new(),
        }
    }

    fn add(&mut self, extension: Extension<SS>) -> &mut Self {
        self.inner.add(extension);
        self
    }

    fn remove(&mut self, name: SS::String) -> bool {
        self.inner.remove(name)
    }

    fn get(&self, name: &str) -> Option<&Extension<SS>> {
        self.inner.get(name)
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
        // This calls the IntoIterator implementation for &Extensions
        (&self.inner).into_iter()
    }
}

// Allow iterating by mutable reference
impl<'a, SS: StringStorage> IntoIterator for &'a mut Extensions<SS> {
    type Item = &'a mut Extension<SS>;
    type IntoIter = std::slice::IterMut<'a, Extension<SS>>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.inner).into_iter()
    }
}

impl<SS: StringStorage> fmt::Display for Extensions<SS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Extension<SS: StringStorage> {
    inner: core::extension::ExtensionCore<SS>,
}

impl<SS: StringStorage> ExtensionTrait<SS> for Extension<SS> {
    fn new(name: SS::String, url: SS::String, version: SS::String) -> Self {
        Self {
            inner: core::extension::ExtensionCore::new(name, url, version),
        }
    }

    fn name(&self) -> &SS::String {
        self.inner.name()
    }

    fn url(&self) -> &SS::String {
        self.inner.url()
    }

    fn version(&self) -> &SS::String {
        self.inner.version()
    }
}
