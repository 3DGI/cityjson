//! Memory-focused benchmarks that capture heap usage with dhat.

#[allow(dead_code)]
mod support;

use rand::Rng;
use std::env;
use std::hint::black_box;
use support::{BenchParams, DEFAULT_SIZE_MEMORY, FAST_SIZE_MEMORY, params_from_env, rng_from_seed};

// Enable dhat heap profiling for the entire benchmark
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

// ==================== DEFAULT BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-default")]
mod default_benches {
    use super::*;

    use cityjson::prelude::*;
    use cityjson::v2_0::*;
    use std::collections::HashMap;

    /// Build a CityModel with the specified vertex index type and number of cityobjects.
    /// Each cityobject will have a solid geometry with 8 vertices (a cube).
    fn build_model<VR: VertexRef>(
        n_cityobjects: usize,
        seed: u64,
    ) -> CityModel<VR, ResourceId32, OwnedStringStorage> {
        let mut model =
            CityModel::<VR, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let mut rng = rng_from_seed(seed);

        // Set basic metadata
        let metadata = model.metadata_mut();
        metadata.set_identifier(CityModelIdentifier::new("memory-benchmark".to_string()));
        metadata.set_reference_system(CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
        ));

        let mut material = Material::new("benchmark_material".to_string());
        material.set_ambient_intensity(Some(0.5));
        material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
        let texture = Texture::new("benchmark_texture.png".to_string(), ImageType::Png);

        for i in 0..n_cityobjects {
            // Generate 8 random vertices for a cube-like solid
            let vertices: Vec<_> = (0..8)
                .map(|_| {
                    let x = rng.random_range(0..100000);
                    let y = rng.random_range(0..100000);
                    let z = rng.random_range(0..1000);
                    model.add_vertex(QuantizedCoordinate::new(x, y, z)).unwrap()
                })
                .collect();

            // Create a CityObject
            let mut cityobject =
                CityObject::new(format!("building-{:06}", i), CityObjectType::Building);

            let attrs = cityobject.attributes_mut();
            let height = 10.0 + (i as f64) * 0.5 + (seed as f64) * 0.001;
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

            // Build a solid geometry using GeometryBuilder
            {
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

                // Add vertices to the builder
                let bv: Vec<_> = vertices
                    .iter()
                    .map(|&v| geometry_builder.add_vertex(v))
                    .collect();

                // Create 6 faces of the cube (simplified solid)
                // Bottom face
                let ring_bottom = geometry_builder
                    .add_ring(&[bv[0], bv[1], bv[2], bv[3]])
                    .unwrap();
                let surface_bottom = geometry_builder.start_surface();
                geometry_builder
                    .add_surface_outer_ring(ring_bottom)
                    .unwrap();
                geometry_builder
                    .set_semantic_surface(None, ground_semantic, false)
                    .unwrap();

                // Top face
                let ring_top = geometry_builder
                    .add_ring(&[bv[4], bv[7], bv[6], bv[5]])
                    .unwrap();
                let surface_top = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_top).unwrap();
                geometry_builder
                    .set_semantic_surface(None, roof_semantic, false)
                    .unwrap();
                geometry_builder
                    .set_material_surface(None, material.clone(), "default".to_string(), true)
                    .unwrap();

                // Front face
                let ring_front = geometry_builder
                    .add_ring(&[bv[0], bv[1], bv[5], bv[4]])
                    .unwrap();
                let surface_front = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_front).unwrap();
                geometry_builder
                    .set_semantic_surface(None, wall_semantic, false)
                    .unwrap();
                let uv0 = geometry_builder.add_uv_coordinate(0.0, 0.0);
                let uv1 = geometry_builder.add_uv_coordinate(1.0, 0.0);
                let uv2 = geometry_builder.add_uv_coordinate(1.0, 1.0);
                let uv3 = geometry_builder.add_uv_coordinate(0.0, 1.0);
                geometry_builder.map_vertex_to_uv(bv[0], uv0);
                geometry_builder.map_vertex_to_uv(bv[1], uv1);
                geometry_builder.map_vertex_to_uv(bv[5], uv2);
                geometry_builder.map_vertex_to_uv(bv[4], uv3);
                geometry_builder
                    .set_texture_ring(None, texture.clone(), "default".to_string(), true)
                    .unwrap();

                // Right face
                let ring_right = geometry_builder
                    .add_ring(&[bv[1], bv[2], bv[6], bv[5]])
                    .unwrap();
                let surface_right = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_right).unwrap();

                // Back face
                let ring_back = geometry_builder
                    .add_ring(&[bv[2], bv[3], bv[7], bv[6]])
                    .unwrap();
                let surface_back = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_back).unwrap();

                // Left face
                let ring_left = geometry_builder
                    .add_ring(&[bv[3], bv[0], bv[4], bv[7]])
                    .unwrap();
                let surface_left = geometry_builder.start_surface();
                geometry_builder.add_surface_outer_ring(ring_left).unwrap();

                // Create the shell from all surfaces
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

                // Build the geometry
                let geometry_ref = geometry_builder.build().unwrap();

                // Add geometry to the CityObject
                cityobject.geometry_mut().push(geometry_ref);
            }

            // Add the CityObject to the model
            model.cityobjects_mut().add(cityobject);
        }

        model
    }

    pub fn run(params: BenchParams) {
        let _profiler = dhat::Profiler::new_heap();
        let model = build_model::<u32>(black_box(params.size), params.seed);
        black_box(&model);
        drop(model);
    }
}

// ==================== NESTED BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-nested")]
mod nested_benches {
    use super::*;
    use cityjson::backend::nested;
    use cityjson::backend::nested::appearance::{ImageType, Material, Texture};
    use cityjson::backend::nested::attributes::AttributeValue;
    use cityjson::backend::nested::geometry::{GeometryType, LoD};
    use cityjson::prelude::*;
    use std::collections::HashMap;

    /// Build a CityModel with the nested backend and the specified number of cityobjects.
    /// Each cityobject will have a solid geometry with 8 vertices (a cube).
    fn build_model(
        n_cityobjects: usize,
        seed: u64,
    ) -> nested::CityModel<OwnedStringStorage, ResourceId32> {
        let mut model = nested::CityModel::<OwnedStringStorage, ResourceId32>::new(CityModelType::CityJSON);
        let mut rng = rng_from_seed(seed);

        // Set basic metadata
        let metadata = model.metadata_mut();
        metadata.set_identifier(nested::metadata::CityModelIdentifier::new(
            "memory-benchmark".to_string(),
        ));
        metadata.set_reference_system(nested::metadata::CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
        ));

        let mut material = Material::new("benchmark_material".to_string());
        material.set_ambient_intensity(Some(0.5));
        material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
        let material_idx = model.add_material(material);
        let texture = Texture::new("benchmark_texture.png".to_string(), ImageType::Png);
        let texture_idx = model.add_texture(texture);
        model.set_default_theme_material(Some("default".to_string()));
        model.set_default_theme_texture(Some("default".to_string()));

        for i in 0..n_cityobjects {
            // Generate 8 random vertices for a cube-like solid
            let vertices: Vec<_> = (0..8)
                .map(|_| {
                    let x = rng.random_range(0..100000);
                    let y = rng.random_range(0..100000);
                    let z = rng.random_range(0..1000);
                    model.add_vertex(QuantizedCoordinate::new(x, y, z)).unwrap()
                })
                .collect();

            // Create a CityObject ID
            let co_id = format!("building-{:06}", i);

            // Create and add a CityObject
            let mut cityobject =
                nested::CityObject::new(nested::cityobject::CityObjectType::Building);

            let attrs = cityobject.attributes_mut();
            let height = 10.0 + (i as f64) * 0.5 + (seed as f64) * 0.001;
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

            // Build a solid geometry using GeometryBuilder
            {
                let mut geometry_builder = nested::GeometryBuilder::new(
                    &mut model,
                    GeometryType::Solid,
                    nested::BuilderMode::Regular,
                )
                .with_lod(LoD::LoD2);

                // Add vertices to the builder
                for &v in &vertices {
                    geometry_builder.add_vertex(v).unwrap();
                }

                // Create 6 faces of the cube (simplified solid)
                // Bottom face (indices 0-3)
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

                // Top face (indices 4-7)
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

                // Front face
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

                // Create the shell from all surfaces
                geometry_builder.start_shell().unwrap();
                geometry_builder.add_shell_surface(surface_bottom).unwrap();
                geometry_builder.add_shell_surface(surface_top).unwrap();
                geometry_builder.add_shell_surface(surface_front).unwrap();
                geometry_builder.add_shell_surface(surface_right).unwrap();
                geometry_builder.add_shell_surface(surface_back).unwrap();
                geometry_builder.add_shell_surface(surface_left).unwrap();
                geometry_builder.end_shell().unwrap();

                // Build the geometry
                let geometry = geometry_builder.build().unwrap();

                // Add geometry to the CityObject
                model.add_geometry_to_cityobject(&co_id, geometry).unwrap();
            }
        }

        model
    }

    pub fn run(params: BenchParams) {
        let _profiler = dhat::Profiler::new_heap();
        let model = build_model(black_box(params.size), params.seed);
        black_box(&model);
        drop(model);
    }
}

fn main() {
    let params = params_from_env(DEFAULT_SIZE_MEMORY, FAST_SIZE_MEMORY);
    #[allow(unused_variables)]
    let backend = env::var("BENCH_BACKEND").unwrap_or_default();

    #[cfg(all(feature = "backend-default", feature = "backend-nested"))]
    {
        if backend == "nested" {
            nested_benches::run(params);
        } else {
            default_benches::run(params);
        }
    }

    #[cfg(all(feature = "backend-default", not(feature = "backend-nested")))]
    {
        default_benches::run(params);
    }

    #[cfg(all(feature = "backend-nested", not(feature = "backend-default")))]
    {
        nested_benches::run(params);
    }
}
