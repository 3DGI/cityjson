//! # CityJSON version 2.0
//!
//! Implementation of the CityJSON types and traits for CityJSON version 2.0.
pub mod citymodel;
pub mod extension;
pub mod metadata;
pub mod transform;

pub use citymodel::CityModel;
pub use extension::{Extension, Extensions};
pub use metadata::{
    BBox, CityModelIdentifier, Contact, ContactRole, ContactType, Date, Metadata, CRS,
};
pub use transform::Transform;
