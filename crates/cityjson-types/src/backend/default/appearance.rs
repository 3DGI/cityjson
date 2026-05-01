use std::{fmt, write};

#[allow(clippy::upper_case_acronyms)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RGB([f32; 3]);

impl RGB {
    #[must_use]
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        Self([red, green, blue])
    }

    #[must_use]
    pub fn to_array(self) -> [f32; 3] {
        self.0
    }
}

impl From<[f32; 3]> for RGB {
    fn from(value: [f32; 3]) -> Self {
        Self(value)
    }
}

impl From<RGB> for [f32; 3] {
    fn from(value: RGB) -> Self {
        value.0
    }
}

impl fmt::Display for RGB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.0[0], self.0[1], self.0[2])
    }
}

#[allow(clippy::upper_case_acronyms)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RGBA([f32; 4]);

impl RGBA {
    #[must_use]
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self([red, green, blue, alpha])
    }

    #[must_use]
    pub fn to_array(self) -> [f32; 4] {
        self.0
    }
}

impl From<[f32; 4]> for RGBA {
    fn from(value: [f32; 4]) -> Self {
        Self(value)
    }
}

impl From<RGBA> for [f32; 4] {
    fn from(value: RGBA) -> Self {
        value.0
    }
}

impl fmt::Display for RGBA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}, {}, {}, {}]",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ImageType {
    #[default]
    Png,
    Jpg,
}

impl fmt::Display for ImageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageType::Png => write!(f, "PNG"),
            ImageType::Jpg => write!(f, "JPG"),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WrapMode {
    Wrap,
    Mirror,
    Clamp,
    Border,
    #[default]
    None,
}

impl fmt::Display for WrapMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WrapMode::Wrap => write!(f, "wrap"),
            WrapMode::Mirror => write!(f, "mirror"),
            WrapMode::Clamp => write!(f, "clamp"),
            WrapMode::Border => write!(f, "border"),
            WrapMode::None => write!(f, "none"),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum TextureType {
    #[default]
    Unknown,
    Specific,
    Typical,
}

impl fmt::Display for TextureType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextureType::Unknown => write!(f, "unknown"),
            TextureType::Specific => write!(f, "specific"),
            TextureType::Typical => write!(f, "typical"),
        }
    }
}
