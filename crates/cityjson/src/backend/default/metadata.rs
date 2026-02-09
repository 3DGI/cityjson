use crate::prelude::StringStorage;
use std::fmt::{Display, Formatter};

/// Bounding Box.
///
/// A wrapper around an array of 6 values: `[minx, miny, minz, maxx, maxy, maxz]`.
///
/// # Examples
/// ```
/// # use cityjson::prelude::*;
/// # use cityjson::v2_0::*;
/// let bbox = BBox::new(84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9);
/// let bbox_height = bbox.height();
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BBox {
    values: [f64; 6],
}

impl BBox {
    #[must_use]
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        Self {
            values: [min_x, min_y, min_z, max_x, max_y, max_z],
        }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[f64] {
        &self.values
    }

    /// Returns the minimum x coordinate.
    #[must_use]
    pub fn min_x(&self) -> f64 {
        self.values[0]
    }

    /// Returns the minimum y coordinate.
    #[must_use]
    pub fn min_y(&self) -> f64 {
        self.values[1]
    }

    /// Returns the minimum z coordinate.
    #[must_use]
    pub fn min_z(&self) -> f64 {
        self.values[2]
    }

    /// Returns the maximum x coordinate.
    #[must_use]
    pub fn max_x(&self) -> f64 {
        self.values[3]
    }

    /// Returns the maximum y coordinate.
    #[must_use]
    pub fn max_y(&self) -> f64 {
        self.values[4]
    }

    /// Returns the maximum z coordinate.
    #[must_use]
    pub fn max_z(&self) -> f64 {
        self.values[5]
    }

    /// Calculates the width (x-axis length) of the bounding box.
    #[must_use]
    pub fn width(&self) -> f64 {
        self.max_x() - self.min_x()
    }

    /// Calculates the length (y-axis length) of the bounding box.
    #[must_use]
    pub fn length(&self) -> f64 {
        self.max_y() - self.min_y()
    }

    /// Calculates the height (z-axis length) of the bounding box.
    #[must_use]
    pub fn height(&self) -> f64 {
        self.max_z() - self.min_z()
    }
}

impl Default for BBox {
    /// Creates a default `BBox` with all coordinates set to 0.0.
    fn default() -> Self {
        Self {
            values: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }
}

impl From<[f64; 6]> for BBox {
    /// Creates a `BBox` from an array of 6 values.
    fn from(values: [f64; 6]) -> Self {
        Self { values }
    }
}

impl From<BBox> for [f64; 6] {
    /// Converts a `BBox` into an array of 6 values.
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

/// An identifier for the dataset.
///
/// # Examples
/// ```
/// # use cityjson::prelude::{BorrowedStringStorage, CityModelIdentifier};
/// let city_id: CityModelIdentifier<BorrowedStringStorage> = CityModelIdentifier::new("44574905-d2d2-4f40-8e96-d39e1ae45f70");
/// ```
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct CityModelIdentifier<SS: StringStorage>(SS::String);

impl<SS: StringStorage> CityModelIdentifier<SS> {
    pub fn new(value: SS::String) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> SS::String {
        self.0
    }
}

impl<SS: StringStorage> Display for CityModelIdentifier<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The date when the dataset was compiled.
///
/// The format is a `"full-date"` per the
/// [RFC 3339, Section 5.6](https://tools.ietf.org/html/rfc3339#section-5.6).
///
/// # Examples
/// ```
/// # use cityjson::prelude::{BorrowedStringStorage, Date};
/// let date: Date<BorrowedStringStorage> = Date::new("1977-02-28");
/// ```
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Date<SS: StringStorage>(SS::String);

impl<SS: StringStorage> Date<SS> {
    pub fn new(value: SS::String) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> SS::String {
        self.0
    }
}

impl<SS: StringStorage> Display for Date<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The coordinate reference system (CRS) of the city model.
///
/// Must be formatted as a URL, according to the
/// [OGC Name Type Specification](https://docs.opengeospatial.org/pol/09-048r5.html#_production_rule_for_specification_element_names).
///
/// # Examples
/// ```
/// # use cityjson::prelude::{BorrowedStringStorage, CRS};
/// let crs: CRS<BorrowedStringStorage> = CRS::new("https://www.opengis.net/def/crs/EPSG/0/7415");
/// ```
#[repr(C)]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct CRS<SS: StringStorage>(SS::String);

impl<SS: StringStorage> CRS<SS> {
    pub fn new(value: SS::String) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> SS::String {
        self.0
    }
}

impl<SS: StringStorage> Display for CRS<SS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
