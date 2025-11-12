use cityjson::prelude::*;
use cityjson::v2_0::*;
use std::sync::mpsc;
use std::thread;

// ============================================================================
// Wire Format Structs - Simulate data structures from parsed CityJSON
// ============================================================================

const NR_BUILDINGS: usize = 10_000;
const BATCH_SIZE: usize = 1_000; // Process buildings in batches to maintain stable memory

/// Represents attribute values as they would appear in parsed JSON
#[derive(Debug, Clone)]
enum WireAttributeValue {
    Null,
    String(String),
    Float(f64),
    Integer(i64),
    Unsigned(u64),
    Bool(bool),
    Vec(Vec<WireAttributeValue>),
    Map(Vec<(String, WireAttributeValue)>),
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

/// Metrics collected from processing a batch of buildings
#[derive(Debug, Default)]
struct BatchMetrics {
    _buildings_processed: usize,
    total_geometries: usize,
    total_surfaces: usize,
    peak_vertices: usize,
    peak_cityobjects: usize,
}

/// Producer-consumer streaming test with batch-based memory management
///
/// This test simulates realistic CityJSON stream processing using batch processing:
/// 1. Producer sends global properties (metadata, transform, geometry templates)
/// 2. Producer sends individual CityObjects with boundary/semantic/material data
/// 3. Consumer processes CityObjects in batches of BATCH_SIZE buildings
/// 4. Each batch uses a fresh CityModel that is dropped after processing
/// 5. Memory stays bounded per batch instead of accumulating across all buildings
///
/// This approach mirrors real-world streaming parsers that process data in chunks
/// rather than holding all data in memory simultaneously.
#[test]
fn test_producer_consumer_stream() -> Result<()> {
    use std::time::Instant;

    // Create a bounded channel for producer-consumer communication with backpressure
    // Buffer size of 10 means producer will block if the consumer is slower
    let (tx, rx) = mpsc::sync_channel::<StreamMessage>(10);

    let start_time = Instant::now();

    // Spawn producer and consumer threads in a thread scope
    let result = thread::scope(|s| {
        // Producer thread - generates CityJSON stream data
        let producer_handle = s.spawn(move || {
            let producer_start = Instant::now();
            producer(tx);
            let producer_duration = producer_start.elapsed();
            println!(
                "Producer finished in {:.2}s",
                producer_duration.as_secs_f64()
            );
        });

        // Consumer thread - ingests and constructs CityModel
        let consumer_handle = s.spawn(move || {
            let consumer_start = Instant::now();
            let result = consumer(rx);
            let consumer_duration = consumer_start.elapsed();
            println!(
                "Consumer finished in {:.2}s",
                consumer_duration.as_secs_f64()
            );
            result
        });

        // Wait for both threads to finish
        producer_handle.join().expect("Producer thread panicked");
        consumer_handle.join().expect("Consumer thread panicked")
    });

    let total_duration = start_time.elapsed();
    let total_duration_secs = total_duration.as_secs_f64();
    let throughput = NR_BUILDINGS as f64 / total_duration_secs;

    println!("\n========== Overall Test Summary ==========");
    println!("Total test duration: {:.2}s", total_duration_secs);
    println!("Throughput: {:.0} buildings/sec", throughput);
    println!(
        "Average processing time per building: {:.3}ms",
        (total_duration_secs / NR_BUILDINGS as f64) * 1000.0
    );
    println!("==========================================\n");

    result
}

/// Producer function that generates CityJSON stream data
///
/// Simulates a newline-delimited CityJSON stream:
/// 1. First line: global properties (metadata, CRS, transform, templates)
/// 2. Subsequent lines: individual CityObjects
/// 3. End of stream
fn producer(tx: mpsc::SyncSender<StreamMessage>) {
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
    let building_count = NR_BUILDINGS as u64;

    for i in 0..building_count {
        // Print producer progresses every 100000 buildings
        if i % 100_000 == 0 && i > 0 {
            println!("Producer: Generated {} / {} buildings", i, building_count);
        }
        // Vary complexity: simple (8 verts), medium (16 verts), complex (24 verts)
        let complexity = i % 3;

        // Base coordinates with offset to create spatial variation
        let base_x = (100 + (i * 50) % 10000) as i64;
        let base_y = (200 + (i * 37) % 10000) as i64; // Use prime to vary the pattern
        let height = (30 + (i % 20) * 3) as i64; // Height varies 30-87

        // Define vertices for this building
        let mut vertices = vec![
            (base_x, base_y, 0),                // 0: bottom-front-left
            (base_x + 20, base_y, 0),           // 1: bottom-front-right
            (base_x + 20, base_y + 20, 0),      // 2: bottom-back-right
            (base_x, base_y + 20, 0),           // 3: bottom-back-left
            (base_x, base_y, height),           // 4: top-front-left
            (base_x + 20, base_y, height),      // 5: top-front-right
            (base_x + 20, base_y + 20, height), // 6: top-back-right
            (base_x, base_y + 20, height),      // 7: top-back-left
        ];

        // Add more vertices for medium complexity (add mid-level)
        if complexity >= 1 {
            let mid_height = height / 2;
            vertices.extend_from_slice(&[
                (base_x, base_y, mid_height),           // 8
                (base_x + 20, base_y, mid_height),      // 9
                (base_x + 20, base_y + 20, mid_height), // 10
                (base_x, base_y + 20, mid_height),      // 11
            ]);
        }

        // Add even more vertices for complex buildings (add details)
        if complexity == 2 {
            vertices.extend_from_slice(&[
                (base_x + 10, base_y, 0),           // 12
                (base_x + 10, base_y + 20, 0),      // 13
                (base_x, base_y + 10, 0),           // 14
                (base_x + 20, base_y + 10, 0),      // 15
                (base_x + 10, base_y, height),      // 16
                (base_x + 10, base_y + 20, height), // 17
                (base_x, base_y + 10, height),      // 18
                (base_x + 20, base_y + 10, height), // 19
            ]);
        }

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

        // Geometry 1: Solid with semantics and materials (complexity varies)
        let (boundaries, semantics, materials) = match complexity {
            0 => {
                // Simple: 6 surfaces (cube)
                let bounds = vec![vec![
                    vec![vec![0, 1, 2, 3]], // Bottom
                    vec![vec![4, 5, 6, 7]], // Top
                    vec![vec![0, 1, 5, 4]], // Front
                    vec![vec![1, 2, 6, 5]], // Right
                    vec![vec![2, 3, 7, 6]], // Back
                    vec![vec![3, 0, 4, 7]], // Left
                ]];
                let sems = vec![
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
                    Some(WireSemantic {
                        surface_type: "WallSurface".to_string(),
                        attributes: vec![],
                    }),
                    Some(WireSemantic {
                        surface_type: "WallSurface".to_string(),
                        attributes: vec![],
                    }),
                ];
                let mats = vec![
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_roof.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                ];
                (bounds, sems, mats)
            }
            1 => {
                // Medium: 10 surfaces (with mid-level)
                let bounds = vec![vec![
                    vec![vec![0, 1, 2, 3]],   // Bottom
                    vec![vec![4, 5, 6, 7]],   // Top
                    vec![vec![0, 1, 9, 8]],   // Front lower
                    vec![vec![8, 9, 5, 4]],   // Front upper
                    vec![vec![1, 2, 10, 9]],  // Right lower
                    vec![vec![9, 10, 6, 5]],  // Right upper
                    vec![vec![2, 3, 11, 10]], // Back lower
                    vec![vec![10, 11, 7, 6]], // Back upper
                    vec![vec![3, 0, 8, 11]],  // Left lower
                    vec![vec![11, 8, 4, 7]],  // Left upper
                ]];
                let sems = vec![
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
                    Some(WireSemantic {
                        surface_type: "WallSurface".to_string(),
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
                    Some(WireSemantic {
                        surface_type: "WallSurface".to_string(),
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
                ];
                let mats = vec![
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_roof.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                ];
                (bounds, sems, mats)
            }
            _ => {
                // Complex: 14 surfaces (with additional details)
                let bounds = vec![vec![
                    vec![vec![0, 1, 2, 3]],   // Bottom
                    vec![vec![4, 5, 6, 7]],   // Top
                    vec![vec![0, 12, 16, 4]], // Front left
                    vec![vec![12, 1, 5, 16]], // Front right
                    vec![vec![1, 15, 19, 5]], // Right front
                    vec![vec![15, 2, 6, 19]], // Right back
                    vec![vec![2, 13, 17, 6]], // Back right
                    vec![vec![13, 3, 7, 17]], // Back left
                    vec![vec![3, 14, 18, 7]], // Left back
                    vec![vec![14, 0, 4, 18]], // Left front
                    vec![vec![12, 15, 1]],    // Bottom detail
                    vec![vec![13, 14, 3]],    // Bottom detail
                    vec![vec![16, 19, 5]],    // Top detail
                    vec![vec![17, 18, 7]],    // Top detail
                ]];
                let sems = vec![
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
                    Some(WireSemantic {
                        surface_type: "WallSurface".to_string(),
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
                    Some(WireSemantic {
                        surface_type: "WallSurface".to_string(),
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
                    Some(WireSemantic {
                        surface_type: "GroundSurface".to_string(),
                        attributes: vec![],
                    }),
                    Some(WireSemantic {
                        surface_type: "GroundSurface".to_string(),
                        attributes: vec![],
                    }),
                    Some(WireSemantic {
                        surface_type: "RoofSurface".to_string(),
                        attributes: vec![],
                    }),
                    Some(WireSemantic {
                        surface_type: "RoofSurface".to_string(),
                        attributes: vec![],
                    }),
                ];
                let mats = vec![
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_roof.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_wall.clone()),
                    ("default".to_string(), material_roof.clone()),
                    ("default".to_string(), material_roof.clone()),
                ];
                (bounds, sems, mats)
            }
        };

        let geometry_solid = WireGeometry {
            geometry_type: "Solid".to_string(),
            lod: "2".to_string(),
            boundaries,
            semantics,
            materials,
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

        // Create comprehensive attributes using all AttributeValue types
        let mut attributes = vec![
            // Float: height, area, volume
            (
                "height".to_string(),
                WireAttributeValue::Float(height as f64),
            ),
            (
                "floorArea".to_string(),
                WireAttributeValue::Float(400.0 + (i % 100) as f64 * 10.0),
            ),
            (
                "volume".to_string(),
                WireAttributeValue::Float(height as f64 * 400.0),
            ),
            // Integer: year of construction
            (
                "yearOfConstruction".to_string(),
                WireAttributeValue::Integer(1950 + ((i % 75) as i64)),
            ),
            (
                "renovationYear".to_string(),
                WireAttributeValue::Integer(2000 + ((i % 25) as i64)),
            ),
            // Unsigned: counts
            (
                "floorCount".to_string(),
                WireAttributeValue::Unsigned(1 + (i % 20)),
            ),
            (
                "roomCount".to_string(),
                WireAttributeValue::Unsigned(3 + (i % 50)),
            ),
            (
                "windowCount".to_string(),
                WireAttributeValue::Unsigned(5 + (i % 30)),
            ),
            // Bool: flags
            (
                "isCommercial".to_string(),
                WireAttributeValue::Bool(i % 5 == 0),
            ),
            (
                "hasElevator".to_string(),
                WireAttributeValue::Bool(i % 3 == 0),
            ),
            (
                "hasParking".to_string(),
                WireAttributeValue::Bool(i % 2 == 0),
            ),
            // String: textual data
            (
                "owner".to_string(),
                WireAttributeValue::String(format!("Owner-{}", i % 100)),
            ),
            (
                "address".to_string(),
                WireAttributeValue::String(format!("{} Main St, City {}", i, i % 50)),
            ),
            (
                "buildingClass".to_string(),
                WireAttributeValue::String(
                    match i % 4 {
                        0 => "residential",
                        1 => "commercial",
                        2 => "industrial",
                        _ => "mixed-use",
                    }
                    .to_string(),
                ),
            ),
            // Vec: arrays of values
            (
                "historicalDates".to_string(),
                WireAttributeValue::Vec(vec![
                    WireAttributeValue::Integer(1950 + (i % 50) as i64),
                    WireAttributeValue::Integer(1980 + (i % 30) as i64),
                    WireAttributeValue::Integer(2010 + (i % 15) as i64),
                ]),
            ),
            (
                "measurements".to_string(),
                WireAttributeValue::Vec(vec![
                    WireAttributeValue::Float(height as f64),
                    WireAttributeValue::Float(20.0),
                    WireAttributeValue::Float(20.0),
                ]),
            ),
            // Map: nested objects
            (
                "energyRating".to_string(),
                WireAttributeValue::Map(vec![
                    (
                        "class".to_string(),
                        WireAttributeValue::String(
                            match i % 7 {
                                0 => "A++",
                                1 => "A+",
                                2 => "A",
                                3 => "B",
                                4 => "C",
                                5 => "D",
                                _ => "E",
                            }
                            .to_string(),
                        ),
                    ),
                    (
                        "consumption".to_string(),
                        WireAttributeValue::Float(50.0 + (i % 150) as f64),
                    ),
                    (
                        "certified".to_string(),
                        WireAttributeValue::Bool(i % 2 == 0),
                    ),
                ]),
            ),
            (
                "ownership".to_string(),
                WireAttributeValue::Map(vec![
                    (
                        "type".to_string(),
                        WireAttributeValue::String(
                            if i % 2 == 0 { "private" } else { "public" }.to_string(),
                        ),
                    ),
                    (
                        "cadastralId".to_string(),
                        WireAttributeValue::String(format!("CAD-{:06}", i)),
                    ),
                    (
                        "registrationYear".to_string(),
                        WireAttributeValue::Integer(1990 + (i % 35) as i64),
                    ),
                ]),
            ),
        ];

        // Add Null for some buildings
        if i % 10 == 0 {
            attributes.push(("futureExpansion".to_string(), WireAttributeValue::Null));
        }

        // Construct CityObject message
        let cityobject = WireCityObjectData {
            id: format!("building-{}", i),
            object_type: "Building".to_string(),
            vertices,
            geometries: vec![geometry_solid, geometry_instance],
            attributes,
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

/// Creates a new CityModel for a batch with templates and metadata from global properties
///
/// # Arguments
///
/// * `global` - Global properties containing metadata, CRS, transform, and geometry templates
///
/// # Returns
///
/// A tuple containing the initialized CityModel and a vector of template resource references
fn create_batch_model(
    global: &WireGlobalProperties,
) -> Result<(
    CityModel<u32, ResourceId32, OwnedStringStorage>,
    Vec<ResourceId32>,
)> {
    let mut model =
        CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

    // Set metadata from wire format
    model
        .metadata_mut()
        .set_identifier(CityModelIdentifier::new(global.metadata_identifier.clone()));
    model
        .metadata_mut()
        .set_reference_system(CRS::new(global.crs.clone()));

    // Set transform
    model.transform_mut().set_scale(global.transform_scale);
    model
        .transform_mut()
        .set_translate(global.transform_translate);

    // Build geometry templates from wire format
    let mut template_refs = Vec::new();
    for wire_template in &global.geometry_templates {
        let template_ref = build_template_from_wire(&mut model, wire_template)?;
        template_refs.push(template_ref);
    }

    Ok((model, template_refs))
}

/// Processes a completed batch and extracts metrics
///
/// # Arguments
///
/// * `model` - The CityModel containing the batch
/// * `batch_num` - The batch number (for logging)
/// * `cumulative_buildings` - Total buildings processed so far across all batches
/// * `total_surfaces` - Total surfaces processed in this batch
///
/// # Returns
///
/// BatchMetrics for this batch
fn process_batch(
    model: &CityModel<u32, ResourceId32, OwnedStringStorage>,
    batch_num: usize,
    cumulative_buildings: usize,
    total_surfaces: usize,
) -> BatchMetrics {
    let buildings_in_batch = model.cityobjects().iter().count();
    let vertices_in_batch = model.vertices().len();
    let geometries_in_batch = model.iter_geometries().count();

    // Print batch progress
    if batch_num.is_multiple_of(100) || batch_num < 10 {
        println!(
            "Batch {}: {} buildings processed (total: {}), {} vertices, {} geometries, {} surfaces",
            batch_num,
            buildings_in_batch,
            cumulative_buildings,
            vertices_in_batch,
            geometries_in_batch,
            total_surfaces
        );
    }

    BatchMetrics {
        _buildings_processed: buildings_in_batch,
        total_geometries: geometries_in_batch,
        total_surfaces,
        peak_vertices: vertices_in_batch,
        peak_cityobjects: buildings_in_batch,
    }
}

/// Consumer function that constructs CityModel from wire format and manages memory
///
/// Simulates realistic CityJSON stream processing with batch processing:
/// 1. Receives GlobalProperties, extracts metadata and templates
/// 2. Processes CityObjects in batches, creating a fresh CityModel for each batch
/// 3. Each batch is processed independently and then dropped to free memory
fn consumer(rx: mpsc::Receiver<StreamMessage>) -> Result<()> {
    // ========================================================================
    // PHASE 1: Receive and store GlobalProperties (needed for all batches)
    // ========================================================================
    let global_props = if let Ok(StreamMessage::GlobalProperties(global)) = rx.recv() {
        println!(
            "Consumer: Received global properties with {} templates",
            global.geometry_templates.len()
        );
        global
    } else {
        return Err(Error::InvalidGeometry(
            "Expected GlobalProperties as first message".to_string(),
        ));
    };

    // ========================================================================
    // PHASE 2: Process CityObjects in batches
    // ========================================================================
    let mut current_batch_num = 0;
    let mut total_buildings_processed = 0;
    let mut cumulative_metrics = BatchMetrics::default();

    // Create first batch
    let (mut current_model, template_refs) = create_batch_model(&global_props)?;
    let mut buildings_in_batch = 0;
    let mut surfaces_in_batch = 0;

    while let Ok(message) = rx.recv() {
        match message {
            StreamMessage::CityObject(wire_co) => {
                // Count surfaces from wire format before processing
                for wire_geom in &wire_co.geometries {
                    if !wire_geom.boundaries.is_empty() {
                        for shell in &wire_geom.boundaries {
                            surfaces_in_batch += shell.len();
                        }
                    }
                }

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
                        current_model
                            .add_vertex(QuantizedCoordinate::new(*x, *y, *z))
                            .expect("Failed to add vertex")
                    })
                    .collect();

                // Build geometries from wire format
                for wire_geom in wire_co.geometries {
                    let geom_ref = build_geometry_from_wire(
                        &mut current_model,
                        &wire_geom,
                        &vertex_refs,
                        &template_refs,
                    )?;
                    cityobject.geometry_mut().push(geom_ref);
                }

                // Add CityObject to model
                current_model.cityobjects_mut().add(cityobject);
                buildings_in_batch += 1;
                total_buildings_processed += 1;

                // When batch is full, process it and create a new batch
                if buildings_in_batch >= BATCH_SIZE {
                    let batch_metrics = process_batch(
                        &current_model,
                        current_batch_num,
                        total_buildings_processed,
                        surfaces_in_batch,
                    );

                    // Accumulate metrics
                    cumulative_metrics.total_geometries += batch_metrics.total_geometries;
                    cumulative_metrics.total_surfaces += batch_metrics.total_surfaces;
                    cumulative_metrics.peak_vertices = cumulative_metrics
                        .peak_vertices
                        .max(batch_metrics.peak_vertices);
                    cumulative_metrics.peak_cityobjects = cumulative_metrics
                        .peak_cityobjects
                        .max(batch_metrics.peak_cityobjects);

                    // Drop current model and create fresh one for next batch
                    drop(current_model);
                    let (new_model, _) = create_batch_model(&global_props)?;
                    current_model = new_model;

                    // Reset batch counters
                    current_batch_num += 1;
                    buildings_in_batch = 0;
                    surfaces_in_batch = 0;
                }
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

    // Process final partial batch if any buildings remain
    if buildings_in_batch > 0 {
        let batch_metrics = process_batch(
            &current_model,
            current_batch_num,
            total_buildings_processed,
            surfaces_in_batch,
        );

        cumulative_metrics.total_geometries += batch_metrics.total_geometries;
        cumulative_metrics.total_surfaces += batch_metrics.total_surfaces;
        cumulative_metrics.peak_vertices = cumulative_metrics
            .peak_vertices
            .max(batch_metrics.peak_vertices);
        cumulative_metrics.peak_cityobjects = cumulative_metrics
            .peak_cityobjects
            .max(batch_metrics.peak_cityobjects);
    }

    // Final assertions and summary
    assert_eq!(
        total_buildings_processed, NR_BUILDINGS,
        "Should have processed {} buildings",
        NR_BUILDINGS
    );

    println!("\n========== Performance Summary ==========");
    println!("Total buildings processed: {}", total_buildings_processed);
    println!("Total batches: {}", current_batch_num + 1);
    println!(
        "Total geometries processed: {}",
        cumulative_metrics.total_geometries
    );
    println!(
        "Total surfaces processed: {}",
        cumulative_metrics.total_surfaces
    );
    println!(
        "Peak vertices per batch: {}",
        cumulative_metrics.peak_vertices
    );
    println!(
        "Peak CityObjects per batch: {}",
        cumulative_metrics.peak_cityobjects
    );
    println!("=========================================\n");

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
        WireAttributeValue::Null => AttributeValue::Null,
        WireAttributeValue::String(s) => AttributeValue::String(s),
        WireAttributeValue::Float(f) => AttributeValue::Float(f),
        WireAttributeValue::Integer(i) => AttributeValue::Integer(i),
        WireAttributeValue::Unsigned(u) => AttributeValue::Unsigned(u),
        WireAttributeValue::Bool(b) => AttributeValue::Bool(b),
        WireAttributeValue::Vec(vec) => {
            let converted: Vec<Box<AttributeValue<OwnedStringStorage, ResourceId32>>> = vec
                .into_iter()
                .map(|v| Box::new(convert_wire_attribute_value(v)))
                .collect();
            AttributeValue::Vec(converted)
        }
        WireAttributeValue::Map(map) => {
            let mut hash_map = std::collections::HashMap::new();
            for (key, value) in map {
                hash_map.insert(key, Box::new(convert_wire_attribute_value(value)));
            }
            AttributeValue::Map(hash_map)
        }
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
