use std::fmt::Debug;

/// Trait for types that can reference a material
pub trait MaterialReference: Clone + Debug {
    fn index(&self) -> Option<u32>;
}

