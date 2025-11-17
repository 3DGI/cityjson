//! CityModel type for the nested backend.
//!

use crate::backend::nested::appearance::Appearance;
use crate::backend::nested::attributes::Attributes;
use crate::backend::nested::cityobject::CityObjects;
use crate::backend::nested::coordinate::Vertices;
use crate::backend::nested::geometry::GeometryTemplates;
use crate::prelude::{QuantizedCoordinate, StringStorage};
use crate::v2_0::extension::Extensions;
use crate::v2_0::metadata::Metadata;
use crate::v2_0::transform::Transform;
#[derive(Debug, Clone)]
pub struct CityModel<SS: StringStorage> {
    pub id: Option<SS::String>,
    pub type_cm: crate::CityModelType,
    pub version: Option<crate::CityJSONVersion>,
    pub transform: Option<Transform>,
    pub cityobjects: CityObjects<SS>,
    pub metadata: Option<Metadata<SS>>,
    pub appearance: Option<Appearance<SS>>,
    pub geometry_templates: Option<GeometryTemplates<SS>>,
    pub extra: Option<Attributes<SS>>,
    pub extensions: Option<Extensions<SS>>,
    pub vertices: Vertices<u32, QuantizedCoordinate>,
}
