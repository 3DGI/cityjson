use std::fmt::Debug;

/// Trait for types that can reference a texture
pub trait TextureReference: Clone + Debug {
    fn index(&self) -> Option<u32>;
}
