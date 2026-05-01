//! Shared core modules for versioned `CityJSON` APIs.

macro_rules! define_string_wrapper {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
        pub struct $name<SS: crate::resources::storage::StringStorage>(SS::String);

        impl<SS: crate::resources::storage::StringStorage> $name<SS> {
            pub fn new(value: SS::String) -> Self {
                Self(value)
            }

            pub fn as_inner(&self) -> &SS::String {
                &self.0
            }

            pub fn into_inner(self) -> SS::String {
                self.0
            }
        }

        impl<SS: crate::resources::storage::StringStorage> Default for $name<SS>
        where
            SS::String: Default,
        {
            fn default() -> Self {
                Self(Default::default())
            }
        }

        impl<SS: crate::resources::storage::StringStorage> std::fmt::Display for $name<SS>
        where
            SS::String: std::fmt::Display,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::Display::fmt(&self.0, f)
            }
        }

        impl<SS: crate::resources::storage::StringStorage> PartialEq<str> for $name<SS>
        where
            SS::String: AsRef<str>,
        {
            fn eq(&self, other: &str) -> bool {
                self.0.as_ref() == other
            }
        }

        impl<SS: crate::resources::storage::StringStorage> PartialEq<&str> for $name<SS>
        where
            SS::String: AsRef<str>,
        {
            fn eq(&self, other: &&str) -> bool {
                self.0.as_ref() == *other
            }
        }

        impl<SS: crate::resources::storage::StringStorage> AsRef<str> for $name<SS>
        where
            SS::String: AsRef<str>,
        {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl<SS: crate::resources::storage::StringStorage> std::borrow::Borrow<str> for $name<SS>
        where
            SS::String: std::borrow::Borrow<str>,
        {
            fn borrow(&self) -> &str {
                std::borrow::Borrow::borrow(&self.0)
            }
        }

        impl From<String> for $name<crate::resources::storage::OwnedStringStorage> {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl<'a> From<&'a str> for $name<crate::resources::storage::BorrowedStringStorage<'a>> {
            fn from(value: &'a str) -> Self {
                Self(value)
            }
        }
    };
}

pub mod appearance {
    pub use crate::backend::default::appearance::{ImageType, RGB, RGBA, TextureType, WrapMode};

    define_string_wrapper!(ThemeName);
}

pub mod attributes {
    pub use crate::backend::default::attributes::{
        AttributeValue, Attributes, BorrowedAttributeValue, BorrowedAttributes,
        OwnedAttributeValue, OwnedAttributes,
    };
}

pub mod boundary {
    pub use crate::backend::default::boundary::nested;
    pub use crate::backend::default::boundary::{
        Boundary, Boundary16, Boundary32, Boundary64, BoundaryCoordinates, BoundaryType,
        BoundaryUniqueCoordinates,
    };
}

pub mod cityobject {
    define_string_wrapper!(CityObjectIdentifier);
}

pub mod coordinate {
    /// Marker trait for coordinate value types.
    pub trait Coordinate: Default + Clone {}

    pub use crate::backend::default::coordinate::{RealWorldCoordinate, UVCoordinate};
}

pub mod extension {
    use crate::backend::default::extension::{ExtensionCore, ExtensionItem, ExtensionsCore};
    use crate::resources::storage::StringStorage;

    #[derive(Debug, Default, Clone, PartialEq)]
    pub struct Extensions<SS: StringStorage> {
        inner: ExtensionsCore<SS, Extension<SS>>,
    }

    impl<SS: StringStorage> Extensions<SS> {
        #[must_use]
        pub fn new() -> Self {
            Self {
                inner: ExtensionsCore::new(),
            }
        }

        pub fn add(&mut self, extension: Extension<SS>) -> &mut Self {
            self.inner.add(extension);
            self
        }

        pub fn remove(&mut self, name: &SS::String) -> bool {
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

        pub fn iter(&self) -> std::slice::Iter<'_, Extension<SS>> {
            self.inner.iter()
        }

        pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Extension<SS>> {
            self.inner.iter_mut()
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
            self.iter()
        }
    }

    impl<'a, SS: StringStorage> IntoIterator for &'a mut Extensions<SS> {
        type Item = &'a mut Extension<SS>;
        type IntoIter = std::slice::IterMut<'a, Extension<SS>>;

        fn into_iter(self) -> Self::IntoIter {
            self.iter_mut()
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
        inner: ExtensionCore<SS>,
    }

    impl<SS: StringStorage> Extension<SS> {
        pub fn new(name: SS::String, url: SS::String, version: SS::String) -> Self {
            Self {
                inner: ExtensionCore::new(name, url, version),
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

    impl<SS: StringStorage> ExtensionItem<SS> for Extension<SS> {
        fn name(&self) -> &SS::String {
            self.inner.name()
        }
    }
}

pub mod geometry {
    pub use crate::backend::default::geometry::{GeometryType, LoD};
}

pub mod metadata {
    pub use crate::backend::default::metadata::{BBox, CRS, CityModelIdentifier, Date};
}

pub mod semantic {
    /// Marker trait for semantic type enums.
    #[allow(dead_code)]
    pub trait SemanticTypeTrait: Default + std::fmt::Display + Clone {}
}

pub mod transform {
    use crate::backend::default::transform::TransformCore;
    use std::fmt::{Display, Formatter};

    /// Transform.
    ///
    /// Specs: <https://www.cityjson.org/specs/1.1.3/#transform-object>.
    #[repr(transparent)]
    #[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
    pub struct Transform(TransformCore);

    impl Display for Transform {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.as_inner())
        }
    }

    impl Transform {
        #[must_use]
        pub fn new() -> Self {
            Self(TransformCore::new())
        }

        #[must_use]
        pub fn scale(&self) -> [f64; 3] {
            self.0.scale()
        }

        #[must_use]
        pub fn translate(&self) -> [f64; 3] {
            self.0.translate()
        }

        pub fn set_scale(&mut self, scale: [f64; 3]) {
            self.0.set_scale(scale);
        }

        pub fn set_translate(&mut self, translate: [f64; 3]) {
            self.0.set_translate(translate);
        }

        pub(crate) fn as_inner(&self) -> &TransformCore {
            &self.0
        }
    }
}

pub mod vertex {
    pub use crate::backend::default::vertex::{
        RawVertexView, VertexIndex, VertexIndex16, VertexIndex32, VertexIndex64, VertexIndexVec,
        VertexIndicesSequence, VertexRef,
    };
}

pub mod vertices {
    pub use crate::backend::default::vertices::{
        GeometryVertices16, GeometryVertices32, GeometryVertices64, UVVertices16, UVVertices32,
        UVVertices64, Vertices,
    };
}
