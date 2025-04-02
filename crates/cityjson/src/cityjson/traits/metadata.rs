use crate::resources::storage::StringStorage;

pub trait MetadataTrait<SS: StringStorage> {}

pub trait BBoxTrait {
    /// Creates a new BBox with the specified coordinates.
    ///
    /// # Parameters
    /// - `min_x`: Minimum x coordinate
    /// - `min_y`: Minimum y coordinate
    /// - `min_z`: Minimum z coordinate
    /// - `max_x`: Maximum x coordinate
    /// - `max_y`: Maximum y coordinate
    /// - `max_z`: Maximum z coordinate
    fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self;
    /// Creates a BBox from an array of 6 values.
    fn from_array(values: [f64; 6]) -> Self;
    /// Returns the underlying array.
    fn as_array(&self) -> &[f64; 6];
    /// Returns the minimum x coordinate.
    fn min_x(&self) -> f64;
    /// Returns the minimum y coordinate.
    fn min_y(&self) -> f64;
    /// Returns the minimum z coordinate.
    fn min_z(&self) -> f64;
    /// Returns the maximum x coordinate.
    fn max_x(&self) -> f64;
    /// Returns the maximum y coordinate.
    fn max_y(&self) -> f64;
    /// Returns the maximum z coordinate.
    fn max_z(&self) -> f64;
    /// Calculates the width (x-axis length) of the bounding box.
    fn width(&self) -> f64;
    /// Calculates the length (y-axis length) of the bounding box.
    fn length(&self) -> f64;
    /// Calculates the height (z-axis length) of the bounding box.
    fn height(&self) -> f64;
}
