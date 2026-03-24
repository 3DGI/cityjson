//! Memory-focused benchmarks that capture heap usage with dhat.

#[allow(dead_code)]
mod support;

use rand::{Rng, RngExt};
use std::hint::black_box;
use support::{BenchParams, DEFAULT_SIZE_MEMORY, FAST_SIZE_MEMORY, params_from_env, rng_from_seed};

// Enable dhat heap profiling for the entire benchmark
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

mod benches {
    use super::{BenchParams, Rng, RngExt, black_box, rng_from_seed};

    use cityjson::error::Result;
    use cityjson::resources::storage::OwnedStringStorage;
    use cityjson::resources::{GeometryHandle, MaterialHandle, TextureHandle};
    use cityjson::v2_0::{
        AttributeValue, CRS, CityModel, CityModelIdentifier, CityModelType, CityObject,
        CityObjectIdentifier, CityObjectType, GeometryDraft, ImageType, LoD, Material,
        RealWorldCoordinate, RingDraft, Semantic, SemanticType, ShellDraft, SurfaceDraft, Texture,
        UVCoordinate, VertexIndex32,
    };
    use std::collections::HashMap;

    type OwnedModel = CityModel<u32, OwnedStringStorage>;
    type OwnedCityObject = CityObject<OwnedStringStorage>;

    fn usize_to_u32(value: usize, context: &str) -> u32 {
        u32::try_from(value).expect(context)
    }

    fn configure_model_metadata(model: &mut OwnedModel) {
        let metadata = model.metadata_mut();
        metadata.set_identifier(CityModelIdentifier::new("memory-benchmark".to_string()));
        metadata.set_reference_system(CRS::new(
            "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
        ));
    }

    fn create_material_and_texture(
        model: &mut OwnedModel,
    ) -> Result<(MaterialHandle, TextureHandle)> {
        let mut material = Material::new("benchmark_material".to_string());
        material.set_ambient_intensity(Some(0.5));
        material.set_diffuse_color(Some([0.8, 0.8, 0.8].into()));
        let texture = Texture::new("benchmark_texture.png".to_string(), ImageType::Png);
        let material_ref = model.add_material(material)?;
        let texture_ref = model.add_texture(texture)?;
        Ok((material_ref, texture_ref))
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
                    .add_vertex(RealWorldCoordinate::new(
                        f64::from(x),
                        f64::from(y),
                        f64::from(z),
                    ))
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
        material_ref: MaterialHandle,
        texture_ref: TextureHandle,
    ) -> Result<GeometryHandle> {
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

        let ground_semantic = model.add_semantic(ground_semantic)?;
        let roof_semantic = model.add_semantic(roof_semantic)?;
        let wall_semantic = model.add_semantic(wall_semantic)?;

        let uv0 = model.add_uv_coordinate(UVCoordinate::new(0.0, 0.0))?;
        let uv1 = model.add_uv_coordinate(UVCoordinate::new(1.0, 0.0))?;
        let uv2 = model.add_uv_coordinate(UVCoordinate::new(1.0, 1.0))?;
        let uv3 = model.add_uv_coordinate(UVCoordinate::new(0.0, 1.0))?;

        let surface_bottom = SurfaceDraft::new(
            RingDraft::new([vertices[0], vertices[1], vertices[2], vertices[3]]),
            std::iter::empty::<RingDraft<u32, OwnedStringStorage>>(),
        )
        .with_semantic(ground_semantic);
        let surface_top = SurfaceDraft::new(
            RingDraft::new([vertices[4], vertices[7], vertices[6], vertices[5]]),
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
        .with_semantic(wall_semantic);
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

    fn add_cityobject<R: Rng + ?Sized>(
        model: &mut OwnedModel,
        rng: &mut R,
        index: usize,
        seed: u32,
        material_ref: MaterialHandle,
        texture_ref: TextureHandle,
    ) {
        let index_u32 = usize_to_u32(index, "cityobject index exceeds u32 range");
        let vertices = add_random_vertices(model, rng);
        let mut cityobject = CityObject::new(
            CityObjectIdentifier::new(format!("building-{index_u32:06}")),
            CityObjectType::Building,
        );

        add_attributes(&mut cityobject, index_u32, seed);
        let geometry_ref =
            build_cube_geometry(model, &vertices, index_u32, material_ref, texture_ref)
                .expect("failed to build geometry");
        cityobject.add_geometry(geometry_ref);

        model
            .cityobjects_mut()
            .add(cityobject)
            .expect("failed to add cityobject to model");
    }

    /// Build a `CityModel` with the specified vertex index type and number of cityobjects.
    /// Each cityobject will have a solid geometry with 8 vertices (a cube).
    fn build_model(n_cityobjects: usize, seed: u64) -> OwnedModel {
        let seed_u32 = u32::try_from(seed).expect("seed exceeds u32 range");
        let mut model = CityModel::<u32, OwnedStringStorage>::new(CityModelType::CityJSON);
        let mut rng = rng_from_seed(seed);
        configure_model_metadata(&mut model);
        let (material_ref, texture_ref) =
            create_material_and_texture(&mut model).expect("failed to create shared resources");

        for index in 0..n_cityobjects {
            add_cityobject(
                &mut model,
                &mut rng,
                index,
                seed_u32,
                material_ref,
                texture_ref,
            );
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
    benches::run(params);
}
