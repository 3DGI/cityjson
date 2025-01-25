use crate::common::storage::{BorrowedStringStorage, OwnedStringStorage, StringStorage};

pub type OwnedTexture = Texture<OwnedStringStorage>;
pub type BorrowedTexture<'a> = Texture<BorrowedStringStorage<'a>>;

pub type RGBA = [f32; 4];

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Texture<S: StringStorage> {
    image_type: ImageType,
    image: S::String,
    wrap_mode: Option<WrapMode>,
    texture_type: Option<TextureType>,
    border_color: Option<RGBA>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ImageType {
    #[default]
    Png,
    Jpg,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextureType {
    #[default]
    Unknown,
    Specific,
    Typical,
}
