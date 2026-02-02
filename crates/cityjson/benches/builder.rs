//! Benchmarks for building CityObjects with and without geometries.

#[allow(dead_code)]
mod support;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use support::{CUBE_VERTICES, DEFAULT_SIZE_BUILDER, FAST_SIZE_BUILDER, params_from_env};

// ==================== DEFAULT BACKEND BENCHMARKS ====================

#[cfg(feature = "backend-default")]
mod default_benches {
    use super::*;

    use cityjson::prelude::*;
    use cityjson::v2_0::*;
    use std::collections::HashMap;

    /// Helper function to build a geometry with semantics, materials, and textures.
    fn build_geometry_with_semantics_materials_textures(
        model: &mut CityModel<u32, ResourceId32, OwnedStringStorage>,
        vertices: &[VertexIndex32],
        index: usize,
        material_data: Option<&(Material<OwnedStringStorage>, ResourceId32)>,
        texture_data: Option<&(Texture<OwnedStringStorage>, ResourceId32)>,
    ) -> Result<ResourceId32> {
        // Create semantic attributes using inline API
        let mut ground_semantic = Semantic::new(SemanticType::GroundSurface);
        let ground_attrs = ground_semantic.attributes_mut();
        ground_attrs.insert(
            "area".to_string(),
            AttributeValue::Float(100.0 + (index as f64) * 0.5),
        );

        let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
        let roof_attrs = roof_semantic.attributes_mut();
        roof_attrs.insert(
            "azimuth".to_string(),
            AttributeValue::Float((index % 360) as f64),
        );
        roof_attrs.insert(
            "slope".to_string(),
            AttributeValue::Float(15.0 + ((index % 30) as f64)),
        );

        let mut wall_north = Semantic::new(SemanticType::WallSurface);
        let wall_attrs = wall_north.attributes_mut();
        wall_attrs.insert(
            "orientation".to_string(),
            AttributeValue::String("north".to_string()),
        );

        // Now create the GeometryBuilder
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
        geometry_builder.set_semantic_surface(None, ground_semantic, false)?;

        // Top surface (Roof)
        let ring_top = geometry_builder.add_ring(&[bv4, bv5, bv6, bv7])?;
        let surface_top = geometry_builder.start_surface();
        geometry_builder.add_surface_outer_ring(ring_top)?;
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

    pub fn build_cityobjects(
        num_cityobjects: usize,
        with_geometries: bool,
        seed: u64,
    ) -> Result<Vec<ResourceId32>> {
        let mut model =
            CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let mut cityobject_refs = Vec::with_capacity(num_cityobjects);

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
            CUBE_VERTICES
                .iter()
                .map(|(x, y, z)| model.add_vertex(QuantizedCoordinate::new(*x, *y, *z)))
                .collect::<Result<Vec<_>>>()?
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

        let seed_offset = (seed as f64) * 0.001;

            let offset = (i as f64) * 100.0;
            cityobject.set_geographical_extent(Some(BBox::new(
                offset + seed_offset,
                offset + seed_offset,
                0.0,
                offset + 50.0 + seed_offset,
                offset + 50.0 + seed_offset,
                20.0,
            )));

            if with_geometries {
                let geometry_ref = build_geometry_with_semantics_materials_textures(
                    &mut model,
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
        let params = params_from_env(DEFAULT_SIZE_BUILDER, FAST_SIZE_BUILDER);
        let mut group = c.benchmark_group("builder");
        let nr_cityobjects = params.size;
        group.throughput(Throughput::Elements(nr_cityobjects as u64));

        group.bench_function("build_without_geometry", |b| {
            b.iter(|| {
                let refs = build_cityobjects(black_box(nr_cityobjects), false, params.seed)
                    .expect("cityobjects builder failed");
                black_box(refs);
            });
        });

        group.finish();
    }

    pub fn bench_build_with_geometry(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_BUILDER, FAST_SIZE_BUILDER);
        let mut group = c.benchmark_group("builder");
        let nr_cityobjects = params.size;
        group.throughput(Throughput::Elements(nr_cityobjects as u64));

        group.bench_function("build_with_geometry", |b| {
            b.iter(|| {
                let refs = build_cityobjects(black_box(nr_cityobjects), true, params.seed)
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
    use cityjson::backend::nested::appearance::{ImageType, Material, Texture};
    use cityjson::backend::nested::attributes::AttributeValue;
    use cityjson::backend::nested::semantics::Semantic;
    use cityjson::prelude::*;
    use std::collections::HashMap;

    /// Helper function to build a geometry with semantics (simplified for nested backend).
    fn build_geometry_with_semantics_materials_textures(
        model: &mut nested::CityModel<OwnedStringStorage, ResourceId32>,
        vertices: &[VertexIndex32],
        index: usize,
        material_idx: Option<usize>,
        texture_idx: Option<usize>,
    ) -> Result<nested::Geometry<OwnedStringStorage, ResourceId32>> {
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
        let mut ground_semantic = Semantic::new(nested::semantics::SemanticType::GroundSurface);
        let ground_attrs = ground_semantic.attributes_mut();
        ground_attrs.insert(
            "area".to_string(),
            AttributeValue::Float(100.0 + (index as f64) * 0.5),
        );
        geometry_builder.set_semantic_surface(0, ground_semantic, false)?;
        let surface_bottom = geometry_builder.end_surface()?;

        // Top surface (Roof)
        let ring_top = geometry_builder.add_ring(&[4, 5, 6, 7])?;
        geometry_builder.start_surface()?;
        geometry_builder.add_surface_outer_ring(ring_top)?;
        let mut roof_semantic = Semantic::new(nested::semantics::SemanticType::RoofSurface);
        let roof_attrs = roof_semantic.attributes_mut();
        roof_attrs.insert(
            "azimuth".to_string(),
            AttributeValue::Float((index % 360) as f64),
        );
        roof_attrs.insert(
            "slope".to_string(),
            AttributeValue::Float(15.0 + ((index % 30) as f64)),
        );
        geometry_builder.set_semantic_surface(1, roof_semantic, true)?;
        let surface_top = geometry_builder.end_surface()?;

        if let Some(material_idx) = material_idx {
            geometry_builder.set_material_surface(
                "default".to_string(),
                surface_top,
                material_idx,
            )?;
        }

        // Front wall
        let ring_front = geometry_builder.add_ring(&[0, 1, 5, 4])?;
        geometry_builder.start_surface()?;
        geometry_builder.add_surface_outer_ring(ring_front)?;
        let mut wall_semantic = Semantic::new(nested::semantics::SemanticType::WallSurface);
        let wall_attrs = wall_semantic.attributes_mut();
        wall_attrs.insert(
            "orientation".to_string(),
            AttributeValue::String("north".to_string()),
        );
        geometry_builder.set_semantic_surface(2, wall_semantic, false)?;
        let surface_front = geometry_builder.end_surface()?;

        if let Some(texture_idx) = texture_idx {
            geometry_builder.add_uv_to_vertex(0, UVCoordinate::new(0.0, 0.0))?;
            geometry_builder.add_uv_to_vertex(1, UVCoordinate::new(1.0, 0.0))?;
            geometry_builder.add_uv_to_vertex(5, UVCoordinate::new(1.0, 1.0))?;
            geometry_builder.add_uv_to_vertex(4, UVCoordinate::new(0.0, 1.0))?;
            geometry_builder.set_texture_ring(
                "default".to_string(),
                ring_front,
                texture_idx,
            )?;
        }

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

    pub fn build_cityobjects(num_cityobjects: usize, with_geometries: bool, seed: u64) -> Result<()> {
        let mut model = nested::CityModel::<OwnedStringStorage, ResourceId32>::new(CityModelType::CityJSON);

        let (material_idx, texture_idx) = if with_geometries {
            let mut material = Material::new("benchmark_material".to_string());
            material.set_ambient_intensity(Some(0.5));
            material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
            let mat_idx = model.add_material(material);
            let texture = Texture::new("benchmark_texture.png".to_string(), ImageType::Png);
            let tex_idx = model.add_texture(texture);
            model.set_default_theme_material(Some("default".to_string()));
            model.set_default_theme_texture(Some("default".to_string()));
            (Some(mat_idx), Some(tex_idx))
        } else {
            (None, None)
        };

        let vertices = if with_geometries {
            CUBE_VERTICES
                .iter()
                .map(|(x, y, z)| model.add_vertex(QuantizedCoordinate::new(*x, *y, *z)))
                .collect::<Result<Vec<_>>>()?
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

            let seed_offset = (seed as f64) * 0.001;
            let offset = (i as f64) * 100.0;
            cityobject.set_geographical_extent(Some(nested::metadata::BBox::new(
                offset + seed_offset,
                offset + seed_offset,
                0.0,
                offset + 50.0 + seed_offset,
                offset + 50.0 + seed_offset,
                20.0,
            )));

            model.add_cityobject(co_id.clone(), cityobject);

            if with_geometries {
                let geometry = build_geometry_with_semantics_materials_textures(
                    &mut model,
                    &vertices,
                    i,
                    material_idx,
                    texture_idx,
                )?;
                model.add_geometry_to_cityobject(&co_id, geometry)?;
            }
        }

        Ok(())
    }

    pub fn bench_build_without_geometry(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_BUILDER, FAST_SIZE_BUILDER);
        let mut group = c.benchmark_group("builder");
        let nr_cityobjects = params.size;
        group.throughput(Throughput::Elements(nr_cityobjects as u64));

        group.bench_function("build_without_geometry", |b| {
            b.iter(|| {
                build_cityobjects(black_box(nr_cityobjects), false, params.seed)
                    .expect("cityobjects builder failed");
            });
        });

        group.finish();
    }

    pub fn bench_build_with_geometry(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_BUILDER, FAST_SIZE_BUILDER);
        let mut group = c.benchmark_group("builder");
        let nr_cityobjects = params.size;
        group.throughput(Throughput::Elements(nr_cityobjects as u64));

        group.bench_function("build_with_geometry", |b| {
            b.iter(|| {
                build_cityobjects(black_box(nr_cityobjects), true, params.seed)
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
