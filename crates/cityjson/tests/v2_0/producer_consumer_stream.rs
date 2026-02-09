use cityjson::backend::default::geometry::GeometryBuilder;
use cityjson::prelude::*;
use cityjson::resources::pool::ResourceId32;
use cityjson::v2_0::*;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

// ============================================================================
// Wire Types + Constants
// ============================================================================

const NR_BUILDINGS: usize = 10_000;
const BATCH_SIZE: usize = 1_000; // Process buildings in batches to maintain stable memory
const STREAM_VERBOSE_ENV: &str = "STREAM_VERBOSE";

const BOUNDARIES_SIMPLE: &[&[&[usize]]] = &[
    &[&[0, 1, 2, 3]], // Bottom
    &[&[4, 5, 6, 7]], // Top
    &[&[0, 1, 5, 4]], // Front
    &[&[1, 2, 6, 5]], // Right
    &[&[2, 3, 7, 6]], // Back
    &[&[3, 0, 4, 7]], // Left
];

const BOUNDARIES_MEDIUM: &[&[&[usize]]] = &[
    &[&[0, 1, 2, 3]],   // Bottom
    &[&[4, 5, 6, 7]],   // Top
    &[&[0, 1, 9, 8]],   // Front lower
    &[&[8, 9, 5, 4]],   // Front upper
    &[&[1, 2, 10, 9]],  // Right lower
    &[&[9, 10, 6, 5]],  // Right upper
    &[&[2, 3, 11, 10]], // Back lower
    &[&[10, 11, 7, 6]], // Back upper
    &[&[3, 0, 8, 11]],  // Left lower
    &[&[11, 8, 4, 7]],  // Left upper
];

const BOUNDARIES_COMPLEX: &[&[&[usize]]] = &[
    &[&[0, 1, 2, 3]],   // Bottom
    &[&[4, 5, 6, 7]],   // Top
    &[&[0, 12, 16, 4]], // Front left
    &[&[12, 1, 5, 16]], // Front right
    &[&[1, 15, 19, 5]], // Right front
    &[&[15, 2, 6, 19]], // Right back
    &[&[2, 13, 17, 6]], // Back right
    &[&[13, 3, 7, 17]], // Back left
    &[&[3, 14, 18, 7]], // Left back
    &[&[14, 0, 4, 18]], // Left front
    &[&[12, 15, 1]],    // Bottom detail
    &[&[13, 14, 3]],    // Bottom detail
    &[&[16, 19, 5]],    // Top detail
    &[&[17, 18, 7]],    // Top detail
];

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
    /// Boundaries in nested Vec format (as in `CityJSON` spec)
    /// For Solid:
    /// Vec<shells> where shell = Vec<surfaces> where surface = Vec<rings> where ring = Vec<`vertex_indices`>
    boundaries: Vec<Vec<Vec<Vec<usize>>>>,
    /// Semantic surface info, one per surface
    semantics: Vec<Option<WireSemantic>>,
    /// Materials per surface, keyed by theme name
    materials: Vec<(String, WireMaterial)>, // (theme, material)
    /// For `GeometryInstance`: reference to template index
    template_ref: Option<usize>,
    /// For `GeometryInstance`: transformation matrix
    transformation_matrix: Option<[f64; 16]>,
}

/// Represents template geometry data in global properties
#[derive(Debug, Clone)]
struct WireTemplateGeometry {
    geometry_type: String,
    lod: String,
    /// Template vertices as real-world coordinates
    template_vertices: Vec<(f64, f64, f64)>,
    /// Boundaries referencing `template_vertices`
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

/// `CityObject` data as it would appear in parsed `CityJSON` stream
#[derive(Debug, Clone)]
struct WireCityObjectData {
    id: String,
    object_type: String,
    /// Quantized vertex coordinates
    vertices: Vec<(i64, i64, i64)>,
    /// Geometries for this `CityObject`
    geometries: Vec<WireGeometry>,
    /// Attributes
    attributes: Vec<(String, WireAttributeValue)>,
}

/// Message types for streaming communication
#[derive(Debug, Clone)]
enum StreamMessage {
    /// First message: global model properties
    GlobalProperties(WireGlobalProperties),
    /// Subsequent messages: individual `CityObjects`
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

// ============================================================================
// Wire Data Generators
// ============================================================================

type WireAttribute = (String, WireAttributeValue);
type OwnedModel = CityModel<u32, OwnedStringStorage>;
type WireBoundaries = Vec<Vec<Vec<Vec<usize>>>>;
type WireSemantics = Vec<Option<WireSemantic>>;
type WireMaterials = Vec<(String, WireMaterial)>;
type SolidGeometryComponents = (WireBoundaries, WireSemantics, WireMaterials);

fn stream_verbose_enabled() -> bool {
    std::env::var(STREAM_VERBOSE_ENV).ok().is_some_and(|val| {
        matches!(
            val.trim().to_lowercase().as_str(),
            "1" | "true" | "yes" | "y" | "on"
        )
    })
}

fn u64_to_u32_checked(value: u64, context: &str) -> u32 {
    u32::try_from(value).unwrap_or_else(|_| panic!("{context} out of u32 range: {value}"))
}

fn u64_to_i64_checked(value: u64, context: &str) -> i64 {
    i64::try_from(value).unwrap_or_else(|_| panic!("{context} out of i64 range: {value}"))
}

fn i64_to_i32_checked(value: i64, context: &str) -> i32 {
    i32::try_from(value).unwrap_or_else(|_| panic!("{context} out of i32 range: {value}"))
}

fn u64_to_f64_checked(value: u64, context: &str) -> f64 {
    f64::from(u64_to_u32_checked(value, context))
}

fn i64_to_f64_checked(value: i64, context: &str) -> f64 {
    f64::from(i64_to_i32_checked(value, context))
}

fn boundaries_from_table(table: &[&[&[usize]]]) -> WireBoundaries {
    vec![
        table
            .iter()
            .map(|surface| surface.iter().map(|ring| ring.to_vec()).collect())
            .collect(),
    ]
}

fn semantics_for(count: usize, pattern: &[&str]) -> WireSemantics {
    let mut semantics = Vec::with_capacity(count);
    for idx in 0..count {
        let surface_type = pattern[idx % pattern.len()].to_string();
        semantics.push(Some(WireSemantic {
            surface_type,
            attributes: vec![],
        }));
    }
    semantics
}

fn materials_for(count: usize, wall: &WireMaterial, roof: &WireMaterial) -> WireMaterials {
    (0..count)
        .map(|idx| {
            let material = if idx == 1 { roof } else { wall };
            ("default".to_string(), material.clone())
        })
        .collect()
}

fn push_numeric_attributes(attributes: &mut Vec<WireAttribute>, i: u64, height: i64) {
    let height_f64 = i64_to_f64_checked(height, "height");
    attributes.extend([
        ("height".to_string(), WireAttributeValue::Float(height_f64)),
        (
            "floorArea".to_string(),
            WireAttributeValue::Float(
                400.0 + u64_to_f64_checked(i % 100, "floorArea offset") * 10.0,
            ),
        ),
        (
            "volume".to_string(),
            WireAttributeValue::Float(height_f64 * 400.0),
        ),
        (
            "yearOfConstruction".to_string(),
            WireAttributeValue::Integer(
                1950 + u64_to_i64_checked(i % 75, "yearOfConstruction offset"),
            ),
        ),
        (
            "renovationYear".to_string(),
            WireAttributeValue::Integer(2000 + u64_to_i64_checked(i % 25, "renovationYear offset")),
        ),
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
    ]);
}

fn push_flag_attributes(attributes: &mut Vec<WireAttribute>, i: u64) {
    attributes.extend([
        (
            "isCommercial".to_string(),
            WireAttributeValue::Bool(i.is_multiple_of(5)),
        ),
        (
            "hasElevator".to_string(),
            WireAttributeValue::Bool(i.is_multiple_of(3)),
        ),
        (
            "hasParking".to_string(),
            WireAttributeValue::Bool(i.is_multiple_of(2)),
        ),
    ]);
}

fn push_text_attributes(attributes: &mut Vec<WireAttribute>, i: u64) {
    let building_class = match i % 4 {
        0 => "residential",
        1 => "commercial",
        2 => "industrial",
        _ => "mixed-use",
    };

    attributes.extend([
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
            WireAttributeValue::String(building_class.to_string()),
        ),
    ]);
}

fn push_vector_attributes(attributes: &mut Vec<WireAttribute>, i: u64, height: i64) {
    attributes.extend([
        (
            "historicalDates".to_string(),
            WireAttributeValue::Vec(vec![
                WireAttributeValue::Integer(
                    1950 + u64_to_i64_checked(i % 50, "historicalDates[0]"),
                ),
                WireAttributeValue::Integer(
                    1980 + u64_to_i64_checked(i % 30, "historicalDates[1]"),
                ),
                WireAttributeValue::Integer(
                    2010 + u64_to_i64_checked(i % 15, "historicalDates[2]"),
                ),
            ]),
        ),
        (
            "measurements".to_string(),
            WireAttributeValue::Vec(vec![
                WireAttributeValue::Float(i64_to_f64_checked(height, "measurements[0]")),
                WireAttributeValue::Float(20.0),
                WireAttributeValue::Float(20.0),
            ]),
        ),
    ]);
}

fn push_map_attributes(attributes: &mut Vec<WireAttribute>, i: u64) {
    let rating_class = match i % 7 {
        0 => "A++",
        1 => "A+",
        2 => "A",
        3 => "B",
        4 => "C",
        5 => "D",
        _ => "E",
    };
    let ownership_type = if i.is_multiple_of(2) {
        "private"
    } else {
        "public"
    };

    attributes.extend([
        (
            "energyRating".to_string(),
            WireAttributeValue::Map(vec![
                (
                    "class".to_string(),
                    WireAttributeValue::String(rating_class.to_string()),
                ),
                (
                    "consumption".to_string(),
                    WireAttributeValue::Float(50.0 + u64_to_f64_checked(i % 150, "consumption")),
                ),
                (
                    "certified".to_string(),
                    WireAttributeValue::Bool(i.is_multiple_of(2)),
                ),
            ]),
        ),
        (
            "ownership".to_string(),
            WireAttributeValue::Map(vec![
                (
                    "type".to_string(),
                    WireAttributeValue::String(ownership_type.to_string()),
                ),
                (
                    "cadastralId".to_string(),
                    WireAttributeValue::String(format!("CAD-{i:06}")),
                ),
                (
                    "registrationYear".to_string(),
                    WireAttributeValue::Integer(
                        1990 + u64_to_i64_checked(i % 35, "registrationYear"),
                    ),
                ),
            ]),
        ),
    ]);
}

fn make_wire_attributes(i: u64, height: i64) -> Vec<WireAttribute> {
    let mut attributes = Vec::with_capacity(19);
    push_numeric_attributes(&mut attributes, i, height);
    push_flag_attributes(&mut attributes, i);
    push_text_attributes(&mut attributes, i);
    push_vector_attributes(&mut attributes, i, height);
    push_map_attributes(&mut attributes, i);

    if i.is_multiple_of(10) {
        attributes.push(("futureExpansion".to_string(), WireAttributeValue::Null));
    }

    attributes
}

fn wire_geometry_solid(
    boundaries: WireBoundaries,
    semantics: WireSemantics,
    materials: WireMaterials,
) -> WireGeometry {
    WireGeometry {
        geometry_type: "Solid".to_string(),
        lod: "2".to_string(),
        boundaries,
        semantics,
        materials,
        template_ref: None,
        transformation_matrix: None,
    }
}

fn wire_geometry_instance(template_ref: usize, transform: [f64; 16]) -> WireGeometry {
    WireGeometry {
        geometry_type: "GeometryInstance".to_string(),
        lod: "1".to_string(),
        boundaries: vec![],
        semantics: vec![],
        materials: vec![],
        template_ref: Some(template_ref),
        transformation_matrix: Some(transform),
    }
}

// ============================================================================
// Producer/Consumer Runtime
// ============================================================================

/// Producer-consumer streaming test with batch-based memory management
///
/// This test simulates realistic `CityJSON` stream processing using batch processing:
/// 1. Producer sends global properties (metadata, transform, geometry templates)
/// 2. Producer sends individual `CityObjects` with boundary/semantic/material data
/// 3. Consumer processes `CityObjects` in batches of `BATCH_SIZE` buildings
/// 4. Each batch uses a fresh `CityModel` that is dropped after processing
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
            producer(&tx);
            let producer_duration = producer_start.elapsed();
            if stream_verbose_enabled() {
                println!(
                    "Producer finished in {:.2}s",
                    producer_duration.as_secs_f64()
                );
            }
        });

        // Consumer thread - ingests and constructs CityModel
        let consumer_handle = s.spawn(move || {
            let consumer_start = Instant::now();
            let result = consumer(&rx);
            let consumer_duration = consumer_start.elapsed();
            if stream_verbose_enabled() {
                println!(
                    "Consumer finished in {:.2}s",
                    consumer_duration.as_secs_f64()
                );
            }
            result
        });

        // Wait for both threads to finish
        producer_handle.join().expect("Producer thread panicked");
        consumer_handle.join().expect("Consumer thread panicked")
    });

    let total_duration = start_time.elapsed();
    let total_duration_secs = total_duration.as_secs_f64();
    let building_count = f64::from(u64_to_u32_checked(nr_buildings(), "NR_BUILDINGS"));
    let throughput = building_count / total_duration_secs;

    println!("\n========== Overall Test Summary ==========");
    println!("Total test duration: {total_duration_secs:.2}s");
    println!("Throughput: {throughput:.0} buildings/sec");
    println!(
        "Average processing time per building: {:.3}ms",
        (total_duration_secs / building_count) * 1000.0
    );
    println!("==========================================\n");

    result
}

fn nr_buildings() -> u64 {
    u64::try_from(NR_BUILDINGS).expect("NR_BUILDINGS must fit in u64")
}

fn build_global_properties() -> WireGlobalProperties {
    WireGlobalProperties {
        metadata_identifier: "streaming-test-model".to_string(),
        crs: "https://www.opengis.net/def/crs/EPSG/0/7415".to_string(),
        transform_scale: [1.0, 1.0, 1.0],
        transform_translate: [0.0, 0.0, 0.0],
        geometry_templates: vec![WireTemplateGeometry {
            geometry_type: "MultiPoint".to_string(),
            lod: "1".to_string(),
            template_vertices: vec![
                (0.0, 0.0, 0.0),
                (5.0, 0.0, 0.0),
                (5.0, 5.0, 0.0),
                (0.0, 5.0, 0.0),
            ],
            boundaries: vec![vec![vec![0, 1, 2, 3]]],
        }],
    }
}

fn building_position(i: u64) -> (i64, i64, i64, u64) {
    let complexity = i % 3;
    let base_x = u64_to_i64_checked(100 + (i * 50) % 10_000, "base_x");
    let base_y = u64_to_i64_checked(200 + (i * 37) % 10_000, "base_y");
    let height = u64_to_i64_checked(30 + (i % 20) * 3, "height");
    (base_x, base_y, height, complexity)
}

fn building_vertices(
    base_x: i64,
    base_y: i64,
    height: i64,
    complexity: u64,
) -> Vec<(i64, i64, i64)> {
    let mut vertices = vec![
        (base_x, base_y, 0),
        (base_x + 20, base_y, 0),
        (base_x + 20, base_y + 20, 0),
        (base_x, base_y + 20, 0),
        (base_x, base_y, height),
        (base_x + 20, base_y, height),
        (base_x + 20, base_y + 20, height),
        (base_x, base_y + 20, height),
    ];

    if complexity >= 1 {
        let mid_height = height / 2;
        vertices.extend_from_slice(&[
            (base_x, base_y, mid_height),
            (base_x + 20, base_y, mid_height),
            (base_x + 20, base_y + 20, mid_height),
            (base_x, base_y + 20, mid_height),
        ]);
    }

    if complexity == 2 {
        vertices.extend_from_slice(&[
            (base_x + 10, base_y, 0),
            (base_x + 10, base_y + 20, 0),
            (base_x, base_y + 10, 0),
            (base_x + 20, base_y + 10, 0),
            (base_x + 10, base_y, height),
            (base_x + 10, base_y + 20, height),
            (base_x, base_y + 10, height),
            (base_x + 20, base_y + 10, height),
        ]);
    }

    vertices
}

fn wall_material() -> WireMaterial {
    WireMaterial {
        name: "wall-material".to_string(),
        ambient_intensity: None,
        diffuse_color: Some([0.7, 0.7, 0.7]),
        emissive_color: None,
        specular_color: None,
        shininess: None,
        transparency: None,
        is_smooth: None,
    }
}

fn roof_material() -> WireMaterial {
    WireMaterial {
        name: "roof-material".to_string(),
        ambient_intensity: None,
        diffuse_color: Some([0.8, 0.2, 0.2]),
        emissive_color: None,
        specular_color: None,
        shininess: Some(0.5),
        transparency: None,
        is_smooth: None,
    }
}

fn solid_geometry_components(
    complexity: u64,
    material_wall: &WireMaterial,
    material_roof: &WireMaterial,
) -> SolidGeometryComponents {
    match complexity {
        0 => (
            boundaries_from_table(BOUNDARIES_SIMPLE),
            semantics_for(
                BOUNDARIES_SIMPLE.len(),
                &[
                    "GroundSurface",
                    "RoofSurface",
                    "WallSurface",
                    "WallSurface",
                    "WallSurface",
                    "WallSurface",
                ],
            ),
            materials_for(BOUNDARIES_SIMPLE.len(), material_wall, material_roof),
        ),
        1 => (
            boundaries_from_table(BOUNDARIES_MEDIUM),
            semantics_for(
                BOUNDARIES_MEDIUM.len(),
                &[
                    "GroundSurface",
                    "RoofSurface",
                    "WallSurface",
                    "WallSurface",
                    "WallSurface",
                    "WallSurface",
                    "WallSurface",
                    "WallSurface",
                    "WallSurface",
                    "WallSurface",
                ],
            ),
            materials_for(BOUNDARIES_MEDIUM.len(), material_wall, material_roof),
        ),
        _ => {
            let mut materials =
                materials_for(BOUNDARIES_COMPLEX.len(), material_wall, material_roof);
            let last = materials.len().saturating_sub(1);
            if let Some(entry) = materials.get_mut(last) {
                entry.1 = material_roof.clone();
            }
            if let Some(entry) = materials.get_mut(last.saturating_sub(1)) {
                entry.1 = material_roof.clone();
            }
            (
                boundaries_from_table(BOUNDARIES_COMPLEX),
                semantics_for(
                    BOUNDARIES_COMPLEX.len(),
                    &[
                        "GroundSurface",
                        "RoofSurface",
                        "WallSurface",
                        "WallSurface",
                        "WallSurface",
                        "WallSurface",
                        "WallSurface",
                        "WallSurface",
                        "WallSurface",
                        "WallSurface",
                        "GroundSurface",
                        "GroundSurface",
                        "RoofSurface",
                        "RoofSurface",
                    ],
                ),
                materials,
            )
        }
    }
}

fn geometry_instance_transform(i: u64) -> [f64; 16] {
    let offset = u64_to_f64_checked(i, "geometry-instance offset") * 10.0;
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, offset, offset, 0.0, 1.0,
    ]
}

fn build_wire_cityobject(i: u64) -> WireCityObjectData {
    let (base_x, base_y, height, complexity) = building_position(i);
    let vertices = building_vertices(base_x, base_y, height, complexity);
    let material_wall = wall_material();
    let material_roof = roof_material();
    let (boundaries, semantics, materials) =
        solid_geometry_components(complexity, &material_wall, &material_roof);
    let geometry_solid = wire_geometry_solid(boundaries, semantics, materials);
    let geometry_instance = wire_geometry_instance(0, geometry_instance_transform(i));
    let attributes = make_wire_attributes(i, height);

    WireCityObjectData {
        id: format!("building-{i}"),
        object_type: "Building".to_string(),
        vertices,
        geometries: vec![geometry_solid, geometry_instance],
        attributes,
    }
}

/// Producer function that generates `CityJSON` stream data
///
/// Simulates a newline-delimited `CityJSON` stream:
/// 1. First line: global properties (metadata, CRS, transform, templates)
/// 2. Subsequent lines: individual `CityObjects`
/// 3. End of stream
fn producer(tx: &mpsc::SyncSender<StreamMessage>) {
    tx.send(StreamMessage::GlobalProperties(build_global_properties()))
        .expect("Failed to send global properties");

    let building_count = nr_buildings();

    for i in 0..building_count {
        if stream_verbose_enabled() && i % 100_000 == 0 && i > 0 {
            println!("Producer: Generated {i} / {building_count} buildings");
        }
        tx.send(StreamMessage::CityObject(build_wire_cityobject(i)))
            .expect("Failed to send CityObject");
    }

    tx.send(StreamMessage::Done)
        .expect("Failed to send completion signal");
}

/// Creates a new `CityModel` for a batch with templates and metadata from global properties
///
/// # Arguments
///
/// * `global` - Global properties containing metadata, CRS, transform, and geometry templates
///
/// # Returns
///
/// A tuple containing the initialized `CityModel` and a vector of template resource references
#[allow(clippy::type_complexity)]
fn create_batch_model(global: &WireGlobalProperties) -> Result<(OwnedModel, Vec<ResourceId32>)> {
    let mut model = OwnedModel::new(CityModelType::CityJSON);

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

// ============================================================================
// Metrics/Reporting Helpers
// ============================================================================

/// Processes a completed batch and extracts metrics
///
/// # Arguments
///
/// * `model` - The `CityModel` containing the batch
/// * `batch_num` - The batch number (for logging)
/// * `cumulative_buildings` - Total buildings processed so far across all batches
/// * `total_surfaces` - Total surfaces processed in this batch
///
/// # Returns
///
/// `BatchMetrics` for this batch
fn process_batch(
    model: &OwnedModel,
    batch_num: usize,
    cumulative_buildings: usize,
    total_surfaces: usize,
) -> BatchMetrics {
    let buildings_in_batch = model.cityobjects().iter().count();
    let vertices_in_batch = model.vertices().len();
    let geometries_in_batch = model.iter_geometries().count();

    // Print batch progress
    if stream_verbose_enabled() && (batch_num.is_multiple_of(100) || batch_num < 10) {
        println!(
            "Batch {batch_num}: {buildings_in_batch} buildings processed (total: {cumulative_buildings}), {vertices_in_batch} vertices, {geometries_in_batch} geometries, {total_surfaces} surfaces"
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

fn accumulate_metrics(cumulative_metrics: &mut BatchMetrics, batch_metrics: &BatchMetrics) {
    cumulative_metrics.total_geometries += batch_metrics.total_geometries;
    cumulative_metrics.total_surfaces += batch_metrics.total_surfaces;
    cumulative_metrics.peak_vertices = cumulative_metrics
        .peak_vertices
        .max(batch_metrics.peak_vertices);
    cumulative_metrics.peak_cityobjects = cumulative_metrics
        .peak_cityobjects
        .max(batch_metrics.peak_cityobjects);
}

fn count_surfaces(geometries: &[WireGeometry]) -> usize {
    geometries
        .iter()
        .flat_map(|wire_geom| wire_geom.boundaries.iter())
        .map(std::vec::Vec::len)
        .sum()
}

fn receive_global_properties(rx: &mpsc::Receiver<StreamMessage>) -> Result<WireGlobalProperties> {
    match rx.recv() {
        Ok(StreamMessage::GlobalProperties(global)) => {
            if stream_verbose_enabled() {
                println!(
                    "Consumer: Received global properties with {} templates",
                    global.geometry_templates.len()
                );
            }
            Ok(global)
        }
        _ => Err(Error::InvalidGeometry(
            "Expected GlobalProperties as first message".to_string(),
        )),
    }
}

fn ingest_wire_cityobject(
    model: &mut OwnedModel,
    template_refs: &[ResourceId32],
    wire_co: WireCityObjectData,
) -> Result<usize> {
    let WireCityObjectData {
        id,
        object_type,
        vertices,
        geometries,
        attributes,
    } = wire_co;
    let surfaces_in_cityobject = count_surfaces(&geometries);

    let mut cityobject = CityObject::new(
        CityObjectIdentifier::new(id),
        parse_city_object_type(&object_type),
    );

    let attrs = cityobject.attributes_mut();
    for (key, wire_value) in attributes {
        attrs.insert(key, convert_wire_attribute_value(wire_value));
    }

    let vertex_refs: Vec<VertexIndex<u32>> = vertices
        .into_iter()
        .map(|(x, y, z)| {
            model
                .add_vertex(QuantizedCoordinate::new(x, y, z))
                .expect("Failed to add vertex")
        })
        .collect();

    for wire_geom in geometries {
        let geom_ref = build_geometry_from_wire(model, &wire_geom, &vertex_refs, template_refs)?;
        cityobject.add_geometry(GeometryRef::from_parts(
            geom_ref.index(),
            geom_ref.generation(),
        ));
    }

    model.cityobjects_mut().add(cityobject)?;
    Ok(surfaces_in_cityobject)
}

struct ConsumerState {
    current_batch_num: usize,
    total_buildings_processed: usize,
    cumulative_metrics: BatchMetrics,
    current_model: OwnedModel,
    template_refs: Vec<ResourceId32>,
    buildings_in_batch: usize,
    surfaces_in_batch: usize,
}

impl ConsumerState {
    fn new(global_props: &WireGlobalProperties) -> Result<Self> {
        let (current_model, template_refs) = create_batch_model(global_props)?;
        Ok(Self {
            current_batch_num: 0,
            total_buildings_processed: 0,
            cumulative_metrics: BatchMetrics::default(),
            current_model,
            template_refs,
            buildings_in_batch: 0,
            surfaces_in_batch: 0,
        })
    }

    fn process_cityobject(&mut self, wire_co: WireCityObjectData) -> Result<()> {
        let surfaces =
            ingest_wire_cityobject(&mut self.current_model, &self.template_refs, wire_co)?;
        self.surfaces_in_batch += surfaces;
        self.buildings_in_batch += 1;
        self.total_buildings_processed += 1;
        Ok(())
    }

    fn finalize_batch(&mut self) {
        let batch_metrics = process_batch(
            &self.current_model,
            self.current_batch_num,
            self.total_buildings_processed,
            self.surfaces_in_batch,
        );
        accumulate_metrics(&mut self.cumulative_metrics, &batch_metrics);
    }

    fn rollover_batch(&mut self, global_props: &WireGlobalProperties) -> Result<()> {
        let (new_model, new_template_refs) = create_batch_model(global_props)?;
        self.current_model = new_model;
        self.template_refs = new_template_refs;
        self.current_batch_num += 1;
        self.buildings_in_batch = 0;
        self.surfaces_in_batch = 0;
        Ok(())
    }

    fn flush_if_full(&mut self, global_props: &WireGlobalProperties) -> Result<()> {
        if self.buildings_in_batch >= BATCH_SIZE {
            self.finalize_batch();
            self.rollover_batch(global_props)?;
        }
        Ok(())
    }

    fn finish(mut self) {
        if self.buildings_in_batch > 0 {
            self.finalize_batch();
        }

        assert_eq!(
            self.total_buildings_processed, NR_BUILDINGS,
            "Should have processed {NR_BUILDINGS} buildings"
        );

        println!("\n========== Performance Summary ==========");
        println!(
            "Total buildings processed: {}",
            self.total_buildings_processed
        );
        println!("Total batches: {}", self.current_batch_num + 1);
        println!(
            "Total geometries processed: {}",
            self.cumulative_metrics.total_geometries
        );
        println!(
            "Total surfaces processed: {}",
            self.cumulative_metrics.total_surfaces
        );
        println!(
            "Peak vertices per batch: {}",
            self.cumulative_metrics.peak_vertices
        );
        println!(
            "Peak CityObjects per batch: {}",
            self.cumulative_metrics.peak_cityobjects
        );
        println!("=========================================\n");
    }
}

/// Consumer function that constructs `CityModel` from wire format and manages memory
///
/// Simulates realistic `CityJSON` stream processing with batch processing:
/// 1. Receives `GlobalProperties`, extracts metadata and templates
/// 2. Processes `CityObjects` in batches, creating a fresh `CityModel` for each batch
/// 3. Each batch is processed independently and then dropped to free memory
fn consumer(rx: &mpsc::Receiver<StreamMessage>) -> Result<()> {
    let global_props = receive_global_properties(rx)?;
    let mut state = ConsumerState::new(&global_props)?;

    while let Ok(message) = rx.recv() {
        match message {
            StreamMessage::CityObject(wire_co) => {
                state.process_cityobject(wire_co)?;
                state.flush_if_full(&global_props)?;
            }
            StreamMessage::Done => {
                if stream_verbose_enabled() {
                    println!("Consumer: Received end-of-stream signal");
                }
                break;
            }
            StreamMessage::GlobalProperties(_) => {
                return Err(Error::InvalidGeometry(
                    "Unexpected GlobalProperties message after stream started".to_string(),
                ));
            }
        }
    }

    state.finish();
    Ok(())
}

// ============================================================================
// Helper Functions: Convert Wire Format → cityjson-rs Types
// ============================================================================

/// Builds a geometry template from wire format
fn build_template_from_wire(
    model: &mut CityModel<u32, OwnedStringStorage>,
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
    model: &mut CityModel<u32, OwnedStringStorage>,
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
            Error::InvalidGeometry(format!("Invalid template reference: {template_idx}"))
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
    for shell in &wire_geom.boundaries {
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
                builder.set_semantic_surface(None, semantic, false)?;
            }

            // Add materials from a wire format
            if let Some((theme, wire_material)) = wire_geom.materials.get(surface_idx) {
                let material = convert_wire_material(wire_material);
                builder.set_material_surface(None, material, theme.clone(), true)?;
            }
        }

        // Add shell
        builder.add_shell(&surface_ids)?;
    }

    builder.build()
}

/// Converts wire semantic to cityjson-rs Semantic
fn convert_wire_semantic(wire_semantic: &WireSemantic) -> Result<Semantic<OwnedStringStorage>> {
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

fn f64_to_f32_checked(value: f64, field: &str) -> f32 {
    assert!(value.is_finite(), "{field} must be finite, got {value}");
    assert!(
        (f64::from(f32::MIN)..=f64::from(f32::MAX)).contains(&value),
        "{field} out of f32 range: {value}"
    );

    value
        .to_string()
        .parse::<f32>()
        .unwrap_or_else(|error| panic!("Failed to convert {field} ({value}) to f32: {error}"))
}

fn f64_color_to_f32(color: [f64; 3], field: &str) -> [f32; 3] {
    [
        f64_to_f32_checked(color[0], field),
        f64_to_f32_checked(color[1], field),
        f64_to_f32_checked(color[2], field),
    ]
}

/// Converts wire material to cityjson-rs Material
fn convert_wire_material(wire_material: &WireMaterial) -> Material<OwnedStringStorage> {
    let mut material = Material::new(wire_material.name.clone());

    if let Some(val) = wire_material.ambient_intensity {
        material.set_ambient_intensity(Some(f64_to_f32_checked(val, "ambient_intensity")));
    }
    if let Some(val) = wire_material.diffuse_color {
        material.set_diffuse_color(Some(f64_color_to_f32(val, "diffuse_color").into()));
    }
    if let Some(val) = wire_material.emissive_color {
        material.set_emissive_color(Some(f64_color_to_f32(val, "emissive_color").into()));
    }
    if let Some(val) = wire_material.specular_color {
        material.set_specular_color(Some(f64_color_to_f32(val, "specular_color").into()));
    }
    if let Some(val) = wire_material.shininess {
        material.set_shininess(Some(f64_to_f32_checked(val, "shininess")));
    }
    if let Some(val) = wire_material.transparency {
        material.set_transparency(Some(f64_to_f32_checked(val, "transparency")));
    }
    if let Some(val) = wire_material.is_smooth {
        material.set_is_smooth(Some(val));
    }

    material
}

/// Converts wire attribute value to inline `AttributeValue`
fn convert_wire_attribute_value(
    wire_value: WireAttributeValue,
) -> AttributeValue<OwnedStringStorage> {
    match wire_value {
        WireAttributeValue::Null => AttributeValue::Null,
        WireAttributeValue::String(s) => AttributeValue::String(s),
        WireAttributeValue::Float(f) => AttributeValue::Float(f),
        WireAttributeValue::Integer(i) => AttributeValue::Integer(i),
        WireAttributeValue::Unsigned(u) => AttributeValue::Unsigned(u),
        WireAttributeValue::Bool(b) => AttributeValue::Bool(b),
        WireAttributeValue::Vec(vec) => {
            let elements: Vec<Box<AttributeValue<OwnedStringStorage>>> = vec
                .into_iter()
                .map(|v| Box::new(convert_wire_attribute_value(v)))
                .collect();
            AttributeValue::Vec(elements)
        }
        WireAttributeValue::Map(map) => {
            let mut element_map = HashMap::new();
            for (map_key, value) in map {
                let elem_value = Box::new(convert_wire_attribute_value(value));
                element_map.insert(map_key, elem_value);
            }
            AttributeValue::Map(element_map)
        }
    }
}

/// Parses `CityObject` type string
fn parse_city_object_type(type_str: &str) -> CityObjectType<OwnedStringStorage> {
    match type_str {
        "Building" => CityObjectType::Building,
        "BuildingPart" => CityObjectType::BuildingPart,
        "Road" => CityObjectType::Road,
        _ => CityObjectType::GenericCityObject,
    }
}

/// Parses `LoD` string
fn parse_lod(lod_str: &str) -> LoD {
    match lod_str {
        "0" => LoD::LoD0,
        "2" => LoD::LoD2,
        "3" => LoD::LoD3,
        _ => LoD::LoD1,
    }
}
