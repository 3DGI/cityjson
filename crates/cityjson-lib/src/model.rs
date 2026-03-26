use std::io::BufRead;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use crate::{json, Result};

#[derive(Debug, Clone)]
pub struct CityModel(pub(crate) cityjson::v2_0::OwnedCityModel);

impl CityModel {
    pub fn new(type_model: cityjson::CityModelType) -> Self {
        Self(cityjson::v2_0::OwnedCityModel::new(type_model))
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        json::from_slice(bytes)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        json::from_file(path)
    }

    pub fn from_stream<R>(reader: R) -> Result<Self>
    where
        R: BufRead,
    {
        json::from_stream(reader)
    }

    pub fn into_inner(self) -> cityjson::v2_0::OwnedCityModel {
        self.0
    }

    pub fn as_inner(&self) -> &cityjson::v2_0::OwnedCityModel {
        &self.0
    }

    pub fn as_inner_mut(&mut self) -> &mut cityjson::v2_0::OwnedCityModel {
        &mut self.0
    }
}

impl Deref for CityModel {
    type Target = cityjson::v2_0::OwnedCityModel;

    fn deref(&self) -> &Self::Target {
        self.as_inner()
    }
}

impl DerefMut for CityModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_inner_mut()
    }
}

impl From<cityjson::v2_0::OwnedCityModel> for CityModel {
    fn from(value: cityjson::v2_0::OwnedCityModel) -> Self {
        Self(value)
    }
}

impl AsRef<cityjson::v2_0::OwnedCityModel> for CityModel {
    fn as_ref(&self) -> &cityjson::v2_0::OwnedCityModel {
        self.as_inner()
    }
}

impl AsMut<cityjson::v2_0::OwnedCityModel> for CityModel {
    fn as_mut(&mut self) -> &mut cityjson::v2_0::OwnedCityModel {
        self.as_inner_mut()
    }
}
