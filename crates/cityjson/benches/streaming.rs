//! Streaming benchmarks for end-to-end producer/consumer performance.

use std::env;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use num::ToPrimitive;

const DEFAULT_SEED: u64 = 12345;

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

#[derive(Debug, Clone)]
enum WireAttributeValue {
    String(String),
    Float(f64),
    Integer(i64),
    Bool(bool),
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
}

#[derive(Debug, Clone)]
struct WireGeometry {
    geometry_type: String,
    lod: String,
    boundaries: Vec<Vec<Vec<Vec<usize>>>>,
    semantics: Vec<Option<WireSemantic>>,
    materials: Vec<(String, WireMaterial)>,
}

#[derive(Debug, Clone)]
struct WireGlobalProperties {
    metadata_identifier: String,
    crs: String,
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

#[derive(Debug)]
struct StreamMetrics {
    total_ms: f64,
    producer_ms: Option<f64>,
    consumer_ms: f64,
    throughput_elem_s: f64,
    mode: String,
}

fn stream_verbose_enabled() -> bool {
    env::var(STREAM_VERBOSE_ENV).ok().is_some_and(|val| {
        matches!(
            val.trim().to_lowercase().as_str(),
            "1" | "true" | "yes" | "y" | "on"
        )
    })
}

fn boundaries_from_table(table: &[&[&[usize]]]) -> Vec<Vec<Vec<Vec<usize>>>> {
    vec![
        table
            .iter()
            .map(|surface| surface.iter().map(|ring| ring.to_vec()).collect())
            .collect(),
    ]
}

fn semantics_for(count: usize, pattern: &[&str]) -> Vec<Option<WireSemantic>> {
    let mut semantics = Vec::with_capacity(count);
    for idx in 0..count {
        let surface_type = pattern[idx % pattern.len()].to_string();
        semantics.push(Some(WireSemantic { surface_type }));
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

fn narrow_i64_from_u64(value: u64) -> i64 {
    i64::try_from(value).expect("value must fit in i64 for benchmark data generation")
}

fn narrow_f64_from_i64(value: i64) -> f64 {
    let narrow = i32::try_from(value).expect("value must fit in i32 for benchmark metrics");
    f64::from(narrow)
}

fn narrow_f64_from_usize(value: usize) -> f64 {
    let narrow = u32::try_from(value).expect("value must fit in u32 for benchmark metrics");
    f64::from(narrow)
}

fn narrow_f32_from_f64(value: f64) -> f32 {
    value
        .to_f32()
        .expect("value must fit in f32 for benchmark material properties")
}

fn narrow_vec3_f32_from_f64(value: [f64; 3]) -> [f32; 3] {
    [
        narrow_f32_from_f64(value[0]),
        narrow_f32_from_f64(value[1]),
        narrow_f32_from_f64(value[2]),
    ]
}

fn make_wire_attributes(i: u64, height: i64) -> Vec<(String, WireAttributeValue)> {
    let year_offset = i64::try_from(i % 75).expect("modulo result must fit in i64");
    vec![
        (
            "height".to_string(),
            WireAttributeValue::Float(narrow_f64_from_i64(height)),
        ),
        (
            "yearOfConstruction".to_string(),
            WireAttributeValue::Integer(1950 + year_offset),
        ),
        (
            "isCommercial".to_string(),
            WireAttributeValue::Bool(i.is_multiple_of(5)),
        ),
        (
            "owner".to_string(),
            WireAttributeValue::String(format!("Owner-{}", i % 100)),
        ),
    ]
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
    }
}

fn producer(tx: &mpsc::SyncSender<StreamMessage>, size: usize, batch_size: usize, seed: u64) {
    let global_props = WireGlobalProperties {
        metadata_identifier: "streaming-benchmark".to_string(),
        crs: "https://www.opengis.net/def/crs/EPSG/0/7415".to_string(),
    };

    tx.send(StreamMessage::GlobalProperties(global_props))
        .expect("Failed to send global properties");

    let building_count = u64::try_from(size).expect("stream size must fit in u64");
    let batch_size_u64 = u64::try_from(batch_size).expect("stream batch size must fit in u64");
    let seed_offset = narrow_i64_from_u64(seed);

    for i in 0..building_count {
        if stream_verbose_enabled() && i > 0 && i.is_multiple_of(100_000) {
            println!("Producer: Generated {i} / {building_count} buildings");
        }

        let i_signed = narrow_i64_from_u64(i);
        let base_x = 100 + (((i_signed + seed_offset) * 50) % 10_000);
        let base_y = 200 + (((i_signed + seed_offset) * 37) % 10_000);
        let height_cycle = narrow_i64_from_u64((i + (seed % 20)) % 20);
        let height = 30 + height_cycle * 3;

        let vertices = vec![
            (base_x, base_y, 0),
            (base_x + 20, base_y, 0),
            (base_x + 20, base_y + 20, 0),
            (base_x, base_y + 20, 0),
            (base_x, base_y, height),
            (base_x + 20, base_y, height),
            (base_x + 20, base_y + 20, height),
            (base_x, base_y + 20, height),
        ];

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

        let boundaries = boundaries_from_table(BOUNDARIES_SIMPLE);
        let semantics = semantics_for(
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
        let materials = materials_for(BOUNDARIES_SIMPLE.len(), &material_wall, &material_roof);

        let geometry_solid = wire_geometry_solid(boundaries, semantics, materials);

        let attributes = make_wire_attributes(i, height);

        let cityobject = WireCityObjectData {
            id: format!("building-{i}"),
            object_type: "Building".to_string(),
            vertices,
            geometries: vec![geometry_solid],
            attributes,
        };

        tx.send(StreamMessage::CityObject(cityobject))
            .expect("Failed to send CityObject");

        if stream_verbose_enabled() && i > 0 && i.is_multiple_of(batch_size_u64) {
            println!("Producer: checkpoint {i}");
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
) {
    if stream_verbose_enabled() && (batch_num.is_multiple_of(100) || batch_num < 10) {
        println!(
            "Batch {batch_num}: {buildings_in_batch} buildings processed (total: {cumulative_buildings}), {vertices_in_batch} vertices, {geometries_in_batch} geometries, {surfaces_in_batch} surfaces"
        );
    }

    let _ = (geometries_in_batch, surfaces_in_batch, vertices_in_batch);
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
    env::var("STREAM_OUT").map_or_else(
        |_| PathBuf::from("target/streaming-metrics.json"),
        PathBuf::from,
    )
}

fn write_metrics(path: &PathBuf, metrics: &StreamMetrics) -> std::io::Result<()> {
    let producer_value = metrics
        .producer_ms
        .map_or_else(|| "null".to_string(), |v| format!("{v:.6}"));
    let payload = format!(
        "{{\n  \"mode\": \"{}\",\n  \"metrics\": {{\n    \"time_ms\": {:.6},\n    \"time_producer_ms\": {},\n    \"time_consumer_ms\": {:.6},\n    \"throughput_elem_s\": {:.6}\n  }}\n}}\n",
        metrics.mode,
        metrics.total_ms,
        producer_value,
        metrics.consumer_ms,
        metrics.throughput_elem_s
    );
    std::fs::write(path, payload)
}

mod default_backend {
    use super::{
        Instant, StreamMessage, StreamMetrics, WireAttributeValue, WireGeometry,
        WireGlobalProperties, WireMaterial, WireSemantic, mpsc, narrow_f32_from_f64,
        narrow_f64_from_usize, narrow_vec3_f32_from_f64, process_batch, producer,
        stream_verbose_enabled, thread,
    };
    use cityjson::backend::default::geometry::GeometryBuilder;
    use cityjson::prelude::*;
    use cityjson::resources::pool::ResourceId32;
    use cityjson::v2_0::{CityModel, CityObject, CityObjectType, Material, Semantic, SemanticType};

    fn create_batch_model(global: &WireGlobalProperties) -> CityModel<u32, OwnedStringStorage> {
        let mut model = CityModel::<u32, OwnedStringStorage>::new(CityModelType::CityJSON);

        model
            .metadata_mut()
            .set_identifier(CityModelIdentifier::new(global.metadata_identifier.clone()));
        model
            .metadata_mut()
            .set_reference_system(CRS::new(global.crs.clone()));
        model
    }

    fn build_geometry_from_wire(
        model: &mut CityModel<u32, OwnedStringStorage>,
        wire_geom: &WireGeometry,
        vertex_refs: &[VertexIndex<u32>],
    ) -> Result<ResourceId32> {
        let lod = parse_lod(&wire_geom.lod);

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
            GeometryBuilder::new(model, geom_type, BuilderMode::Regular).with_lod(lod);

        let bv: Vec<_> = vertex_refs
            .iter()
            .map(|vref| builder.add_vertex(*vref))
            .collect();

        for shell in &wire_geom.boundaries {
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

    fn convert_wire_material(wire_material: &WireMaterial) -> Material<OwnedStringStorage> {
        let mut material = Material::new(wire_material.name.clone());

        if let Some(val) = wire_material.ambient_intensity {
            material.set_ambient_intensity(Some(narrow_f32_from_f64(val)));
        }
        if let Some(val) = wire_material.diffuse_color {
            material.set_diffuse_color(Some(narrow_vec3_f32_from_f64(val).into()));
        }
        if let Some(val) = wire_material.emissive_color {
            material.set_emissive_color(Some(narrow_vec3_f32_from_f64(val).into()));
        }
        if let Some(val) = wire_material.specular_color {
            material.set_specular_color(Some(narrow_vec3_f32_from_f64(val).into()));
        }
        if let Some(val) = wire_material.shininess {
            material.set_shininess(Some(narrow_f32_from_f64(val)));
        }
        if let Some(val) = wire_material.transparency {
            material.set_transparency(Some(narrow_f32_from_f64(val)));
        }
        if let Some(val) = wire_material.is_smooth {
            material.set_is_smooth(Some(val));
        }

        material
    }

    fn convert_wire_attribute_value(
        wire_value: WireAttributeValue,
    ) -> AttributeValue<OwnedStringStorage> {
        match wire_value {
            WireAttributeValue::String(s) => AttributeValue::String(s),
            WireAttributeValue::Float(f) => AttributeValue::Float(f),
            WireAttributeValue::Integer(i) => AttributeValue::Integer(i),
            WireAttributeValue::Bool(b) => AttributeValue::Bool(b),
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
                producer(&tx, size, batch, seed);
                producer_start.elapsed()
            });

            let consumer_handle = s.spawn(move || {
                let consumer_start = Instant::now();
                let result = consumer(&rx, size, batch);
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

        let (elapsed_total_seconds, elapsed_producer_seconds) = match mode {
            "consumer" => (consumer_duration.as_secs_f64(), None),
            _ => (
                total_duration.as_secs_f64(),
                Some(producer_duration.as_secs_f64()),
            ),
        };

        let throughput = narrow_f64_from_usize(size) / elapsed_total_seconds;
        let elapsed_total_millis = elapsed_total_seconds * 1000.0;
        let elapsed_producer_millis = elapsed_producer_seconds.map(|v| v * 1000.0);
        let elapsed_consumer_millis = consumer_duration.as_secs_f64() * 1000.0;

        Ok(StreamMetrics {
            total_ms: elapsed_total_millis,
            producer_ms: elapsed_producer_millis,
            consumer_ms: elapsed_consumer_millis,
            throughput_elem_s: throughput,
            mode: mode.to_string(),
        })
    }

    fn consumer(rx: &mpsc::Receiver<StreamMessage>, size: usize, batch_size: usize) -> Result<()> {
        let global_props = if let Ok(StreamMessage::GlobalProperties(global)) = rx.recv() {
            if stream_verbose_enabled() {
                println!("Consumer: Received global properties");
            }
            global
        } else {
            return Err(Error::InvalidGeometry(
                "Expected GlobalProperties as first message".to_string(),
            ));
        };

        let mut current_batch_num = 0;
        let mut total_buildings_processed = 0;

        let mut current_model = create_batch_model(&global_props);
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
                        CityObjectIdentifier::new(wire_co.id.clone()),
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
                        let geom_ref =
                            build_geometry_from_wire(&mut current_model, &wire_geom, &vertex_refs)?;
                        cityobject.add_geometry(GeometryRef::from_parts(
                            geom_ref.index(),
                            geom_ref.generation(),
                        ));
                    }

                    current_model.cityobjects_mut().add(cityobject)?;
                    buildings_in_batch += 1;
                    total_buildings_processed += 1;

                    if buildings_in_batch >= batch_size {
                        process_batch(
                            buildings_in_batch,
                            current_model.vertices().len(),
                            current_model.iter_geometries().count(),
                            surfaces_in_batch,
                            current_batch_num,
                            total_buildings_processed,
                        );

                        drop(current_model);
                        current_model = create_batch_model(&global_props);

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
            process_batch(
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
                "Expected {size} buildings, processed {total_buildings_processed}"
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

    let metrics = default_backend::run(&mode, size, batch, seed).expect("default stream failed");

    if let Err(err) = write_metrics(&out_path, &metrics) {
        eprintln!("Failed to write streaming metrics: {err}");
    }

    println!("streaming mode: {}", metrics.mode);
    println!("time_ms: {:.4}", metrics.total_ms);
    if let Some(producer_ms) = metrics.producer_ms {
        println!("time_producer_ms: {producer_ms:.4}");
    }
    println!("time_consumer_ms: {:.4}", metrics.consumer_ms);
    println!("throughput_elem_s: {:.2}", metrics.throughput_elem_s);
}
