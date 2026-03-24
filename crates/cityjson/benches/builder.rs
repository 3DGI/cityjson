//! Benchmarks for building `CityObjects` with minimal and full-feature geometries.

#[allow(dead_code)]
mod support;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use support::{CUBE_VERTICES, DEFAULT_SIZE_BUILDER, FAST_SIZE_BUILDER, params_from_env};

mod benches {
    use super::{
        CUBE_VERTICES, Criterion, DEFAULT_SIZE_BUILDER, FAST_SIZE_BUILDER, Throughput, black_box,
        params_from_env,
    };

    use cityjson::error::Result;
    use cityjson::resources::storage::OwnedStringStorage;
    use cityjson::resources::{CityObjectHandle, GeometryHandle, MaterialHandle, TextureHandle};
    use cityjson::v2_0::{
        AttributeValue, BBox, CityModel, CityModelType, CityObject, CityObjectIdentifier,
        CityObjectType, GeometryDraft, ImageType, LoD, Material, RealWorldCoordinate, RingDraft,
        Semantic, SemanticType, ShellDraft, SurfaceDraft, Texture, UVCoordinate, VertexIndex32,
    };
    use std::collections::HashMap;

    fn build_geometry_minimal(
        model: &mut CityModel<u32, OwnedStringStorage>,
        vertices: &[VertexIndex32],
    ) -> Result<GeometryHandle> {
        let surface_bottom = SurfaceDraft::new(
            RingDraft::new([vertices[0], vertices[1], vertices[2], vertices[3]]),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        );
        let surface_top = SurfaceDraft::new(
            RingDraft::new([vertices[4], vertices[7], vertices[6], vertices[5]]),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        );
        let surface_front = SurfaceDraft::new(
            RingDraft::new([vertices[0], vertices[1], vertices[5], vertices[4]]),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        );
        let surface_right = SurfaceDraft::new(
            RingDraft::new([vertices[1], vertices[2], vertices[6], vertices[5]]),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        );
        let surface_back = SurfaceDraft::new(
            RingDraft::new([vertices[2], vertices[3], vertices[7], vertices[6]]),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        );
        let surface_left = SurfaceDraft::new(
            RingDraft::new([vertices[3], vertices[0], vertices[4], vertices[7]]),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        );
        let shell = ShellDraft::new([
            surface_bottom,
            surface_top,
            surface_front,
            surface_right,
            surface_back,
            surface_left,
        ]);
        GeometryDraft::solid(
            Some(LoD::LoD2),
            shell,
            std::iter::empty::<ShellDraft<u32, OwnedStringStorage>>(),
        )
        .insert_into(model)
    }

    /// Helper function to build a geometry with semantics, materials, and textures.
    fn build_geometry_full_feature(
        model: &mut CityModel<u32, OwnedStringStorage>,
        vertices: &[VertexIndex32],
        index: u32,
        material_ref: MaterialHandle,
        texture_ref: TextureHandle,
    ) -> Result<GeometryHandle> {
        let mut ground_semantic = Semantic::new(SemanticType::GroundSurface);
        let ground_attrs = ground_semantic.attributes_mut();
        ground_attrs.insert(
            "area".to_string(),
            AttributeValue::Float(100.0 + f64::from(index) * 0.5),
        );

        let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
        let roof_attrs = roof_semantic.attributes_mut();
        roof_attrs.insert(
            "azimuth".to_string(),
            AttributeValue::Float(f64::from(index % 360)),
        );
        roof_attrs.insert(
            "slope".to_string(),
            AttributeValue::Float(15.0 + f64::from(index % 30)),
        );

        let mut wall_north = Semantic::new(SemanticType::WallSurface);
        let wall_attrs = wall_north.attributes_mut();
        wall_attrs.insert(
            "orientation".to_string(),
            AttributeValue::String("north".to_string()),
        );

        let ground_semantic = model.add_semantic(ground_semantic)?;
        let roof_semantic = model.add_semantic(roof_semantic)?;
        let wall_north = model.add_semantic(wall_north)?;

        let uv0 = model.add_uv_coordinate(UVCoordinate::new(0.0, 0.0))?;
        let uv1 = model.add_uv_coordinate(UVCoordinate::new(1.0, 0.0))?;
        let uv2 = model.add_uv_coordinate(UVCoordinate::new(1.0, 1.0))?;
        let uv3 = model.add_uv_coordinate(UVCoordinate::new(0.0, 1.0))?;

        let surface_bottom = SurfaceDraft::new(
            RingDraft::new([vertices[0], vertices[3], vertices[2], vertices[1]]),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        )
        .with_semantic(ground_semantic);
        let surface_top = SurfaceDraft::new(
            RingDraft::new([vertices[4], vertices[5], vertices[6], vertices[7]]),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        )
        .with_semantic(roof_semantic)
        .with_material("default".to_string(), material_ref);
        let surface_front = SurfaceDraft::new(
            RingDraft::new([vertices[0], vertices[1], vertices[5], vertices[4]]).with_texture(
                "default".to_string(),
                texture_ref,
                [uv0, uv1, uv2, uv3],
            ),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        )
        .with_semantic(wall_north);
        let surface_back = SurfaceDraft::new(
            RingDraft::new([vertices[2], vertices[3], vertices[7], vertices[6]]),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        );
        let surface_left = SurfaceDraft::new(
            RingDraft::new([vertices[0], vertices[4], vertices[7], vertices[3]]),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        );
        let surface_right = SurfaceDraft::new(
            RingDraft::new([vertices[1], vertices[2], vertices[6], vertices[5]]),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        );

        let shell = ShellDraft::new([
            surface_bottom,
            surface_top,
            surface_front,
            surface_back,
            surface_left,
            surface_right,
        ]);
        GeometryDraft::solid(
            Some(LoD::LoD2_2),
            shell,
            std::iter::empty::<ShellDraft<u32, OwnedStringStorage>>(),
        )
        .insert_into(model)
    }

    pub fn build_cityobjects_minimal(num_cityobjects: usize) -> Result<Vec<CityObjectHandle>> {
        let mut model = CityModel::<u32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let mut cityobject_refs = Vec::with_capacity(num_cityobjects);

        let vertices = CUBE_VERTICES
            .iter()
            .map(|(x, y, z)| model.add_vertex(RealWorldCoordinate::new(*x, *y, *z)))
            .collect::<Result<Vec<_>>>()?;

        for i in 0..num_cityobjects {
            let co_id = format!("cityobject-{i}");
            let co_type = match i % 5 {
                0 => CityObjectType::Building,
                1 => CityObjectType::BuildingPart,
                2 => CityObjectType::Road,
                3 => CityObjectType::PlantCover,
                _ => CityObjectType::GenericCityObject,
            };

            let mut cityobject = CityObject::new(CityObjectIdentifier::new(co_id.clone()), co_type);
            let geometry_ref = build_geometry_minimal(&mut model, &vertices)?;
            cityobject.add_geometry(geometry_ref);

            let co_ref = model.cityobjects_mut().add(cityobject)?;
            cityobject_refs.push(co_ref);
        }

        Ok(cityobject_refs)
    }

    pub fn build_cityobjects_full(
        num_cityobjects: usize,
        seed: u64,
    ) -> Result<Vec<CityObjectHandle>> {
        let mut model = CityModel::<u32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let mut cityobject_refs = Vec::with_capacity(num_cityobjects);
        let seed_u32 = u32::try_from(seed).expect("seed exceeds u32 range");

        let mut material = Material::new("benchmark_material".to_string());
        material.set_ambient_intensity(Some(0.5));
        material.set_diffuse_color(Some([0.8, 0.8, 0.8].into()));
        let mat_ref = model.add_material(material)?;
        let texture = Texture::new("benchmark_texture.png".to_string(), ImageType::Png);
        let tex_ref = model.add_texture(texture)?;

        let vertices = CUBE_VERTICES
            .iter()
            .map(|(x, y, z)| model.add_vertex(RealWorldCoordinate::new(*x, *y, *z)))
            .collect::<Result<Vec<_>>>()?;

        for i in 0..num_cityobjects {
            let index_u32 = u32::try_from(i).expect("cityobject index exceeds u32 range");
            let index_signed = i64::from(index_u32);
            let co_id = format!("cityobject-{i}");
            let co_type = match i % 5 {
                0 => CityObjectType::Building,
                1 => CityObjectType::BuildingPart,
                2 => CityObjectType::Road,
                3 => CityObjectType::PlantCover,
                _ => CityObjectType::GenericCityObject,
            };

            let mut cityobject = CityObject::new(CityObjectIdentifier::new(co_id.clone()), co_type);

            let attrs = cityobject.attributes_mut();
            let height = 10.0 + f64::from(index_u32) * 0.5 + f64::from(seed_u32) * 0.001;
            attrs.insert("attr_null".to_string(), AttributeValue::Null);
            attrs.insert("attr_bool".to_string(), AttributeValue::Bool(i % 2 == 0));
            attrs.insert(
                "attr_unsigned".to_string(),
                AttributeValue::Unsigned(u64::from(index_u32)),
            );
            attrs.insert(
                "attr_integer".to_string(),
                AttributeValue::Integer(index_signed),
            );
            attrs.insert("attr_float".to_string(), AttributeValue::Float(height));
            attrs.insert(
                "attr_string".to_string(),
                AttributeValue::String(format!("name-{i}")),
            );
            attrs.insert(
                "attr_vec".to_string(),
                AttributeValue::Vec(vec![
                    Box::new(AttributeValue::Integer(index_signed)),
                    Box::new(AttributeValue::Float(height)),
                ]),
            );
            let mut attr_map = HashMap::new();
            attr_map.insert(
                "key".to_string(),
                Box::new(AttributeValue::String("value".to_string())),
            );
            attrs.insert("attr_map".to_string(), AttributeValue::Map(attr_map));

            let seed_offset = f64::from(seed_u32) * 0.001;
            let offset = f64::from(index_u32) * 100.0;
            cityobject.set_geographical_extent(Some(BBox::new(
                offset + seed_offset,
                offset + seed_offset,
                0.0,
                offset + 50.0 + seed_offset,
                offset + 50.0 + seed_offset,
                20.0,
            )));

            let geometry_ref =
                build_geometry_full_feature(&mut model, &vertices, index_u32, mat_ref, tex_ref)?;
            cityobject.add_geometry(geometry_ref);

            let co_ref = model.cityobjects_mut().add(cityobject)?;
            cityobject_refs.push(co_ref);
        }

        Ok(cityobject_refs)
    }

    pub fn bench_build_minimal_geometry(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_BUILDER, FAST_SIZE_BUILDER);
        let mut group = c.benchmark_group("builder");
        let nr_cityobjects = params.size;
        group.throughput(Throughput::Elements(
            u64::try_from(nr_cityobjects).expect("cityobject count exceeds u64 range"),
        ));

        group.bench_function("build_minimal_geometry", |b| {
            b.iter(|| {
                let refs = build_cityobjects_minimal(black_box(nr_cityobjects))
                    .expect("cityobjects builder failed");
                black_box(refs);
            });
        });

        group.finish();
    }

    pub fn bench_build_full_feature(c: &mut Criterion) {
        let params = params_from_env(DEFAULT_SIZE_BUILDER, FAST_SIZE_BUILDER);
        let mut group = c.benchmark_group("builder");
        let nr_cityobjects = params.size;
        group.throughput(Throughput::Elements(
            u64::try_from(nr_cityobjects).expect("cityobject count exceeds u64 range"),
        ));

        group.bench_function("build_full_feature", |b| {
            b.iter(|| {
                let refs = build_cityobjects_full(black_box(nr_cityobjects), params.seed)
                    .expect("cityobjects builder failed");
                black_box(refs);
            });
        });

        group.finish();
    }
}

criterion_group!(
    benches,
    benches::bench_build_minimal_geometry,
    benches::bench_build_full_feature
);

criterion_main!(benches);
