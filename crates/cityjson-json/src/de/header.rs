use cityjson::{CityJSONVersion, CityModelType};

use crate::errors::{Error, Result};

pub(crate) struct RootHeader {
    pub(crate) type_citymodel: CityModelType,
    pub(crate) version: CityJSONVersion,
}

pub(crate) fn parse_root_header(type_name: &str, version: Option<&str>) -> Result<RootHeader> {
    let type_citymodel = CityModelType::try_from(type_name)
        .map_err(|_| Error::UnsupportedType(type_name.to_owned()))?;
    let version = match (type_citymodel, version) {
        (CityModelType::CityJSONFeature, None) => CityJSONVersion::V2_0,
        (_, Some(version)) => CityJSONVersion::try_from(version)
            .map_err(|_| Error::UnsupportedVersion(version.to_owned()))?,
        (_, None) => return Err(Error::MalformedRootObject("missing root version")),
    };

    Ok(RootHeader {
        type_citymodel,
        version,
    })
}
