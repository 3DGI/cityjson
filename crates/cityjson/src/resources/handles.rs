//! # Opaque resource handle types
//!
//! This module defines type-safe opaque handles for different resource types.
//! Each handle wraps a [`ResourceId32`] internally but provides compile-time type safety,
//! preventing accidental mixing of different resource references.
//!
//! Handles use the newtype pattern with `#[repr(transparent)]` for zero runtime overhead
//! while maintaining maximum type safety.

use crate::resources::pool::ResourceId32;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

/// Trait for converting between typed handles and raw `ResourceId32` values.
/// This is an internal trait used at API boundaries to convert between
/// opaque handles and the underlying storage type.
#[allow(dead_code)]
pub(crate) trait HandleType: Copy + Clone + PartialEq + Eq + Hash + Default {
    /// Convert from a raw `ResourceId32` to this handle type.
    fn from_raw(raw: ResourceId32) -> Self;

    /// Convert this handle to a raw `ResourceId32`.
    fn to_raw(self) -> ResourceId32;

    /// Check if this handle is null (has index 0 and generation 0).
    fn is_null(self) -> bool {
        let raw = self.to_raw();
        raw.index() == 0 && raw.generation() == 0
    }
}

/// Macro to define a newtype handle around `ResourceId32`.
macro_rules! define_handle {
    (
        $(#[$meta:meta])*
        $name:ident
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(ResourceId32);

        #[allow(dead_code)]
        impl $name {
            /// Check if this handle is null.
            pub fn is_null(self) -> bool {
                self.0.index() == 0 && self.0.generation() == 0
            }

            /// Create a typed handle from index and generation.
            pub fn from_parts(index: u32, generation: u16) -> Self {
                Self(ResourceId32::new(index, generation))
            }

            /// Get the underlying index.
            pub(crate) fn index(self) -> u32 {
                self.0.index()
            }

            /// Get the underlying generation.
            pub(crate) fn generation(self) -> u16 {
                self.0.generation()
            }

            /// Create a handle from raw parts (internal use only).
            pub(crate) fn from_raw_parts(index: u32, generation: u16) -> Self {
                Self(ResourceId32::new(index, generation))
            }

            /// Get the raw parts (internal use only).
            pub(crate) fn to_raw_parts(self) -> (u32, u16) {
                (self.0.index(), self.0.generation())
            }

            /// Convert to raw `ResourceId32` (internal use only).
            pub(crate) fn to_raw(self) -> ResourceId32 {
                self.0
            }

            /// Convert from raw `ResourceId32` (internal use only).
            pub(crate) fn from_raw(raw: ResourceId32) -> Self {
                Self(raw)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}(..)", stringify!($name))
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}(..)", stringify!($name))
            }
        }

        impl HandleType for $name {
            fn from_raw(raw: ResourceId32) -> Self {
                Self(raw)
            }

            fn to_raw(self) -> ResourceId32 {
                self.0
            }
        }
    };
}

define_handle! {
    /// Handle to a Geometry resource.
    GeometryRef
}

define_handle! {
    /// Handle to a Template Geometry resource (for instance geometry).
    TemplateGeometryRef
}

define_handle! {
    /// Handle to a Semantic resource.
    SemanticRef
}

define_handle! {
    /// Handle to a Material resource.
    MaterialRef
}

define_handle! {
    /// Handle to a Texture resource.
    TextureRef
}

define_handle! {
    /// Handle to a `CityObject` resource.
    CityObjectRef
}

/// Handle to an Attribute resource.
/// Attributes are stored in the global `AttributePool`.
#[repr(transparent)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AttributeRef(ResourceId32);

#[allow(dead_code)]
impl AttributeRef {
    /// Create a typed handle from index and generation.
    #[must_use]
    pub fn from_parts(index: u32, generation: u16) -> Self {
        Self(ResourceId32::new(index, generation))
    }

    /// Check if this handle is null.
    #[must_use]
    pub fn is_null(self) -> bool {
        self.0.index() == 0 && self.0.generation() == 0
    }

    /// Get the underlying index.
    pub(crate) fn index(self) -> u32 {
        self.0.index()
    }

    /// Get the underlying generation.
    pub(crate) fn generation(self) -> u16 {
        self.0.generation()
    }

    /// Create a handle from raw parts (internal use only).
    pub(crate) fn from_raw_parts(index: u32, generation: u16) -> Self {
        Self(ResourceId32::new(index, generation))
    }

    /// Get the raw parts (internal use only).
    pub(crate) fn to_raw_parts(self) -> (u32, u16) {
        (self.0.index(), self.0.generation())
    }

    /// Convert to raw `ResourceId32` (internal use only).
    pub(crate) fn to_raw(self) -> ResourceId32 {
        self.0
    }

    /// Convert from raw `ResourceId32` (internal use only).
    pub(crate) fn from_raw(raw: ResourceId32) -> Self {
        Self(raw)
    }
}

impl Display for AttributeRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AttributeRef(..)")
    }
}

impl std::fmt::Debug for AttributeRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AttributeRef(..)")
    }
}

impl HandleType for AttributeRef {
    fn from_raw(raw: ResourceId32) -> Self {
        Self(raw)
    }

    fn to_raw(self) -> ResourceId32 {
        self.0
    }
}
