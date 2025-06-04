/// Trait representing any type of coordinate.
///
/// This trait serves as a marker for types that can be used as coordinates
/// in the CityJSON model. It's implemented by all coordinate types in this module.
pub trait Coordinate: Default + Clone {}
