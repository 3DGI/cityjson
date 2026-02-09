use crate::resources::storage::StringStorage;
use std::fmt::{Display, Formatter};

macro_rules! define_string_wrapper {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
        pub struct $name<SS: StringStorage>(SS::String);

        impl<SS: StringStorage> $name<SS> {
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

        impl<SS: StringStorage> Default for $name<SS>
        where
            SS::String: Default,
        {
            fn default() -> Self {
                Self(Default::default())
            }
        }

        impl<SS: StringStorage> Display for $name<SS>
        where
            SS::String: Display,
        {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        impl<SS: StringStorage> PartialEq<str> for $name<SS>
        where
            SS::String: AsRef<str>,
        {
            fn eq(&self, other: &str) -> bool {
                self.0.as_ref() == other
            }
        }

        impl<SS: StringStorage> PartialEq<&str> for $name<SS>
        where
            SS::String: AsRef<str>,
        {
            fn eq(&self, other: &&str) -> bool {
                self.0.as_ref() == *other
            }
        }
    };
}

define_string_wrapper!(CityObjectIdentifier);
define_string_wrapper!(ThemeName);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RGB([f32; 3]);

impl RGB {
    #[must_use]
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        Self([red, green, blue])
    }

    #[must_use]
    pub fn as_array(self) -> [f32; 3] {
        self.0
    }
}

impl From<[f32; 3]> for RGB {
    fn from(value: [f32; 3]) -> Self {
        Self(value)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RGBA([f32; 4]);

impl RGBA {
    #[must_use]
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self([red, green, blue, alpha])
    }

    #[must_use]
    pub fn as_array(self) -> [f32; 4] {
        self.0
    }
}

impl From<[f32; 4]> for RGBA {
    fn from(value: [f32; 4]) -> Self {
        Self(value)
    }
}
