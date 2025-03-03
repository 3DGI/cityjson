pub mod material;
pub mod texture;

use std::fmt;
pub use material::*;
pub use texture::*;

pub type RGB = [f32; 3];
pub type RGBA = [f32; 4];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
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
