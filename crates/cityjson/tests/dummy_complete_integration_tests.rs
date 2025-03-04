use cityjson::prelude::*;
use cityjson::v1_1::*;

#[test]
fn build_dummy_complete() -> Result<()> {
    // A CityModel for CityJSON v1.1, that uses u32 indices and owned strings.
    let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new();

    // Set metadata
    let metadata = model.metadata_mut();
    metadata.set_identifier("eaeceeaa-3f66-429a-b81d-bbc6140b8c1c");
    metadata.set_reference_system("https://www.opengis.net/def/crs/EPSG/0/2355");
    metadata.set_contact_name("3DGI");
    metadata.set_email_address("info@3dgi.nl");


    println!("{}", &model);
    Ok(())
}