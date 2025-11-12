macro_rules! impl_core_transform_methods {
    ($type:ty) => {
        impl $type {
            pub fn new() -> Self {
                Self(transform::TransformCore::new())
            }
            pub fn scale(&self) -> [f64; 3] {
                self.0.scale()
            }
            pub fn translate(&self) -> [f64; 3] {
                self.0.translate()
            }
            pub fn set_scale(&mut self, scale: [f64; 3]) {
                self.0.set_scale(scale);
            }
            pub fn set_translate(&mut self, translate: [f64; 3]) {
                self.0.set_translate(translate);
            }

            pub(crate) fn as_inner(&self) -> &transform::TransformCore {
                &self.0
            }
            #[allow(unused)]
            pub(crate) fn as_inner_mut(&mut self) -> &mut transform::TransformCore {
                &mut self.0
            }
        }
    };
}
pub(crate) use impl_core_transform_methods;

macro_rules! impl_extension_trait {
    () => {
        impl<SS: crate::resources::storage::StringStorage>
            crate::cityjson::traits::extension::ExtensionTrait<SS> for Extension<SS>
        {
            fn new(name: SS::String, url: SS::String, version: SS::String) -> Self {
                Self {
                    inner: crate::cityjson::core::extension::ExtensionCore::new(name, url, version),
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
    };
}
pub(crate) use impl_extension_trait;

macro_rules! impl_extensions_trait {
    () => {
        impl<SS: crate::resources::storage::StringStorage>
            crate::cityjson::traits::extension::ExtensionsTrait<SS, Extension<SS>>
            for Extensions<SS>
        {
            fn new() -> Self {
                Self {
                    inner: crate::cityjson::core::extension::ExtensionsCore::new(),
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

        impl<SS: crate::resources::storage::StringStorage> IntoIterator for Extensions<SS> {
            type Item = Extension<SS>;
            type IntoIter = std::vec::IntoIter<Self::Item>;

            fn into_iter(self) -> Self::IntoIter {
                self.inner.into_iter()
            }
        }

        impl<'a, SS: crate::resources::storage::StringStorage> IntoIterator for &'a Extensions<SS> {
            type Item = &'a Extension<SS>;
            type IntoIter = std::slice::Iter<'a, Extension<SS>>;

            fn into_iter(self) -> Self::IntoIter {
                (&self.inner).into_iter()
            }
        }

        impl<'a, SS: crate::resources::storage::StringStorage> IntoIterator
            for &'a mut Extensions<SS>
        {
            type Item = &'a mut Extension<SS>;
            type IntoIter = std::slice::IterMut<'a, Extension<SS>>;

            fn into_iter(self) -> Self::IntoIter {
                (&mut self.inner).into_iter()
            }
        }

        impl<SS: crate::resources::storage::StringStorage> std::fmt::Display for Extensions<SS> {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.inner)
            }
        }
    };
}
pub(crate) use impl_extensions_trait;
