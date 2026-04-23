//! A quick start example on the main parts of a `CityJSON` model.

use cityjson::v2_0::{CityJSONVersion, CityModel, CityModelType};
fn main() {
    let model = CityModel::<u32>::new(CityModelType::CityJSON);

    assert_eq!(model.version(), Some(CityJSONVersion::V2_0));
    assert!(model.cityobjects().is_empty());

    assert_eq!(model.iter_geometries().count(), 0);
    assert_eq!(model.iter_geometry_templates().count(), 0);
    assert!(model.template_vertices().is_empty());
    assert_eq!(model.iter_semantics().count(), 0);
    assert_eq!(model.iter_materials().count(), 0);
    assert_eq!(model.iter_textures().count(), 0);
    assert!(model.vertices_texture().is_empty());

    assert!(model.vertices().is_empty());
    assert_eq!(model.transform(), None);

    assert_eq!(model.metadata(), None);
    assert_eq!(model.extra(), None);
    assert_eq!(model.extensions(), None);
}
