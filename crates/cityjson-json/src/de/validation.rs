use cityjson::resources::storage::StringStorage;
use cityjson::v2_0::{
    CityObjectType, ContactRole, ContactType, ImageType, LoD, SemanticType, TextureType, WrapMode,
};
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

pub(crate) fn parse_lod(value: Option<&str>) -> Result<Option<LoD>> {
    match value {
        None => Ok(None),
        Some("0") => Ok(Some(LoD::LoD0)),
        Some("0.0") => Ok(Some(LoD::LoD0_0)),
        Some("0.1") => Ok(Some(LoD::LoD0_1)),
        Some("0.2") => Ok(Some(LoD::LoD0_2)),
        Some("0.3") => Ok(Some(LoD::LoD0_3)),
        Some("1") => Ok(Some(LoD::LoD1)),
        Some("1.0") => Ok(Some(LoD::LoD1_0)),
        Some("1.1") => Ok(Some(LoD::LoD1_1)),
        Some("1.2") => Ok(Some(LoD::LoD1_2)),
        Some("1.3") => Ok(Some(LoD::LoD1_3)),
        Some("2") => Ok(Some(LoD::LoD2)),
        Some("2.0") => Ok(Some(LoD::LoD2_0)),
        Some("2.1") => Ok(Some(LoD::LoD2_1)),
        Some("2.2") => Ok(Some(LoD::LoD2_2)),
        Some("2.3") => Ok(Some(LoD::LoD2_3)),
        Some("3") => Ok(Some(LoD::LoD3)),
        Some("3.0") => Ok(Some(LoD::LoD3_0)),
        Some("3.1") => Ok(Some(LoD::LoD3_1)),
        Some("3.2") => Ok(Some(LoD::LoD3_2)),
        Some("3.3") => Ok(Some(LoD::LoD3_3)),
        Some(other) => Err(Error::InvalidValue(format!(
            "unsupported geometry lod value '{other}'"
        ))),
    }
}

pub(crate) fn parse_contact_role(value: &str) -> Result<ContactRole> {
    match value {
        "author" => Ok(ContactRole::Author),
        "co-author" => Ok(ContactRole::CoAuthor),
        "processor" => Ok(ContactRole::Processor),
        "pointOfContact" => Ok(ContactRole::PointOfContact),
        "owner" => Ok(ContactRole::Owner),
        "user" => Ok(ContactRole::User),
        "distributor" => Ok(ContactRole::Distributor),
        "originator" => Ok(ContactRole::Originator),
        "custodian" => Ok(ContactRole::Custodian),
        "resourceProvider" => Ok(ContactRole::ResourceProvider),
        "rightsHolder" => Ok(ContactRole::RightsHolder),
        "sponsor" => Ok(ContactRole::Sponsor),
        "principalInvestigator" => Ok(ContactRole::PrincipalInvestigator),
        "stakeholder" => Ok(ContactRole::Stakeholder),
        "publisher" => Ok(ContactRole::Publisher),
        _ => Err(Error::InvalidValue(format!(
            "unsupported pointOfContact.role value '{value}'"
        ))),
    }
}

pub(crate) fn parse_contact_type(value: &str) -> Result<ContactType> {
    match value {
        "individual" => Ok(ContactType::Individual),
        "organization" => Ok(ContactType::Organization),
        _ => Err(Error::InvalidValue(format!(
            "unsupported pointOfContact.contactType value '{value}'"
        ))),
    }
}

pub(crate) fn parse_image_type(value: &str) -> Result<ImageType> {
    match value {
        "PNG" => Ok(ImageType::Png),
        "JPG" => Ok(ImageType::Jpg),
        _ => Err(Error::InvalidValue(format!(
            "unsupported texture image type '{value}'"
        ))),
    }
}

pub(crate) fn parse_wrap_mode(value: &str) -> Result<WrapMode> {
    match value {
        "wrap" => Ok(WrapMode::Wrap),
        "mirror" => Ok(WrapMode::Mirror),
        "clamp" => Ok(WrapMode::Clamp),
        "border" => Ok(WrapMode::Border),
        "none" => Ok(WrapMode::None),
        _ => Err(Error::InvalidValue(format!(
            "unsupported texture wrapMode value '{value}'"
        ))),
    }
}

pub(crate) fn parse_texture_type(value: &str) -> Result<TextureType> {
    match value {
        "unknown" => Ok(TextureType::Unknown),
        "specific" => Ok(TextureType::Specific),
        "typical" => Ok(TextureType::Typical),
        _ => Err(Error::InvalidValue(format!(
            "unsupported texture textureType value '{value}'"
        ))),
    }
}

pub(crate) fn parse_semantic_type<'de, SS: StringStorage>(
    value: &'de str,
) -> Result<SemanticType<SS>>
where
    SS::String: From<&'de str>,
{
    Ok(match value {
        "RoofSurface" => SemanticType::RoofSurface,
        "GroundSurface" => SemanticType::GroundSurface,
        "WallSurface" => SemanticType::WallSurface,
        "ClosureSurface" => SemanticType::ClosureSurface,
        "OuterCeilingSurface" => SemanticType::OuterCeilingSurface,
        "OuterFloorSurface" => SemanticType::OuterFloorSurface,
        "Window" => SemanticType::Window,
        "Door" => SemanticType::Door,
        "InteriorWallSurface" => SemanticType::InteriorWallSurface,
        "CeilingSurface" => SemanticType::CeilingSurface,
        "FloorSurface" => SemanticType::FloorSurface,
        "WaterSurface" => SemanticType::WaterSurface,
        "WaterGroundSurface" => SemanticType::WaterGroundSurface,
        "WaterClosureSurface" => SemanticType::WaterClosureSurface,
        "TrafficArea" => SemanticType::TrafficArea,
        "AuxiliaryTrafficArea" => SemanticType::AuxiliaryTrafficArea,
        "TransportationMarking" => SemanticType::TransportationMarking,
        "TransportationHole" => SemanticType::TransportationHole,
        _ if value.starts_with('+') => SemanticType::Extension(SS::String::from(value)),
        _ => {
            return Err(Error::InvalidValue(format!(
                "invalid Semantic type: {value}"
            )));
        }
    })
}

pub(crate) fn parse_cityobject_type<'de, SS: StringStorage>(
    value: &'de str,
) -> Result<CityObjectType<SS>>
where
    SS::String: From<&'de str>,
{
    Ok(match value {
        "Bridge" => CityObjectType::Bridge,
        "BridgePart" => CityObjectType::BridgePart,
        "BridgeInstallation" => CityObjectType::BridgeInstallation,
        "BridgeConstructiveElement" => CityObjectType::BridgeConstructiveElement,
        "BridgeRoom" => CityObjectType::BridgeRoom,
        "BridgeFurniture" => CityObjectType::BridgeFurniture,
        "Building" => CityObjectType::Building,
        "BuildingPart" => CityObjectType::BuildingPart,
        "BuildingInstallation" => CityObjectType::BuildingInstallation,
        "BuildingConstructiveElement" => CityObjectType::BuildingConstructiveElement,
        "BuildingFurniture" => CityObjectType::BuildingFurniture,
        "BuildingStorey" => CityObjectType::BuildingStorey,
        "BuildingRoom" => CityObjectType::BuildingRoom,
        "BuildingUnit" => CityObjectType::BuildingUnit,
        "CityFurniture" => CityObjectType::CityFurniture,
        "CityObjectGroup" => CityObjectType::CityObjectGroup,
        "Default" => CityObjectType::Default,
        "GenericCityObject" => CityObjectType::GenericCityObject,
        "LandUse" => CityObjectType::LandUse,
        "OtherConstruction" => CityObjectType::OtherConstruction,
        "PlantCover" => CityObjectType::PlantCover,
        "SolitaryVegetationObject" => CityObjectType::SolitaryVegetationObject,
        "TINRelief" => CityObjectType::TINRelief,
        "WaterBody" => CityObjectType::WaterBody,
        "Road" => CityObjectType::Road,
        "Railway" => CityObjectType::Railway,
        "Waterway" => CityObjectType::Waterway,
        "TransportSquare" => CityObjectType::TransportSquare,
        "Tunnel" => CityObjectType::Tunnel,
        "TunnelPart" => CityObjectType::TunnelPart,
        "TunnelInstallation" => CityObjectType::TunnelInstallation,
        "TunnelConstructiveElement" => CityObjectType::TunnelConstructiveElement,
        "TunnelHollowSpace" => CityObjectType::TunnelHollowSpace,
        "TunnelFurniture" => CityObjectType::TunnelFurniture,
        _ if value.starts_with('+') => CityObjectType::Extension(SS::String::from(value)),
        _ => {
            return Err(Error::InvalidValue(format!(
                "invalid CityObject type '{value}'"
            )));
        }
    })
}
