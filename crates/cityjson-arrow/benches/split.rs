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

fn sample_model() -> OwnedCityModel {
    let mut model = OwnedCityModel::new(CityModelType::CityJSON);
    model
        .metadata_mut()
        .set_identifier(cityjson::v2_0::CityModelIdentifier::new(
            "sample-citymodel".to_string(),
        ));
    model.metadata_mut().set_title("Sample".to_string());

    let roof = model
        .add_semantic(OwnedSemantic::new(SemanticType::RoofSurface))
        .expect("semantic");
    let wall = model
        .add_semantic(OwnedSemantic::new(SemanticType::WallSurface))
        .expect("semantic");

    for vertex in [
        cityjson::v2_0::RealWorldCoordinate::new(0.0, 0.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(1.0, 0.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(1.0, 1.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(0.0, 1.0, 0.0),
        cityjson::v2_0::RealWorldCoordinate::new(0.5, 0.5, 1.0),
    ] {
        model.add_vertex(vertex).expect("vertex");
    }

    let boundary: Boundary<u32> = vec![
        vec![vec![0_u32, 1, 4, 0]],
        vec![vec![1_u32, 2, 4, 1]],
        vec![vec![2_u32, 3, 4, 2]],
        vec![vec![3_u32, 0, 4, 3]],
        vec![vec![0_u32, 3, 2, 1, 0]],
    ]
    .try_into()
    .expect("boundary");
    let mut semantics = SemanticMap::new();
    semantics.add_surface(Some(wall));
    semantics.add_surface(Some(wall));
    semantics.add_surface(Some(wall));
    semantics.add_surface(Some(wall));
    semantics.add_surface(Some(roof));

    let geometry = Geometry::from_stored_parts(StoredGeometryParts {
        type_geometry: cityjson::v2_0::GeometryType::MultiSurface,
        lod: Some(LoD::LoD2_2),
        boundaries: Some(boundary),
        semantics: Some(semantics),
        materials: None,
        textures: None,
        instance: None,
    });
    let geometry_handle = model.add_geometry(geometry).expect("geometry");

    let mut building = CityObject::new(
        CityObjectIdentifier::new("building-1".to_string()),
        CityObjectType::Building,
    );
    building.add_geometry(geometry_handle);
    building.attributes_mut().insert(
        "name".to_string(),
        AttributeValue::String("Sample Building".to_string()),
    );
    model.cityobjects_mut().add(building).expect("cityobject");

    model
}

fn split_benches(c: &mut Criterion) {
    let model = sample_model();
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

    c.bench_function("convert_encode_parts", |b| {
        b.iter(|| {
            let _ = encode_parts(&model).expect("encode parts");
        });
    });
    c.bench_function("convert_decode_parts", |b| {
        b.iter(|| {
            let _ = decode_parts(&parts).expect("decode parts");
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
