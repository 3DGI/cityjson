mod temporary_json;

use crate::{CityModel, Result};

pub(crate) trait CityJsonBoundary {
    fn import_document_v2(bytes: &[u8]) -> Result<CityModel>;
    fn merge_feature_v2(model: &mut CityModel, bytes: &[u8]) -> Result<()>;
}

pub(crate) struct TemporaryCityJsonBoundary;

impl CityJsonBoundary for TemporaryCityJsonBoundary {
    fn import_document_v2(bytes: &[u8]) -> Result<CityModel> {
        temporary_json::import_document(bytes)
    }

    fn merge_feature_v2(model: &mut CityModel, bytes: &[u8]) -> Result<()> {
        temporary_json::merge_feature(model, bytes)
    }
}

pub(crate) type ActiveCityJsonBoundary = TemporaryCityJsonBoundary;
