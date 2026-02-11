//! Benchmarks for processing and querying `CityModels`.

#[allow(dead_code)]
mod support;

use criterion::{Criterion, criterion_group, criterion_main};
use rand::Rng;
use std::hint::black_box;
use support::{DEFAULT_SIZE_PROCESSOR, FAST_SIZE_PROCESSOR, params_from_env, rng_from_seed};

mod benches {
    use super::{
        Criterion, DEFAULT_SIZE_PROCESSOR, FAST_SIZE_PROCESSOR, Rng, black_box, params_from_env,
        rng_from_seed,
    };

    use cityjson::prelude::*;
    use cityjson::v2_0::{
        CityModel, CityObject, CityObjectType, GeometryBuilder, Material, Semantic, SemanticType,
        Texture,
    };
    use std::collections::HashMap;

    type AttrValue = AttributeValue<OwnedStringStorage>;
    type OwnedModel = CityModel<u32, OwnedStringStorage>;
    type OwnedCityObject = CityObject<OwnedStringStorage>;

    macro_rules! add_surface {
        ($builder:expr, $vertices:expr, [$a:expr, $b:expr, $c:expr, $d:expr], $ring_error:literal, $surface_error:literal) => {{
            let ring = $builder
                .add_ring(&[$vertices[$a], $vertices[$b], $vertices[$c], $vertices[$d]])
                .expect($ring_error);
            let surface = $builder.start_surface();
            $builder.add_surface_outer_ring(ring).expect($surface_error);
            surface
        }};
    }

    macro_rules! map_front_texture {
        ($builder:expr, $vertices:expr, $texture:expr) => {{
            let uv0 = $builder.add_uv_coordinate(0.0, 0.0);
            let uv1 = $builder.add_uv_coordinate(1.0, 0.0);
            let uv2 = $builder.add_uv_coordinate(1.0, 1.0);
            let uv3 = $builder.add_uv_coordinate(0.0, 1.0);
            $builder.map_vertex_to_uv($vertices[0], uv0);
            $builder.map_vertex_to_uv($vertices[1], uv1);
            $builder.map_vertex_to_uv($vertices[5], uv2);
            $builder.map_vertex_to_uv($vertices[4], uv3);
            $builder
                .set_texture_ring(None, $texture.clone(), "default".to_string(), true)
                .expect("failed to set texture");
        }};
    }

    fn usize_to_u32(value: usize, context: &str) -> u32 {
        u32::try_from(value).expect(context)
    }

    fn configure_model_metadata(model: &mut OwnedModel) {
        let metadata = model.metadata_mut();
        metadata.set_identifier(CityModelIdentifier::new("benchmark-model".to_string()));
        metadata.set_reference_system(CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
        ));
    }

    fn create_material_and_texture() -> (Material<OwnedStringStorage>, Texture<OwnedStringStorage>)
    {
        let mut material = Material::new("benchmark_material".to_string());
        material.set_ambient_intensity(Some(0.5));
        material.set_diffuse_color(Some([0.8, 0.8, 0.8].into()));
        let texture = Texture::new("benchmark_texture.png".to_string(), ImageType::Png);
        (material, texture)
    }

    fn add_random_vertices<R: Rng + ?Sized>(
        model: &mut OwnedModel,
        rng: &mut R,
    ) -> Vec<VertexIndex32> {
        (0..8)
            .map(|_| {
                let x = rng.random_range(0..100_000);
                let y = rng.random_range(0..100_000);
                let z = rng.random_range(0..1_000);
                model
                    .add_vertex(QuantizedCoordinate::new(x, y, z))
                    .expect("failed to add vertex")
            })
            .collect()
    }

    fn compute_height(index: u32, seed: u32) -> f64 {
        10.0 + f64::from(index) * 0.5 + f64::from(seed) * 0.001
    }

    fn add_attributes(cityobject: &mut OwnedCityObject, index: u32, seed: u32) {
        let attrs = cityobject.attributes_mut();
        let index_i64 = i64::from(index);
        let height = compute_height(index, seed);

        attrs.insert("attr_null".to_string(), AttributeValue::Null);
        attrs.insert(
            "attr_bool".to_string(),
            AttributeValue::Bool(index.is_multiple_of(2)),
        );
        attrs.insert(
            "attr_unsigned".to_string(),
            AttributeValue::Unsigned(u64::from(index)),
        );
        attrs.insert(
            "attr_integer".to_string(),
            AttributeValue::Integer(index_i64),
        );
        attrs.insert("attr_float".to_string(), AttributeValue::Float(height));
        attrs.insert(
            "attr_string".to_string(),
            AttributeValue::String(format!("name-{index}")),
        );
        attrs.insert(
            "attr_vec".to_string(),
            AttributeValue::Vec(vec![
                Box::new(AttributeValue::Integer(index_i64)),
                Box::new(AttributeValue::Float(height)),
            ]),
        );

        let mut attr_map = HashMap::new();
        attr_map.insert(
            "key".to_string(),
            Box::new(AttributeValue::String("value".to_string())),
        );
        attrs.insert("attr_map".to_string(), AttributeValue::Map(attr_map));
    }

    fn build_cube_geometry(
        model: &mut OwnedModel,
        vertices: &[VertexIndex32],
        index: u32,
        material: &Material<OwnedStringStorage>,
        texture: &Texture<OwnedStringStorage>,
    ) -> GeometryRef {
        let mut geometry_builder =
            GeometryBuilder::new(model, GeometryType::Solid, BuilderMode::Regular)
                .with_lod(LoD::LoD2);

        let mut ground_semantic = Semantic::new(SemanticType::GroundSurface);
        ground_semantic.attributes_mut().insert(
            "area".to_string(),
            AttributeValue::Float(100.0 + f64::from(index) * 0.5),
        );

        let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
        roof_semantic.attributes_mut().insert(
            "azimuth".to_string(),
            AttributeValue::Float(f64::from(index % 360)),
        );
        roof_semantic.attributes_mut().insert(
            "slope".to_string(),
            AttributeValue::Float(15.0 + f64::from(index % 30)),
        );

        let mut wall_semantic = Semantic::new(SemanticType::WallSurface);
        wall_semantic.attributes_mut().insert(
            "orientation".to_string(),
            AttributeValue::String("north".to_string()),
        );

        let builder_vertices: Vec<_> = vertices
            .iter()
            .map(|&vertex| geometry_builder.add_vertex(vertex))
            .collect();

        let surface_bottom = add_surface!(
            geometry_builder,
            builder_vertices,
            [0, 1, 2, 3],
            "failed to add bottom ring",
            "failed to add bottom surface ring"
        );
        geometry_builder
            .set_semantic_surface(None, ground_semantic, false)
            .expect("failed to set ground semantics");

        let surface_top = add_surface!(
            geometry_builder,
            builder_vertices,
            [4, 7, 6, 5],
            "failed to add top ring",
            "failed to add top surface ring"
        );
        geometry_builder
            .set_semantic_surface(None, roof_semantic, false)
            .expect("failed to set roof semantics");
        geometry_builder
            .set_material_surface(None, material.clone(), "default".to_string(), true)
            .expect("failed to set material");

        let surface_front = add_surface!(
            geometry_builder,
            builder_vertices,
            [0, 1, 5, 4],
            "failed to add front ring",
            "failed to add front surface ring"
        );
        geometry_builder
            .set_semantic_surface(None, wall_semantic, false)
            .expect("failed to set wall semantics");

        map_front_texture!(geometry_builder, builder_vertices, texture);

        let surface_right = add_surface!(
            geometry_builder,
            builder_vertices,
            [1, 2, 6, 5],
            "failed to add right ring",
            "failed to add right surface ring"
        );

        let surface_back = add_surface!(
            geometry_builder,
            builder_vertices,
            [2, 3, 7, 6],
            "failed to add back ring",
            "failed to add back surface ring"
        );

        let surface_left = add_surface!(
            geometry_builder,
            builder_vertices,
            [3, 0, 4, 7],
            "failed to add left ring",
            "failed to add left surface ring"
        );

        geometry_builder
            .add_shell(&[
                surface_bottom,
                surface_top,
                surface_front,
                surface_right,
                surface_back,
                surface_left,
            ])
            .expect("failed to add shell");

        geometry_builder
            .build_geometry()
            .expect("failed to build geometry")
    }

    fn add_cityobject<R: Rng + ?Sized>(
        model: &mut OwnedModel,
        rng: &mut R,
        index: usize,
        seed: u32,
        material: &Material<OwnedStringStorage>,
        texture: &Texture<OwnedStringStorage>,
    ) {
        let index_u32 = usize_to_u32(index, "cityobject index exceeds u32 range");
        let vertices = add_random_vertices(model, rng);
        let mut cityobject = CityObject::new(
            CityObjectIdentifier::new(format!("building-{index_u32:06}")),
            CityObjectType::Building,
        );

        add_attributes(&mut cityobject, index_u32, seed);
        let geometry_ref = build_cube_geometry(model, &vertices, index_u32, material, texture);
        cityobject.add_geometry(geometry_ref);

        model
            .cityobjects_mut()
            .add(cityobject)
            .expect("failed to add cityobject to model");
    }

    fn accumulate_attribute_value(value: &AttrValue, acc: &mut u64) {
        match value {
            AttributeValue::Null => *acc = acc.wrapping_add(1),
            AttributeValue::Bool(value) => *acc = acc.wrapping_add(if *value { 2 } else { 3 }),
            AttributeValue::Unsigned(value) => *acc = acc.wrapping_add(*value),
            AttributeValue::Integer(value) => *acc = acc.wrapping_add(value.cast_unsigned()),
            AttributeValue::Float(value) => *acc = acc.wrapping_add(value.to_bits()),
            AttributeValue::String(value) => *acc = acc.wrapping_add(value.len() as u64),
            AttributeValue::Vec(values) => {
                *acc = acc.wrapping_add(values.len() as u64);
                for value in values {
                    accumulate_attribute_value(value, acc);
                }
            }
            AttributeValue::Map(values) => {
                *acc = acc.wrapping_add(values.len() as u64);
                for (key, value) in values {
                    *acc = acc.wrapping_add(key.len() as u64);
                    accumulate_attribute_value(value, acc);
                }
            }
            AttributeValue::Geometry(_) => *acc = acc.wrapping_add(7),
            _ => {}
        }
    }

    fn compute_full_feature_stats(model: &CityModel<u32, OwnedStringStorage>) -> u64 {
        let mut acc = 0u64;

        for (_id, cityobject) in model.cityobjects().iter() {
            if let Some(attributes) = cityobject.attributes() {
                for (key, value) in attributes.iter() {
                    acc = acc.wrapping_add(key.len() as u64);
                    accumulate_attribute_value(value, &mut acc);
                }
            }

            if let Some(geometries) = cityobject.geometry() {
                for geometry_ref in geometries {
                    if let Some(geometry) = model.get_geometry(*geometry_ref) {
                        if let Some(semantics) = geometry.semantics() {
                            for semantic_ref in semantics.surfaces().iter().flatten() {
                                if let Some(semantic) = model.get_semantic(*semantic_ref) {
                                    acc = acc.wrapping_add(1);
                                    match semantic.type_semantic() {
                                        SemanticType::RoofSurface => acc = acc.wrapping_add(2),
                                        SemanticType::GroundSurface => acc = acc.wrapping_add(3),
                                        SemanticType::WallSurface => acc = acc.wrapping_add(5),
                                        SemanticType::Extension(name) => {
                                            acc = acc.wrapping_add(name.len() as u64);
                                        }
                                        _ => acc = acc.wrapping_add(1),
                                    }

                                    if let Some(attrs) = semantic.attributes() {
                                        for (key, value) in attrs.iter() {
                                            acc = acc.wrapping_add(key.len() as u64);
                                            accumulate_attribute_value(value, &mut acc);
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(materials) = geometry.materials() {
                            for (theme, mapping) in materials.iter() {
                                acc = acc.wrapping_add(theme.len() as u64);
                                for material_ref in mapping.surfaces().iter().flatten() {
                                    let _ = material_ref;
                                    acc = acc.wrapping_add(1);
                                }
                            }
                        }

                        if let Some(textures) = geometry.textures() {
                            for (theme, mapping) in textures.iter() {
                                acc = acc.wrapping_add(theme.len() as u64);
                                acc = acc.wrapping_add(mapping.vertices().len() as u64);
                                for texture_ref in mapping.ring_textures().iter().flatten() {
                                    let _ = texture_ref;
                                    acc = acc.wrapping_add(1);
                                }
                            }
                        }
                    }
                }
            }
        }

        acc
    }

    /// Generate a citymodel with n cityobjects, each with a solid geometry type.
    fn generate_citymodel(n: usize, seed: u64) -> OwnedModel {
        let seed_u32 = u32::try_from(seed).expect("seed exceeds u32 range");
        let mut model = CityModel::<u32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let mut rng = rng_from_seed(seed);
        configure_model_metadata(&mut model);
        let (material, texture) = create_material_and_texture();

        for index in 0..n {
            add_cityobject(&mut model, &mut rng, index, seed_u32, &material, &texture);
        }

        model
    }

    fn compute_mean_component(sum: i64, count: usize) -> f64 {
        let sum_i32 = i32::try_from(sum).expect("coordinate sum exceeds i32 range");
        let count_u32 = u32::try_from(count).expect("vertex count exceeds u32 range");
        f64::from(sum_i32) / f64::from(count_u32)
    }

    /// Compute the mean x,y,z coordinate for each geometry of each cityobject
    fn compute_mean_coordinates(
        model: &CityModel<u32, OwnedStringStorage>,
    ) -> Vec<(f64, f64, f64)> {
        let mut means = Vec::new();

        for (_id, cityobject) in model.cityobjects().iter() {
            if let Some(geometries) = cityobject.geometry() {
                for geometry_ref in geometries {
                    if let Some(geometry) = model.get_geometry(*geometry_ref)
                        && let Some(boundary) = geometry.boundaries()
                    {
                        let vertex_indices = boundary.vertices();

                        if vertex_indices.is_empty() {
                            continue;
                        }

                        let mut sum_x = 0i64;
                        let mut sum_y = 0i64;
                        let mut sum_z = 0i64;
                        let mut count = 0usize;

                        for vertex_idx in vertex_indices {
                            if let Some(vertex) = model.get_vertex(*vertex_idx) {
                                sum_x += vertex.x();
                                sum_y += vertex.y();
                                sum_z += vertex.z();
                                count += 1;
                            }
                        }

                        if count > 0 {
                            let mean_x = compute_mean_component(sum_x, count);
                            let mean_y = compute_mean_component(sum_y, count);
                            let mean_z = compute_mean_component(sum_z, count);
                            means.push((mean_x, mean_y, mean_z));
                        }
                    }
                }
            }
        }

        means
    }

    pub fn benchmark_mean_coordinates(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_PROCESSOR, FAST_SIZE_PROCESSOR);
        let model = generate_citymodel(params.size, params.seed);

        c.bench_function("compute_mean_coordinates", |b| {
            b.iter(|| {
                let means = compute_mean_coordinates(black_box(&model));
                black_box(means);
            });
        });
    }

    pub fn benchmark_full_feature_stats(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_PROCESSOR, FAST_SIZE_PROCESSOR);
        let model = generate_citymodel(params.size, params.seed);

        c.bench_function("compute_full_feature_stats", |b| {
            b.iter(|| {
                let stats = compute_full_feature_stats(black_box(&model));
                black_box(stats);
            });
        });
    }
}

criterion_group!(
    benches,
    benches::benchmark_mean_coordinates,
    benches::benchmark_full_feature_stats
);

criterion_main!(benches);
