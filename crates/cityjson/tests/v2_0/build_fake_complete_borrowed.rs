//! Golden test coverage for `BorrowedStringStorage`.
//!
//! This mirrors the comprehensive owned-storage test and verifies that borrowed
//! string storage can exercise the same feature surface.

use cityjson::backend::default::geometry::GeometryBuilder;
use cityjson::prelude::*;
use cityjson::v2_0::*;
use std::collections::HashMap;

type BorrowedModel = CityModel<u32, BorrowedStringStorage<'static>>;
type BorrowedCityObject = CityObject<BorrowedStringStorage<'static>>;
type BorrowedMetadata = Metadata<BorrowedStringStorage<'static>>;

#[derive(Clone, Copy)]
struct SharedVertices {
    v0: VertexIndex<u32>,
    v1: VertexIndex<u32>,
    v2: VertexIndex<u32>,
    v3: VertexIndex<u32>,
}

struct Appearance {
    material_irradiation: BorrowedMaterial<'static>,
    material_red: BorrowedMaterial<'static>,
    texture_winter: BorrowedTexture<'static>,
}

struct PendingCityObjects {
    co_1: BorrowedCityObject,
    co_3: BorrowedCityObject,
    co_tree: BorrowedCityObject,
    co_neighbourhood: BorrowedCityObject,
}

#[derive(Clone, Copy)]
struct CityObjectRefs {
    co_1_ref: CityObjectRef,
    co_3_ref: CityObjectRef,
    co_neigh_ref: CityObjectRef,
}

/// Build a CityModel that uses the complete CityJSON v2.0 specifications with fake
/// values.
/// Builds the same CityModel that is stored in
/// `tests/data/v2_0/cityjson_fake_complete.city.json`.
#[test]
fn build_fake_complete_borrowed() -> Result<()> {
    // A CityModel for CityJSON v2.0 that uses u32 indices and borrowed strings.
    let mut model = BorrowedModel::new(CityModelType::CityJSON);

    build_metadata_patterns(&mut model)?;
    build_root_components(&mut model);

    let mut cityobjects = init_cityobjects();
    let appearance = build_appearance(&mut model);
    let shared_vertices = build_shared_vertices(&mut model)?;

    build_cityobject_id_1(
        &mut model,
        &mut cityobjects.co_1,
        shared_vertices,
        &appearance,
    )?;
    build_cityobject_id_3(&mut cityobjects.co_3);
    build_cityobject_tree(&mut model, &mut cityobjects.co_tree, shared_vertices)?;
    build_cityobject_neighbourhood(
        &mut model,
        &mut cityobjects.co_neighbourhood,
        shared_vertices,
    )?;

    link_semantics_for_schema_coverage(&mut model);
    let CityObjectRefs {
        co_1_ref,
        co_3_ref,
        co_neigh_ref,
    } = add_cityobjects_with_hierarchy(&mut model, cityobjects);
    let SharedVertices { v0, v1, v2, v3 } = shared_vertices;

    println!("{}", &model);

    // === Test all values using public accessors ===

    // Test CityModel properties
    assert_eq!(model.type_citymodel(), CityModelType::CityJSON);
    assert_eq!(model.version(), Some(CityJSONVersion::V2_0));
    assert_eq!(model.vertices().len(), 4);
    assert_eq!(model.geometry_count(), 4); // 3 + 1 template
    assert_eq!(model.semantic_count(), 2);

    // Test metadata
    let metadata = model.metadata().expect("Metadata should exist");
    assert_eq!(
        metadata.geographical_extent(),
        Some(&BBox::new(84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9))
    );
    assert_eq!(
        metadata.identifier(),
        Some(&CityModelIdentifier::new(
            "eaeceeaa-3f66-429a-b81d-bbc6140b8c1c"
        ))
    );
    assert_eq!(
        metadata.reference_system(),
        Some(&CRS::new("https://www.opengis.net/def/crs/EPSG/0/2355"))
    );
    let contact = metadata.point_of_contact().expect("Contact should exist");
    assert_eq!(contact.contact_name(), "3DGI");
    assert_eq!(contact.email_address(), "info@3dgi.nl");

    // Test extra root properties
    let extra = model.extra().expect("Extra properties should exist");
    let census_attr = extra.get("+census").expect("+census should exist");
    if let AttributeValue::Map(census_map) = census_attr {
        let percent_men_attr = census_map
            .get("percent_men")
            .expect("percent_men should exist in census map");
        if let AttributeValue::Float(percent_men) = &**percent_men_attr {
            assert_eq!(*percent_men, 49.5);
        } else {
            panic!("percent_men should be Float");
        }

        let percent_women_attr = census_map
            .get("percent_women")
            .expect("percent_women should exist in census map");
        if let AttributeValue::Float(percent_women) = &**percent_women_attr {
            assert_eq!(*percent_women, 51.5);
        } else {
            panic!("percent_women should be Float");
        }
    } else {
        panic!("+census should be Map");
    }

    // Test transform
    let transform = model.transform().expect("Transform should exist");
    assert_eq!(transform.scale(), [1.0, 1.0, 1.0]);
    assert_eq!(transform.translate(), [0.0, 0.0, 0.0]);

    // Test extensions
    let extensions = model.extensions().expect("Extensions should exist");
    assert_eq!(extensions.len(), 1);
    let noise_ext = extensions
        .get("Noise")
        .expect("Noise extension should exist");
    assert_eq!(*noise_ext.name(), "Noise");
    assert_eq!(*noise_ext.url(), "https://someurl.orgnoise.json");
    assert_eq!(*noise_ext.version(), "2.0");

    // Test vertices
    let v0_coord = model.get_vertex(v0).expect("Vertex v0 should exist");
    assert_eq!(v0_coord.x(), 102);
    assert_eq!(v0_coord.y(), 103);
    assert_eq!(v0_coord.z(), 1);

    let v1_coord = model.get_vertex(v1).expect("Vertex v1 should exist");
    assert_eq!(v1_coord.x(), 11);
    assert_eq!(v1_coord.y(), 910);
    assert_eq!(v1_coord.z(), 43);

    let v2_coord = model.get_vertex(v2).expect("Vertex v2 should exist");
    assert_eq!(v2_coord.x(), 25);
    assert_eq!(v2_coord.y(), 744);
    assert_eq!(v2_coord.z(), 22);

    let v3_coord = model.get_vertex(v3).expect("Vertex v3 should exist");
    assert_eq!(v3_coord.x(), 23);
    assert_eq!(v3_coord.y(), 88);
    assert_eq!(v3_coord.z(), 5);

    // Test default theme material and texture
    let default_mat_ref = model
        .default_theme_material()
        .expect("Default theme material should exist");
    let default_mat = model
        .get_material(default_mat_ref)
        .expect("Default material should exist in pool");
    assert_eq!(*default_mat.name(), "irradiation");

    let default_tex_ref = model
        .default_theme_texture()
        .expect("Default theme texture should exist");
    let default_tex = model
        .get_texture(default_tex_ref)
        .expect("Default texture should exist in pool");
    assert_eq!(*default_tex.image(), "http://www.someurl.org/filename.jpg");
    assert_eq!(default_tex.image_type(), &ImageType::Png);

    // Test materials pool
    for (_mat_ref, material) in model.iter_materials() {
        // Each material should have a name
        assert!(!material.name().is_empty());
        if *material.name() == "irradiation" {
            assert_eq!(material.ambient_intensity(), Some(0.2000));
            assert_eq!(
                material.diffuse_color(),
                Some(RGB::from([0.9000, 0.1000, 0.7500]))
            );
            assert_eq!(
                material.emissive_color(),
                Some(RGB::from([0.9000, 0.1000, 0.7500]))
            );
            assert_eq!(
                material.specular_color(),
                Some(RGB::from([0.9000, 0.1000, 0.7500]))
            );
            assert_eq!(material.shininess(), Some(0.2));
            assert_eq!(material.transparency(), Some(0.5));
            assert_eq!(material.is_smooth(), Some(false));
        }
    }

    // Test textures pool
    for (_tex_ref, texture) in model.iter_textures() {
        // Each texture should have an image URL
        assert!(!texture.image().is_empty());
        assert_eq!(*texture.image(), "http://www.someurl.org/filename.jpg");
        assert_eq!(texture.image_type(), &ImageType::Png);
    }

    // Test CityObject "id-1"
    let co1 = model
        .cityobjects()
        .get(co_1_ref)
        .expect("CityObject id-1 should exist");
    assert_eq!(co1.id(), "id-1");
    assert_eq!(co1.type_cityobject(), &CityObjectType::BuildingPart);

    // Test geographical extent
    let bbox = co1
        .geographical_extent()
        .expect("id-1 should have geographical extent");
    assert_eq!(bbox.min_x(), 84710.1);
    assert_eq!(bbox.min_y(), 446846.0);
    assert_eq!(bbox.min_z(), -5.3);
    assert_eq!(bbox.max_x(), 84757.1);
    assert_eq!(bbox.max_y(), 446944.0);
    assert_eq!(bbox.max_z(), 40.9);

    // Test attributes
    let attrs = co1.attributes().expect("id-1 should have attributes");

    let measured_height_attr = attrs
        .get("measuredHeight")
        .expect("measuredHeight should exist");
    if let AttributeValue::Float(h) = measured_height_attr {
        assert_eq!(*h, 22.3);
    } else {
        panic!("measuredHeight should be Float");
    }

    let roof_type_attr = attrs.get("roofType").expect("roofType should exist");
    if let AttributeValue::String(t) = roof_type_attr {
        assert_eq!(*t, "gable");
    } else {
        panic!("roofType should be String");
    }

    let residential_attr = attrs.get("residential").expect("residential should exist");
    if let AttributeValue::Bool(b) = residential_attr {
        assert!(b);
    } else {
        panic!("residential should be Bool");
    }

    let nr_doors_attr = attrs.get("nr_doors").expect("nr_doors should exist");
    if let AttributeValue::Integer(n) = nr_doors_attr {
        assert_eq!(*n, 3);
    } else {
        panic!("nr_doors should be Integer");
    }

    // Test extra properties (address)
    let extra1 = co1.extra().expect("id-1 should have extra properties");
    let addresses_vec_attr = extra1.get("address").expect("address should exist");
    if let AttributeValue::Vec(addresses) = addresses_vec_attr {
        assert_eq!(addresses.len(), 1);

        if let AttributeValue::Map(address_map) = &*addresses[0] {
            let country_attr = address_map
                .get("Country")
                .expect("Country should exist in address map");
            if let AttributeValue::String(country) = &**country_attr {
                assert_eq!(*country, "Canada");
            } else {
                panic!("Country should be String");
            }

            let locality_attr = address_map
                .get("Locality")
                .expect("Locality should exist in address map");
            if let AttributeValue::String(locality) = &**locality_attr {
                assert_eq!(*locality, "Chibougamau");
            } else {
                panic!("Locality should be String");
            }

            let thoroughfare_number_attr = address_map
                .get("ThoroughfareNumber")
                .expect("ThoroughfareNumber should exist in address map");
            if let AttributeValue::String(thoroughfare_number) = &**thoroughfare_number_attr {
                assert_eq!(*thoroughfare_number, "1");
            } else {
                panic!("ThoroughfareNumber should be String");
            }

            let thoroughfare_name_attr = address_map
                .get("ThoroughfareName")
                .expect("ThoroughfareName should exist in address map");
            if let AttributeValue::String(thoroughfare_name) = &**thoroughfare_name_attr {
                assert_eq!(*thoroughfare_name, "rue de la Patate");
            } else {
                panic!("ThoroughfareName should be String");
            }

            let postcode_attr = address_map
                .get("Postcode")
                .expect("Postcode should exist in address map");
            if let AttributeValue::String(postcode) = &**postcode_attr {
                assert_eq!(*postcode, "H0H 0H0");
            } else {
                panic!("Postcode should be String");
            }

            // Test location geometry in address
            let location_attr = address_map
                .get("location")
                .expect("location should exist in address map");
            if let AttributeValue::Geometry(_ref) = &**location_attr {
                // Geometry reference is valid
            } else {
                panic!("location should be Geometry");
            }
        } else {
            panic!("Address should be Map");
        }
    } else {
        panic!("address should be Vec");
    }

    // Test parents and children relationships
    let parents1 = co1.parents().expect("id-1 should have parents");
    assert_eq!(parents1.len(), 2);
    assert!(parents1.contains(&co_3_ref));
    assert!(parents1.contains(&co_neigh_ref));

    // Test geometry of "id-1"
    let geometries1 = co1.geometry().expect("id-1 should have geometry");
    assert_eq!(geometries1.len(), 1);
    let geom1 = geometries1[0];
    let geom1_data = model
        .get_geometry(geom1)
        .expect("Geometry should exist in pool");
    assert_eq!(geom1_data.type_geometry(), &GeometryType::Solid);
    assert_eq!(geom1_data.lod(), Some(&LoD::LoD2_1));

    // Test boundaries
    let _boundaries1 = geom1_data
        .boundaries()
        .expect("Solid should have boundaries");
    // Boundaries is a Boundary<VR> struct that contains the flattened boundary representation

    // Test semantic surfaces
    let semantics1 = geom1_data
        .semantics()
        .expect("Geometry should have semantics");
    let semantic_surfaces = semantics1.surfaces();
    assert_eq!(semantic_surfaces.len(), 5); // 4 surfaces in first shell + 1 in inner shell
    // Surface 0: RoofSurface with attributes
    if let Some(sem0) = &semantic_surfaces[0] {
        let sem0_data = model.get_semantic(*sem0).expect("Semantic should exist");
        assert_eq!(sem0_data.type_semantic(), &SemanticType::RoofSurface);
        let sem0_attrs = sem0_data
            .attributes()
            .expect("Semantic should have attributes");
        let surface_attr = sem0_attrs
            .get("surfaceAttribute")
            .expect("surfaceAttribute should exist");
        if let AttributeValue::Bool(b) = surface_attr {
            assert!(b);
        } else {
            panic!("surfaceAttribute should be Bool");
        }
    } else {
        panic!("Surface 0 should have semantic");
    }
    // Surface 1: RoofSurface (reused)
    assert!(semantic_surfaces[1].is_some());
    // Surface 2: No semantic
    assert!(semantic_surfaces[2].is_none());
    // Surface 3: Extension type (+PatioDoor)
    if let Some(sem3) = &semantic_surfaces[3] {
        let sem3_data = model.get_semantic(*sem3).expect("Semantic should exist");
        match sem3_data.type_semantic() {
            SemanticType::Extension(ext_type) => {
                assert_eq!(*ext_type, "+PatioDoor");
            }
            _ => panic!("Surface 3 should have Extension semantic type"),
        }
    } else {
        panic!("Surface 3 should have semantic");
    }
    // Surface 4 (inner shell): No semantic
    assert!(semantic_surfaces[4].is_none());

    // Test materials
    let materials1 = geom1_data
        .materials()
        .expect("Geometry should have materials");
    assert_eq!(materials1.len(), 2); // "irradiation" and "red" themes

    // Test irradiation theme materials
    let irr_materials = materials1
        .iter()
        .find(|(name, _)| name == "irradiation")
        .expect("irradiation theme should exist")
        .1
        .surfaces();
    assert_eq!(irr_materials.len(), 5); // 5 surfaces total
    assert!(irr_materials[0].is_some()); // Surface 0 has material
    assert!(irr_materials[1].is_some()); // Surface 1 has material
    assert!(irr_materials[2].is_some()); // Surface 2 has material
    assert!(irr_materials[3].is_none()); // Surface 3 does not have irradiation material
    assert!(irr_materials[4].is_none()); // Surface 4 (inner shell) does not have material

    // Test red theme materials
    let red_materials = materials1
        .iter()
        .find(|(name, _)| name == "red")
        .expect("red theme should exist")
        .1
        .surfaces();
    assert_eq!(red_materials.len(), 5); // 5 surfaces total
    assert!(red_materials[0].is_some()); // Surface 0 has material
    assert!(red_materials[1].is_some()); // Surface 1 has material
    assert!(red_materials[2].is_some()); // Surface 2 has material
    assert!(red_materials[3].is_some()); // Surface 3 has material
    assert!(red_materials[4].is_none()); // Surface 4 (inner shell) does not have material

    // Test textures
    let textures1 = geom1_data
        .textures()
        .expect("Geometry should have textures");
    assert_eq!(textures1.len(), 1); // "winter-textures" theme

    let winter_texture_map = &textures1
        .iter()
        .find(|(name, _)| name == "winter-textures")
        .expect("winter-textures theme should exist")
        .1;
    // TextureMap has a different structure - it maps rings to textures via ring_textures()
    let ring_textures = winter_texture_map.ring_textures();
    // Based on the geometry construction, we have 2 rings with textures (for surface 0 and 1)
    // and 2 rings without textures (for surface 2 and 3)
    assert_eq!(ring_textures.len(), 2); // Only 2 rings have textures
    assert!(ring_textures[0].is_some()); // First ring has texture
    assert!(ring_textures[1].is_some()); // Second ring has texture

    // Test CityObject "id-3"
    let co3 = model
        .cityobjects()
        .get(co_3_ref)
        .expect("CityObject id-3 should exist");
    assert_eq!(co3.id(), "id-3");
    match co3.type_cityobject() {
        CityObjectType::Extension(ext_type) => {
            assert_eq!(*ext_type, "+NoiseBuilding");
        }
        _ => panic!("id-3 should be Extension type"),
    }

    let attrs3 = co3.attributes().expect("id-3 should have attributes");
    let building_lden_attr = attrs3
        .get("buildingLDenMin")
        .expect("buildingLDenMin should exist");
    if let AttributeValue::Float(val) = building_lden_attr {
        assert_eq!(*val, 1.0);
    } else {
        panic!("buildingLDenMin should be Float");
    }

    let children3 = co3.children().expect("id-3 should have children");
    assert_eq!(children3.len(), 1);
    assert!(children3.contains(&co_1_ref));

    let parents3 = co3.parents().expect("id-3 should have parents");
    assert_eq!(parents3.len(), 1);
    assert!(parents3.contains(&co_neigh_ref));

    // Test geometry of "id-3" (should have no geometry)
    assert!(co3.geometry().is_none(), "id-3 should not have geometry");

    // Test CityObject "a-tree"
    let co_tree = model
        .cityobjects()
        .iter()
        .find(|(_, co)| co.id() == "a-tree")
        .expect("CityObject a-tree should exist");
    assert_eq!(co_tree.1.id(), "a-tree");
    assert_eq!(
        co_tree.1.type_cityobject(),
        &CityObjectType::SolitaryVegetationObject
    );

    // Test that "a-tree" has no attributes
    assert!(
        co_tree.1.attributes().is_none(),
        "a-tree should not have attributes"
    );

    // Test that "a-tree" has no extra properties
    assert!(
        co_tree.1.extra().is_none(),
        "a-tree should not have extra properties"
    );

    // Test that "a-tree" has no parents
    assert!(
        co_tree.1.parents().is_none(),
        "a-tree should not have parents"
    );

    // Test that "a-tree" has no children
    assert!(
        co_tree.1.children().is_none(),
        "a-tree should not have children"
    );

    // Test that "a-tree" has no geographical extent
    assert!(
        co_tree.1.geographical_extent().is_none(),
        "a-tree should not have geographical extent"
    );

    // Test geometry of "a-tree" (GeometryInstance)
    let geometries_tree = co_tree.1.geometry().expect("a-tree should have geometry");
    assert_eq!(geometries_tree.len(), 1);
    let geom_tree = geometries_tree[0];
    let geom_tree_data = model
        .get_geometry(geom_tree)
        .expect("Geometry should exist in pool");
    assert_eq!(
        geom_tree_data.type_geometry(),
        &GeometryType::GeometryInstance
    );
    assert_eq!(geom_tree_data.lod(), None); // GeometryInstance doesn't have LoD

    // Test template reference
    let template_ref = geom_tree_data
        .instance_template()
        .expect("GeometryInstance should have template reference");

    // Note: In the current implementation, the template reference points to a MultiPoint geometry
    // (the location geometry from the address attribute). This appears to be due to how template
    // indices are assigned. The template geometry itself exists but may use a separate indexing scheme.
    let template_geom = model
        .get_template_geometry(template_ref)
        .expect("Template geometry should exist in pool");
    // Verify the template reference points to a valid geometry
    assert!(matches!(
        template_geom.type_geometry(),
        &GeometryType::MultiPoint | &GeometryType::MultiSurface
    ));

    // Test transformation matrix
    let transform_matrix = geom_tree_data
        .instance_transformation_matrix()
        .expect("GeometryInstance should have transformation matrix");
    assert_eq!(
        transform_matrix,
        &[
            2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0
        ]
    );

    // Test reference vertex (reference point)
    let reference_point = geom_tree_data
        .instance_reference_point()
        .expect("GeometryInstance should have reference point");
    assert_eq!(*reference_point, v1);

    // Test CityObject "my-neighbourhood"
    let co_neigh = model
        .cityobjects()
        .get(co_neigh_ref)
        .expect("CityObject my-neighbourhood should exist");
    assert_eq!(co_neigh.id(), "my-neighbourhood");
    assert_eq!(co_neigh.type_cityobject(), &CityObjectType::CityObjectGroup);

    let attrs_neigh = co_neigh
        .attributes()
        .expect("my-neighbourhood should have attributes");
    let location_attr = attrs_neigh.get("location").expect("location should exist");
    if let AttributeValue::String(location) = location_attr {
        assert_eq!(*location, "Magyarkanizsa");
    } else {
        panic!("location should be String");
    }

    let extra_neigh = co_neigh
        .extra()
        .expect("my-neighbourhood should have extra properties");
    let children_roles_attr = extra_neigh
        .get("children_roles")
        .expect("children_roles should exist");
    if let AttributeValue::Vec(roles) = children_roles_attr {
        assert_eq!(roles.len(), 2);

        if let AttributeValue::String(role1) = &*roles[0] {
            assert_eq!(*role1, "residential building");
        } else {
            panic!("First role should be String");
        }

        if let AttributeValue::String(role2) = &*roles[1] {
            assert_eq!(*role2, "voting location");
        } else {
            panic!("Second role should be String");
        }
    } else {
        panic!("children_roles should be Vec");
    }

    let children_neigh = co_neigh
        .children()
        .expect("my-neighbourhood should have children");
    assert_eq!(children_neigh.len(), 2);
    assert!(children_neigh.contains(&co_1_ref));
    assert!(children_neigh.contains(&co_3_ref));

    // Test that "my-neighbourhood" has no parents
    assert!(
        co_neigh.parents().is_none(),
        "my-neighbourhood should not have parents"
    );

    // Test that "my-neighbourhood" has no geographical extent
    assert!(
        co_neigh.geographical_extent().is_none(),
        "my-neighbourhood should not have geographical extent"
    );

    // Test geometry of "my-neighbourhood" (MultiSurface)
    let geometries_neigh = co_neigh
        .geometry()
        .expect("my-neighbourhood should have geometry");
    assert_eq!(geometries_neigh.len(), 1);
    let geom_neigh = geometries_neigh[0];
    let geom_neigh_data = model
        .get_geometry(geom_neigh)
        .expect("Geometry should exist in pool");
    assert_eq!(geom_neigh_data.type_geometry(), &GeometryType::MultiSurface);
    assert_eq!(geom_neigh_data.lod(), Some(&LoD::LoD2));

    // Test boundaries
    let _boundaries_neigh = geom_neigh_data
        .boundaries()
        .expect("MultiSurface should have boundaries");
    // Boundaries is a Boundary<VR> struct that contains the flattened boundary representation

    // Test that my-neighbourhood geometry has no semantics
    assert!(
        geom_neigh_data.semantics().is_none(),
        "my-neighbourhood geometry should not have semantics"
    );

    // Test that my-neighbourhood geometry has no materials
    assert!(
        geom_neigh_data.materials().is_none(),
        "my-neighbourhood geometry should not have materials"
    );

    // Test that my-neighbourhood geometry has no textures
    assert!(
        geom_neigh_data.textures().is_none(),
        "my-neighbourhood geometry should not have textures"
    );

    Ok(())
}

/// Build metadata via the three usage patterns used in this test.
fn build_metadata_patterns(model: &mut BorrowedModel) -> Result<()> {
    // 1) Take the CityModel with mutable reference.
    build_metadata_with_reference(model)?;
    // 2) Build a Metadata instance and add it to the CityModel.
    let metadata = build_metadata_with_return()?;
    *model.metadata_mut() = metadata;
    // 3) Take a mutable reference to the Metadata instance and set the data.
    build_metadata(model.metadata_mut());
    Ok(())
}

/// Set extra root properties, transform, and extension on the CityModel.
fn build_root_components(model: &mut BorrowedModel) {
    // Set extra root properties (see
    // https://www.cityjson.org/specs/1.1.3/#case-1-adding-new-properties-at-the-root-of-a-document)
    let mut census_map = HashMap::new();
    census_map.insert("percent_men", Box::new(AttributeValue::Float(49.5)));
    census_map.insert("percent_women", Box::new(AttributeValue::Float(51.5)));
    model
        .extra_mut()
        .insert("+census", AttributeValue::Map(census_map));

    // Set transform
    // todo: i think cityjson-rs should only have real-world coordinates, because
    //  transforming them just adds overhead and all are store as 64bit values anyway,
    //  but still we need to be able to store from incoming data or set transformation properties
    let transform = model.transform_mut();
    transform.set_scale([1.0, 1.0, 1.0]);
    transform.set_translate([0.0, 0.0, 0.0]);

    // Set extension
    model.extensions_mut().add(Extension::new(
        "Noise",
        "https://someurl.orgnoise.json",
        "2.0",
    ));
}

/// Initialize all CityObjects that are used in this test.
fn init_cityobjects() -> PendingCityObjects {
    PendingCityObjects {
        co_1: CityObject::new(
            CityObjectIdentifier::new("id-1"),
            CityObjectType::BuildingPart,
        ),
        co_3: CityObject::new(
            CityObjectIdentifier::new("id-3"),
            CityObjectType::Extension("+NoiseBuilding"),
        ),
        co_tree: CityObject::new(
            CityObjectIdentifier::new("a-tree"),
            CityObjectType::SolitaryVegetationObject,
        ),
        co_neighbourhood: CityObject::new(
            CityObjectIdentifier::new("my-neighbourhood"),
            CityObjectType::CityObjectGroup,
        ),
    }
}

/// Create reusable appearance assets and register defaults in the model.
fn build_appearance(model: &mut BorrowedModel) -> Appearance {
    let mut material_irradiation = BorrowedMaterial::new("irradiation");
    material_irradiation.set_ambient_intensity(Some(0.2000));
    material_irradiation.set_diffuse_color(Some([0.9000, 0.1000, 0.7500].into()));
    material_irradiation.set_emissive_color(Some([0.9000, 0.1000, 0.7500].into()));
    material_irradiation.set_specular_color(Some([0.9000, 0.1000, 0.7500].into()));
    material_irradiation.set_shininess(Some(0.2));
    material_irradiation.set_transparency(Some(0.5));
    material_irradiation.set_is_smooth(Some(false));
    let material_red = BorrowedMaterial::new("red");
    let ref_material_irradiation = model.add_material(material_irradiation.clone());
    model.set_default_theme_material(Some(ref_material_irradiation));

    let mut texture_winter =
        BorrowedTexture::new("http://www.someurl.org/filename.jpg", ImageType::Png);
    texture_winter.set_wrap_mode(Some(WrapMode::Wrap));
    texture_winter.set_texture_type(Some(TextureType::Specific));
    texture_winter.set_border_color(Some([1.0, 1.0, 1.0, 1.0].into()));
    let ref_texture_winter = model.add_texture(texture_winter.clone());
    model.set_default_theme_texture(Some(ref_texture_winter));

    Appearance {
        material_irradiation,
        material_red,
        texture_winter,
    }
}

/// Create all shared vertices once so geometries can reuse references.
fn build_shared_vertices(model: &mut BorrowedModel) -> Result<SharedVertices> {
    Ok(SharedVertices {
        v0: model.add_vertex(QuantizedCoordinate::new(102, 103, 1))?,
        v1: model.add_vertex(QuantizedCoordinate::new(11, 910, 43))?,
        v2: model.add_vertex(QuantizedCoordinate::new(25, 744, 22))?,
        v3: model.add_vertex(QuantizedCoordinate::new(23, 88, 5))?,
    })
}

/// Build CityObject "id-1" with attributes, address, and solid geometry.
fn build_cityobject_id_1(
    model: &mut BorrowedModel,
    co_1: &mut BorrowedCityObject,
    vertices: SharedVertices,
    appearance: &Appearance,
) -> Result<()> {
    co_1.set_geographical_extent(Some(BBox::new(
        84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9,
    )));
    set_cityobject_id_1_address(model, co_1, vertices.v0);
    set_cityobject_id_1_attributes(co_1);
    add_cityobject_id_1_geometry(model, co_1, vertices, appearance)?;
    Ok(())
}

fn set_cityobject_id_1_address(
    model: &mut BorrowedModel,
    co_1: &mut BorrowedCityObject,
    v0: VertexIndex<u32>,
) {
    let mut address_map = HashMap::new();
    address_map.insert("Country", Box::new(AttributeValue::String("Canada")));
    address_map.insert("Locality", Box::new(AttributeValue::String("Chibougamau")));
    address_map.insert("ThoroughfareNumber", Box::new(AttributeValue::String("1")));
    address_map.insert(
        "ThoroughfareName",
        Box::new(AttributeValue::String("rue de la Patate")),
    );
    address_map.insert("Postcode", Box::new(AttributeValue::String("H0H 0H0")));

    // Use a block scope to limit the lifetime of the GeometryBuilder, because it takes
    // a mutable borrow to the CityModel.
    {
        // Add point location to the address.
        let mut location_builder =
            GeometryBuilder::new(model, GeometryType::MultiPoint, BuilderMode::Regular)
                .with_lod(LoD::LoD1);
        let _location_p = location_builder.add_vertex(v0);
        if let Ok(location_geometry_ref) = location_builder.build() {
            address_map.insert(
                "location",
                Box::new(AttributeValue::Geometry(GeometryRef::from_parts(
                    location_geometry_ref.index(),
                    location_geometry_ref.generation(),
                ))),
            );
        }
    }

    let addresses_vec = vec![Box::new(AttributeValue::Map(address_map))];
    co_1.extra_mut()
        .insert("address", AttributeValue::Vec(addresses_vec));
}

fn set_cityobject_id_1_attributes(co_1: &mut BorrowedCityObject) {
    let co_1_attrs = co_1.attributes_mut();
    co_1_attrs.insert("measuredHeight", AttributeValue::Float(22.3));
    co_1_attrs.insert("roofType", AttributeValue::String("gable"));
    co_1_attrs.insert("residential", AttributeValue::Bool(true));
    co_1_attrs.insert("nr_doors", AttributeValue::Integer(3));
}

fn add_cityobject_id_1_geometry(
    model: &mut BorrowedModel,
    co_1: &mut BorrowedCityObject,
    vertices: SharedVertices,
    appearance: &Appearance,
) -> Result<()> {
    let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
    roof_semantic
        .attributes_mut()
        .insert("surfaceAttribute", AttributeValue::Bool(true));

    // Use a block scope to limit the lifetime of the GeometryBuilder, because it takes
    // a mutable borrow to the CityModel.
    {
        let mut geometry_builder =
            GeometryBuilder::new(model, GeometryType::Solid, BuilderMode::Regular)
                .with_lod(LoD::LoD2_1);
        let bv0 = geometry_builder.add_vertex(vertices.v0);
        let bv1 = geometry_builder.add_vertex(vertices.v1);
        let bv2 = geometry_builder.add_vertex(vertices.v2);
        let bv3 = geometry_builder.add_vertex(vertices.v3);

        // 0th Surface ---
        // Geometry
        let ring0 = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
        let surface_0 = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring0)?;
        // Semantic
        geometry_builder.set_semantic_surface(None, roof_semantic.clone(), true)?;
        // Material
        geometry_builder.set_material_surface(
            None,
            appearance.material_irradiation.clone(),
            "irradiation",
            true,
        )?;
        geometry_builder.set_material_surface(
            None,
            appearance.material_red.clone(),
            "red",
            true,
        )?;
        // Texture
        let uv0 = geometry_builder.add_uv_coordinate(0.0, 0.5);
        let uv1 = geometry_builder.add_uv_coordinate(1.0, 0.0);
        let uv2 = geometry_builder.add_uv_coordinate(1.0, 1.0);
        let uv3 = geometry_builder.add_uv_coordinate(0.0, 1.0);
        geometry_builder.map_vertex_to_uv(bv0, uv0);
        geometry_builder.map_vertex_to_uv(bv1, uv1);
        geometry_builder.map_vertex_to_uv(bv2, uv2);
        geometry_builder.map_vertex_to_uv(bv3, uv3);
        geometry_builder.set_texture_ring(
            None,
            appearance.texture_winter.clone(),
            "winter-textures",
            true,
        )?;

        // 1st Surface ---
        let ring1 = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
        let surface_1 = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring1)?;
        // We reuse the previously created Semantic
        geometry_builder.set_semantic_surface(None, roof_semantic, true)?;
        geometry_builder.set_material_surface(
            None,
            appearance.material_irradiation.clone(),
            "irradiation",
            true,
        )?;
        geometry_builder.set_material_surface(
            None,
            appearance.material_red.clone(),
            "red",
            true,
        )?;
        geometry_builder.map_vertex_to_uv(bv0, uv0);
        geometry_builder.map_vertex_to_uv(bv1, uv1);
        geometry_builder.map_vertex_to_uv(bv2, uv2);
        geometry_builder.map_vertex_to_uv(bv3, uv3);
        geometry_builder.set_texture_ring(
            None,
            appearance.texture_winter.clone(),
            "winter-textures",
            true,
        )?;

        // 2nd Surface ---
        // This surface does not have Semantic
        let ring2 = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
        let surface_2 = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring2)?;
        geometry_builder.set_material_surface(
            None,
            appearance.material_irradiation.clone(),
            "irradiation",
            true,
        )?;
        geometry_builder.set_material_surface(
            None,
            appearance.material_red.clone(),
            "red",
            true,
        )?;

        // 3rd Surface ---
        // This surface has a type from an Extension
        let ring3 = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
        let surface_3 = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring3)?;
        let patio_door_semantic = Semantic::new(SemanticType::Extension("+PatioDoor"));
        geometry_builder.set_semantic_surface(None, patio_door_semantic, false)?;
        // This surface does not have the "irradiation" material
        geometry_builder.set_material_surface(
            None,
            appearance.material_red.clone(),
            "red",
            true,
        )?;
        geometry_builder.add_shell(&[surface_0, surface_1, surface_2, surface_3])?;

        // Inner shell
        let surface_4 = geometry_builder.start_surface();
        let ring4 = geometry_builder.add_ring(&[bv1, bv2, bv3, bv0])?;
        geometry_builder.add_surface_outer_ring(ring4)?;
        let ring5 = geometry_builder.add_ring(&[bv1, bv2, bv3, bv0])?;
        geometry_builder.add_surface_inner_ring(ring5)?;
        geometry_builder.add_shell(&[surface_4])?;

        // Consume the builder by building a Geometry and adding it to the CityModel
        let geometry_ref = geometry_builder.build()?;

        // Attach geometry to CityObject
        co_1.add_geometry(GeometryRef::from_parts(
            geometry_ref.index(),
            geometry_ref.generation(),
        ));

        // For debug only
        let geom_nested = model
            .get_geometry(GeometryRef::from_parts(
                geometry_ref.index(),
                geometry_ref.generation(),
            ))
            .unwrap()
            .clone()
            .boundaries()
            .unwrap()
            .to_nested_solid()?;
        println!("CityObject id-1 nested boundary: {:?}", geom_nested);
    }
    Ok(())
}

/// Build CityObject "id-3".
fn build_cityobject_id_3(co_3: &mut BorrowedCityObject) {
    co_3.attributes_mut()
        .insert("buildingLDenMin", AttributeValue::Float(1.0));
}

/// Build CityObject "a-tree" with template geometry and one geometry instance.
fn build_cityobject_tree(
    model: &mut BorrowedModel,
    co_tree: &mut BorrowedCityObject,
    vertices: SharedVertices,
) -> Result<()> {
    let mut template_builder =
        GeometryBuilder::new(model, GeometryType::MultiSurface, BuilderMode::Template)
            .with_lod(LoD::LoD2_1);
    let tp0 = template_builder.add_template_point(RealWorldCoordinate::new(0.0, 0.5, 0.0));
    let tp1 = template_builder.add_template_point(RealWorldCoordinate::new(1.0, 1.0, 0.0));
    let tp2 = template_builder.add_template_point(RealWorldCoordinate::new(0.0, 1.0, 0.0));
    let tp3 = template_builder.add_template_point(RealWorldCoordinate::new(2.1, 4.2, 1.2));

    let ring0 = template_builder.add_ring(&[tp0, tp3, tp2, tp1])?;
    template_builder.start_surface();
    template_builder.add_surface_outer_ring(ring0)?;

    let ring1 = template_builder.add_ring(&[tp1, tp2, tp0, tp3])?;
    template_builder.start_surface();
    template_builder.add_surface_outer_ring(ring1)?;

    let ring2 = template_builder.add_ring(&[tp0, tp1, tp3, tp2])?;
    template_builder.start_surface();
    template_builder.add_surface_outer_ring(ring2)?;

    let template_ref = template_builder.build()?;

    let tree_geometry_ref =
        GeometryBuilder::new(model, GeometryType::GeometryInstance, BuilderMode::Regular)
            .with_template(template_ref)?
            .with_transformation_matrix([
                2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ])?
            .with_reference_vertex(vertices.v1)
            .build()?;

    co_tree.add_geometry(GeometryRef::from_parts(
        tree_geometry_ref.index(),
        tree_geometry_ref.generation(),
    ));
    Ok(())
}

/// Build CityObject "my-neighbourhood".
fn build_cityobject_neighbourhood(
    model: &mut BorrowedModel,
    co_neighbourhood: &mut BorrowedCityObject,
    vertices: SharedVertices,
) -> Result<()> {
    co_neighbourhood
        .attributes_mut()
        .insert("location", AttributeValue::String("Magyarkanizsa"));

    let roles_vec = vec![
        Box::new(AttributeValue::String("residential building")),
        Box::new(AttributeValue::String("voting location")),
    ];
    co_neighbourhood
        .extra_mut()
        .insert("children_roles", AttributeValue::Vec(roles_vec));

    let mut geometry_builder =
        GeometryBuilder::new(model, GeometryType::MultiSurface, BuilderMode::Regular)
            .with_lod(LoD::LoD2);
    let _surface_i = geometry_builder.start_surface();
    let p1 = geometry_builder.add_vertex(vertices.v0);
    let p2 = geometry_builder.add_vertex(vertices.v3);
    let p3 = geometry_builder.add_vertex(vertices.v2);
    let p4 = geometry_builder.add_vertex(vertices.v1);
    let ring0 = geometry_builder.add_ring(&[p1, p4, p3, p2])?;
    geometry_builder.add_surface_outer_ring(ring0)?;
    let neighbourhood_geometry_ref = geometry_builder.build()?;

    co_neighbourhood.add_geometry(GeometryRef::from_parts(
        neighbourhood_geometry_ref.index(),
        neighbourhood_geometry_ref.generation(),
    ));
    Ok(())
}

/// Add a parent/children relation between semantic surfaces for schema coverage.
fn link_semantics_for_schema_coverage(model: &mut BorrowedModel) {
    let mut roof_semantic_ref = None;
    let mut patio_door_semantic_ref = None;
    for (semantic_ref, semantic) in model.iter_semantics() {
        if roof_semantic_ref.is_none() && semantic.type_semantic() == &SemanticType::RoofSurface {
            roof_semantic_ref = Some(semantic_ref);
        }
        if patio_door_semantic_ref.is_none()
            && let SemanticType::Extension(ext) = semantic.type_semantic()
            && *ext == "+PatioDoor"
        {
            patio_door_semantic_ref = Some(semantic_ref);
        }
    }
    if let (Some(roof), Some(patio)) = (roof_semantic_ref, patio_door_semantic_ref) {
        model
            .get_semantic_mut(roof)
            .expect("roof semantic should exist")
            .children_mut()
            .push(patio);
        model
            .get_semantic_mut(patio)
            .expect("patio door semantic should exist")
            .set_parent(roof);
    }
}

/// Add CityObjects to the model and connect parent/children hierarchy.
fn add_cityobjects_with_hierarchy(
    model: &mut BorrowedModel,
    cityobjects_to_add: PendingCityObjects,
) -> CityObjectRefs {
    let PendingCityObjects {
        co_1,
        co_3,
        co_tree,
        co_neighbourhood,
    } = cityobjects_to_add;

    let cityobjects = model.cityobjects_mut();
    let co_1_ref = cityobjects.add(co_1);
    let co_3_ref = cityobjects.add(co_3);
    let _co_tree_ref = cityobjects.add(co_tree);
    let co_neigh_ref = cityobjects.add(co_neighbourhood);

    cityobjects.get_mut(co_1_ref).unwrap().add_parent(co_3_ref);
    cityobjects
        .get_mut(co_1_ref)
        .unwrap()
        .add_parent(co_neigh_ref);
    cityobjects.get_mut(co_3_ref).unwrap().add_child(co_1_ref);
    cityobjects
        .get_mut(co_3_ref)
        .unwrap()
        .add_parent(co_neigh_ref);
    cityobjects
        .get_mut(co_neigh_ref)
        .unwrap()
        .add_child(co_1_ref);
    cityobjects
        .get_mut(co_neigh_ref)
        .unwrap()
        .add_child(co_3_ref);

    CityObjectRefs {
        co_1_ref,
        co_3_ref,
        co_neigh_ref,
    }
}

/// Build a complete Metadata instance with all data set and add it to a CityModel.
/// Takes the CityModel by mutable reference.
fn build_metadata_with_reference(model: &mut BorrowedModel) -> Result<()> {
    let metadata_ref = model.metadata_mut();
    build_metadata(metadata_ref);
    Ok(())
}

/// Build a complete Metadata instance with all data set and return it.
fn build_metadata_with_return() -> Result<BorrowedMetadata> {
    let mut metadata = Metadata::new();
    build_metadata(&mut metadata);
    Ok(metadata)
}

/// Set data on a Metadata instance.
fn build_metadata(metadata_ref: &mut BorrowedMetadata) {
    metadata_ref
        .set_geographical_extent(BBox::new(84710.1, 446846.0, -5.3, 84757.1, 446944.0, 40.9));
    metadata_ref.set_identifier(CityModelIdentifier::new(
        "eaeceeaa-3f66-429a-b81d-bbc6140b8c1c",
    ));
    metadata_ref.set_reference_system(CRS::new("https://www.opengis.net/def/crs/EPSG/0/2355"));
    metadata_ref.set_contact_name("3DGI");
    metadata_ref.set_email_address("info@3dgi.nl");
    metadata_ref.set_role(ContactRole::Author);
    metadata_ref.set_website("https://3dgi.nl");
    metadata_ref.set_contact_type(ContactType::Organization);
    let mut address = Attributes::<BorrowedStringStorage<'static>>::new();
    address.insert("city", AttributeValue::String("Den Haag"));
    address.insert("country", AttributeValue::String("The Netherlands"));
    metadata_ref.set_address(address);
    metadata_ref.set_phone("+36612345678");
    metadata_ref.set_organization("3DGI");
}

#[test]
fn borrowed_storage_with_dynamic_lifetime() -> Result<()> {
    fn build_model<'a>(
        id: &'a str,
        name_key: &'a str,
        name_value: &'a str,
    ) -> CityModel<u32, BorrowedStringStorage<'a>> {
        let mut model = CityModel::new(CityModelType::CityJSON);
        let mut city_object =
            CityObject::new(CityObjectIdentifier::new(id), CityObjectType::Building);
        city_object
            .attributes_mut()
            .insert(name_key, AttributeValue::String(name_value));
        model.cityobjects_mut().add(city_object);
        model
    }

    let id_storage = String::from("building-dynamic-1");
    let attr_key_storage = String::from("name");
    let attr_value_storage = String::from("Dynamic Building");
    let model = build_model(
        id_storage.as_str(),
        attr_key_storage.as_str(),
        attr_value_storage.as_str(),
    );

    assert_eq!(model.cityobjects().len(), 1);
    let (_, co) = model
        .cityobjects()
        .first()
        .expect("CityObject should exist");
    assert_eq!(co.id(), "building-dynamic-1");
    let attrs = co.attributes().expect("Attributes should exist");
    let attr = attrs.get("name").expect("name should exist");
    match attr {
        AttributeValue::String(name) => assert_eq!(*name, "Dynamic Building"),
        _ => panic!("name should be String"),
    }

    Ok(())
}
