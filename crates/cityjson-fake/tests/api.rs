use cityjson_fake::prelude::*;

// ─── Basic construction ───────────────────────────────────────────────────────

/// Can we use the top-level generation helpers?
#[test]
fn generate_helpers() {
    let config = CJFakeConfig::default();
    let model = generate_model(config.clone(), Some(1));
    assert_eq!(model.cityobjects().len(), 1);

    let json = generate_string(config, Some(1)).expect("serialization should succeed");
    assert!(json.starts_with('{'));
}

/// Can we fake a valid `CityJSON` with the default parameters?
#[test]
fn default() {
    let cm: CityModel<u32, OwnedStringStorage> = CityModelBuilder::default().build();
    assert_eq!(cm.cityobjects().len(), 1);
}

/// Can we fake a valid `CityJSON` with a seed?
#[test]
fn seed() {
    let cm: CityModel<u32, OwnedStringStorage> =
        CityModelBuilder::new(CJFakeConfig::default(), Some(10))
            .metadata(None)
            .vertices()
            .materials(None)
            .textures(None)
            .attributes(None)
            .cityobjects()
            .build();
    assert_eq!(cm.cityobjects().len(), 1);
}

/// Can we fake a valid `CityJSON` with custom builders?
#[test]
fn custom_builders() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            min_cityobjects: 2,
            max_cityobjects: 2,
            ..Default::default()
        },
        materials: MaterialConfig {
            min_materials: 2,
            max_materials: 2,
            nr_themes_materials: 2,
            ..Default::default()
        },
        textures: TextureConfig {
            min_textures: 1,
            max_textures: 1,
            nr_themes_textures: 1,
            ..Default::default()
        },
        templates: TemplateConfig {
            use_templates: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let cm: CityModel = CityModelBuilder::new(config, None)
        .metadata(None)
        .vertices()
        .materials(None)
        .textures(None)
        .attributes(None)
        .cityobjects()
        .build();

    assert_eq!(cm.cityobjects().len(), 2);

    let materials: Vec<_> = cm.iter_materials().collect();
    assert_eq!(materials.len(), 2);
    let (_, first_material) = materials[0];
    assert!(!first_material.name().is_empty());

    let textures: Vec<_> = cm.iter_textures().collect();
    assert_eq!(textures.len(), 1);
    let (_, first_texture) = textures[0];
    assert!(!first_texture.image().is_empty());
}

/// City objects can carry multiple geometries when configured to do so.
#[test]
fn multiple_geometries_per_cityobject() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            allowed_types_cityobject: Some(vec![CityObjectType::Building]),
            min_cityobjects: 2,
            max_cityobjects: 2,
            ..Default::default()
        },
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::MultiSurface]),
            min_members_cityobject_geometries: 2,
            max_members_cityobject_geometries: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(42))
        .vertices()
        .cityobjects()
        .build();

    for (_, co) in cm.cityobjects().iter() {
        assert_eq!(co.geometry().map(<[GeometryHandle]>::len), Some(2));
    }
}

/// `GenericCityObject` can be generated directly when requested.
#[test]
fn generic_cityobject_allowed() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            allowed_types_cityobject: Some(vec![CityObjectType::GenericCityObject]),
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::MultiSurface]),
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(11))
        .vertices()
        .cityobjects()
        .build();

    for (_, co) in cm.cityobjects().iter() {
        assert_eq!(co.type_cityobject(), &CityObjectType::GenericCityObject);
    }
}

/// `CityObjectGroup` members are wired through `children`, `parents`, and `children_roles`.
#[test]
fn cityobject_group_members() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            allowed_types_cityobject: Some(vec![CityObjectType::CityObjectGroup]),
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::MultiSurface]),
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(7))
        .vertices()
        .cityobjects()
        .build();

    assert_eq!(cm.cityobjects().len(), 3);

    let mut saw_group_with_members = false;
    for (group_handle, co) in cm.cityobjects().iter() {
        assert_eq!(co.type_cityobject(), &CityObjectType::CityObjectGroup);
        let children = co.children().expect("group children should exist");
        let roles = co
            .extra()
            .and_then(|extra| extra.get("children_roles"))
            .expect("group children_roles should exist");
        if let OwnedAttributeValue::Vec(role_values) = roles {
            assert_eq!(role_values.len(), children.len());
        } else {
            panic!("children_roles should be a list");
        }

        if !children.is_empty() {
            saw_group_with_members = true;
        }

        for child_handle in children {
            let child = cm.cityobjects().get(*child_handle).unwrap();
            assert!(child
                .parents()
                .is_some_and(|parents| parents.contains(&group_handle)));
        }
    }

    assert!(
        saw_group_with_members,
        "expected at least one group to have members"
    );
}

// ─── Geometry type coverage ───────────────────────────────────────────────────

/// A config restricted to `MultiPoint` produces `MultiPoint` geometries.
#[test]
fn geometry_multipoint() {
    let config = CJFakeConfig {
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::MultiPoint]),
            ..Default::default()
        },
        cityobjects: CityObjectConfig {
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(42))
        .vertices()
        .cityobjects()
        .build();
    assert_eq!(cm.cityobjects().len(), 3);
    for (_, co) in cm.cityobjects().iter() {
        if let Some(geom_handles) = co.geometry() {
            for &h in geom_handles {
                let g = cm.get_geometry(h).unwrap();
                assert_eq!(g.type_geometry(), &GeometryType::MultiPoint);
            }
        }
    }
}

/// A config restricted to `MultiLineString` produces `MultiLineString` geometries.
#[test]
fn geometry_multilinestring() {
    let config = CJFakeConfig {
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::MultiLineString]),
            ..Default::default()
        },
        cityobjects: CityObjectConfig {
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(42))
        .vertices()
        .cityobjects()
        .build();
    assert_eq!(cm.cityobjects().len(), 3);
    for (_, co) in cm.cityobjects().iter() {
        if let Some(geom_handles) = co.geometry() {
            for &h in geom_handles {
                let g = cm.get_geometry(h).unwrap();
                assert_eq!(g.type_geometry(), &GeometryType::MultiLineString);
            }
        }
    }
}

/// A config restricted to `MultiSurface` produces `MultiSurface` geometries.
#[test]
fn geometry_multisurface() {
    let config = CJFakeConfig {
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::MultiSurface]),
            ..Default::default()
        },
        cityobjects: CityObjectConfig {
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(42))
        .vertices()
        .cityobjects()
        .build();
    assert_eq!(cm.cityobjects().len(), 3);
    for (_, co) in cm.cityobjects().iter() {
        if let Some(geom_handles) = co.geometry() {
            for &h in geom_handles {
                let g = cm.get_geometry(h).unwrap();
                assert_eq!(g.type_geometry(), &GeometryType::MultiSurface);
            }
        }
    }
}

/// A config restricted to Solid produces Solid geometries.
#[test]
fn geometry_solid() {
    let config = CJFakeConfig {
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::Solid]),
            ..Default::default()
        },
        cityobjects: CityObjectConfig {
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(42))
        .vertices()
        .cityobjects()
        .build();
    assert_eq!(cm.cityobjects().len(), 3);
    for (_, co) in cm.cityobjects().iter() {
        if let Some(geom_handles) = co.geometry() {
            for &h in geom_handles {
                let g = cm.get_geometry(h).unwrap();
                assert_eq!(g.type_geometry(), &GeometryType::Solid);
            }
        }
    }
}

/// A config restricted to `MultiSolid` produces `MultiSolid` geometries.
#[test]
fn geometry_multisolid() {
    let config = CJFakeConfig {
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::MultiSolid]),
            ..Default::default()
        },
        cityobjects: CityObjectConfig {
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(42))
        .vertices()
        .cityobjects()
        .build();
    assert_eq!(cm.cityobjects().len(), 3);
    for (_, co) in cm.cityobjects().iter() {
        if let Some(geom_handles) = co.geometry() {
            for &h in geom_handles {
                let g = cm.get_geometry(h).unwrap();
                assert_eq!(g.type_geometry(), &GeometryType::MultiSolid);
            }
        }
    }
}

/// A config restricted to `CompositeSurface` produces `CompositeSurface` geometries.
#[test]
fn geometry_composite_surface() {
    let config = CJFakeConfig {
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::CompositeSurface]),
            ..Default::default()
        },
        cityobjects: CityObjectConfig {
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(42))
        .vertices()
        .cityobjects()
        .build();
    assert_eq!(cm.cityobjects().len(), 3);
    for (_, co) in cm.cityobjects().iter() {
        if let Some(geom_handles) = co.geometry() {
            for &h in geom_handles {
                let g = cm.get_geometry(h).unwrap();
                assert_eq!(g.type_geometry(), &GeometryType::CompositeSurface);
            }
        }
    }
}

/// Composite surfaces honor their configured member count.
#[test]
fn composite_surface_member_count() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            allowed_types_cityobject: Some(vec![CityObjectType::Building]),
            min_cityobjects: 1,
            max_cityobjects: 1,
            ..Default::default()
        },
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::CompositeSurface]),
            min_members_compositesurface: 4,
            max_members_compositesurface: 4,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(42))
        .vertices()
        .cityobjects()
        .build();
    let (_, co) = cm.cityobjects().first().unwrap();
    let geom = cm.get_geometry(co.geometry().unwrap()[0]).unwrap();
    assert_eq!(geom.type_geometry(), &GeometryType::CompositeSurface);
    assert_eq!(geom.boundaries().unwrap().surfaces().len(), 4);
}

/// A config restricted to `CompositeSolid` produces `CompositeSolid` geometries.
#[test]
fn geometry_composite_solid() {
    let config = CJFakeConfig {
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::CompositeSolid]),
            ..Default::default()
        },
        cityobjects: CityObjectConfig {
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(42))
        .vertices()
        .cityobjects()
        .build();
    assert_eq!(cm.cityobjects().len(), 3);
    for (_, co) in cm.cityobjects().iter() {
        if let Some(geom_handles) = co.geometry() {
            for &h in geom_handles {
                let g = cm.get_geometry(h).unwrap();
                assert_eq!(g.type_geometry(), &GeometryType::CompositeSolid);
            }
        }
    }
}

/// Composite solids honor their configured member count.
#[test]
fn composite_solid_member_count() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            allowed_types_cityobject: Some(vec![CityObjectType::Building]),
            min_cityobjects: 1,
            max_cityobjects: 1,
            ..Default::default()
        },
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::CompositeSolid]),
            min_members_compositesolid: 2,
            max_members_compositesolid: 2,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(42))
        .vertices()
        .cityobjects()
        .build();
    let (_, co) = cm.cityobjects().first().unwrap();
    let geom = cm.get_geometry(co.geometry().unwrap()[0]).unwrap();
    assert_eq!(geom.type_geometry(), &GeometryType::CompositeSolid);
    assert_eq!(geom.boundaries().unwrap().solids().len(), 2);
}

// ─── Feature wiring ───────────────────────────────────────────────────────────

/// Generated cityobjects contain non-empty attributes when attributes are enabled.
#[test]
fn attributes_non_empty() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            min_cityobjects: 2,
            max_cityobjects: 2,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(1))
        .vertices()
        .attributes(None)
        .cityobjects()
        .build();
    let mut any_attrs = false;
    for (_, co) in cm.cityobjects().iter() {
        if let Some(attrs) = co.attributes() {
            if !attrs.is_empty() {
                any_attrs = true;
            }
        }
    }
    assert!(
        any_attrs,
        "expected at least one cityobject with attributes"
    );
}

/// Cityobjects preserve the generated type rather than collapsing to Building.
#[test]
fn cityobject_type_preserved() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            allowed_types_cityobject: Some(vec![CityObjectType::Road]),
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(7))
        .vertices()
        .cityobjects()
        .build();
    for (_, co) in cm.cityobjects().iter() {
        assert_eq!(co.type_cityobject(), &CityObjectType::Road);
    }
}

/// Generated geometries expose semantics when a supporting cityobject type is used.
#[test]
fn semantics_attached() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            allowed_types_cityobject: Some(vec![CityObjectType::Building]),
            min_cityobjects: 2,
            max_cityobjects: 2,
            ..Default::default()
        },
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::MultiSurface]),
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(5))
        .vertices()
        .cityobjects()
        .build();
    let mut found_semantic = false;
    for (_, co) in cm.cityobjects().iter() {
        if let Some(geom_handles) = co.geometry() {
            for &h in geom_handles {
                let g = cm.get_geometry(h).unwrap();
                if g.semantics().is_some() {
                    found_semantic = true;
                }
            }
        }
    }
    assert!(
        found_semantic,
        "expected at least one geometry with semantics"
    );
}

/// Generated geometries expose materials when materials are enabled.
#[test]
fn materials_attached() {
    let config = CJFakeConfig {
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::MultiSurface]),
            ..Default::default()
        },
        cityobjects: CityObjectConfig {
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        materials: MaterialConfig {
            min_materials: 2,
            max_materials: 2,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(11))
        .vertices()
        .materials(None)
        .cityobjects()
        .build();
    assert_eq!(cm.iter_materials().count(), 2);
    let mut found_material = false;
    for (_, co) in cm.cityobjects().iter() {
        if let Some(geom_handles) = co.geometry() {
            for &h in geom_handles {
                let g = cm.get_geometry(h).unwrap();
                if g.materials().is_some() {
                    found_material = true;
                }
            }
        }
    }
    assert!(
        found_material,
        "expected at least one geometry with materials"
    );
}

/// Generated geometries expose textures when textures are enabled.
#[test]
fn textures_attached() {
    let config = CJFakeConfig {
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::MultiSurface]),
            ..Default::default()
        },
        cityobjects: CityObjectConfig {
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        textures: TextureConfig {
            min_textures: 1,
            max_textures: 1,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(13))
        .vertices()
        .textures(None)
        .cityobjects()
        .build();
    assert_eq!(cm.iter_textures().count(), 1);
    let mut found_texture = false;
    for (_, co) in cm.cityobjects().iter() {
        if let Some(geom_handles) = co.geometry() {
            for &h in geom_handles {
                let g = cm.get_geometry(h).unwrap();
                if g.textures().is_some() {
                    found_texture = true;
                }
            }
        }
    }
    assert!(
        found_texture,
        "expected at least one geometry with textures"
    );
}

/// Parent objects have children and child objects have parents when hierarchy is enabled.
#[test]
fn parent_child_links() {
    let config = CJFakeConfig {
        cityobjects: CityObjectConfig {
            allowed_types_cityobject: Some(vec![CityObjectType::Building]),
            cityobject_hierarchy: true,
            min_cityobjects: 4,
            max_cityobjects: 4,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(99))
        .vertices()
        .cityobjects()
        .build();

    let mut has_parent = false;
    let mut has_children = false;
    for (_, co) in cm.cityobjects().iter() {
        if co.parents().is_some_and(|p| !p.is_empty()) {
            has_parent = true;
        }
        if co.children().is_some_and(|c| !c.is_empty()) {
            has_children = true;
        }
    }
    assert!(
        has_parent,
        "expected at least one child with a parent reference"
    );
    assert!(
        has_children,
        "expected at least one parent with child references"
    );
}

/// Metadata includes `point_of_contact` when built.
#[test]
fn metadata_point_of_contact() {
    let cm: CityModel = CityModelBuilder::new(CJFakeConfig::default(), Some(3))
        .metadata(None)
        .vertices()
        .cityobjects()
        .build();
    let meta = cm.metadata().expect("metadata should be set");
    let poc = meta.point_of_contact();
    assert!(poc.is_some(), "expected point_of_contact to be set");
    let contact = poc.unwrap();
    assert!(!contact.contact_name().is_empty());
    assert!(!contact.email_address().is_empty());
}

/// Metadata contains identifier, `reference_date`, `reference_system`, and title.
#[test]
fn metadata_fields() {
    let cm: CityModel = CityModelBuilder::new(CJFakeConfig::default(), Some(4))
        .metadata(None)
        .vertices()
        .cityobjects()
        .build();
    let meta = cm.metadata().expect("metadata should be set");
    assert!(meta.identifier().is_some());
    assert!(meta.reference_date().is_some());
    assert!(meta.reference_system().is_some());
    assert!(meta.title().is_some());
    assert!(meta.geographical_extent().is_some());
}

/// Enabling templates inserts template geometries and cityobjects reference instances.
#[test]
fn template_geometries() {
    let config = CJFakeConfig {
        templates: TemplateConfig {
            use_templates: true,
            min_templates: 1,
            max_templates: 2,
        },
        cityobjects: CityObjectConfig {
            min_cityobjects: 2,
            max_cityobjects: 2,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            ..Default::default()
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(77))
        .vertices()
        .cityobjects()
        .build();
    assert_eq!(cm.cityobjects().len(), 2);
    // Each cityobject should have a GeometryInstance geometry
    for (_, co) in cm.cityobjects().iter() {
        if let Some(handles) = co.geometry() {
            for &h in handles {
                let g = cm.get_geometry(h).unwrap();
                assert_eq!(g.type_geometry(), &GeometryType::GeometryInstance);
            }
        }
    }
}

/// Cityobjects with geometry have a `geographical_extent`.
#[test]
fn geographical_extent() {
    let config = CJFakeConfig {
        geometry: GeometryConfig {
            allowed_types_geometry: Some(vec![GeometryType::MultiSurface]),
            ..Default::default()
        },
        cityobjects: CityObjectConfig {
            min_cityobjects: 3,
            max_cityobjects: 3,
            ..Default::default()
        },
        vertices: VertexConfig {
            min_vertices: 8,
            max_vertices: 8,
            min_coordinate: 0.0,
            max_coordinate: 100.0,
        },
        ..Default::default()
    };
    let cm: CityModel = CityModelBuilder::new(config, Some(55))
        .vertices()
        .cityobjects()
        .build();
    for (_, co) in cm.cityobjects().iter() {
        if co.geometry().is_some_and(|g| !g.is_empty()) {
            assert!(
                co.geographical_extent().is_some(),
                "cityobject with geometry should have geographical_extent"
            );
        }
    }
}
