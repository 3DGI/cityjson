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
use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::tempdir;

const LARGE_BENCHMARK_BUILDINGS: usize = 4_096;

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
}

criterion_group!(benches, split_benches);
criterion_main!(benches);
