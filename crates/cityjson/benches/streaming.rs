//! Streaming benchmarks for end-to-end producer/consumer performance.

#[allow(dead_code)]
mod support;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use support::DEFAULT_SEED;

const DEFAULT_STREAM_SIZE: usize = 10_000;
const FAST_STREAM_SIZE: usize = 1_000;
const DEFAULT_STREAM_BATCH: usize = 1_000;

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

#[derive(Debug, Clone)]
struct WireSemantic {
    surface_type: String,
    #[allow(dead_code)]
    attributes: Vec<(String, WireAttributeValue)>,
}

#[derive(Debug, Clone)]
struct WireGeometry {
    geometry_type: String,
    lod: String,
    boundaries: Vec<Vec<Vec<Vec<usize>>>>,
    semantics: Vec<Option<WireSemantic>>,
    materials: Vec<(String, WireMaterial)>,
    template_ref: Option<usize>,
    transformation_matrix: Option<[f64; 16]>,
}

#[derive(Debug, Clone)]
struct WireTemplateGeometry {
    geometry_type: String,
    lod: String,
    template_vertices: Vec<(f64, f64, f64)>,
    #[allow(dead_code)]
    boundaries: Vec<Vec<Vec<usize>>>,
}

#[derive(Debug, Clone)]
struct WireGlobalProperties {
    metadata_identifier: String,
    crs: String,
    transform_scale: [f64; 3],
    transform_translate: [f64; 3],
    geometry_templates: Vec<WireTemplateGeometry>,
}

#[derive(Debug, Clone)]
struct WireCityObjectData {
    id: String,
    object_type: String,
    vertices: Vec<(i64, i64, i64)>,
    geometries: Vec<WireGeometry>,
    attributes: Vec<(String, WireAttributeValue)>,
}

#[derive(Debug, Clone)]
enum StreamMessage {
    GlobalProperties(WireGlobalProperties),
    CityObject(WireCityObjectData),
    Done,
}

#[derive(Debug, Default)]
struct BatchMetrics {
    _buildings_processed: usize,
}

#[derive(Debug)]
struct StreamMetrics {
    total_s: f64,
    producer_s: Option<f64>,
    consumer_s: f64,
    throughput_buildings_s: f64,
    mode: String,
}

fn stream_verbose_enabled() -> bool {
    env::var(STREAM_VERBOSE_ENV)
        .ok()
        .map(|val| {
            matches!(
                val.trim().to_lowercase().as_str(),
                "1" | "true" | "yes" | "y" | "on"
            )
        })
        .unwrap_or(false)
}

fn boundaries_from_table(table: &[&[&[usize]]]) -> Vec<Vec<Vec<Vec<usize>>>> {
    vec![table
        .iter()
        .map(|surface| surface.iter().map(|ring| ring.to_vec()).collect())
        .collect()]
}

fn semantics_for(count: usize, pattern: &[&str]) -> Vec<Option<WireSemantic>> {
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

fn materials_for(
    count: usize,
    wall: &WireMaterial,
    roof: &WireMaterial,
) -> Vec<(String, WireMaterial)> {
    (0..count)
        .map(|idx| {
            let material = if idx == 1 { roof } else { wall };
            ("default".to_string(), material.clone())
        })
        .collect()
}

fn make_wire_attributes(i: u64, height: i64) -> Vec<(String, WireAttributeValue)> {
    let mut attributes = vec![
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
        (
            "yearOfConstruction".to_string(),
            WireAttributeValue::Integer(1950 + ((i % 75) as i64)),
        ),
        (
            "renovationYear".to_string(),
            WireAttributeValue::Integer(2000 + ((i % 25) as i64)),
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
                    WireAttributeValue::Bool(i.is_multiple_of(2)),
                ),
            ]),
        ),
        (
            "ownership".to_string(),
            WireAttributeValue::Map(vec![
                (
                    "type".to_string(),
                    WireAttributeValue::String(
                        if i.is_multiple_of(2) {
                            "private"
                        } else {
                            "public"
                        }
                        .to_string(),
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

    if i.is_multiple_of(10) {
        attributes.push(("futureExpansion".to_string(), WireAttributeValue::Null));
    }

    attributes
}

fn wire_geometry_solid(
    boundaries: Vec<Vec<Vec<Vec<usize>>>>,
    semantics: Vec<Option<WireSemantic>>,
    materials: Vec<(String, WireMaterial)>,
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

fn producer(
    tx: mpsc::SyncSender<StreamMessage>,
    size: usize,
    batch_size: usize,
    seed: u64,
) {
    let global_props = WireGlobalProperties {
        metadata_identifier: "streaming-benchmark".to_string(),
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
    };

    tx.send(StreamMessage::GlobalProperties(global_props))
        .expect("Failed to send global properties");

    let building_count = size as u64;
    let seed_offset = seed as i64;

    for i in 0..building_count {
        if stream_verbose_enabled() && i % 100_000 == 0 && i > 0 {
            println!("Producer: Generated {} / {} buildings", i, building_count);
        }

        let complexity = i % 3;
        let base_x = 100 + (((i as i64 + seed_offset) * 50) % 10_000);
        let base_y = 200 + (((i as i64 + seed_offset) * 37) % 10_000);
        let height = 30 + ((i + (seed % 20)) % 20) as i64 * 3;

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

        let (boundaries, semantics, materials) = match complexity {
            0 => {
                let bounds = boundaries_from_table(BOUNDARIES_SIMPLE);
                let sems = semantics_for(
                    BOUNDARIES_SIMPLE.len(),
                    &[
                        "GroundSurface",
                        "RoofSurface",
                        "WallSurface",
                        "WallSurface",
                        "WallSurface",
                        "WallSurface",
                    ],
                );
                let mats = materials_for(BOUNDARIES_SIMPLE.len(), &material_wall, &material_roof);
                (bounds, sems, mats)
            }
            1 => {
                let bounds = boundaries_from_table(BOUNDARIES_MEDIUM);
                let sems = semantics_for(
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
                );
                let mats = materials_for(BOUNDARIES_MEDIUM.len(), &material_wall, &material_roof);
                (bounds, sems, mats)
            }
            _ => {
                let bounds = boundaries_from_table(BOUNDARIES_COMPLEX);
                let sems = semantics_for(
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
                );
                let mut mats =
                    materials_for(BOUNDARIES_COMPLEX.len(), &material_wall, &material_roof);
                let last = mats.len().saturating_sub(1);
                if let Some(entry) = mats.get_mut(last) {
                    entry.1 = material_roof.clone();
                }
                if let Some(entry) = mats.get_mut(last.saturating_sub(1)) {
                    entry.1 = material_roof.clone();
                }
                (bounds, sems, mats)
            }
        };

        let geometry_solid = wire_geometry_solid(boundaries, semantics, materials);

        let geometry_instance = wire_geometry_instance(
            0,
            [
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
            ],
        );

        let attributes = make_wire_attributes(i, height);

        let cityobject = WireCityObjectData {
            id: format!("building-{}", i),
            object_type: "Building".to_string(),
            vertices,
            geometries: vec![geometry_solid, geometry_instance],
            attributes,
        };

        tx.send(StreamMessage::CityObject(cityobject))
            .expect("Failed to send CityObject");

        if stream_verbose_enabled() && i > 0 && i.is_multiple_of(batch_size as u64) {
            println!("Producer: checkpoint {}", i);
        }
    }

    tx.send(StreamMessage::Done)
        .expect("Failed to send completion signal");
}

fn process_batch(
    buildings_in_batch: usize,
    vertices_in_batch: usize,
    geometries_in_batch: usize,
    surfaces_in_batch: usize,
    batch_num: usize,
    cumulative_buildings: usize,
) -> BatchMetrics {
    if stream_verbose_enabled() && (batch_num.is_multiple_of(100) || batch_num < 10) {
        println!(
            "Batch {}: {} buildings processed (total: {}), {} vertices, {} geometries, {} surfaces",
            batch_num,
            buildings_in_batch,
            cumulative_buildings,
            vertices_in_batch,
            geometries_in_batch,
            surfaces_in_batch
        );
    }

    let _ = (geometries_in_batch, surfaces_in_batch, vertices_in_batch);
    BatchMetrics {
        _buildings_processed: buildings_in_batch,
    }
}

fn parse_stream_mode() -> String {
    env::var("STREAM_MODE").unwrap_or_else(|_| "e2e".to_string())
}

fn stream_size_from_env() -> usize {
    if let Some(value) = env::var("STREAM_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        return value;
    }
    if let Some(value) = env::var("BENCH_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
    {
        return value;
    }
    let mode = env::var("BENCH_MODE").unwrap_or_else(|_| "full".to_string());
    if mode == "fast" {
        FAST_STREAM_SIZE
    } else {
        DEFAULT_STREAM_SIZE
    }
}

fn stream_batch_from_env() -> usize {
    env::var("STREAM_BATCH")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_STREAM_BATCH)
        .max(1)
}

fn stream_seed_from_env() -> u64 {
    env::var("BENCH_SEED")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_SEED)
}

fn stream_out_path() -> PathBuf {
    env::var("STREAM_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target/streaming-metrics.json"))
}

fn write_metrics(path: &PathBuf, metrics: &StreamMetrics) -> std::io::Result<()> {
    let producer_value = metrics
        .producer_s
        .map(|v| format!("{:.6}", v))
        .unwrap_or_else(|| "null".to_string());
    let payload = format!(
        "{{\n  \"mode\": \"{}\",\n  \"metrics\": {{\n    \"stream_total_s\": {:.6},\n    \"stream_producer_s\": {},\n    \"stream_consumer_s\": {:.6},\n    \"stream_throughput_buildings_s\": {:.6}\n  }}\n}}\n",
        metrics.mode,
        metrics.total_s,
        producer_value,
        metrics.consumer_s,
        metrics.throughput_buildings_s
    );
    std::fs::write(path, payload)
}

#[cfg(feature = "backend-default")]
mod default_backend {
    use super::*;
    use cityjson::prelude::*;
    use cityjson::v2_0::*;

    #[allow(clippy::type_complexity)]
    fn create_batch_model(
        global: &WireGlobalProperties,
    ) -> Result<(
        CityModel<u32, ResourceId32, OwnedStringStorage>,
        Vec<ResourceId32>,
    )> {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        model
            .metadata_mut()
            .set_identifier(CityModelIdentifier::new(global.metadata_identifier.clone()));
        model
            .metadata_mut()
            .set_reference_system(CRS::new(global.crs.clone()));

        model.transform_mut().set_scale(global.transform_scale);
        model
            .transform_mut()
            .set_translate(global.transform_translate);

        let mut template_refs = Vec::new();
        for wire_template in &global.geometry_templates {
            let template_ref = build_template_from_wire(&mut model, wire_template)?;
            template_refs.push(template_ref);
        }

        Ok((model, template_refs))
    }

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

        for (x, y, z) in &wire_template.template_vertices {
            builder.add_template_point(RealWorldCoordinate::new(*x, *y, *z));
        }

        builder.build()
    }

    fn build_geometry_from_wire(
        model: &mut CityModel<u32, ResourceId32, OwnedStringStorage>,
        wire_geom: &WireGeometry,
        vertex_refs: &[VertexIndex<u32>],
        template_refs: &[ResourceId32],
    ) -> Result<ResourceId32> {
        let lod = parse_lod(&wire_geom.lod);

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

        let bv: Vec<_> = vertex_refs
            .iter()
            .map(|vref| builder.add_vertex(*vref))
            .collect();

        for shell in wire_geom.boundaries.iter() {
            let mut surface_ids = Vec::new();

            for (surface_idx, surface_rings) in shell.iter().enumerate() {
                let surface_id = builder.start_surface();
                surface_ids.push(surface_id);

                if let Some(outer_ring_indices) = surface_rings.first() {
                    let ring_verts: Vec<usize> =
                        outer_ring_indices.iter().map(|&idx| bv[idx]).collect();
                    let ring_id = builder.add_ring(&ring_verts)?;
                    builder.add_surface_outer_ring(ring_id)?;
                }

                for inner_ring_indices in surface_rings.iter().skip(1) {
                    let ring_verts: Vec<usize> =
                        inner_ring_indices.iter().map(|&idx| bv[idx]).collect();
                    let ring_id = builder.add_ring(&ring_verts)?;
                    builder.add_surface_inner_ring(ring_id)?;
                }

                if let Some(wire_semantic) = wire_geom
                    .semantics
                    .get(surface_idx)
                    .and_then(|s| s.as_ref())
                {
                    let semantic = convert_wire_semantic(wire_semantic)?;
                    builder.set_semantic_surface(None, semantic, false)?;
                }

                if let Some((theme, wire_material)) = wire_geom.materials.get(surface_idx) {
                    let material = convert_wire_material(wire_material);
                    builder.set_material_surface(None, material, theme.clone(), true)?;
                }
            }

            builder.add_shell(&surface_ids)?;
        }

        builder.build()
    }

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
                let elements: Vec<Box<AttributeValue<OwnedStringStorage, ResourceId32>>> = vec
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

    fn parse_city_object_type(type_str: &str) -> CityObjectType<OwnedStringStorage> {
        match type_str {
            "Building" => CityObjectType::Building,
            "BuildingPart" => CityObjectType::BuildingPart,
            "Road" => CityObjectType::Road,
            _ => CityObjectType::GenericCityObject,
        }
    }

    fn parse_lod(lod_str: &str) -> LoD {
        match lod_str {
            "0" => LoD::LoD0,
            "1" => LoD::LoD1,
            "2" => LoD::LoD2,
            "3" => LoD::LoD3,
            _ => LoD::LoD1,
        }
    }

    pub fn run(mode: &str, size: usize, batch: usize, seed: u64) -> Result<StreamMetrics> {
        let (tx, rx) = mpsc::sync_channel::<StreamMessage>(10);

        let total_start = Instant::now();

        let result = thread::scope(|s| {
            let producer_handle = s.spawn(move || {
                let producer_start = Instant::now();
                producer(tx, size, batch, seed);
                producer_start.elapsed()
            });

            let consumer_handle = s.spawn(move || {
                let consumer_start = Instant::now();
                let result = consumer(rx, size, batch);
                let consumer_duration = consumer_start.elapsed();
                (result, consumer_duration)
            });

            let producer_duration = producer_handle.join().expect("Producer thread panicked");
            let (consumer_result, consumer_duration) =
                consumer_handle.join().expect("Consumer thread panicked");

            (producer_duration, consumer_duration, consumer_result)
        });

        let total_duration = total_start.elapsed();
        let (producer_duration, consumer_duration, consumer_result) = result;

        consumer_result?;

        let (total_s, producer_s) = match mode {
            "consumer" => (consumer_duration.as_secs_f64(), None),
            _ => (total_duration.as_secs_f64(), Some(producer_duration.as_secs_f64())),
        };

        let throughput = size as f64 / total_s;

        Ok(StreamMetrics {
            total_s,
            producer_s,
            consumer_s: consumer_duration.as_secs_f64(),
            throughput_buildings_s: throughput,
            mode: mode.to_string(),
        })
    }

    fn consumer(rx: mpsc::Receiver<StreamMessage>, size: usize, batch_size: usize) -> Result<()> {
        let global_props = if let Ok(StreamMessage::GlobalProperties(global)) = rx.recv() {
            if stream_verbose_enabled() {
                println!(
                    "Consumer: Received global properties with {} templates",
                    global.geometry_templates.len()
                );
            }
            global
        } else {
            return Err(Error::InvalidGeometry(
                "Expected GlobalProperties as first message".to_string(),
            ));
        };

        let mut current_batch_num = 0;
        let mut total_buildings_processed = 0;

        let (mut current_model, template_refs) = create_batch_model(&global_props)?;
        let mut buildings_in_batch = 0;
        let mut surfaces_in_batch = 0;

        while let Ok(message) = rx.recv() {
            match message {
                StreamMessage::CityObject(wire_co) => {
                    for wire_geom in &wire_co.geometries {
                        if !wire_geom.boundaries.is_empty() {
                            for shell in &wire_geom.boundaries {
                                surfaces_in_batch += shell.len();
                            }
                        }
                    }

                    let mut cityobject = CityObject::new(
                        wire_co.id.clone(),
                        parse_city_object_type(&wire_co.object_type),
                    );

                    let attrs = cityobject.attributes_mut();
                    for (key, wire_value) in wire_co.attributes {
                        let attr_value = convert_wire_attribute_value(wire_value);
                        attrs.insert(key, attr_value);
                    }

                    let vertex_refs: Vec<VertexIndex<u32>> = wire_co
                        .vertices
                        .iter()
                        .map(|(x, y, z)| {
                            current_model
                                .add_vertex(QuantizedCoordinate::new(*x, *y, *z))
                                .expect("Failed to add vertex")
                        })
                        .collect();

                    for wire_geom in wire_co.geometries {
                        let geom_ref = build_geometry_from_wire(
                            &mut current_model,
                            &wire_geom,
                            &vertex_refs,
                            &template_refs,
                        )?;
                        cityobject.geometry_mut().push(geom_ref);
                    }

                    current_model.cityobjects_mut().add(cityobject);
                    buildings_in_batch += 1;
                    total_buildings_processed += 1;

                    if buildings_in_batch >= batch_size {
                        let _ = process_batch(
                            buildings_in_batch,
                            current_model.vertices().len(),
                            current_model.iter_geometries().count(),
                            surfaces_in_batch,
                            current_batch_num,
                            total_buildings_processed,
                        );

                        drop(current_model);
                        let (new_model, _) = create_batch_model(&global_props)?;
                        current_model = new_model;

                        current_batch_num += 1;
                        buildings_in_batch = 0;
                        surfaces_in_batch = 0;
                    }
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

        if buildings_in_batch > 0 {
            let _ = process_batch(
                buildings_in_batch,
                current_model.vertices().len(),
                current_model.iter_geometries().count(),
                surfaces_in_batch,
                current_batch_num,
                total_buildings_processed,
            );
        }

        if total_buildings_processed != size {
            return Err(Error::InvalidGeometry(format!(
                "Expected {} buildings, processed {}",
                size, total_buildings_processed
            )));
        }

        Ok(())
    }
}

#[cfg(feature = "backend-nested")]
mod nested_backend {
    use super::*;
    use cityjson::backend::nested;
    use cityjson::backend::nested::appearance::Material;
    use cityjson::backend::nested::attributes::AttributeValue;
    use cityjson::backend::nested::geometry::{Geometry, GeometryBuilder, GeometryType, LoD};
    use cityjson::backend::nested::semantics::{Semantic, SemanticType};
    use cityjson::prelude::{
        CityModelType, Error, OwnedStringStorage, QuantizedCoordinate, ResourceId32, VertexIndex,
    };

    fn create_batch_model(
        global: &WireGlobalProperties,
    ) -> cityjson::prelude::Result<(nested::CityModel<OwnedStringStorage, ResourceId32>, usize, usize)>
    {
        let mut model =
            nested::CityModel::<OwnedStringStorage, ResourceId32>::new(CityModelType::CityJSON);

        model
            .metadata_mut()
            .set_identifier(nested::metadata::CityModelIdentifier::new(
                global.metadata_identifier.clone(),
            ));
        model
            .metadata_mut()
            .set_reference_system(nested::metadata::CRS::new(global.crs.clone()));

        let wall_idx = model.add_material(convert_wire_material(&WireMaterial {
            name: "wall-material".to_string(),
            ambient_intensity: None,
            diffuse_color: Some([0.7, 0.7, 0.7]),
            emissive_color: None,
            specular_color: None,
            shininess: None,
            transparency: None,
            is_smooth: None,
        }));
        let roof_idx = model.add_material(convert_wire_material(&WireMaterial {
            name: "roof-material".to_string(),
            ambient_intensity: None,
            diffuse_color: Some([0.8, 0.2, 0.2]),
            emissive_color: None,
            specular_color: None,
            shininess: Some(0.5),
            transparency: None,
            is_smooth: None,
        }));

        Ok((model, wall_idx, roof_idx))
    }

    fn build_geometry_from_wire(
        model: &mut nested::CityModel<OwnedStringStorage, ResourceId32>,
        wire_geom: &WireGeometry,
        vertex_refs: &[VertexIndex<u32>],
        wall_material_idx: usize,
        roof_material_idx: usize,
    ) -> cityjson::prelude::Result<Geometry<OwnedStringStorage, ResourceId32>> {
        let lod = parse_lod(&wire_geom.lod);

        if wire_geom.geometry_type == "GeometryInstance" {
            return Err(Error::InvalidGeometry(
                "GeometryInstance not supported in nested backend".to_string(),
            ));
        }

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

        let mut builder =
            GeometryBuilder::new(model, geom_type, nested::BuilderMode::Regular).with_lod(lod);

        for vref in vertex_refs {
            builder.add_vertex(*vref)?;
        }
        let bv: Vec<usize> = (0..vertex_refs.len()).collect();

        for shell in wire_geom.boundaries.iter() {
            builder.start_shell()?;
            let mut surface_ids = Vec::new();

            for (surface_idx, surface_rings) in shell.iter().enumerate() {
                builder.start_surface()?;

                if let Some(outer_ring_indices) = surface_rings.first() {
                    let ring_verts: Vec<usize> =
                        outer_ring_indices.iter().map(|&idx| bv[idx]).collect();
                    let ring_id = builder.add_ring(&ring_verts)?;
                    builder.add_surface_outer_ring(ring_id)?;
                }

                for inner_ring_indices in surface_rings.iter().skip(1) {
                    let ring_verts: Vec<usize> =
                        inner_ring_indices.iter().map(|&idx| bv[idx]).collect();
                    let ring_id = builder.add_ring(&ring_verts)?;
                    builder.add_surface_inner_ring(ring_id)?;
                }

                let surface_id = builder.end_surface()?;
                surface_ids.push(surface_id);

                if let Some(wire_semantic) = wire_geom
                    .semantics
                    .get(surface_idx)
                    .and_then(|s| s.as_ref())
                {
                    let semantic = convert_wire_semantic(wire_semantic)?;
                    builder.set_semantic_surface(surface_id, semantic, false)?;
                }

                if let Some((_theme, wire_material)) = wire_geom.materials.get(surface_idx) {
                    let material_idx = if wire_material.name == "roof-material" {
                        roof_material_idx
                    } else {
                        wall_material_idx
                    };
                    builder.set_material_surface("default".to_string(), surface_id, material_idx)?;
                }
            }

            for surface_id in surface_ids {
                builder.add_shell_surface(surface_id)?;
            }
            builder.end_shell()?;
        }

        builder.build()
    }

    fn convert_wire_semantic(
        wire_semantic: &WireSemantic,
    ) -> cityjson::prelude::Result<Semantic<OwnedStringStorage, ResourceId32>> {
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
                let elements: Vec<Box<AttributeValue<OwnedStringStorage, ResourceId32>>> = vec
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

    fn parse_city_object_type(type_str: &str) -> nested::cityobject::CityObjectType<OwnedStringStorage> {
        match type_str {
            "Building" => nested::cityobject::CityObjectType::Building,
            "BuildingPart" => nested::cityobject::CityObjectType::BuildingPart,
            "Road" => nested::cityobject::CityObjectType::Road,
            _ => nested::cityobject::CityObjectType::GenericCityObject,
        }
    }

    fn parse_lod(lod_str: &str) -> LoD {
        match lod_str {
            "0" => LoD::LoD0,
            "1" => LoD::LoD1,
            "2" => LoD::LoD2,
            "3" => LoD::LoD3,
            _ => LoD::LoD1,
        }
    }

    pub fn run(mode: &str, size: usize, batch: usize, seed: u64) -> cityjson::prelude::Result<StreamMetrics> {
        let (tx, rx) = mpsc::sync_channel::<StreamMessage>(10);

        let total_start = Instant::now();

        let result = thread::scope(|s| {
            let producer_handle = s.spawn(move || {
                let producer_start = Instant::now();
                producer(tx, size, batch, seed);
                producer_start.elapsed()
            });

            let consumer_handle = s.spawn(move || {
                let consumer_start = Instant::now();
                let result = consumer(rx, size, batch);
                let consumer_duration = consumer_start.elapsed();
                (result, consumer_duration)
            });

            let producer_duration = producer_handle.join().expect("Producer thread panicked");
            let (consumer_result, consumer_duration) =
                consumer_handle.join().expect("Consumer thread panicked");

            (producer_duration, consumer_duration, consumer_result)
        });

        let total_duration = total_start.elapsed();
        let (producer_duration, consumer_duration, consumer_result) = result;

        consumer_result?;

        let (total_s, producer_s) = match mode {
            "consumer" => (consumer_duration.as_secs_f64(), None),
            _ => (total_duration.as_secs_f64(), Some(producer_duration.as_secs_f64())),
        };

        let throughput = size as f64 / total_s;

        Ok(StreamMetrics {
            total_s,
            producer_s,
            consumer_s: consumer_duration.as_secs_f64(),
            throughput_buildings_s: throughput,
            mode: mode.to_string(),
        })
    }

    fn consumer(
        rx: mpsc::Receiver<StreamMessage>,
        size: usize,
        batch_size: usize,
    ) -> cityjson::prelude::Result<()> {
        let global_props = if let Ok(StreamMessage::GlobalProperties(global)) = rx.recv() {
            if stream_verbose_enabled() {
                println!(
                    "Consumer: Received global properties with {} templates",
                    global.geometry_templates.len()
                );
            }
            global
        } else {
            return Err(Error::InvalidGeometry(
                "Expected GlobalProperties as first message".to_string(),
            ));
        };

        let mut current_batch_num = 0;
        let mut total_buildings_processed = 0;
        let (mut current_model, mut wall_idx, mut roof_idx) = create_batch_model(&global_props)?;
        let mut buildings_in_batch = 0;
        let mut surfaces_in_batch = 0;

        while let Ok(message) = rx.recv() {
            match message {
                StreamMessage::CityObject(wire_co) => {
                    for wire_geom in &wire_co.geometries {
                        if !wire_geom.boundaries.is_empty() {
                            for shell in &wire_geom.boundaries {
                                surfaces_in_batch += shell.len();
                            }
                        }
                    }

                    let mut cityobject =
                        nested::CityObject::new(parse_city_object_type(&wire_co.object_type));

                    let attrs = cityobject.attributes_mut();
                    for (key, wire_value) in wire_co.attributes {
                        let attr_value = convert_wire_attribute_value(wire_value);
                        attrs.insert(key, attr_value);
                    }

                    let vertex_refs: Vec<VertexIndex<u32>> = wire_co
                        .vertices
                        .iter()
                        .map(|(x, y, z)| {
                            current_model
                                .add_vertex(QuantizedCoordinate::new(*x, *y, *z))
                                .expect("Failed to add vertex")
                        })
                        .collect();

                    for wire_geom in wire_co.geometries {
                        if wire_geom.geometry_type == "GeometryInstance" {
                            continue;
                        }
                        let geom = build_geometry_from_wire(
                            &mut current_model,
                            &wire_geom,
                            &vertex_refs,
                            wall_idx,
                            roof_idx,
                        )?;
                        cityobject.geometry_mut().push(geom);
                    }

                    current_model.add_cityobject(wire_co.id.clone(), cityobject);
                    buildings_in_batch += 1;
                    total_buildings_processed += 1;

                    if buildings_in_batch >= batch_size {
                        let geometries_in_batch: usize = current_model
                            .cityobjects()
                            .values()
                            .map(|co| co.geometry().map_or(0, |g| g.len()))
                            .sum();
                        let _ = process_batch(
                            buildings_in_batch,
                            current_model.vertices().len(),
                            geometries_in_batch,
                            surfaces_in_batch,
                            current_batch_num,
                            total_buildings_processed,
                        );

                        drop(current_model);
                        let (new_model, new_wall_idx, new_roof_idx) =
                            create_batch_model(&global_props)?;
                        current_model = new_model;
                        wall_idx = new_wall_idx;
                        roof_idx = new_roof_idx;

                        current_batch_num += 1;
                        buildings_in_batch = 0;
                        surfaces_in_batch = 0;
                    }
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

        if buildings_in_batch > 0 {
            let geometries_in_batch: usize = current_model
                .cityobjects()
                .values()
                .map(|co| co.geometry().map_or(0, |g| g.len()))
                .sum();
            let _ = process_batch(
                buildings_in_batch,
                current_model.vertices().len(),
                geometries_in_batch,
                surfaces_in_batch,
                current_batch_num,
                total_buildings_processed,
            );
        }

        if total_buildings_processed != size {
            return Err(Error::InvalidGeometry(format!(
                "Expected {} buildings, processed {}",
                size, total_buildings_processed
            )));
        }

        Ok(())
    }
}

fn main() {
    let mode = parse_stream_mode();
    let size = stream_size_from_env();
    let batch = stream_batch_from_env();
    let seed = stream_seed_from_env();
    let out_path = stream_out_path();

    #[cfg(all(feature = "backend-default", feature = "backend-nested"))]
    let metrics = {
        let backend = env::var("BENCH_BACKEND").unwrap_or_default();
        if backend == "nested" {
            nested_backend::run(&mode, size, batch, seed).expect("nested stream failed")
        } else {
            default_backend::run(&mode, size, batch, seed).expect("default stream failed")
        }
    };

    #[cfg(all(feature = "backend-default", not(feature = "backend-nested")))]
    let metrics = default_backend::run(&mode, size, batch, seed).expect("default stream failed");

    #[cfg(all(feature = "backend-nested", not(feature = "backend-default")))]
    let metrics = nested_backend::run(&mode, size, batch, seed).expect("nested stream failed");

    if let Err(err) = write_metrics(&out_path, &metrics) {
        eprintln!("Failed to write streaming metrics: {}", err);
    }

    println!("streaming mode: {}", metrics.mode);
    println!("stream_total_s: {:.4}", metrics.total_s);
    if let Some(producer_s) = metrics.producer_s {
        println!("stream_producer_s: {:.4}", producer_s);
    }
    println!("stream_consumer_s: {:.4}", metrics.consumer_s);
    println!(
        "stream_throughput_buildings_s: {:.2}",
        metrics.throughput_buildings_s
    );
}
