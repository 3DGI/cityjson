use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{BorrowedCityModel, CityModel, OwnedCityModel, VertexRef};
use serde::Serialize;

pub use crate::de::ParseStringStorage;
use crate::errors::Result;

/// Parse a `CityJSON` document into a [`CityModel`].
///
/// # Errors
///
/// Returns an error if the input is not valid `CityJSON`.
pub fn from_str<'de, SS>(input: &'de str) -> Result<CityModel<u32, SS>>
where
    SS: ParseStringStorage<'de>,
    SS::String: From<&'de str>,
{
    crate::de::from_str_generic::<SS>(input)
}

/// Parse a `CityJSON` document into an [`OwnedCityModel`].
///
/// # Errors
///
/// Returns an error if the input is not valid `CityJSON`.
pub fn from_str_owned(input: &str) -> Result<OwnedCityModel> {
    crate::de::from_str_owned(input)
}

/// Parse a `CityJSON` document into a [`BorrowedCityModel`].
///
/// # Errors
///
/// Returns an error if the input is not valid `CityJSON`.
pub fn from_str_borrowed(input: &str) -> Result<BorrowedCityModel<'_>> {
    crate::de::from_str_borrowed(input)
}

/// Serialize a [`CityModel`] to a `CityJSON` string.
///
/// # Errors
///
/// Returns an error if the model cannot be serialized.
pub fn to_string<VR, SS>(model: &CityModel<VR, SS>) -> Result<String>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    Ok(serde_json::to_string(&as_json(model))?)
}

/// Serialize a [`CityModel`] to a `CityJSON` string, validating default themes.
///
/// # Errors
///
/// Returns an error if the model fails validation or cannot be serialized.
pub fn to_string_validated<VR, SS>(model: &CityModel<VR, SS>) -> Result<String>
where
    VR: VertexRef + Serialize,
    SS: StringStorage,
{
    model.validate_default_themes()?;
    Ok(serde_json::to_string(&as_json(model))?)
}

pub fn as_json<VR, SS>(model: &CityModel<VR, SS>) -> SerializableCityModel<'_, VR, SS>
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
