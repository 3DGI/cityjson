use cityjson_fake::attribute::{AttributesBuilder, AttributesFaker};
use cityjson_fake::material::MaterialBuilder;
use cityjson_fake::metadata::MetadataBuilder;
use cityjson_fake::prelude::*;
use cityjson_fake::texture::TextureBuilder;
use cityjson_fake::vertex::{RealWorldCoordinateFaker, VerticesFaker};
use fake::{Dummy, Fake};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::sample::select;
use proptest::test_runner::FileFailurePersistence;
use rand::SeedableRng;
use rand::prelude::SmallRng;
use std::path::Path;

fn city_object_type_strategy()
-> impl Strategy<Value = Option<Vec<CityObjectType<OwnedStringStorage>>>> {
    let types = vec![
        CityObjectType::Bridge,
        CityObjectType::Building,
        CityObjectType::CityFurniture,
        CityObjectType::GenericCityObject,
        CityObjectType::Default,
        CityObjectType::LandUse,
        CityObjectType::OtherConstruction,
        CityObjectType::PlantCover,
        CityObjectType::SolitaryVegetationObject,
        CityObjectType::TINRelief,
        CityObjectType::WaterBody,
        CityObjectType::Road,
        CityObjectType::Railway,
        CityObjectType::Waterway,
        CityObjectType::TransportSquare,
        CityObjectType::Tunnel,
    ];
    let nr_types = types.len();
    vec(select(types), 1..=nr_types).prop_map(Some)
}

fn geometry_type_strategy() -> impl Strategy<Value = Option<Vec<GeometryType>>> {
    let types = vec![
        GeometryType::MultiPoint,
        GeometryType::MultiLineString,
        GeometryType::MultiSurface,
        GeometryType::CompositeSurface,
        GeometryType::Solid,
        GeometryType::MultiSolid,
        GeometryType::CompositeSolid,
        GeometryType::GeometryInstance,
    ];
    let nr_types = types.len();
    prop_oneof![Just(None), vec(select(types), 1..=nr_types).prop_map(Some),]
}

fn lod_strategy() -> impl Strategy<Value = Option<Vec<LoD>>> {
    let lods = vec![
        LoD::LoD0,
        LoD::LoD0_0,
        LoD::LoD0_1,
        LoD::LoD0_2,
        LoD::LoD0_3,
        LoD::LoD1,
        LoD::LoD1_0,
        LoD::LoD1_1,
        LoD::LoD1_2,
        LoD::LoD1_3,
        LoD::LoD2,
        LoD::LoD2_0,
        LoD::LoD2_1,
        LoD::LoD2_2,
        LoD::LoD2_3,
        LoD::LoD3,
        LoD::LoD3_0,
        LoD::LoD3_1,
        LoD::LoD3_2,
        LoD::LoD3_3,
    ];
    let nr_lods = lods.len();
    prop_oneof![Just(None), vec(select(lods), 1..=nr_lods).prop_map(Some),]
}

fn semantic_type_strategy() -> impl Strategy<Value = Option<Vec<SemanticType<OwnedStringStorage>>>>
{
    let types = vec![
        SemanticType::RoofSurface,
        SemanticType::GroundSurface,
        SemanticType::WallSurface,
        SemanticType::ClosureSurface,
        SemanticType::OuterCeilingSurface,
        SemanticType::OuterFloorSurface,
        SemanticType::Window,
        SemanticType::Door,
        SemanticType::InteriorWallSurface,
        SemanticType::CeilingSurface,
        SemanticType::FloorSurface,
        SemanticType::WaterSurface,
        SemanticType::WaterGroundSurface,
        SemanticType::WaterClosureSurface,
        SemanticType::TrafficArea,
        SemanticType::AuxiliaryTrafficArea,
        SemanticType::TransportationMarking,
        SemanticType::TransportationHole,
    ];
    let nr_types = types.len();
    prop_oneof![Just(None), vec(select(types), 1..=nr_types).prop_map(Some),]
}

fn option_bool_strategy() -> impl Strategy<Value = Option<bool>> {
    prop_oneof![Just(None), Just(Some(true)), Just(Some(false))]
}

fn attribute_depth(value: &OwnedAttributeValue) -> usize {
    match value {
        OwnedAttributeValue::Vec(values) => {
            1 + values.iter().map(attribute_depth).max().unwrap_or(0)
        }
        OwnedAttributeValue::Map(values) => {
            1 + values.values().map(attribute_depth).max().unwrap_or(0)
        }
        _ => 0,
    }
}

fn first_level_building_types() -> Vec<CityObjectType<OwnedStringStorage>> {
    vec![
        CityObjectType::Bridge,
        CityObjectType::Building,
        CityObjectType::CityFurniture,
        CityObjectType::GenericCityObject,
        CityObjectType::LandUse,
        CityObjectType::OtherConstruction,
        CityObjectType::PlantCover,
        CityObjectType::SolitaryVegetationObject,
        CityObjectType::TINRelief,
        CityObjectType::WaterBody,
        CityObjectType::Road,
        CityObjectType::Railway,
        CityObjectType::Waterway,
        CityObjectType::TransportSquare,
        CityObjectType::Tunnel,
    ]
}

proptest! {
    #![proptest_config(ProptestConfig{
        cases: 64,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("proptest-regressions"))),
        ..Default::default()
    })]

    /// Exercise the full generation surface and verify the observable config effects.
    #[test]
    fn fuzz_config(
        allowed_types_cityobject in city_object_type_strategy(),
        allowed_types_geometry in geometry_type_strategy(),
        allowed_lods in lod_strategy(),
        cityobject_hierarchy in any::<bool>(),
        cityobject_count in 1u32..=3,
        children_count in 1u32..=3,
        min_coordinate in -1000.0f64..=-1.0f64,
        max_coordinate in 1.0f64..=1000.0f64,
        count_value in 1u32..=3,
        materials_enabled in any::<bool>(),
        textures_enabled in any::<bool>(),
        use_templates in any::<bool>(),
        metadata_enabled in any::<bool>(),
        attributes_enabled in any::<bool>(),
        semantics_enabled in any::<bool>(),
        texture_allow_none in any::<bool>(),
        attributes_max_depth in 0u8..=3,
        generate_ambient_intensity in option_bool_strategy(),
        generate_diffuse_color in option_bool_strategy(),
        generate_emissive_color in option_bool_strategy(),
        generate_specular_color in option_bool_strategy(),
        generate_shininess in option_bool_strategy(),
        generate_transparency in option_bool_strategy(),
        metadata_geographical_extent in any::<bool>(),
        metadata_identifier in any::<bool>(),
        metadata_reference_date in any::<bool>(),
        metadata_reference_system in any::<bool>(),
        metadata_title in any::<bool>(),
        metadata_point_of_contact in any::<bool>(),
        attributes_random_keys in any::<bool>(),
        attributes_random_values in any::<bool>(),
        allowed_types_semantic in semantic_type_strategy(),
        call_transform in any::<bool>(),
    ) {
        let mut allowed_types_cityobject = allowed_types_cityobject;
        if cityobject_hierarchy
            && !allowed_types_cityobject
                .as_ref()
                .is_some_and(|types| types.iter().any(|t| first_level_building_types().contains(t)))
        {
            let mut types = allowed_types_cityobject.unwrap_or_default();
            types.push(CityObjectType::Building);
            allowed_types_cityobject = Some(types);
        }

        let config = CJFakeConfig {
            cityobjects: CityObjectConfig {
                allowed_types_cityobject,
                min_cityobjects: cityobject_count,
                max_cityobjects: cityobject_count,
                cityobject_hierarchy,
                min_children: children_count,
                max_children: children_count,
            },
            geometry: GeometryConfig {
                allowed_types_geometry,
                allowed_lods,
                min_members_multipoint: count_value,
                max_members_multipoint: count_value,
                min_members_multilinestring: count_value,
                max_members_multilinestring: count_value,
                min_members_multisurface: count_value,
                max_members_multisurface: count_value,
                min_members_solid: count_value,
                max_members_solid: count_value,
                min_members_multisolid: count_value,
                max_members_multisolid: count_value,
                min_members_compositesurface: count_value,
                max_members_compositesurface: count_value,
                min_members_compositesolid: count_value,
                max_members_compositesolid: count_value,
                min_members_cityobject_geometries: count_value,
                max_members_cityobject_geometries: count_value,
            },
            vertices: VertexConfig {
                min_coordinate,
                max_coordinate,
                min_vertices: count_value,
                max_vertices: count_value,
            },
            materials: MaterialConfig {
                materials_enabled,
                min_materials: count_value,
                max_materials: count_value,
                nr_themes_materials: count_value.max(1),
                generate_ambient_intensity,
                generate_diffuse_color,
                generate_emissive_color,
                generate_specular_color,
                generate_shininess,
                generate_transparency,
            },
            textures: TextureConfig {
                textures_enabled,
                min_textures: count_value,
                max_textures: count_value,
                nr_themes_textures: count_value.max(1),
                max_vertices_texture: count_value + 4,
                texture_allow_none,
            },
            templates: TemplateConfig {
                use_templates,
                min_templates: count_value,
                max_templates: count_value,
            },
            metadata: MetadataConfig {
                metadata_enabled,
                metadata_geographical_extent,
                metadata_identifier,
                metadata_reference_date,
                metadata_reference_system,
                metadata_title,
                metadata_point_of_contact,
            },
            attributes: AttributeConfig {
                attributes_enabled,
                min_attributes: count_value,
                max_attributes: count_value,
                attributes_max_depth,
                attributes_random_keys,
                attributes_random_values,
            },
            semantics: SemanticConfig {
                semantics_enabled,
                allowed_types_semantic: allowed_types_semantic.clone(),
            },
            ..Default::default()
        };

        let model = {
            let builder = CityModelBuilder::<u32, OwnedStringStorage>::new(config.clone(), Some(11));
            let builder = if call_transform {
                builder.transform()
            } else {
                builder
            };
            builder
                .metadata(None)
                .vertices()
                .materials(None)
                .textures(None)
                .attributes(None)
                .cityobjects()
                .build()
        };

        let helper_model = generate_model(config.clone(), Some(11));
        assert_eq!(helper_model.cityobjects().len(), model.cityobjects().len());

        #[cfg(feature = "json")]
        {
            let json =
                generate_string(config.clone(), Some(11)).expect("JSON serialization should succeed");
            assert!(json.starts_with('{'));

            let bytes =
                generate_vec(config.clone(), Some(11)).expect("byte serialization should succeed");
            assert!(bytes.starts_with(b"{"));

            let built_json = CityModelBuilder::<u32, OwnedStringStorage>::new(
                config.clone(),
                Some(11),
            )
            .metadata(None)
            .vertices()
            .materials(None)
            .textures(None)
            .attributes(None)
            .cityobjects()
            .build_string()
            .expect("builder serialization should succeed");
            assert!(built_json.starts_with('{'));

            let built_vec = CityModelBuilder::<u32, OwnedStringStorage>::new(config.clone(), Some(11))
                .metadata(None)
                .vertices()
                .materials(None)
                .textures(None)
                .attributes(None)
                .cityobjects()
                .build_vec()
                .expect("builder byte serialization should succeed");
            assert!(built_vec.starts_with(b"{"));
        }

        if call_transform {
            assert!(model.transform().is_some());
        }

        if cityobject_hierarchy {
            assert!(model.cityobjects().len() >= cityobject_count as usize);
        } else {
            assert_eq!(model.cityobjects().len(), cityobject_count as usize);
        }

        for vertex in model.vertices().as_slice() {
            assert!(vertex.x() >= min_coordinate && vertex.x() <= max_coordinate);
            assert!(vertex.y() >= min_coordinate && vertex.y() <= max_coordinate);
            assert!(vertex.z() >= min_coordinate && vertex.z() <= max_coordinate);
        }

        if materials_enabled {
            assert_eq!(model.iter_materials().count(), count_value as usize);
            for (_, material) in model.iter_materials() {
                if let Some(expected) = generate_ambient_intensity {
                    assert_eq!(material.ambient_intensity().is_some(), expected);
                }
                if let Some(expected) = generate_diffuse_color {
                    assert_eq!(material.diffuse_color().is_some(), expected);
                }
                if let Some(expected) = generate_emissive_color {
                    assert_eq!(material.emissive_color().is_some(), expected);
                }
                if let Some(expected) = generate_specular_color {
                    assert_eq!(material.specular_color().is_some(), expected);
                }
                if let Some(expected) = generate_shininess {
                    assert_eq!(material.shininess().is_some(), expected);
                }
                if let Some(expected) = generate_transparency {
                    assert_eq!(material.transparency().is_some(), expected);
                }
            }
        } else {
            assert_eq!(model.iter_materials().count(), 0);
        }

        if textures_enabled {
            assert_eq!(model.iter_textures().count(), count_value as usize);
            for (_, texture) in model.iter_textures() {
                assert!(!texture.image().is_empty());
            }
        } else {
            assert_eq!(model.iter_textures().count(), 0);
        }

        if metadata_enabled {
            let meta = model.metadata().expect("metadata should be generated");
            assert_eq!(meta.geographical_extent().is_some(), metadata_geographical_extent);
            assert_eq!(meta.identifier().is_some(), metadata_identifier);
            assert_eq!(meta.reference_date().is_some(), metadata_reference_date);
            assert_eq!(meta.reference_system().is_some(), metadata_reference_system);
            assert_eq!(meta.title().is_some(), metadata_title);
            assert_eq!(meta.point_of_contact().is_some(), metadata_point_of_contact);
        } else {
            assert!(model.metadata().is_none());
        }

        if attributes_enabled {
            let mut saw_attributes = false;
            for (_, cityobject) in model.cityobjects().iter() {
                if let Some(attrs) = cityobject.attributes() {
                    if !attrs.is_empty() {
                        saw_attributes = true;
                    }
                    if !attributes_random_keys {
                        for key in attrs.keys() {
                            assert!(key.starts_with("attr_"));
                        }
                    }
                    if !attributes_random_values {
                        for value in attrs.values() {
                            assert!(matches!(value, OwnedAttributeValue::String(s) if s == "default"));
                        }
                    }
                    for value in attrs.values() {
                        assert!(attribute_depth(value) <= attributes_max_depth as usize);
                    }
                }
            }
            assert!(saw_attributes);
        } else {
            for (_, cityobject) in model.cityobjects().iter() {
                assert!(cityobject.attributes().is_none_or(Attributes::is_empty));
            }
        }

        if !semantics_enabled {
            assert_eq!(model.iter_semantics().count(), 0);
        } else if let Some(allowed) = &allowed_types_semantic {
            for (_, semantic) in model.iter_semantics() {
                assert!(allowed.contains(semantic.type_semantic()));
            }
        }

        for (_, cityobject) in model.cityobjects().iter() {
            if let Some(geometry_handles) = cityobject.geometry() {
                for geometry_handle in geometry_handles {
                    let geometry = model
                        .get_geometry(*geometry_handle)
                        .expect("geometry should exist");
                    if let Some(allowed) = &config.geometry.allowed_lods {
                        assert!(geometry.lod().is_none_or(|lod| allowed.contains(lod)));
                    }
                }
            }
        }
    }

    /// Exercise the standalone builders and faker helpers directly.
    #[test]
    fn fuzz_builders(
        material_use_name in any::<bool>(),
        material_ambient in any::<bool>(),
        material_diffuse in any::<bool>(),
        material_emissive in any::<bool>(),
        material_specular in any::<bool>(),
        material_shininess in any::<bool>(),
        material_transparency in any::<bool>(),
        texture_use_image_type in any::<bool>(),
        texture_use_image in any::<bool>(),
        metadata_geographical_extent in any::<bool>(),
        metadata_identifier in any::<bool>(),
        metadata_reference_date in any::<bool>(),
        metadata_reference_system in any::<bool>(),
        metadata_title in any::<bool>(),
        metadata_point_of_contact in any::<bool>(),
        attributes_random_keys in any::<bool>(),
        attributes_random_values in any::<bool>(),
        attributes_max_depth in 0u8..=3,
        attributes_min in 0u32..=3,
        attributes_extra in 0u32..=3,
        vertex_min in -1000.0f64..=-1.0f64,
        vertex_max in 1.0f64..=1000.0f64,
    ) {
        let default_model: CityModel<u32, OwnedStringStorage> =
            CityModelBuilder::<u32, OwnedStringStorage>::default().build();
        assert_eq!(default_model.cityobjects().len(), 1);

        let default_material: Material<OwnedStringStorage> = MaterialBuilder::default().build();
        assert_eq!(default_material.name(), "material");

        let mut material_rng = SmallRng::seed_from_u64(1);
        let material: Material<OwnedStringStorage> = {
            let mut builder = MaterialBuilder::new(&mut material_rng);
            if material_use_name {
                builder = builder.name();
            }
            if material_ambient {
                builder = builder.ambient_intensity();
            }
            if material_diffuse {
                builder = builder.diffuse_color();
            }
            if material_emissive {
                builder = builder.emissive_color();
            }
            if material_specular {
                builder = builder.specular_color();
            }
            if material_shininess {
                builder = builder.shininess();
            }
            if material_transparency {
                builder = builder.transparency();
            }
            builder.build()
        };
        assert!(!material.name().is_empty());
        assert_eq!(material.ambient_intensity().is_some(), material_ambient);
        assert_eq!(material.diffuse_color().is_some(), material_diffuse);
        assert_eq!(material.emissive_color().is_some(), material_emissive);
        assert_eq!(material.specular_color().is_some(), material_specular);
        assert_eq!(material.shininess().is_some(), material_shininess);
        assert_eq!(material.transparency().is_some(), material_transparency);

        let default_texture: Texture<OwnedStringStorage> = TextureBuilder::default().build();
        assert!(!default_texture.image().is_empty());

        let mut texture_rng = SmallRng::seed_from_u64(2);
        let texture: Texture<OwnedStringStorage> = {
            let mut builder = TextureBuilder::new(&mut texture_rng);
            if texture_use_image_type {
                builder = builder.image_type();
            }
            if texture_use_image {
                builder = builder.image();
            }
            builder.build()
        };
        assert!(!texture.image().is_empty());
        if texture_use_image {
            let expected_extension = match texture.image_type() {
                ImageType::Png => "png",
                ImageType::Jpg => "jpg",
                _ => unreachable!("unexpected image type"),
            };
            assert_eq!(
                Path::new(texture.image()).extension().and_then(|ext| ext.to_str()),
                Some(expected_extension)
            );
        }

        let config = CJFakeConfig::default();
        let mut metadata_rng = SmallRng::seed_from_u64(3);
        let metadata: Metadata<OwnedStringStorage> = {
            let mut builder = MetadataBuilder::new(&config, &mut metadata_rng);
            if metadata_geographical_extent {
                builder = builder.geographical_extent();
            }
            if metadata_identifier {
                builder = builder.identifier();
            }
            if metadata_reference_date {
                builder = builder.reference_date();
            }
            if metadata_reference_system {
                builder = builder.reference_system();
            }
            if metadata_title {
                builder = builder.title();
            }
            if metadata_point_of_contact {
                builder = builder.point_of_contact();
            }
            builder.build()
        };
        assert_eq!(
            metadata.geographical_extent().is_some(),
            metadata_geographical_extent
        );
        assert_eq!(metadata.identifier().is_some(), metadata_identifier);
        assert_eq!(metadata.reference_date().is_some(), metadata_reference_date);
        assert_eq!(
            metadata.reference_system().is_some(),
            metadata_reference_system
        );
        assert_eq!(metadata.title().is_some(), metadata_title);
        assert_eq!(
            metadata.point_of_contact().is_some(),
            metadata_point_of_contact
        );

        let mut attributes_rng = SmallRng::seed_from_u64(4);
        let faker = AttributesFaker {
            random_keys: attributes_random_keys,
            random_values: attributes_random_values,
            max_depth: attributes_max_depth,
            min_attrs: attributes_min,
            max_attrs: attributes_min + attributes_extra,
        };
        let attributes = faker.generate(&mut attributes_rng);
        assert!(attributes.len() >= attributes_min as usize);
        assert!(attributes.len() <= (attributes_min + attributes_extra) as usize);
        if !attributes_random_keys {
            assert!(attributes.keys().all(|key| key.starts_with("attr_")));
        }
        if !attributes_random_values {
            assert!(
                attributes
                    .values()
                    .all(|value| matches!(value, OwnedAttributeValue::String(text) if text == "default"))
            );
        }

        let mut builder_rng = SmallRng::seed_from_u64(5);
        let built_attributes = AttributesBuilder::new()
            .with_random_attributes(&mut builder_rng)
            .build();
        assert!(!built_attributes.is_empty());

        let mut coordinate_rng = SmallRng::seed_from_u64(6);
        let coordinate: RealWorldCoordinate = Dummy::dummy_with_rng(
            &RealWorldCoordinateFaker::new(vertex_min, vertex_max),
            &mut coordinate_rng,
        );
        assert!(coordinate.x() >= vertex_min && coordinate.x() <= vertex_max);
        assert!(coordinate.y() >= vertex_min && coordinate.y() <= vertex_max);
        assert!(coordinate.z() >= vertex_min && coordinate.z() <= vertex_max);
    }

    /// The vertex faker honors the configured count and coordinate range.
    #[test]
    fn fuzz_vertices(
        min_vertices in 1u32..=4,
        extra_vertices in 0u32..=4,
        min_coordinate in -1000.0f64..=-1.0f64,
        max_coordinate in 1.0f64..=1000.0f64,
    ) {
        let max_vertices = min_vertices + extra_vertices;
        let config = CJFakeConfig {
            vertices: VertexConfig {
                min_coordinate,
                max_coordinate,
                min_vertices,
                max_vertices,
            },
            ..Default::default()
        };

        let mut rng = SmallRng::seed_from_u64(99);
        let vertices: Vertices<u32, RealWorldCoordinate> =
            VerticesFaker::new(&config).fake_with_rng(&mut rng);

        assert!(vertices.len() >= min_vertices as usize);
        assert!(vertices.len() <= max_vertices as usize);
        for vertex in vertices.as_slice() {
            assert!(vertex.x() >= min_coordinate && vertex.x() <= max_coordinate);
            assert!(vertex.y() >= min_coordinate && vertex.y() <= max_coordinate);
            assert!(vertex.z() >= min_coordinate && vertex.z() <= max_coordinate);
        }

        let mut coordinate_rng = SmallRng::seed_from_u64(100);
        let coordinate: RealWorldCoordinate = Dummy::dummy_with_rng(
            &RealWorldCoordinateFaker::new(min_coordinate, max_coordinate),
            &mut coordinate_rng,
        );
        assert!(coordinate.x() >= min_coordinate && coordinate.x() <= max_coordinate);
        assert!(coordinate.y() >= min_coordinate && coordinate.y() <= max_coordinate);
        assert!(coordinate.z() >= min_coordinate && coordinate.z() <= max_coordinate);
    }
}
