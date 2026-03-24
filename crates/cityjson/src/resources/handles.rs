//! # Opaque resource handle types
//!
//! Type-safe opaque handles for different resource types.
//! Each handle wraps a `ResourceId32` internally.
//! Handles use `#[repr(transparent)]` for zero runtime overhead.
//! Apart from the `Default` implementation, handles cannot be created directly,
//! to enforce the integrity of the `ResourcePool`. Thus, all handles are created by
//! the `ResourcePool`.
//!
//! # Examples
//!
//! ```
//! use cityjson::resources::handles::{GeometryHandle, MaterialHandle};
//!
//! let geometry = GeometryHandle::default();
//! let material = MaterialHandle::default();
//!
//! assert!(geometry.is_null());
//! assert!(material.is_null());
//! assert_eq!(format!("{geometry}"), "GeometryHandle");
//! assert_eq!(format!("{material:?}"), "MaterialHandle(index=0, generation=0)");
//! ```

use crate::resources::id::ResourceId32;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

/// Internal trait for converting between typed handles and raw `ResourceId32` values.
#[allow(dead_code)]
pub(crate) trait HandleType: Copy + Clone + PartialEq + Eq + Hash + Default {
    fn from_raw(raw: ResourceId32) -> Self;
    fn to_raw(self) -> ResourceId32;

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
            pub fn is_null(self) -> bool {
                self.0.index() == 0 && self.0.generation() == 0
            }

            pub(crate) fn from_parts(index: u32, generation: u16) -> Self {
                Self(ResourceId32::new(index, generation))
            }

            pub(crate) fn index(self) -> u32 {
                self.0.index()
            }

            pub(crate) fn generation(self) -> u16 {
                self.0.generation()
            }

            pub(crate) fn from_raw_parts(index: u32, generation: u16) -> Self {
                Self(ResourceId32::new(index, generation))
            }

            pub(crate) fn to_raw_parts(self) -> (u32, u16) {
                (self.0.index(), self.0.generation())
            }

            pub(crate) fn to_raw(self) -> ResourceId32 {
                self.0
            }

            pub(crate) fn from_raw(raw: ResourceId32) -> Self {
                Self(raw)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", stringify!($name))
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}(index={}, generation={})", stringify!($name), self.0.index(), self.0.generation())
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
    /// Handle to a Geometry.
    GeometryHandle
}

define_handle! {
    /// Handle to a `GeometryTemplate`.
    GeometryTemplateHandle
}

define_handle! {
    /// Handle to a Semantic.
    SemanticHandle
}

define_handle! {
    /// Handle to a Material.
    MaterialHandle
}

define_handle! {
    /// Handle to a Texture.
    TextureHandle
}

define_handle! {
    /// Handle to a `CityObject`.
    CityObjectHandle
}

#[inline]
pub(crate) fn cast_handle_slice<H: HandleType>(raw: &[ResourceId32]) -> &[H] {
    const {
        assert!(std::mem::size_of::<H>() == std::mem::size_of::<ResourceId32>());
        assert!(std::mem::align_of::<H>() == std::mem::align_of::<ResourceId32>());
    }

    // SAFETY: all exported handle types are `#[repr(transparent)]` wrappers over `ResourceId32`,
    // and compile-time layout assertions above guarantee identical size/alignment.
    unsafe { std::slice::from_raw_parts(raw.as_ptr().cast::<H>(), raw.len()) }
}

#[inline]
pub(crate) fn cast_option_handle_slice<H: HandleType>(
    raw: &[Option<ResourceId32>],
) -> &[Option<H>] {
    const {
        assert!(std::mem::size_of::<Option<H>>() == std::mem::size_of::<Option<ResourceId32>>());
        assert!(std::mem::align_of::<Option<H>>() == std::mem::align_of::<Option<ResourceId32>>());
    }

    // SAFETY: handle types are `#[repr(transparent)]` wrappers over `ResourceId32`, and
    // `Option<Handle>` has identical layout to `Option<ResourceId32>` due compile-time checks.
    unsafe { std::slice::from_raw_parts(raw.as_ptr().cast::<Option<H>>(), raw.len()) }
}
