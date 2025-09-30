use cityjson::prelude::*;
use cityjson::v2_0::*;
use std::sync::mpsc;
use std::thread;

// ============================================================================
// Wire Format Structs - Simulate data structures from parsed CityJSON
// ============================================================================

/// Represents attribute values as they would appear in parsed JSON
#[derive(Debug, Clone)]
enum WireAttributeValue {
    #[allow(dead_code)]
    String(String),
    Float(f64),
    Integer(i64),
    #[allow(dead_code)]
    Bool(bool),
}

/// Represents material data as it would appear in parsed JSON
#[derive(Debug, Clone)]
struct WireMaterial {
    name: String,
    ambient_intensity: Option<f64>,
    diffuse_color: Option<[f64; 3]>,
    emissive_color: Option<[f64; 3]>,
    specular_color: Option<[f64; 3]>,
    shininess: Option<f64>,
    transparency: Option<f64>,
    is_smooth: Option<bool>,
}

/// Represents semantic surface data as it would appear in parsed JSON
#[derive(Debug, Clone)]
struct WireSemantic {
    surface_type: String, // "RoofSurface", "WallSurface", etc.
    #[allow(dead_code)]
    attributes: Vec<(String, WireAttributeValue)>,
}

/// Represents geometry data as it would appear in parsed JSON
#[derive(Debug, Clone)]
struct WireGeometry {
    geometry_type: String, // "Solid", "MultiSurface", "GeometryInstance"
    lod: String,
    /// Boundaries in nested Vec format (as in CityJSON spec)
    /// For Solid:
    /// Vec<shells> where shell = Vec<surfaces> where surface = Vec<rings> where ring = Vec<vertex_indices>
    boundaries: Vec<Vec<Vec<Vec<usize>>>>,
    /// Semantic surface info, one per surface
    semantics: Vec<Option<WireSemantic>>,
    /// Materials per surface, keyed by theme name
    materials: Vec<(String, WireMaterial)>, // (theme, material)
    /// For GeometryInstance: reference to template index
    template_ref: Option<usize>,
    /// For GeometryInstance: transformation matrix
    transformation_matrix: Option<[f64; 16]>,
}

/// Represents template geometry data in global properties
#[derive(Debug, Clone)]
struct WireTemplateGeometry {
    geometry_type: String,
    lod: String,
    /// Template vertices as real-world coordinates
    template_vertices: Vec<(f64, f64, f64)>,
    /// Boundaries referencing template_vertices
    #[allow(dead_code)]
    boundaries: Vec<Vec<Vec<usize>>>, // For MultiPoint: just Vec of point indices
}

/// Global properties sent as the first message in the stream
#[derive(Debug, Clone)]
struct WireGlobalProperties {
    metadata_identifier: String,
    crs: String,
    transform_scale: [f64; 3],
    transform_translate: [f64; 3],
    geometry_templates: Vec<WireTemplateGeometry>,
}

/// CityObject data as it would appear in parsed CityJSON stream
#[derive(Debug, Clone)]
struct WireCityObjectData {
    id: String,
    object_type: String,
    /// Quantized vertex coordinates
    vertices: Vec<(i64, i64, i64)>,
    /// Geometries for this CityObject
    geometries: Vec<WireGeometry>,
    /// Attributes
    attributes: Vec<(String, WireAttributeValue)>,
}

/// Message types for streaming communication
#[derive(Debug, Clone)]
enum StreamMessage {
    /// First message: global model properties
    GlobalProperties(WireGlobalProperties),
    /// Subsequent messages: individual CityObjects
    CityObject(WireCityObjectData),
    /// Final message: end of stream
    Done,
}

/// Producer-consumer streaming test with memory management
///
/// This test simulates realistic CityJSON stream processing:
/// 1. Producer sends global properties (metadata, transform, geometry templates)
/// 2. Producer sends individual CityObjects with boundary/semantic/material data
/// 3. Consumer constructs cityjson-rs types from wire format (simulating deserialization)
/// 4. Consumer processes and removes CityObjects to maintain stable memory
#[test]
fn test_producer_consumer_stream() -> Result<()> {
    // Create a channel for producer-consumer communication
    let (tx, rx) = mpsc::channel::<StreamMessage>();

    // Spawn producer and consumer threads in a thread scope
    thread::scope(|s| {
        // Producer thread - generates CityJSON stream data
        s.spawn(move || {
            producer(tx);
        });

        // Consumer thread - ingests and constructs CityModel
        let consumer_handle = s.spawn(move || consumer(rx));

        // Wait for consumer to finish and get results
        consumer_handle.join().expect("Consumer thread panicked")
    })
}

/// Producer function that generates CityJSON stream data
///
/// Simulates a newline-delimited CityJSON stream:
/// 1. First line: global properties (metadata, CRS, transform, templates)
/// 2. Subsequent lines: individual CityObjects
/// 3. End of stream
fn producer(tx: mpsc::Sender<StreamMessage>) {
    // ========================================================================
    // STEP 1: Send global properties (first message in stream)
    // ========================================================================
    let global_props = WireGlobalProperties {
        metadata_identifier: "streaming-test-model".to_string(),
        crs: "https://www.opengis.net/def/crs/EPSG/0/7415".to_string(),
        transform_scale: [1.0, 1.0, 1.0],
        transform_translate: [0.0, 0.0, 0.0],
        geometry_templates: vec![
            // Template for GeometryInstance (shared across all buildings)
            WireTemplateGeometry {
                geometry_type: "MultiPoint".to_string(),
                lod: "1".to_string(),
                template_vertices: vec![
                    (0.0, 0.0, 0.0),
                    (5.0, 0.0, 0.0),
                    (5.0, 5.0, 0.0),
                    (0.0, 5.0, 0.0),
                ],
                boundaries: vec![vec![vec![0, 1, 2, 3]]],
            },
        ],
    };

    tx.send(StreamMessage::GlobalProperties(global_props))
        .expect("Failed to send global properties");

    // ========================================================================
    // STEP 2: Send CityObject messages (one per building)
    // ========================================================================
    let building_count = 5;

    for i in 0..building_count {
        // Define vertices for this building (simple box)
        let vertices = vec![
            (100 + i * 50, 200 + i * 50, 0),  // 0: bottom-front-left
            (120 + i * 50, 200 + i * 50, 0),  // 1: bottom-front-right
            (120 + i * 50, 220 + i * 50, 0),  // 2: bottom-back-right
            (100 + i * 50, 220 + i * 50, 0),  // 3: bottom-back-left
            (100 + i * 50, 200 + i * 50, 30), // 4: top-front-left
            (120 + i * 50, 200 + i * 50, 30), // 5: top-front-right
            (120 + i * 50, 220 + i * 50, 30), // 6: top-back-right
            (100 + i * 50, 220 + i * 50, 30), // 7: top-back-left
        ];

        // Define materials for this building
        let material_wall = WireMaterial {
            name: "wall-material".to_string(),
            ambient_intensity: None,
            diffuse_color: Some([0.7, 0.7, 0.7]),
            emissive_color: None,
            specular_color: None,
            shininess: None,
            transparency: None,
            is_smooth: None,
        };

        let material_roof = WireMaterial {
            name: "roof-material".to_string(),
            ambient_intensity: None,
            diffuse_color: Some([0.8, 0.2, 0.2]),
            emissive_color: None,
            specular_color: None,
            shininess: Some(0.5),
            transparency: None,
            is_smooth: None,
        };

        // Geometry 1: Solid with semantics and materials
        let geometry_solid = WireGeometry {
            geometry_type: "Solid".to_string(),
            lod: "2".to_string(),
            // Boundaries: [shells[surfaces[rings[vertex_indices]]]]
            boundaries: vec![
                // Shell 0
                vec![
                    // Surface 0: Bottom (GroundSurface)
                    vec![vec![0, 1, 2, 3]],
                    // Surface 1: Top/Roof (RoofSurface)
                    vec![vec![4, 5, 6, 7]],
                    // Surface 2: Front wall (WallSurface)
                    vec![vec![0, 1, 5, 4]],
                    // Surface 3: Right wall (WallSurface)
                    vec![vec![1, 2, 6, 5]],
                ],
            ],
            semantics: vec![
                Some(WireSemantic {
                    surface_type: "GroundSurface".to_string(),
                    attributes: vec![],
                }),
                Some(WireSemantic {
                    surface_type: "RoofSurface".to_string(),
                    attributes: vec![],
                }),
                Some(WireSemantic {
                    surface_type: "WallSurface".to_string(),
                    attributes: vec![],
                }),
                Some(WireSemantic {
                    surface_type: "WallSurface".to_string(),
                    attributes: vec![],
                }),
            ],
            materials: vec![
                ("default".to_string(), material_wall.clone()),
                ("default".to_string(), material_roof.clone()),
                ("default".to_string(), material_wall.clone()),
                ("default".to_string(), material_wall.clone()),
            ],
            template_ref: None,
            transformation_matrix: None,
        };

        // Geometry 2: GeometryInstance referencing template
        let geometry_instance = WireGeometry {
            geometry_type: "GeometryInstance".to_string(),
            lod: "1".to_string(),
            boundaries: vec![], // Not used for GeometryInstance
            semantics: vec![],
            materials: vec![],
            template_ref: Some(0), // References first template
            transformation_matrix: Some([
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                0.0,
                i as f64 * 10.0,
                i as f64 * 10.0,
                0.0,
                1.0,
            ]),
        };

        // Construct CityObject message
        let cityobject = WireCityObjectData {
            id: format!("building-{}", i),
            object_type: "Building".to_string(),
            vertices,
            geometries: vec![geometry_solid, geometry_instance],
            attributes: vec![
                (
                    "height".to_string(),
                    WireAttributeValue::Float(30.0 + i as f64 * 5.0),
                ),
                (
                    "yearOfConstruction".to_string(),
                    WireAttributeValue::Integer(2000 + i),
                ),
            ],
        };

        // Send CityObject to consumer
        tx.send(StreamMessage::CityObject(cityobject))
            .expect("Failed to send CityObject");
    }

    // ========================================================================
    // STEP 3: Signal end of stream
    // ========================================================================
    tx.send(StreamMessage::Done)
        .expect("Failed to send completion signal");
}

/// Consumer function that constructs CityModel from wire format and manages memory
///
/// Simulates realistic CityJSON stream processing:
/// 1. Receives GlobalProperties, builds templates and sets metadata
/// 2. Receives CityObjects, constructs cityjson-rs types from wire format
/// 3. Processes and removes CityObjects to maintain stable memory
fn consumer(rx: mpsc::Receiver<StreamMessage>) -> Result<()> {
    // Initialize CityModel
    let mut model =
        CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

    // Storage for template references (built from GlobalProperties)
    let mut template_refs: Vec<ResourceId32> = Vec::new();

    // Track memory metrics
    let mut buildings_processed = 0;
    let mut max_cityobjects = 0;
    let mut max_vertices = 0;

    // ========================================================================
    // PHASE 1: Process GlobalProperties (first message)
    // ========================================================================
    if let Ok(StreamMessage::GlobalProperties(global)) = rx.recv() {
        // Set metadata from a wire format
        model
            .metadata_mut()
            .set_identifier(CityModelIdentifier::new(global.metadata_identifier));
        model
            .metadata_mut()
            .set_reference_system(CRS::new(global.crs));

        // Set transform
        model.transform_mut().set_scale(global.transform_scale);
        model
            .transform_mut()
            .set_translate(global.transform_translate);

        // Build geometry templates from a wire format
        for wire_template in global.geometry_templates {
            let template_ref = build_template_from_wire(&mut model, &wire_template)?;
            template_refs.push(template_ref);
        }

        println!(
            "Consumer: Processed global properties, {} templates built",
            template_refs.len()
        );
    } else {
        return Err(Error::InvalidGeometry(
            "Expected GlobalProperties as first message".to_string(),
        ));
    }

    // ========================================================================
    // PHASE 2: Process CityObject messages
    // ========================================================================
    while let Ok(message) = rx.recv() {
        match message {
            StreamMessage::CityObject(wire_co) => {
                // Construct CityObject from wire format
                let mut cityobject = CityObject::new(
                    wire_co.id.clone(),
                    parse_city_object_type(&wire_co.object_type),
                );

                // Add attributes from wire format
                let attrs = cityobject.attributes_mut();
                for (key, wire_value) in wire_co.attributes {
                    attrs.insert(key, convert_wire_attribute_value(wire_value));
                }

                // Add vertices to the model
                let vertex_refs: Vec<VertexIndex<u32>> = wire_co
                    .vertices
                    .iter()
                    .map(|(x, y, z)| {
                        model
                            .add_vertex(QuantizedCoordinate::new(*x, *y, *z))
                            .expect("Failed to add vertex")
                    })
                    .collect();

                // Build geometries from wire format
                for wire_geom in wire_co.geometries {
                    let geom_ref = build_geometry_from_wire(
                        &mut model,
                        &wire_geom,
                        &vertex_refs,
                        &template_refs,
                    )?;
                    cityobject.geometry_mut().push(geom_ref);
                }

                // Add CityObject to model
                let cityobject_ref = model.cityobjects_mut().add(cityobject);

                // Track memory metrics
                max_cityobjects = max_cityobjects.max(model.cityobjects().len());
                max_vertices = max_vertices.max(model.vertices().len());

                // Process the CityObject (extract information, validate, etc.)
                let processed_object = model.cityobjects().get(cityobject_ref).unwrap();
                println!(
                    "Processed: {} (type: {}, geometries: {})",
                    processed_object.id(),
                    processed_object.type_cityobject(),
                    processed_object.geometry().map_or(0, |g| g.len())
                );

                // Remove CityObject from the model to maintain stable memory
                let removed = model.cityobjects_mut().remove(cityobject_ref);
                assert!(removed.is_some(), "Failed to remove CityObject");

                buildings_processed += 1;

                // Verify memory is bounded-count active objects via iterator
                let active_objects = model.cityobjects().iter().count();
                assert_eq!(
                    active_objects, 0,
                    "CityObjects should be removed after processing"
                );
            }
            StreamMessage::Done => {
                println!("Consumer: Received end-of-stream signal");
                break;
            }
            StreamMessage::GlobalProperties(_) => {
                return Err(Error::InvalidGeometry(
                    "Unexpected GlobalProperties message after stream started".to_string(),
                ));
            }
        }
    }

    // Final assertions
    assert_eq!(buildings_processed, 5, "Should have processed 5 buildings");

    // Verify all CityObjects were removed (count active objects via iterator)
    let final_active_objects = model.cityobjects().iter().count();
    assert_eq!(final_active_objects, 0, "All CityObjects should be removed");

    println!(
        "Stream processing complete: {} buildings processed, max_objects={}, max_vertices={}",
        buildings_processed, max_cityobjects, max_vertices
    );

    Ok(())
}

// ============================================================================
// Helper Functions: Convert Wire Format → cityjson-rs Types
// ============================================================================

/// Builds a geometry template from wire format
fn build_template_from_wire(
    model: &mut CityModel<u32, ResourceId32, OwnedStringStorage>,
    wire_template: &WireTemplateGeometry,
) -> Result<ResourceId32> {
    let geom_type = match wire_template.geometry_type.as_str() {
        "MultiPoint" => GeometryType::MultiPoint,
        "MultiSurface" => GeometryType::MultiSurface,
        _ => {
            return Err(Error::InvalidGeometry(format!(
                "Unsupported template geometry type: {}",
                wire_template.geometry_type
            )));
        }
    };

    let lod = parse_lod(&wire_template.lod);

    let mut builder = GeometryBuilder::new(model, geom_type, BuilderMode::Template).with_lod(lod);

    // Add template points
    for (x, y, z) in &wire_template.template_vertices {
        builder.add_template_point(RealWorldCoordinate::new(*x, *y, *z));
    }

    // Build template geometry
    builder.build()
}

/// Builds a geometry from wire format
fn build_geometry_from_wire(
    model: &mut CityModel<u32, ResourceId32, OwnedStringStorage>,
    wire_geom: &WireGeometry,
    vertex_refs: &[VertexIndex<u32>],
    template_refs: &[ResourceId32],
) -> Result<ResourceId32> {
    let lod = parse_lod(&wire_geom.lod);

    // Handle GeometryInstance separately
    if wire_geom.geometry_type == "GeometryInstance" {
        let template_idx = wire_geom.template_ref.ok_or_else(|| {
            Error::InvalidGeometry("GeometryInstance missing template_ref".to_string())
        })?;
        let template_ref = template_refs.get(template_idx).ok_or_else(|| {
            Error::InvalidGeometry(format!("Invalid template reference: {}", template_idx))
        })?;

        return GeometryBuilder::new(model, GeometryType::GeometryInstance, BuilderMode::Regular)
            .with_template(*template_ref)?
            .with_transformation_matrix(wire_geom.transformation_matrix.unwrap())?
            .with_reference_vertex(vertex_refs[0])
            .build();
    }

    // Handle regular geometries (Solid, MultiSurface, etc.)
    let geom_type = match wire_geom.geometry_type.as_str() {
        "Solid" => GeometryType::Solid,
        "MultiSurface" => GeometryType::MultiSurface,
        _ => {
            return Err(Error::InvalidGeometry(format!(
                "Unsupported geometry type: {}",
                wire_geom.geometry_type
            )));
        }
    };

    let mut builder = GeometryBuilder::new(model, geom_type, BuilderMode::Regular).with_lod(lod);

    // Add vertices to builder
    let bv: Vec<_> = vertex_refs
        .iter()
        .map(|vref| builder.add_vertex(*vref))
        .collect();

    // Build geometry from boundaries
    for shell in wire_geom.boundaries.iter() {
        let mut surface_ids = Vec::new();

        for (surface_idx, surface_rings) in shell.iter().enumerate() {
            let surface_id = builder.start_surface();
            surface_ids.push(surface_id);

            // Add an outer ring
            if let Some(outer_ring_indices) = surface_rings.first() {
                let ring_verts: Vec<usize> =
                    outer_ring_indices.iter().map(|&idx| bv[idx]).collect();
                let ring_id = builder.add_ring(&ring_verts)?;
                builder.add_surface_outer_ring(ring_id)?;
            }

            // Add inner rings (if any)
            for inner_ring_indices in surface_rings.iter().skip(1) {
                let ring_verts: Vec<usize> =
                    inner_ring_indices.iter().map(|&idx| bv[idx]).collect();
                let ring_id = builder.add_ring(&ring_verts)?;
                builder.add_surface_inner_ring(ring_id)?;
            }

            // Add semantic from wire format
            if let Some(wire_semantic) = wire_geom
                .semantics
                .get(surface_idx)
                .and_then(|s| s.as_ref())
            {
                let semantic = convert_wire_semantic(wire_semantic)?;
                builder.set_semantic_surface(None, semantic)?;
            }

            // Add materials from a wire format
            if let Some((theme, wire_material)) = wire_geom.materials.get(surface_idx) {
                let material = convert_wire_material(wire_material);
                builder.set_material_surface(None, material, theme.clone())?;
            }
        }

        // Add shell
        builder.add_shell(&surface_ids)?;
    }

    builder.build()
}

/// Converts wire semantic to cityjson-rs Semantic
fn convert_wire_semantic(
    wire_semantic: &WireSemantic,
) -> Result<Semantic<ResourceId32, OwnedStringStorage>> {
    let semantic_type = match wire_semantic.surface_type.as_str() {
        "RoofSurface" => SemanticType::RoofSurface,
        "GroundSurface" => SemanticType::GroundSurface,
        "WallSurface" => SemanticType::WallSurface,
        "ClosureSurface" => SemanticType::ClosureSurface,
        "OuterCeilingSurface" => SemanticType::OuterCeilingSurface,
        "OuterFloorSurface" => SemanticType::OuterFloorSurface,
        "Window" => SemanticType::Window,
        "Door" => SemanticType::Door,
        _ => {
            return Err(Error::InvalidGeometry(format!(
                "Unknown semantic type: {}",
                wire_semantic.surface_type
            )));
        }
    };

    Ok(Semantic::new(semantic_type))
}

/// Converts wire material to cityjson-rs Material
fn convert_wire_material(wire_material: &WireMaterial) -> Material<OwnedStringStorage> {
    let mut material = Material::new(wire_material.name.clone());

    if let Some(val) = wire_material.ambient_intensity {
        material.set_ambient_intensity(Some(val as f32));
    }
    if let Some(val) = wire_material.diffuse_color {
        material.set_diffuse_color(Some([val[0] as f32, val[1] as f32, val[2] as f32]));
    }
    if let Some(val) = wire_material.emissive_color {
        material.set_emissive_color(Some([val[0] as f32, val[1] as f32, val[2] as f32]));
    }
    if let Some(val) = wire_material.specular_color {
        material.set_specular_color(Some([val[0] as f32, val[1] as f32, val[2] as f32]));
    }
    if let Some(val) = wire_material.shininess {
        material.set_shininess(Some(val as f32));
    }
    if let Some(val) = wire_material.transparency {
        material.set_transparency(Some(val as f32));
    }
    if let Some(val) = wire_material.is_smooth {
        material.set_is_smooth(Some(val));
    }

    material
}

/// Converts wire attribute value to cityjson-rs AttributeValue
fn convert_wire_attribute_value(
    wire_value: WireAttributeValue,
) -> AttributeValue<OwnedStringStorage, ResourceId32> {
    match wire_value {
        WireAttributeValue::String(s) => AttributeValue::String(s),
        WireAttributeValue::Float(f) => AttributeValue::Float(f),
        WireAttributeValue::Integer(i) => AttributeValue::Integer(i),
        WireAttributeValue::Bool(b) => AttributeValue::Bool(b),
    }
}

/// Parses CityObject type string
fn parse_city_object_type(type_str: &str) -> CityObjectType<OwnedStringStorage> {
    match type_str {
        "Building" => CityObjectType::Building,
        "BuildingPart" => CityObjectType::BuildingPart,
        "Road" => CityObjectType::Road,
        _ => CityObjectType::GenericCityObject,
    }
}

/// Parses LoD string
fn parse_lod(lod_str: &str) -> LoD {
    match lod_str {
        "0" => LoD::LoD0,
        "1" => LoD::LoD1,
        "2" => LoD::LoD2,
        "3" => LoD::LoD3,
        _ => LoD::LoD1,
    }
}
