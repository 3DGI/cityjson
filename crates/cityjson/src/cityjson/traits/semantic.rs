/// Marker trait for semantic type enums.
///
/// This trait is implemented by semantic type enums across different CityJSON versions.
/// It requires Default, Display, and Clone implementations.
pub trait SemanticTypeTrait: Default + std::fmt::Display + Clone {}
