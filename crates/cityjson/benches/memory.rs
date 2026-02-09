//! Memory-focused benchmarks that capture heap usage with dhat.

#[allow(dead_code)]
mod support;

use rand::Rng;
use std::hint::black_box;
use support::{BenchParams, DEFAULT_SIZE_MEMORY, FAST_SIZE_MEMORY, params_from_env, rng_from_seed};

// Enable dhat heap profiling for the entire benchmark
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

mod benches {
    use super::{rng_from_seed, Rng, BenchParams, black_box};

    use cityjson::backend::default::geometry::GeometryBuilder;
    use cityjson::prelude::*;
    use cityjson::v2_0::{CityModel, Material, Texture, CityObject, CityObjectType, Semantic, SemanticType};
    use std::collections::HashMap;

    /// Build a `CityModel` with the specified vertex index type and number of cityobjects.
    /// Each cityobject will have a solid geometry with 8 vertices (a cube).
    fn build_model<VR: VertexRef>(
        n_cityobjects: usize,
        seed: u64,
    ) -> CityModel<VR, OwnedStringStorage> {
        let mut model = CityModel::<VR, OwnedStringStorage>::new(CityModelType::CityJSON);
        let mut rng = rng_from_seed(seed);

        // Set basic metadata
        let metadata = model.metadata_mut();
        metadata.set_identifier(CityModelIdentifier::new("memory-benchmark".to_string()));
        metadata.set_reference_system(CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
        ));

        let mut material = Material::new("benchmark_material".to_string());
        material.set_ambient_intensity(Some(0.5));
        material.set_diffuse_color(Some([0.8, 0.8, 0.8].into()));
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
            let mut cityobject = CityObject::new(
                CityObjectIdentifier::new(format!("building-{i:06}")),
                CityObjectType::Building,
            );

            let attrs = cityobject.attributes_mut();
            let height = 10.0 + (i as f64) * 0.5 + (seed as f64) * 0.001;
            attrs.insert("attr_null".to_string(), AttributeValue::Null);
            attrs.insert("attr_bool".to_string(), AttributeValue::Bool(i % 2 == 0));
            attrs.insert(
                "attr_unsigned".to_string(),
                AttributeValue::Unsigned(i as u64),
            );
            attrs.insert(
                "attr_integer".to_string(),
                AttributeValue::Integer(i as i64),
            );
            attrs.insert("attr_float".to_string(), AttributeValue::Float(height));
            attrs.insert(
                "attr_string".to_string(),
                AttributeValue::String(format!("name-{i}")),
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
                cityobject.add_geometry(GeometryRef::from_parts(
                    geometry_ref.index(),
                    geometry_ref.generation(),
                ));
            }

            // Add the CityObject to the model
            model
                .cityobjects_mut()
                .add(cityobject)
                .expect("failed to add cityobject to model");
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

fn main() {
    let params = params_from_env(DEFAULT_SIZE_MEMORY, FAST_SIZE_MEMORY);
    benches::run(params);
}
