mod common;

use std::collections::HashMap;

use cityarrow::internal::{decode_parts, encode_parts, read_stream_parts, write_stream_parts};
use cityarrow::{ModelDecoder, ModelEncoder};
use cityjson::CityModelType;
use cityjson::v2_0::{
    AttributeValue, Boundary, CityObject, CityObjectIdentifier, CityObjectType, Geometry, LoD,
    OwnedCityModel, OwnedSemantic, SemanticMap, SemanticType, StoredGeometryParts,
};
use cityparquet::{
    PackageReader, PackageWriter, read_package_parts_file, write_package_parts_file,
};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use tempfile::tempdir;

const LARGE_BENCHMARK_BUILDINGS: usize = 4_096;
const ATTRIBUTE_HEAVY_BENCHMARK_BUILDINGS: usize = 1_024;
const SHARED_CORPUS_WRITE_CASES: &[&str] = &[
    "stress_appearance_and_validation",
    "stress_geometry_flattening",
    "stress_deep_boundary",
];

fn benchmark_vertices(building_index: usize) -> [cityjson::v2_0::RealWorldCoordinate; 5] {
    let grid_x = f64::from(u32::try_from(building_index % 64).expect("grid x fits into u32")) * 2.0;
    let grid_y = f64::from(u32::try_from(building_index / 64).expect("grid y fits into u32")) * 2.0;
    let apex_height = 1.0
        + f64::from(u32::try_from(building_index % 5).expect("apex height fits into u32")) * 0.1;
    [
        cityjson::v2_0::RealWorldCoordinate::new(grid_x, grid_y, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(grid_x + 1.0, grid_y, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(grid_x + 1.0, grid_y + 1.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(grid_x, grid_y + 1.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(grid_x + 0.5, grid_y + 0.5, apex_height),
    ]
}

fn benchmark_boundary(vertex_start: u32) -> Boundary<u32> {
    vec![
        vec![vec![
            vertex_start,
            vertex_start + 1,
            vertex_start + 4,
            vertex_start,
        ]],
        vec![vec![
            vertex_start + 1,
            vertex_start + 2,
            vertex_start + 4,
            vertex_start + 1,
        ]],
        vec![vec![
            vertex_start + 2,
            vertex_start + 3,
            vertex_start + 4,
            vertex_start + 2,
        ]],
        vec![vec![
            vertex_start + 3,
            vertex_start,
            vertex_start + 4,
            vertex_start + 3,
        ]],
        vec![vec![
            vertex_start,
            vertex_start + 3,
            vertex_start + 2,
            vertex_start + 1,
            vertex_start,
        ]],
    ]
    .try_into()
    .expect("boundary")
}

fn benchmark_geometry(
    vertex_start: u32,
    roof: cityjson::prelude::SemanticHandle,
    wall: cityjson::prelude::SemanticHandle,
) -> Geometry<u32, cityjson::prelude::OwnedStringStorage> {
    let mut semantics = SemanticMap::new();
    semantics.add_surface(Some(wall));
    semantics.add_surface(Some(wall));
    semantics.add_surface(Some(wall));
    semantics.add_surface(Some(wall));
    semantics.add_surface(Some(roof));
    Geometry::from_stored_parts(StoredGeometryParts {
        type_geometry: cityjson::v2_0::GeometryType::MultiSurface,
        lod: Some(LoD::LoD2_2),
        boundaries: Some(benchmark_boundary(vertex_start)),
        semantics: Some(semantics),
        materials: None,
        textures: None,
        instance: None,
    })
}

fn benchmark_building(
    building_index: usize,
    geometry_handle: cityjson::prelude::GeometryHandle,
) -> CityObject<cityjson::prelude::OwnedStringStorage> {
    let mut building = CityObject::new(
        CityObjectIdentifier::new(format!("building-{building_index}")),
        CityObjectType::Building,
    );
    building.add_geometry(geometry_handle);
    building.attributes_mut().insert(
        "name".to_string(),
        AttributeValue::String(format!("Benchmark Building {building_index}")),
    );
    building.attributes_mut().insert(
        "floors".to_string(),
        AttributeValue::Unsigned(
            5 + u64::from(u32::try_from(building_index % 7).expect("floor count fits into u32")),
        ),
    );
    building
}

fn large_benchmark_model(building_count: usize) -> OwnedCityModel {
    let mut model = OwnedCityModel::new(CityModelType::CityJSON);
    model
        .metadata_mut()
        .set_identifier(cityjson::v2_0::CityModelIdentifier::new(
            "benchmark-citymodel".to_string(),
        ));
    model.metadata_mut().set_title("Benchmark".to_string());

    let roof = model
        .add_semantic(OwnedSemantic::new(SemanticType::RoofSurface))
        .expect("semantic");
    let wall = model
        .add_semantic(OwnedSemantic::new(SemanticType::WallSurface))
        .expect("semantic");

    for building_index in 0..building_count {
        let vertex_start =
            u32::try_from(model.vertices().as_slice().len()).expect("vertex count fits into u32");
        let vertices = benchmark_vertices(building_index);
        let _ = model.add_vertices(&vertices).expect("vertices");
        let geometry = benchmark_geometry(vertex_start, roof, wall);
        let geometry_handle = model.add_geometry(geometry).expect("geometry");
        let building = benchmark_building(building_index, geometry_handle);
        model.cityobjects_mut().add(building).expect("cityobject");
    }

    model
}

fn attribute_heavy_benchmark_building(
    building_index: usize,
    geometry_handle: cityjson::prelude::GeometryHandle,
) -> CityObject<cityjson::prelude::OwnedStringStorage> {
    let floors =
        5 + u64::from(u32::try_from(building_index % 7).expect("floor count fits into u32"));
    let mut building = CityObject::new(
        CityObjectIdentifier::new(format!("attribute-heavy-building-{building_index}")),
        CityObjectType::Building,
    );
    building.add_geometry(geometry_handle);
    building.attributes_mut().insert(
        "name".to_string(),
        AttributeValue::String(format!("Attribute Heavy Building {building_index}")),
    );
    building.attributes_mut().insert(
        "classification".to_string(),
        AttributeValue::String(
            match building_index % 3 {
                0 => "residential",
                1 => "mixed-use",
                _ => "civic",
            }
            .to_string(),
        ),
    );
    building.attributes_mut().insert(
        "profile".to_string(),
        AttributeValue::Map(HashMap::from([
            ("floors".to_string(), AttributeValue::Unsigned(floors)),
            (
                "height_m".to_string(),
                AttributeValue::Float(12.5 + f64::from(building_index as u32) * 0.05),
            ),
            (
                "district".to_string(),
                if building_index % 4 == 0 {
                    AttributeValue::Null
                } else {
                    AttributeValue::String(format!("district-{}", building_index % 8))
                },
            ),
            (
                "energy".to_string(),
                AttributeValue::Map(HashMap::from([
                    (
                        "rating".to_string(),
                        AttributeValue::String(
                            match building_index % 4 {
                                0 => "A",
                                1 => "B",
                                2 => "C",
                                _ => "D",
                            }
                            .to_string(),
                        ),
                    ),
                    (
                        "score".to_string(),
                        AttributeValue::Integer(70 + i64::from((building_index % 20) as u32)),
                    ),
                    (
                        "audited".to_string(),
                        AttributeValue::Bool(building_index % 2 == 0),
                    ),
                ])),
            ),
        ])),
    );
    building.attributes_mut().insert(
        "labels".to_string(),
        AttributeValue::Vec(vec![
            AttributeValue::String(format!("block-{}", building_index % 32)),
            AttributeValue::String(format!("cluster-{}", building_index % 16)),
            AttributeValue::String(format!("survey-{}", building_index % 5)),
        ]),
    );
    building.attributes_mut().insert(
        "flags".to_string(),
        AttributeValue::Vec(vec![
            AttributeValue::Bool(building_index % 2 == 0),
            AttributeValue::Bool(building_index % 3 == 0),
            AttributeValue::Bool(building_index % 5 == 0),
        ]),
    );
    building.attributes_mut().insert(
        "history".to_string(),
        AttributeValue::Map(HashMap::from([
            (
                "renovation_years".to_string(),
                AttributeValue::Vec(vec![
                    AttributeValue::Unsigned(1998 + u64::from((building_index % 10) as u32)),
                    AttributeValue::Unsigned(2010 + u64::from((building_index % 7) as u32)),
                    AttributeValue::Unsigned(2020 + u64::from((building_index % 5) as u32)),
                ]),
            ),
            (
                "occupancy".to_string(),
                AttributeValue::Map(HashMap::from([
                    (
                        "weekday".to_string(),
                        AttributeValue::Unsigned(40 + u64::from((building_index % 60) as u32)),
                    ),
                    (
                        "weekend".to_string(),
                        AttributeValue::Unsigned(15 + u64::from((building_index % 25) as u32)),
                    ),
                ])),
            ),
        ])),
    );
    building
}

fn attribute_heavy_benchmark_model(building_count: usize) -> OwnedCityModel {
    let mut model = OwnedCityModel::new(CityModelType::CityJSON);
    model
        .metadata_mut()
        .set_identifier(cityjson::v2_0::CityModelIdentifier::new(
            "attribute-heavy-benchmark-citymodel".to_string(),
        ));
    model
        .metadata_mut()
        .set_title("Attribute Heavy Benchmark".to_string());

    let roof = model
        .add_semantic(OwnedSemantic::new(SemanticType::RoofSurface))
        .expect("semantic");
    let wall = model
        .add_semantic(OwnedSemantic::new(SemanticType::WallSurface))
        .expect("semantic");

    for building_index in 0..building_count {
        let vertex_start =
            u32::try_from(model.vertices().as_slice().len()).expect("vertex count fits into u32");
        let vertices = benchmark_vertices(building_index);
        let _ = model.add_vertices(&vertices).expect("vertices");
        let geometry = benchmark_geometry(vertex_start, roof, wall);
        let geometry_handle = model.add_geometry(geometry).expect("geometry");
        let building = attribute_heavy_benchmark_building(building_index, geometry_handle);
        model.cityobjects_mut().add(building).expect("cityobject");
    }

    model
}

fn split_benches(c: &mut Criterion) {
    // Keep the benchmark model large enough to surface conversion overhead that does not show up
    // on the single-building correctness fixture used in roundtrip tests.
    let model = large_benchmark_model(LARGE_BENCHMARK_BUILDINGS);
    let parts = encode_parts(&model).expect("encode parts");

    let mut stream_bytes = Vec::new();
    ModelEncoder
        .encode(&model, &mut stream_bytes)
        .expect("encode stream");

    let mut stream_parts_bytes = Vec::new();
    write_stream_parts(&parts, &mut stream_parts_bytes).expect("encode stream parts");

    let dir = tempdir().expect("tempdir");
    let model_path = dir.path().join("model.cityarrow");
    let parts_path = dir.path().join("parts.cityarrow");
    PackageWriter
        .write_file(&model_path, &model)
        .expect("write model package");
    write_package_parts_file(&parts_path, &parts).expect("write parts package");

    c.bench_function("encode_parts", |b| {
        b.iter(|| {
            let _ = encode_parts(&model).expect("encode parts");
        });
    });
    c.bench_function("decode_parts", |b| {
        b.iter(|| {
            let _ = decode_parts(&parts).expect("decode parts");
        });
    });
    c.bench_function("stream_roundtrip", |b| {
        b.iter(|| {
            let mut bytes = Vec::new();
            ModelEncoder
                .encode(&model, &mut bytes)
                .expect("encode stream");
            let _ = ModelDecoder
                .decode(bytes.as_slice())
                .expect("decode stream");
        });
    });
    c.bench_function("package_roundtrip", |b| {
        b.iter(|| {
            PackageWriter
                .write_file(&model_path, &model)
                .expect("write package");
            let _ = PackageReader.read_file(&model_path).expect("read package");
        });
    });
    c.bench_function("stream_write_model", |b| {
        b.iter(|| {
            let mut bytes = Vec::new();
            ModelEncoder
                .encode(&model, &mut bytes)
                .expect("encode stream");
        });
    });
    c.bench_function("stream_read_model", |b| {
        b.iter(|| {
            let _ = ModelDecoder
                .decode(stream_bytes.as_slice())
                .expect("decode stream");
        });
    });
    c.bench_function("stream_write_parts", |b| {
        b.iter(|| {
            let mut bytes = Vec::new();
            write_stream_parts(&parts, &mut bytes).expect("encode stream parts");
        });
    });
    c.bench_function("stream_read_parts", |b| {
        b.iter(|| {
            let _ = read_stream_parts(stream_parts_bytes.as_slice()).expect("decode stream parts");
        });
    });
    c.bench_function("package_write_model", |b| {
        b.iter(|| {
            let _ = PackageWriter
                .write_file(&model_path, &model)
                .expect("write package");
        });
    });
    c.bench_function("package_read_model", |b| {
        b.iter(|| {
            let _ = PackageReader.read_file(&model_path).expect("read package");
        });
    });
    c.bench_function("package_write_parts", |b| {
        b.iter(|| {
            let _ = write_package_parts_file(&parts_path, &parts).expect("write parts package");
        });
    });
    c.bench_function("package_read_parts", |b| {
        b.iter(|| {
            let _ = read_package_parts_file(&parts_path).expect("read parts package");
        });
    });
    c.bench_function("package_read_manifest", |b| {
        b.iter(|| {
            let _ = PackageReader
                .read_manifest(&model_path)
                .expect("read manifest");
        });
    });

    local_attribute_write_benches(c);
    shared_corpus_write_benches(c);
}

fn shared_corpus_write_benches(c: &mut Criterion) {
    for case in common::load_named_write_cases(SHARED_CORPUS_WRITE_CASES) {
        bench_write_case(
            c,
            &format!("shared_corpus/{}", case.name),
            Some(case.input_bytes),
            &case.model,
        );
    }
}

fn local_attribute_write_benches(c: &mut Criterion) {
    let model = attribute_heavy_benchmark_model(ATTRIBUTE_HEAVY_BENCHMARK_BUILDINGS);
    bench_write_case(c, "local_attribute_heavy", None, &model);
}

fn bench_write_case(
    c: &mut Criterion,
    group_name: &str,
    input_bytes: Option<u64>,
    model: &OwnedCityModel,
) {
    let mut group = c.benchmark_group(group_name);
    if let Some(input_bytes) = input_bytes {
        group.throughput(Throughput::Bytes(input_bytes));
    }

    let dir = tempdir().expect("tempdir");
    let file_stem = group_name.replace('/', "-");
    let model_path = dir.path().join(format!("{file_stem}.cityarrow"));
    let parts_path = dir.path().join(format!("{file_stem}-parts.cityarrow"));

    group.bench_function("encode_parts", |b| {
        b.iter(|| {
            let _ = encode_parts(model).expect("encode parts");
        });
    });
    group.bench_function("stream_write_model", |b| {
        b.iter(|| {
            let mut bytes = Vec::new();
            ModelEncoder
                .encode(model, &mut bytes)
                .expect("encode stream");
        });
    });
    group.bench_function("stream_write_parts", |b| {
        let parts = encode_parts(model).expect("encode parts");
        b.iter(|| {
            let mut bytes = Vec::new();
            write_stream_parts(&parts, &mut bytes).expect("encode stream parts");
        });
    });
    group.bench_function("package_write_model", |b| {
        b.iter(|| {
            let _ = PackageWriter
                .write_file(&model_path, model)
                .expect("write package");
        });
    });
    group.bench_function("package_write_parts", |b| {
        let parts = encode_parts(model).expect("encode parts");
        b.iter(|| {
            let _ = write_package_parts_file(&parts_path, &parts).expect("write parts package");
        });
    });
    group.finish();
}

criterion_group!(benches, split_benches);
criterion_main!(benches);
