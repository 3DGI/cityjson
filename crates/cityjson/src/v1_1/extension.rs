use crate::cityjson::traits;
use crate::cityjson::core;
use crate::prelude::{ExtensionTrait, StringStorage};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Extensions<SS: StringStorage> {
    inner: core::extension::Extensions<SS>
}

impl<SS: StringStorage> traits::extension::ExtensionsTrait<SS, Extension<SS>> for Extensions<SS> {
    fn new() -> Self {
        Self { inner: core::extension::Extensions::new() }
    }

    fn add(&mut self, extension: Extension<SS>) -> &mut Self {
        self.inner.add(extension.inner)
    }

    fn remove(&mut self, name: SS::String) -> bool {
        self.remove(name)
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

#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Extension<SS: StringStorage> {
    inner: core::extension::Extension<SS>
}

impl<SS: StringStorage> ExtensionTrait<SS> for Extension<SS> {
    fn new(name: SS::String, url: SS::String, version: SS::String) -> Self {
        Self{
            inner: core::extension::Extension::new(name, url, version)
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

impl<SS: StringStorage> From<core::extension::Extension<SS>> for Extension<SS> {
    fn from(value: core::extension::Extension<SS>) -> Self {
        Self {
            inner: value
        }
    }
}
