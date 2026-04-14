#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

mod de;
pub mod errors;
mod ser;
pub mod v2_0;

#[doc(hidden)]
pub use cityjson::prelude;
#[doc(hidden)]
pub use cityjson::v2_0::{BorrowedCityModel, CityModel, OwnedCityModel};
#[doc(hidden)]
pub use cityjson::{CityJSONVersion, CityModelType};

pub use errors::{Error, Result};
pub use v2_0::{
    AutoTransformOptions, CityJSONSeqWriteOptions, CityJSONSeqWriteReport, FeatureObject,
    FeatureParts, ParseStringStorage, SerializableCityModel, as_json,
    from_feature_parts_owned_with_base, from_feature_str_owned, from_feature_str_owned_with_base,
    from_str, from_str_borrowed, from_str_owned, merge_feature_stream, read_feature_stream,
    to_string_feature, write_cityjsonseq_auto_transform_refs, write_cityjsonseq_with_transform_refs,
};
