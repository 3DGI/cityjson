//! # CityModel
//!
//! Represents a [CityJSON object](https://www.cityjson.org/specs/1.1.3/#cityjson-object).
use crate::cityjson::citymodel::GenericCityModel;
use crate::resources::pool::DefaultResourcePool;
use crate::resources::storage::OwnedStringStorage;
use crate::v1_1::appearance::material::Material;
use crate::v1_1::appearance::texture::Texture;
use crate::v1_1::geometry::Geometry;
use crate::v1_1::semantic::Semantic;

pub type CityModel<VR, RR, SS> = GenericCityModel<
    VR,
    RR,
    DefaultResourcePool<Semantic<VR, SS>, RR>,
    DefaultResourcePool<Material<SS>, RR>,
    DefaultResourcePool<Texture<SS>, RR>,
    OwnedStringStorage,
    Geometry<VR, RR>,
    Semantic<VR, SS>,
    Material<SS>,
    Texture<SS>,
>;
