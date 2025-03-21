use crate::prelude::{Attributes, BBoxTrait};
use std::fmt::{Display, Formatter};

pub type Metadata<SS, RR> = Attributes<SS, RR>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BBox {
    values: [f64; 6],
}

impl BBoxTrait for BBox {
    fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            values: [min_x, min_y, min_z, max_x, max_y, max_z],
        }
    }

    fn from_array(values: [f64; 6]) -> Self {
        Self { values }
    }

    fn as_array(&self) -> &[f64; 6] {
        &self.values
    }

    fn as_array_mut(&mut self) -> &mut [f64; 6] {
        &mut self.values
    }

    fn min_x(&self) -> f64 {
        self.values[0]
    }

    fn min_y(&self) -> f64 {
        self.values[1]
    }

    fn min_z(&self) -> f64 {
        self.values[2]
    }

    fn max_x(&self) -> f64 {
        self.values[3]
    }

    fn max_y(&self) -> f64 {
        self.values[4]
    }

    fn max_z(&self) -> f64 {
        self.values[5]
    }

    fn set_min_x(&mut self, value: f64) {
        self.values[0] = value;
    }

    fn set_min_y(&mut self, value: f64) {
        self.values[1] = value;
    }

    fn set_min_z(&mut self, value: f64) {
        self.values[2] = value;
    }

    fn set_max_x(&mut self, value: f64) {
        self.values[3] = value;
    }

    fn set_max_y(&mut self, value: f64) {
        self.values[4] = value;
    }

    fn set_max_z(&mut self, value: f64) {
        self.values[5] = value;
    }

    fn width(&self) -> f64 {
        self.max_x() - self.min_x()
    }

    fn length(&self) -> f64 {
        self.max_y() - self.min_y()
    }

    fn height(&self) -> f64 {
        self.max_z() - self.min_z()
    }
}

impl Default for BBox {
    /// Creates a default BBox with all coordinates set to 0.0.
    fn default() -> Self {
        Self {
            values: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }
}

impl From<[f64; 6]> for BBox {
    /// Creates a BBox from an array of 6 values.
    fn from(values: [f64; 6]) -> Self {
        Self { values }
    }
}

impl From<BBox> for [f64; 6] {
    /// Converts a BBox into an array of 6 values.
    fn from(bbox: BBox) -> Self {
        bbox.values
    }
}

impl Display for BBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}, {}, {}, {}, {}, {}]",
            self.min_x(),
            self.min_y(),
            self.min_z(),
            self.max_x(),
            self.max_y(),
            self.max_z()
        )
    }
}
