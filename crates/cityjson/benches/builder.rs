//! Benchmarks for building CityObjects with and without geometries
//!
//! This benchmark tests the performance of building CityModels with complex objects including
//! attributes, geometries with semantics, materials, and textures.
//!
//! ## Running Benchmarks
//!
//! Run with specific backend:
//! ```bash
//! # Default backend (flattened representation)
//! cargo bench --bench builder --features backend-default
//!
//! # Nested backend (JSON-like representation)
//! cargo bench --bench builder --features backend-nested
//!
//! # Both backends (for comparison)
//! cargo bench --bench builder --features backend-both
//! ```

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

// ==================== DEFAULT BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-default")]
mod default_benches {
    use super::*;
    use cityjson::backend::default::*;
    use cityjson::cityjson::core::attributes::{AttributeOwnerType, OwnedAttributePool};
    use cityjson::prelude::*;
    use cityjson::v2_0::*;
    use std::collections::HashMap;

    /// Helper function to build a geometry with semantics, materials, and textures.
    fn build_geometry_with_semantics_materials_textures(
        model: &mut CityModel<u32, ResourceId32, OwnedStringStorage>,
        pool: &mut OwnedAttributePool,
        vertices: &[VertexIndex32],
        index: usize,
        material_data: Option<&(Material<OwnedStringStorage>, ResourceId32)>,
        texture_data: Option<&(Texture<OwnedStringStorage>, ResourceId32)>,
    ) -> Result<ResourceId32> {
        let mut geometry_builder =
            GeometryBuilder::new(model, GeometryType::Solid, BuilderMode::Regular)
                .with_lod(LoD::LoD2_2);

        let bv0 = geometry_builder.add_vertex(vertices[0]);
        let bv1 = geometry_builder.add_vertex(vertices[1]);
        let bv2 = geometry_builder.add_vertex(vertices[2]);
        let bv3 = geometry_builder.add_vertex(vertices[3]);
        let bv4 = geometry_builder.add_vertex(vertices[4]);
        let bv5 = geometry_builder.add_vertex(vertices[5]);
        let bv6 = geometry_builder.add_vertex(vertices[6]);
        let bv7 = geometry_builder.add_vertex(vertices[7]);

        // Bottom surface
        let ring_bottom = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
        let surface_bottom = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_bottom)?;
        let mut ground_semantic = Semantic::new(SemanticType::GroundSurface);
        let ground_attrs = ground_semantic.attributes_mut();
        let area_id = pool.add_float(
            "area".to_string(),
            true,
            100.0 + (index as f64) * 0.5,
            AttributeOwnerType::Semantic,
            None,
        );
        ground_attrs.insert("area".to_string(), area_id);
        geometry_builder.set_semantic_surface(None, ground_semantic, false)?;

        // Top surface (Roof)
        let ring_top = geometry_builder.add_ring(&[bv4, bv5, bv6, bv7])?;
        let surface_top = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_top)?;
        let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
        let roof_attrs = roof_semantic.attributes_mut();
        let azimuth_id = pool.add_float(
            "azimuth".to_string(),
            true,
            (index % 360) as f64,
            AttributeOwnerType::Semantic,
            None,
        );
        let slope_id = pool.add_float(
            "slope".to_string(),
            true,
            15.0 + ((index % 30) as f64),
            AttributeOwnerType::Semantic,
            None,
        );
        let roof_area_id = pool.add_float(
            "area".to_string(),
            true,
            200.0 + (index as f64) * 1.2,
            AttributeOwnerType::Semantic,
            None,
        );
        roof_attrs.insert("azimuth".to_string(), azimuth_id);
        roof_attrs.insert("slope".to_string(), slope_id);
        roof_attrs.insert("area".to_string(), roof_area_id);
        geometry_builder.set_semantic_surface(None, roof_semantic, false)?;

        if let Some((material, _mat_ref)) = material_data {
            geometry_builder.set_material_surface(
                None,
                material.clone(),
                "default".to_string(),
                true,
            )?;
        }

        // Front wall
        let ring_front = geometry_builder.add_ring(&[bv0, bv1, bv5, bv4])?;
        let surface_front = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_front)?;
        let mut wall_north = Semantic::new(SemanticType::WallSurface);
        let wall_north_attrs = wall_north.attributes_mut();
        let orientation_n_id = pool.add_string(
            "orientation".to_string(),
            true,
            "north".to_string(),
            AttributeOwnerType::Semantic,
            None,
        );
        wall_north_attrs.insert("orientation".to_string(), orientation_n_id);
        geometry_builder.set_semantic_surface(None, wall_north, false)?;

        if let Some((texture, _tex_ref)) = texture_data {
            let uv0 = geometry_builder.add_uv_coordinate(0.0, 0.0);
            let uv1 = geometry_builder.add_uv_coordinate(1.0, 0.0);
            let uv2 = geometry_builder.add_uv_coordinate(1.0, 1.0);
            let uv3 = geometry_builder.add_uv_coordinate(0.0, 1.0);
            geometry_builder.map_vertex_to_uv(bv0, uv0);
            geometry_builder.map_vertex_to_uv(bv1, uv1);
            geometry_builder.map_vertex_to_uv(bv5, uv2);
            geometry_builder.map_vertex_to_uv(bv4, uv3);
            geometry_builder.set_texture_ring(
                None,
                texture.clone(),
                "default".to_string(),
                true,
            )?;
        }

        // Back, left, right walls (simplified)
        let ring_back = geometry_builder.add_ring(&[bv2, bv3, bv7, bv6])?;
        geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_back)?;

        let ring_left = geometry_builder.add_ring(&[bv0, bv4, bv7, bv3])?;
        geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_left)?;

        let ring_right = geometry_builder.add_ring(&[bv1, bv2, bv6, bv5])?;
        geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_right)?;

        let shell_surfaces = vec![
            surface_bottom,
            surface_top,
            surface_front,
            surface_front + 1,
            surface_front + 2,
            surface_front + 3,
        ];
        geometry_builder.add_shell(&shell_surfaces)?;

        let geometry_ref = geometry_builder.build()?;
        Ok(geometry_ref)
    }

    pub fn build_cityobjects(config: (usize, bool)) -> Result<Vec<ResourceId32>> {
        let num_cityobjects = config.0;
        let with_geometries = config.1;
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let mut cityobject_refs = Vec::with_capacity(num_cityobjects);
        let mut pool = OwnedAttributePool::new();

        let (material_ref, texture_ref) = if with_geometries {
            let mut material = Material::new("benchmark_material".to_string());
            material.set_ambient_intensity(Some(0.5));
            material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
            let mat_ref = model.add_material(material.clone());
            let texture = Texture::new("benchmark_texture.png".to_string(), ImageType::Png);
            let tex_ref = model.add_texture(texture.clone());
            (Some((material, mat_ref)), Some((texture, tex_ref)))
        } else {
            (None, None)
        };

        let vertices = if with_geometries {
            vec![
                model.add_vertex(QuantizedCoordinate::new(0, 0, 0))?,
                model.add_vertex(QuantizedCoordinate::new(1000, 0, 0))?,
                model.add_vertex(QuantizedCoordinate::new(1000, 1000, 0))?,
                model.add_vertex(QuantizedCoordinate::new(0, 1000, 0))?,
                model.add_vertex(QuantizedCoordinate::new(0, 0, 500))?,
                model.add_vertex(QuantizedCoordinate::new(1000, 0, 500))?,
                model.add_vertex(QuantizedCoordinate::new(1000, 1000, 500))?,
                model.add_vertex(QuantizedCoordinate::new(0, 1000, 500))?,
            ]
        } else {
            Vec::new()
        };

        for i in 0..num_cityobjects {
            let co_id = format!("cityobject-{}", i);
            let co_type = match i % 5 {
                0 => CityObjectType::Building,
                1 => CityObjectType::BuildingPart,
                2 => CityObjectType::Road,
                3 => CityObjectType::PlantCover,
                _ => CityObjectType::GenericCityObject,
            };

            let mut cityobject = CityObject::new(co_id.clone(), co_type);

            let attrs = cityobject.attributes_mut();
            let measured_height_id = pool.add_float(
                "measuredHeight".to_string(),
                true,
                10.0 + (i as f64) * 0.5,
                AttributeOwnerType::CityObject,
                None,
            );
            attrs.insert("measuredHeight".to_string(), measured_height_id);

            let offset = (i as f64) * 100.0;
            cityobject.set_geographical_extent(Some(BBox::new(
                offset,
                offset,
                0.0,
                offset + 50.0,
                offset + 50.0,
                20.0,
            )));

            if with_geometries {
                let geometry_ref = build_geometry_with_semantics_materials_textures(
                    &mut model,
                    &mut pool,
                    &vertices,
                    i,
                    material_ref.as_ref(),
                    texture_ref.as_ref(),
                )?;
                cityobject.geometry_mut().push(geometry_ref);
            }

            let co_ref = model.cityobjects_mut().add(cityobject);
            cityobject_refs.push(co_ref);
        }

        Ok(cityobject_refs)
    }

    pub fn bench_build_without_geometry(c: &mut Criterion) {
        let mut group = c.benchmark_group("builder");
        let nr_cityobjects = 10_000_usize;
        group.throughput(Throughput::Elements(nr_cityobjects as u64));

        group.bench_function("default/build_without_geometry", |b| {
            b.iter(|| {
                let refs = build_cityobjects(black_box((nr_cityobjects, false)))
                    .expect("cityobjects builder failed");
                black_box(refs);
            });
        });

        group.finish();
    }

    pub fn bench_build_with_geometry(c: &mut Criterion) {
        let mut group = c.benchmark_group("builder");
        let nr_cityobjects = 10_000_usize;
        group.throughput(Throughput::Elements(nr_cityobjects as u64));

        group.bench_function("default/build_with_geometry", |b| {
            b.iter(|| {
                let refs = build_cityobjects(black_box((nr_cityobjects, true)))
                    .expect("cityobjects builder failed");
                black_box(refs);
            });
        });

        group.finish();
    }
}

// ==================== NESTED BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-nested")]
mod nested_benches {
    use super::*;
    use cityjson::backend::nested;
    use cityjson::prelude::*;
    use std::collections::HashMap;

    /// Helper function to build a geometry with semantics (simplified for nested backend).
    fn build_geometry_with_semantics(
        model: &mut nested::CityModel<OwnedStringStorage>,
        vertices: &[VertexIndex32],
        index: usize,
    ) -> Result<nested::Geometry<OwnedStringStorage>> {
        let mut geometry_builder =
            nested::GeometryBuilder::new(model, GeometryType::Solid, nested::BuilderMode::Regular)
                .with_lod(LoD::LoD2_2);

        // Add vertices
        for &v in vertices {
            geometry_builder.add_vertex(v)?;
        }

        // Bottom surface
        let ring_bottom = geometry_builder.add_ring(&[0, 3, 2, 1])?;
        geometry_builder.start_surface()?;
        geometry_builder.add_surface_outer_ring(ring_bottom)?;
        let mut ground_attrs = HashMap::new();
        ground_attrs.insert(
            "area".to_string(),
            nested::AttributeValue::Float(100.0 + (index as f64) * 0.5),
        );
        let ground_semantic = nested::Semantic::new_with_attributes(
            nested::semantics::SemanticType::GroundSurface,
            ground_attrs,
        );
        geometry_builder.set_semantic_surface(None, ground_semantic)?;
        let surface_bottom = geometry_builder.end_surface()?;

        // Top surface (Roof)
        let ring_top = geometry_builder.add_ring(&[4, 5, 6, 7])?;
        geometry_builder.start_surface()?;
        geometry_builder.add_surface_outer_ring(ring_top)?;
        let mut roof_attrs = HashMap::new();
        roof_attrs.insert(
            "azimuth".to_string(),
            nested::AttributeValue::Float((index % 360) as f64),
        );
        roof_attrs.insert(
            "slope".to_string(),
            nested::AttributeValue::Float(15.0 + ((index % 30) as f64)),
        );
        let roof_semantic = nested::Semantic::new_with_attributes(
            nested::semantics::SemanticType::RoofSurface,
            roof_attrs,
        );
        geometry_builder.set_semantic_surface(None, roof_semantic)?;
        let surface_top = geometry_builder.end_surface()?;

        // Front wall
        let ring_front = geometry_builder.add_ring(&[0, 1, 5, 4])?;
        geometry_builder.start_surface()?;
        geometry_builder.add_surface_outer_ring(ring_front)?;
        let mut wall_attrs = HashMap::new();
        wall_attrs.insert(
            "orientation".to_string(),
            nested::AttributeValue::String("north".to_string()),
        );
        let wall_semantic = nested::Semantic::new_with_attributes(
            nested::semantics::SemanticType::WallSurface,
            wall_attrs,
        );
        geometry_builder.set_semantic_surface(None, wall_semantic)?;
        let surface_front = geometry_builder.end_surface()?;

        // Back, left, right walls (simplified, no semantics)
        let ring_back = geometry_builder.add_ring(&[2, 3, 7, 6])?;
        geometry_builder.start_surface()?;
        geometry_builder.add_surface_outer_ring(ring_back)?;
        let surface_back = geometry_builder.end_surface()?;

        let ring_left = geometry_builder.add_ring(&[0, 4, 7, 3])?;
        geometry_builder.start_surface()?;
        geometry_builder.add_surface_outer_ring(ring_left)?;
        let surface_left = geometry_builder.end_surface()?;

        let ring_right = geometry_builder.add_ring(&[1, 2, 6, 5])?;
        geometry_builder.start_surface()?;
        geometry_builder.add_surface_outer_ring(ring_right)?;
        let surface_right = geometry_builder.end_surface()?;

        // Create shell
        geometry_builder.start_shell()?;
        geometry_builder.add_shell_surface(surface_bottom)?;
        geometry_builder.add_shell_surface(surface_top)?;
        geometry_builder.add_shell_surface(surface_front)?;
        geometry_builder.add_shell_surface(surface_back)?;
        geometry_builder.add_shell_surface(surface_left)?;
        geometry_builder.add_shell_surface(surface_right)?;
        geometry_builder.end_shell()?;

        let geometry = geometry_builder.build()?;
        Ok(geometry)
    }

    pub fn build_cityobjects(config: (usize, bool)) -> Result<()> {
        let num_cityobjects = config.0;
        let with_geometries = config.1;
        let mut model = nested::CityModel::<OwnedStringStorage>::new(CityModelType::CityJSON);

        let vertices = if with_geometries {
            vec![
                model.add_vertex(QuantizedCoordinate::new(0, 0, 0))?,
                model.add_vertex(QuantizedCoordinate::new(1000, 0, 0))?,
                model.add_vertex(QuantizedCoordinate::new(1000, 1000, 0))?,
                model.add_vertex(QuantizedCoordinate::new(0, 1000, 0))?,
                model.add_vertex(QuantizedCoordinate::new(0, 0, 500))?,
                model.add_vertex(QuantizedCoordinate::new(1000, 0, 500))?,
                model.add_vertex(QuantizedCoordinate::new(1000, 1000, 500))?,
                model.add_vertex(QuantizedCoordinate::new(0, 1000, 500))?,
            ]
        } else {
            Vec::new()
        };

        for i in 0..num_cityobjects {
            let co_id = format!("cityobject-{}", i);
            let co_type = match i % 5 {
                0 => nested::cityobject::CityObjectType::Building,
                1 => nested::cityobject::CityObjectType::BuildingPart,
                2 => nested::cityobject::CityObjectType::Road,
                3 => nested::cityobject::CityObjectType::PlantCover,
                _ => nested::cityobject::CityObjectType::GenericCityObject,
            };

            let mut cityobject = nested::CityObject::new(co_type);

            // Add attributes using nested backend's inline AttributeValue
            let mut attrs = HashMap::new();
            attrs.insert(
                "measuredHeight".to_string(),
                nested::AttributeValue::Float(10.0 + (i as f64) * 0.5),
            );
            cityobject.set_attributes(Some(attrs));

            let offset = (i as f64) * 100.0;
            cityobject.set_geographical_extent(Some(BBox::new(
                offset,
                offset,
                0.0,
                offset + 50.0,
                offset + 50.0,
                20.0,
            )));

            model.add_cityobject(co_id.clone(), cityobject);

            if with_geometries {
                let geometry = build_geometry_with_semantics(&mut model, &vertices, i)?;
                model.add_geometry_to_cityobject(&co_id, geometry)?;
            }
        }

        Ok(())
    }

    pub fn bench_build_without_geometry(c: &mut Criterion) {
        let mut group = c.benchmark_group("builder");
        let nr_cityobjects = 10_000_usize;
        group.throughput(Throughput::Elements(nr_cityobjects as u64));

        group.bench_function("nested/build_without_geometry", |b| {
            b.iter(|| {
                build_cityobjects(black_box((nr_cityobjects, false)))
                    .expect("cityobjects builder failed");
            });
        });

        group.finish();
    }

    pub fn bench_build_with_geometry(c: &mut Criterion) {
        let mut group = c.benchmark_group("builder");
        let nr_cityobjects = 10_000_usize;
        group.throughput(Throughput::Elements(nr_cityobjects as u64));

        group.bench_function("nested/build_with_geometry", |b| {
            b.iter(|| {
                build_cityobjects(black_box((nr_cityobjects, true)))
                    .expect("cityobjects builder failed");
            });
        });

        group.finish();
    }
}

// ==================== CRITERION GROUPS ====================

#[cfg(all(feature = "backend-default", not(feature = "backend-nested")))]
criterion_group!(
    benches,
    default_benches::bench_build_without_geometry,
    default_benches::bench_build_with_geometry
);

#[cfg(all(feature = "backend-nested", not(feature = "backend-default")))]
criterion_group!(
    benches,
    nested_benches::bench_build_without_geometry,
    nested_benches::bench_build_with_geometry
);

#[cfg(all(feature = "backend-default", feature = "backend-nested"))]
criterion_group!(
    benches,
    default_benches::bench_build_without_geometry,
    default_benches::bench_build_with_geometry,
    nested_benches::bench_build_without_geometry,
    nested_benches::bench_build_with_geometry
);

criterion_main!(benches);
