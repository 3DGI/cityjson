use crate::{CityModel, Error, Result};

pub fn cleanup(model: &CityModel) -> Result<CityModel> {
    cityjson_json::cleanup(model).map_err(Error::from)
}

pub fn extract<'a, I>(model: &CityModel, cityobject_ids: I) -> Result<CityModel>
where
    I: IntoIterator<Item = &'a str>,
{
    cityjson_json::extract(model, cityobject_ids).map_err(Error::from)
}

pub fn append(target: &mut CityModel, source: &CityModel) -> Result<()> {
    cityjson_json::append(target, source).map_err(Error::from)
}

pub fn merge<I>(models: I) -> Result<CityModel>
where
    I: IntoIterator<Item = CityModel>,
{
    cityjson_json::merge(models).map_err(Error::from)
}
