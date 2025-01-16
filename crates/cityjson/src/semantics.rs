use std::fmt::Debug;

/// Trait for types that can reference a semantic
pub trait SemanticReference: Clone + Debug {
    fn index(&self) -> Option<u32>;
}