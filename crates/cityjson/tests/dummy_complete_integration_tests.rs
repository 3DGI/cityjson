use std::collections::HashMap;
use cityjson::prelude::*;
use cityjson::v1_1::*;

#[test]
fn build_dummy_complete_owned() -> Result<()> {
    // A CityModel for CityJSON v1.1, that uses u32 indices and owned strings.
    let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();

    // Set metadata
    let metadata = model.metadata_mut();
    metadata.set_identifier("eaeceeaa-3f66-429a-b81d-bbc6140b8c1c");
    metadata.set_reference_system("https://www.opengis.net/def/crs/EPSG/0/2355");
    metadata.set_contact_name("3DGI");
    metadata.set_email_address("info@3dgi.nl");

    // Set extra root properties (see https://www.cityjson.org/specs/1.1.3/#case-1-adding-new-properties-at-the-root-of-a-document)
    let extra = model.extra_mut();
    let mut census_map = HashMap::new();
    census_map.insert("percent_men".to_string(), Box::new(AttributeValue::Float(49.5)));
    census_map.insert("percent_women".to_string(), Box::new(AttributeValue::Float(51.5)));
    extra.insert("+census".to_string(), AttributeValue::Map(census_map));

    println!("{}", &model);
    Ok(())
}