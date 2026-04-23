#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

mod de;
pub mod errors;
mod facade;
mod ser;
pub mod v2_0;

#[doc(hidden)]
pub use cityjson::prelude;
#[doc(hidden)]
pub use cityjson::v2_0::{CityModel, OwnedCityModel};
#[doc(hidden)]
pub use cityjson::{CityJSONVersion, CityModelType};

pub use errors::{Error, Result};
pub use facade::{
    Probe, RootKind, append, cleanup, extract, merge, merge_cityjsonseq_slice,
    merge_feature_stream_slice, probe, staged,
};
pub use v2_0::{
    CityJsonSeqReader, CityJsonSeqWriteOptions, CityJsonSeqWriteReport, FeatureStreamTransform,
    ReadOptions, WriteOptions, read_feature, read_feature_stream, read_feature_with_base,
    read_model, to_vec, write_feature_stream, write_model,
};
