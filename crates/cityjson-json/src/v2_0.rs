use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{BorrowedCityModel, CityModel, OwnedCityModel, VertexRef};
use serde::Serialize;

use crate::errors::Result;

pub fn from_str_owned(input: &str) -> Result<OwnedCityModel> {
    crate::de::from_str_owned(input)
}

pub fn from_str_borrowed<'a>(input: &'a str) -> Result<BorrowedCityModel<'a>> {
    crate::de::from_str_borrowed(input)
}

pub fn to_string<VR, SS>(model: &CityModel<VR, SS>) -> Result<String>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    Ok(serde_json::to_string(&as_json(model))?)
}

pub fn to_string_validated<VR, SS>(model: &CityModel<VR, SS>) -> Result<String>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    model.validate_default_themes()?;
    Ok(serde_json::to_string(&as_json(model))?)
}

pub fn as_json<'a, VR, SS>(model: &'a CityModel<VR, SS>) -> SerializableCityModel<'a, VR, SS>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    SerializableCityModel { model }
}

pub struct SerializableCityModel<'a, VR, SS>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    pub(crate) model: &'a CityModel<VR, SS>,
}

impl<VR, SS> Serialize for SerializableCityModel<'_, VR, SS>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        crate::ser::citymodel_to_json_value(self.model)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}
