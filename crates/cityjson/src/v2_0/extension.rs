use crate::cityjson::core;
use crate::macros::{impl_extension_trait, impl_extensions_trait};
use crate::prelude::StringStorage;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Extensions<SS: StringStorage> {
    inner: core::extension::ExtensionsCore<SS, Extension<SS>>,
}

impl_extensions_trait!();

#[repr(transparent)]
#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Extension<SS: StringStorage> {
    inner: core::extension::ExtensionCore<SS>,
}

impl_extension_trait!();
