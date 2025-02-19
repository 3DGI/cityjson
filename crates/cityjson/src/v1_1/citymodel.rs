//! # CityModel
//!
//! Represents a [CityJSON object](https://www.cityjson.org/specs/1.1.3/#cityjson-object).
use crate::common::citymodel::GenericCityModel;
use crate::common::storage::OwnedStringStorage;
use crate::resources::pool::{DefaultResourcePool, ResourceRef};
use crate::v1_1::material::Material;
use crate::v1_1::semantic::Semantic;
use crate::v1_1::texture::Texture;

pub type CityModel<VR, RR, S> = GenericCityModel<
    VR,
    RR,
    DefaultResourcePool<Semantic<VR, S>, RR>,
    DefaultResourcePool<Material<S>, RR>,
    DefaultResourcePool<Texture<S>, RR>,
    OwnedStringStorage, Semantic<VR, S>, Material<S>, Texture<S>,
>;

