//! Benchmark that builds solid geometries at multiple sizes.

#[allow(dead_code)]
mod support;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use support::{
    DEFAULT_SIZE_BACKEND_COMPARE, FAST_SIZE_BACKEND_COMPARE, comparison_sizes, params_from_env,
};

// ==================== DEFAULT BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-default")]
mod default_benches {
    use super::*;

    use cityjson::prelude::*;
    use cityjson::v2_0::*;
    use std::collections::HashMap;

    fn build_simple_solid(n_buildings: usize) {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        // Pre-create vertices for a cube
        let v0 = model.add_vertex(QuantizedCoordinate::new(0, 0, 0)).unwrap();
        let v1 = model
            .add_vertex(QuantizedCoordinate::new(1000, 0, 0))
            .unwrap();
        let v2 = model
            .add_vertex(QuantizedCoordinate::new(1000, 1000, 0))
            .unwrap();
        let v3 = model
            .add_vertex(QuantizedCoordinate::new(0, 1000, 0))
            .unwrap();
        let v4 = model
            .add_vertex(QuantizedCoordinate::new(0, 0, 500))
            .unwrap();
        let v5 = model
            .add_vertex(QuantizedCoordinate::new(1000, 0, 500))
            .unwrap();
        let v6 = model
            .add_vertex(QuantizedCoordinate::new(1000, 1000, 500))
            .unwrap();
        let v7 = model
            .add_vertex(QuantizedCoordinate::new(0, 1000, 500))
            .unwrap();

        for i in 0..n_buildings {
            let mut cityobject =
                CityObject::new(format!("building-{}", i), CityObjectType::Building);

            // Build a simple cube geometry
            let mut geometry_builder =
                GeometryBuilder::new(&mut model, GeometryType::Solid, BuilderMode::Regular)
                    .with_lod(LoD::LoD2);

            let bv0 = geometry_builder.add_vertex(v0);
            let bv1 = geometry_builder.add_vertex(v1);
            let bv2 = geometry_builder.add_vertex(v2);
            let bv3 = geometry_builder.add_vertex(v3);
            let bv4 = geometry_builder.add_vertex(v4);
            let bv5 = geometry_builder.add_vertex(v5);
            let bv6 = geometry_builder.add_vertex(v6);
            let bv7 = geometry_builder.add_vertex(v7);

            // Bottom face
            let ring_bottom = geometry_builder.add_ring(&[bv0, bv1, bv2, bv3]).unwrap();
            let surface_bottom = geometry_builder.start_surface();
            geometry_builder
                .add_surface_outer_ring(ring_bottom)
                .unwrap();

            // Top face
            let ring_top = geometry_builder.add_ring(&[bv4, bv7, bv6, bv5]).unwrap();
            let surface_top = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_top).unwrap();

            // Front face
            let ring_front = geometry_builder.add_ring(&[bv0, bv1, bv5, bv4]).unwrap();
            let surface_front = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_front).unwrap();

            // Right face
            let ring_right = geometry_builder.add_ring(&[bv1, bv2, bv6, bv5]).unwrap();
            let surface_right = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_right).unwrap();

            // Back face
            let ring_back = geometry_builder.add_ring(&[bv2, bv3, bv7, bv6]).unwrap();
            let surface_back = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_back).unwrap();

            // Left face
            let ring_left = geometry_builder.add_ring(&[bv3, bv0, bv4, bv7]).unwrap();
            let surface_left = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_left).unwrap();

            // Create the shell
            geometry_builder
                .add_shell(&[
                    surface_bottom,
                    surface_top,
                    surface_front,
                    surface_right,
                    surface_back,
                    surface_left,
                ])
                .unwrap();

            let geometry_ref = geometry_builder.build().unwrap();
            cityobject.geometry_mut().push(geometry_ref);

            model.cityobjects_mut().add(cityobject);
        }
    }

    fn build_full_solid(n_buildings: usize) {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);

        let mut material = Material::new("benchmark_material".to_string());
        material.set_ambient_intensity(Some(0.5));
        material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
        let texture = Texture::new("benchmark_texture.png".to_string(), ImageType::Png);

        // Pre-create vertices for a cube
        let v0 = model.add_vertex(QuantizedCoordinate::new(0, 0, 0)).unwrap();
        let v1 = model
            .add_vertex(QuantizedCoordinate::new(1000, 0, 0))
            .unwrap();
        let v2 = model
            .add_vertex(QuantizedCoordinate::new(1000, 1000, 0))
            .unwrap();
        let v3 = model
            .add_vertex(QuantizedCoordinate::new(0, 1000, 0))
            .unwrap();
        let v4 = model
            .add_vertex(QuantizedCoordinate::new(0, 0, 500))
            .unwrap();
        let v5 = model
            .add_vertex(QuantizedCoordinate::new(1000, 0, 500))
            .unwrap();
        let v6 = model
            .add_vertex(QuantizedCoordinate::new(1000, 1000, 500))
            .unwrap();
        let v7 = model
            .add_vertex(QuantizedCoordinate::new(0, 1000, 500))
            .unwrap();

        for i in 0..n_buildings {
            let mut cityobject =
                CityObject::new(format!("building-{}", i), CityObjectType::Building);

            let attrs = cityobject.attributes_mut();
            let height = 10.0 + (i as f64) * 0.5;
            attrs.insert("attr_null".to_string(), AttributeValue::Null);
            attrs.insert("attr_bool".to_string(), AttributeValue::Bool(i % 2 == 0));
            attrs.insert("attr_unsigned".to_string(), AttributeValue::Unsigned(i as u64));
            attrs.insert("attr_integer".to_string(), AttributeValue::Integer(i as i64));
            attrs.insert("attr_float".to_string(), AttributeValue::Float(height));
            attrs.insert(
                "attr_string".to_string(),
                AttributeValue::String(format!("name-{}", i)),
            );
            attrs.insert(
                "attr_vec".to_string(),
                AttributeValue::Vec(vec![
                    Box::new(AttributeValue::Integer(i as i64)),
                    Box::new(AttributeValue::Float(height)),
                ]),
            );
            let mut attr_map = HashMap::new();
            attr_map.insert(
                "key".to_string(),
                Box::new(AttributeValue::String("value".to_string())),
            );
            attrs.insert("attr_map".to_string(), AttributeValue::Map(attr_map));
            attrs.insert(
                "attr_geometry".to_string(),
                AttributeValue::Geometry(ResourceId32::new(0, 0)),
            );

            let mut geometry_builder =
                GeometryBuilder::new(&mut model, GeometryType::Solid, BuilderMode::Regular)
                    .with_lod(LoD::LoD2);

            let mut ground_semantic = Semantic::new(SemanticType::GroundSurface);
            let ground_attrs = ground_semantic.attributes_mut();
            ground_attrs.insert(
                "area".to_string(),
                AttributeValue::Float(100.0 + (i as f64) * 0.5),
            );

            let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
            let roof_attrs = roof_semantic.attributes_mut();
            roof_attrs.insert(
                "azimuth".to_string(),
                AttributeValue::Float((i % 360) as f64),
            );
            roof_attrs.insert(
                "slope".to_string(),
                AttributeValue::Float(15.0 + ((i % 30) as f64)),
            );

            let mut wall_semantic = Semantic::new(SemanticType::WallSurface);
            let wall_attrs = wall_semantic.attributes_mut();
            wall_attrs.insert(
                "orientation".to_string(),
                AttributeValue::String("north".to_string()),
            );

            let bv0 = geometry_builder.add_vertex(v0);
            let bv1 = geometry_builder.add_vertex(v1);
            let bv2 = geometry_builder.add_vertex(v2);
            let bv3 = geometry_builder.add_vertex(v3);
            let bv4 = geometry_builder.add_vertex(v4);
            let bv5 = geometry_builder.add_vertex(v5);
            let bv6 = geometry_builder.add_vertex(v6);
            let bv7 = geometry_builder.add_vertex(v7);

            let ring_bottom = geometry_builder.add_ring(&[bv0, bv1, bv2, bv3]).unwrap();
            let surface_bottom = geometry_builder.start_surface();
            geometry_builder
                .add_surface_outer_ring(ring_bottom)
                .unwrap();
            geometry_builder
                .set_semantic_surface(None, ground_semantic, false)
                .unwrap();

            let ring_top = geometry_builder.add_ring(&[bv4, bv7, bv6, bv5]).unwrap();
            let surface_top = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_top).unwrap();
            geometry_builder
                .set_semantic_surface(None, roof_semantic, false)
                .unwrap();
            geometry_builder
                .set_material_surface(None, material.clone(), "default".to_string(), true)
                .unwrap();

            let ring_front = geometry_builder.add_ring(&[bv0, bv1, bv5, bv4]).unwrap();
            let surface_front = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_front).unwrap();
            geometry_builder
                .set_semantic_surface(None, wall_semantic, false)
                .unwrap();

            let uv0 = geometry_builder.add_uv_coordinate(0.0, 0.0);
            let uv1 = geometry_builder.add_uv_coordinate(1.0, 0.0);
            let uv2 = geometry_builder.add_uv_coordinate(1.0, 1.0);
            let uv3 = geometry_builder.add_uv_coordinate(0.0, 1.0);
            geometry_builder.map_vertex_to_uv(bv0, uv0);
            geometry_builder.map_vertex_to_uv(bv1, uv1);
            geometry_builder.map_vertex_to_uv(bv5, uv2);
            geometry_builder.map_vertex_to_uv(bv4, uv3);
            geometry_builder
                .set_texture_ring(None, texture.clone(), "default".to_string(), true)
                .unwrap();

            let ring_right = geometry_builder.add_ring(&[bv1, bv2, bv6, bv5]).unwrap();
            let surface_right = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_right).unwrap();

            let ring_back = geometry_builder.add_ring(&[bv2, bv3, bv7, bv6]).unwrap();
            let surface_back = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_back).unwrap();

            let ring_left = geometry_builder.add_ring(&[bv3, bv0, bv4, bv7]).unwrap();
            let surface_left = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_left).unwrap();

            geometry_builder
                .add_shell(&[
                    surface_bottom,
                    surface_top,
                    surface_front,
                    surface_right,
                    surface_back,
                    surface_left,
                ])
                .unwrap();

            let geometry_ref = geometry_builder.build().unwrap();
            cityobject.geometry_mut().push(geometry_ref);

            model.cityobjects_mut().add(cityobject);
        }
    }

    pub fn bench_build_solids(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_BACKEND_COMPARE, FAST_SIZE_BACKEND_COMPARE);
        let sizes = comparison_sizes(params.size);
        let mut group = c.benchmark_group("backend_comparison");

        for n in sizes.iter() {
            group.throughput(Throughput::Elements(*n as u64));
            group.bench_with_input(BenchmarkId::new("build_solids", n), n, |b, &n| {
                b.iter(|| {
                    build_simple_solid(black_box(n));
                });
            });
        }

        group.finish();
    }

    pub fn bench_build_solids_full(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_BACKEND_COMPARE, FAST_SIZE_BACKEND_COMPARE);
        let sizes = comparison_sizes(params.size);
        let mut group = c.benchmark_group("backend_comparison");

        for n in sizes.iter() {
            group.throughput(Throughput::Elements(*n as u64));
            group.bench_with_input(BenchmarkId::new("build_solids_full", n), n, |b, &n| {
                b.iter(|| {
                    build_full_solid(black_box(n));
                });
            });
        }

        group.finish();
    }
}

// ==================== NESTED BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-nested")]
mod nested_benches {
    use super::*;
    use cityjson::backend::nested;
    use cityjson::backend::nested::appearance::{ImageType, Material, Texture};
    use cityjson::backend::nested::attributes::AttributeValue;
    use cityjson::prelude::*;
    use std::collections::HashMap;

    fn build_simple_solid(n_buildings: usize) {
        let mut model = nested::CityModel::<OwnedStringStorage, ResourceId32>::new(CityModelType::CityJSON);

        // Pre-create vertices for a cube
        let v0 = model.add_vertex(QuantizedCoordinate::new(0, 0, 0)).unwrap();
        let v1 = model
            .add_vertex(QuantizedCoordinate::new(1000, 0, 0))
            .unwrap();
        let v2 = model
            .add_vertex(QuantizedCoordinate::new(1000, 1000, 0))
            .unwrap();
        let v3 = model
            .add_vertex(QuantizedCoordinate::new(0, 1000, 0))
            .unwrap();
        let v4 = model
            .add_vertex(QuantizedCoordinate::new(0, 0, 500))
            .unwrap();
        let v5 = model
            .add_vertex(QuantizedCoordinate::new(1000, 0, 500))
            .unwrap();
        let v6 = model
            .add_vertex(QuantizedCoordinate::new(1000, 1000, 500))
            .unwrap();
        let v7 = model
            .add_vertex(QuantizedCoordinate::new(0, 1000, 500))
            .unwrap();

        for i in 0..n_buildings {
            let co_id = format!("building-{}", i);
            let cityobject = nested::CityObject::new(nested::cityobject::CityObjectType::Building);

            // Add the cityobject first
            model.add_cityobject(co_id.clone(), cityobject);

            // Build a simple cube geometry
            let mut geometry_builder = nested::GeometryBuilder::new(
                &mut model,
                GeometryType::Solid,
                nested::BuilderMode::Regular,
            )
            .with_lod(LoD::LoD2);

            geometry_builder.add_vertex(v0).unwrap();
            geometry_builder.add_vertex(v1).unwrap();
            geometry_builder.add_vertex(v2).unwrap();
            geometry_builder.add_vertex(v3).unwrap();
            geometry_builder.add_vertex(v4).unwrap();
            geometry_builder.add_vertex(v5).unwrap();
            geometry_builder.add_vertex(v6).unwrap();
            geometry_builder.add_vertex(v7).unwrap();

            // Bottom face (indices 0-3)
            let ring_bottom = geometry_builder.add_ring(&[0, 1, 2, 3]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder
                .add_surface_outer_ring(ring_bottom)
                .unwrap();
            let surface_bottom = geometry_builder.end_surface().unwrap();

            // Top face (indices 4-7)
            let ring_top = geometry_builder.add_ring(&[4, 7, 6, 5]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_top).unwrap();
            let surface_top = geometry_builder.end_surface().unwrap();

            // Front face
            let ring_front = geometry_builder.add_ring(&[0, 1, 5, 4]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_front).unwrap();
            let surface_front = geometry_builder.end_surface().unwrap();

            // Right face
            let ring_right = geometry_builder.add_ring(&[1, 2, 6, 5]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_right).unwrap();
            let surface_right = geometry_builder.end_surface().unwrap();

            // Back face
            let ring_back = geometry_builder.add_ring(&[2, 3, 7, 6]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_back).unwrap();
            let surface_back = geometry_builder.end_surface().unwrap();

            // Left face
            let ring_left = geometry_builder.add_ring(&[3, 0, 4, 7]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_left).unwrap();
            let surface_left = geometry_builder.end_surface().unwrap();

            // Create the shell
            geometry_builder.start_shell().unwrap();
            geometry_builder.add_shell_surface(surface_bottom).unwrap();
            geometry_builder.add_shell_surface(surface_top).unwrap();
            geometry_builder.add_shell_surface(surface_front).unwrap();
            geometry_builder.add_shell_surface(surface_right).unwrap();
            geometry_builder.add_shell_surface(surface_back).unwrap();
            geometry_builder.add_shell_surface(surface_left).unwrap();
            geometry_builder.end_shell().unwrap();

            // Build the geometry and add it to the cityobject
            let geometry = geometry_builder.build().unwrap();
            model.add_geometry_to_cityobject(&co_id, geometry).unwrap();
        }
    }

    fn build_full_solid(n_buildings: usize) {
        let mut model = nested::CityModel::<OwnedStringStorage, ResourceId32>::new(CityModelType::CityJSON);

        let mut material = Material::new("benchmark_material".to_string());
        material.set_ambient_intensity(Some(0.5));
        material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
        let material_idx = model.add_material(material);
        let texture = Texture::new("benchmark_texture.png".to_string(), ImageType::Png);
        let texture_idx = model.add_texture(texture);
        model.set_default_theme_material(Some("default".to_string()));
        model.set_default_theme_texture(Some("default".to_string()));

        // Pre-create vertices for a cube
        let v0 = model.add_vertex(QuantizedCoordinate::new(0, 0, 0)).unwrap();
        let v1 = model
            .add_vertex(QuantizedCoordinate::new(1000, 0, 0))
            .unwrap();
        let v2 = model
            .add_vertex(QuantizedCoordinate::new(1000, 1000, 0))
            .unwrap();
        let v3 = model
            .add_vertex(QuantizedCoordinate::new(0, 1000, 0))
            .unwrap();
        let v4 = model
            .add_vertex(QuantizedCoordinate::new(0, 0, 500))
            .unwrap();
        let v5 = model
            .add_vertex(QuantizedCoordinate::new(1000, 0, 500))
            .unwrap();
        let v6 = model
            .add_vertex(QuantizedCoordinate::new(1000, 1000, 500))
            .unwrap();
        let v7 = model
            .add_vertex(QuantizedCoordinate::new(0, 1000, 500))
            .unwrap();

        for i in 0..n_buildings {
            let co_id = format!("building-{}", i);
            let mut cityobject =
                nested::CityObject::new(nested::cityobject::CityObjectType::Building);
            let attrs = cityobject.attributes_mut();
            let height = 10.0 + (i as f64) * 0.5;
            attrs.insert("attr_null".to_string(), AttributeValue::Null);
            attrs.insert("attr_bool".to_string(), AttributeValue::Bool(i % 2 == 0));
            attrs.insert("attr_unsigned".to_string(), AttributeValue::Unsigned(i as u64));
            attrs.insert("attr_integer".to_string(), AttributeValue::Integer(i as i64));
            attrs.insert("attr_float".to_string(), AttributeValue::Float(height));
            attrs.insert(
                "attr_string".to_string(),
                AttributeValue::String(format!("name-{}", i)),
            );
            attrs.insert(
                "attr_vec".to_string(),
                AttributeValue::Vec(vec![
                    Box::new(AttributeValue::Integer(i as i64)),
                    Box::new(AttributeValue::Float(height)),
                ]),
            );
            let mut attr_map = HashMap::new();
            attr_map.insert(
                "key".to_string(),
                Box::new(AttributeValue::String("value".to_string())),
            );
            attrs.insert("attr_map".to_string(), AttributeValue::Map(attr_map));
            attrs.insert(
                "attr_geometry".to_string(),
                AttributeValue::Geometry(Box::new(nested::Geometry::new(
                    GeometryType::Solid,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ))),
            );

            model.add_cityobject(co_id.clone(), cityobject);

            let mut geometry_builder = nested::GeometryBuilder::new(
                &mut model,
                GeometryType::Solid,
                nested::BuilderMode::Regular,
            )
            .with_lod(LoD::LoD2);

            geometry_builder.add_vertex(v0).unwrap();
            geometry_builder.add_vertex(v1).unwrap();
            geometry_builder.add_vertex(v2).unwrap();
            geometry_builder.add_vertex(v3).unwrap();
            geometry_builder.add_vertex(v4).unwrap();
            geometry_builder.add_vertex(v5).unwrap();
            geometry_builder.add_vertex(v6).unwrap();
            geometry_builder.add_vertex(v7).unwrap();

            let ring_bottom = geometry_builder.add_ring(&[0, 1, 2, 3]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder
                .add_surface_outer_ring(ring_bottom)
                .unwrap();
            let mut ground_semantic =
                nested::semantics::Semantic::new(nested::semantics::SemanticType::GroundSurface);
            let ground_attrs = ground_semantic.attributes_mut();
            ground_attrs.insert(
                "area".to_string(),
                AttributeValue::Float(100.0 + (i as f64) * 0.5),
            );
            geometry_builder
                .set_semantic_surface(0, ground_semantic, false)
                .unwrap();
            let surface_bottom = geometry_builder.end_surface().unwrap();

            let ring_top = geometry_builder.add_ring(&[4, 7, 6, 5]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_top).unwrap();
            let mut roof_semantic =
                nested::semantics::Semantic::new(nested::semantics::SemanticType::RoofSurface);
            let roof_attrs = roof_semantic.attributes_mut();
            roof_attrs.insert(
                "azimuth".to_string(),
                AttributeValue::Float((i % 360) as f64),
            );
            roof_attrs.insert(
                "slope".to_string(),
                AttributeValue::Float(15.0 + ((i % 30) as f64)),
            );
            geometry_builder
                .set_semantic_surface(1, roof_semantic, true)
                .unwrap();
            let surface_top = geometry_builder.end_surface().unwrap();
            geometry_builder
                .set_material_surface("default".to_string(), surface_top, material_idx)
                .unwrap();

            let ring_front = geometry_builder.add_ring(&[0, 1, 5, 4]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_front).unwrap();
            let mut wall_semantic =
                nested::semantics::Semantic::new(nested::semantics::SemanticType::WallSurface);
            let wall_attrs = wall_semantic.attributes_mut();
            wall_attrs.insert(
                "orientation".to_string(),
                AttributeValue::String("north".to_string()),
            );
            geometry_builder
                .set_semantic_surface(2, wall_semantic, false)
                .unwrap();
            let surface_front = geometry_builder.end_surface().unwrap();
            geometry_builder
                .add_uv_to_vertex(0, UVCoordinate::new(0.0, 0.0))
                .unwrap();
            geometry_builder
                .add_uv_to_vertex(1, UVCoordinate::new(1.0, 0.0))
                .unwrap();
            geometry_builder
                .add_uv_to_vertex(5, UVCoordinate::new(1.0, 1.0))
                .unwrap();
            geometry_builder
                .add_uv_to_vertex(4, UVCoordinate::new(0.0, 1.0))
                .unwrap();
            geometry_builder
                .set_texture_ring("default".to_string(), ring_front, texture_idx)
                .unwrap();

            let ring_right = geometry_builder.add_ring(&[1, 2, 6, 5]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_right).unwrap();
            let surface_right = geometry_builder.end_surface().unwrap();

            let ring_back = geometry_builder.add_ring(&[2, 3, 7, 6]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_back).unwrap();
            let surface_back = geometry_builder.end_surface().unwrap();

            let ring_left = geometry_builder.add_ring(&[3, 0, 4, 7]).unwrap();
            geometry_builder.start_surface().unwrap();
            geometry_builder.add_surface_outer_ring(ring_left).unwrap();
            let surface_left = geometry_builder.end_surface().unwrap();

            geometry_builder.start_shell().unwrap();
            geometry_builder.add_shell_surface(surface_bottom).unwrap();
            geometry_builder.add_shell_surface(surface_top).unwrap();
            geometry_builder.add_shell_surface(surface_front).unwrap();
            geometry_builder.add_shell_surface(surface_right).unwrap();
            geometry_builder.add_shell_surface(surface_back).unwrap();
            geometry_builder.add_shell_surface(surface_left).unwrap();
            geometry_builder.end_shell().unwrap();

            let geometry = geometry_builder.build().unwrap();
            model.add_geometry_to_cityobject(&co_id, geometry).unwrap();
        }
    }

    pub fn bench_build_solids(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_BACKEND_COMPARE, FAST_SIZE_BACKEND_COMPARE);
        let sizes = comparison_sizes(params.size);
        let mut group = c.benchmark_group("backend_comparison");

        for n in sizes.iter() {
            group.throughput(Throughput::Elements(*n as u64));
            group.bench_with_input(BenchmarkId::new("build_solids", n), n, |b, &n| {
                b.iter(|| {
                    build_simple_solid(black_box(n));
                });
            });
        }

        group.finish();
    }

    pub fn bench_build_solids_full(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_BACKEND_COMPARE, FAST_SIZE_BACKEND_COMPARE);
        let sizes = comparison_sizes(params.size);
        let mut group = c.benchmark_group("backend_comparison");

        for n in sizes.iter() {
            group.throughput(Throughput::Elements(*n as u64));
            group.bench_with_input(BenchmarkId::new("build_solids_full", n), n, |b, &n| {
                b.iter(|| {
                    build_full_solid(black_box(n));
                });
            });
        }

        group.finish();
    }
}

// ==================== CRITERION GROUPS ====================

#[cfg(all(feature = "backend-default", not(feature = "backend-nested")))]
criterion_group!(
    benches,
    default_benches::bench_build_solids,
    default_benches::bench_build_solids_full
);

#[cfg(all(feature = "backend-nested", not(feature = "backend-default")))]
criterion_group!(
    benches,
    nested_benches::bench_build_solids,
    nested_benches::bench_build_solids_full
);

#[cfg(all(feature = "backend-default", feature = "backend-nested"))]
criterion_group!(
    benches,
    default_benches::bench_build_solids,
    default_benches::bench_build_solids_full,
    nested_benches::bench_build_solids,
    nested_benches::bench_build_solids_full
);

criterion_main!(benches);
