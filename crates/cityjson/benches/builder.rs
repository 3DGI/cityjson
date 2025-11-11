//! Benchmarks that build objects
use cityjson::prelude::*;
use cityjson::v2_0::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::collections::HashMap;

/// Helper function to build a geometry with semantics, materials, and textures
fn build_geometry_with_semantics_materials_textures(
    model: &mut CityModel<u32, ResourceId32, OwnedStringStorage>,
    vertices: &[VertexIndex32],
    index: usize,
    material_data: Option<&(Material<OwnedStringStorage>, ResourceId32)>,
    texture_data: Option<&(Texture<OwnedStringStorage>, ResourceId32)>,
) -> Result<ResourceId32> {
    // Create a Solid geometry
    let mut geometry_builder =
        GeometryBuilder::new(model, GeometryType::Solid, BuilderMode::Regular)
            .with_lod(LoD::LoD2_2);

    // Add vertices to the geometry builder
    let bv0 = geometry_builder.add_vertex(vertices[0]);
    let bv1 = geometry_builder.add_vertex(vertices[1]);
    let bv2 = geometry_builder.add_vertex(vertices[2]);
    let bv3 = geometry_builder.add_vertex(vertices[3]);
    let bv4 = geometry_builder.add_vertex(vertices[4]);
    let bv5 = geometry_builder.add_vertex(vertices[5]);
    let bv6 = geometry_builder.add_vertex(vertices[6]);
    let bv7 = geometry_builder.add_vertex(vertices[7]);

    // Build a simple box (6 surfaces)

    // Bottom surface (Ground)
    let ring_bottom = geometry_builder.add_ring(&[bv0, bv3, bv2, bv1])?;
    let surface_bottom = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_bottom)?;

    // Add semantic: GroundSurface
    let mut ground_semantic = Semantic::new(SemanticType::GroundSurface);
    let sem_attrs = ground_semantic.attributes_mut();
    sem_attrs.insert(
        "surfaceType".to_string(),
        AttributeValue::String("ground".to_string()),
    );
    geometry_builder.set_semantic_surface(None, ground_semantic)?;

    // Top surface (Roof)
    let ring_top = geometry_builder.add_ring(&[bv4, bv5, bv6, bv7])?;
    let surface_top = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_top)?;

    // Add semantic: RoofSurface
    let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
    let roof_attrs = roof_semantic.attributes_mut();
    roof_attrs.insert(
        "roofType".to_string(),
        AttributeValue::String("flat".to_string()),
    );
    roof_attrs.insert(
        "solarPanels".to_string(),
        AttributeValue::Bool(index % 3 == 0),
    );
    geometry_builder.set_semantic_surface(None, roof_semantic)?;

    // Add material to roof if available
    if let Some((material, _mat_ref)) = material_data {
        geometry_builder.set_material_surface(None, material.clone(), "default".to_string())?;
    }

    // Front wall (WallSurface)
    let ring_front = geometry_builder.add_ring(&[bv0, bv1, bv5, bv4])?;
    let surface_front = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_front)?;

    // Add semantic: WallSurface
    let mut wall_semantic = Semantic::new(SemanticType::WallSurface);
    let wall_attrs = wall_semantic.attributes_mut();
    wall_attrs.insert(
        "orientation".to_string(),
        AttributeValue::String("north".to_string()),
    );
    geometry_builder.set_semantic_surface(None, wall_semantic.clone())?;

    // Add texture to wall if available
    if let Some((texture, _tex_ref)) = texture_data {
        // Add UV coordinates
        let uv0 = geometry_builder.add_uv_coordinate(0.0, 0.0);
        let uv1 = geometry_builder.add_uv_coordinate(1.0, 0.0);
        let uv2 = geometry_builder.add_uv_coordinate(1.0, 1.0);
        let uv3 = geometry_builder.add_uv_coordinate(0.0, 1.0);

        // Map vertices to UV coordinates
        geometry_builder.map_vertex_to_uv(bv0, uv0);
        geometry_builder.map_vertex_to_uv(bv1, uv1);
        geometry_builder.map_vertex_to_uv(bv5, uv2);
        geometry_builder.map_vertex_to_uv(bv4, uv3);

        // Apply texture to the ring
        geometry_builder.set_texture_ring(None, texture.clone(), "default".to_string())?;
    }

    // Back wall
    let ring_back = geometry_builder.add_ring(&[bv2, bv3, bv7, bv6])?;
    let surface_back = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_back)?;
    wall_semantic.attributes_mut().insert(
        "orientation".to_string(),
        AttributeValue::String("south".to_string()),
    );
    geometry_builder.set_semantic_surface(None, wall_semantic.clone())?;

    // Left wall
    let ring_left = geometry_builder.add_ring(&[bv0, bv4, bv7, bv3])?;
    let surface_left = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_left)?;
    wall_semantic.attributes_mut().insert(
        "orientation".to_string(),
        AttributeValue::String("west".to_string()),
    );
    geometry_builder.set_semantic_surface(None, wall_semantic.clone())?;

    // Right wall
    let ring_right = geometry_builder.add_ring(&[bv1, bv2, bv6, bv5])?;
    let surface_right = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_right)?;
    wall_semantic.attributes_mut().insert(
        "orientation".to_string(),
        AttributeValue::String("east".to_string()),
    );
    geometry_builder.set_semantic_surface(None, wall_semantic)?;

    // Create shell from all surfaces
    let shell_surfaces = vec![
        surface_bottom,
        surface_top,
        surface_front,
        surface_back,
        surface_left,
        surface_right,
    ];
    geometry_builder.add_shell(&shell_surfaces)?;

    // Build and return the geometry
    let geometry_ref = geometry_builder.build()?;
    Ok(geometry_ref)
}

/// Builds a collection of CityObjects with optional geometries, semantics, materials, and textures.
///
/// # Arguments
///
/// * `num_cityobjects` - The number of CityObjects to create
/// * `with_geometries` - If true, creates geometries with semantics, materials, and textures
///
/// # Returns
///
/// Returns a vector of ResourceId32 references to the created CityObjects
///
/// # Examples
///
/// ```ignore
/// let mut model = CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
/// let cityobject_refs = build_cityobjects(&mut model, 100, true)?;
/// ```
pub fn build_cityobjects(config: (usize, bool)) -> Result<Vec<ResourceId32>> {
    let num_cityobjects = config.0;
    let with_geometries = config.1;
    let mut model =
        CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
    let mut cityobject_refs = Vec::with_capacity(num_cityobjects);

    // Create materials and textures if geometries are needed
    let (material_ref, texture_ref) = if with_geometries {
        // Create a comprehensive material
        let mut material = Material::new("benchmark_material".to_string());
        material.set_ambient_intensity(Some(0.5));
        material.set_diffuse_color(Some([0.8, 0.8, 0.8]));
        material.set_emissive_color(Some([0.0, 0.0, 0.0]));
        material.set_specular_color(Some([1.0, 1.0, 1.0]));
        material.set_shininess(Some(0.8));
        material.set_transparency(Some(0.0));
        material.set_is_smooth(Some(true));
        let mat_ref = model.add_material(material.clone());

        // Create a texture
        let texture = Texture::new("benchmark_texture.png".to_string(), ImageType::Png);
        let tex_ref = model.add_texture(texture.clone());

        (Some((material, mat_ref)), Some((texture, tex_ref)))
    } else {
        (None, None)
    };

    // Pre-create some vertices for geometry reuse
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

    // Build CityObjects
    for i in 0..num_cityobjects {
        let co_id = format!("cityobject-{}", i);

        // Vary the CityObject types for diversity
        let co_type = match i % 5 {
            0 => CityObjectType::Building,
            1 => CityObjectType::BuildingPart,
            2 => CityObjectType::Road,
            3 => CityObjectType::PlantCover,
            _ => CityObjectType::GenericCityObject,
        };

        let mut cityobject = CityObject::new(co_id.clone(), co_type);

        // Add attributes to the CityObject
        let attrs = cityobject.attributes_mut();
        attrs.insert(
            "measuredHeight".to_string(),
            AttributeValue::Float(10.0 + (i as f64) * 0.5),
        );
        attrs.insert(
            "yearOfConstruction".to_string(),
            AttributeValue::Integer(2000 + (i as i64 % 24)),
        );
        attrs.insert(
            "function".to_string(),
            AttributeValue::String(format!("function_{}", i % 10)),
        );
        attrs.insert("active".to_string(), AttributeValue::Bool(i % 2 == 0));

        // Add complex nested attributes
        let mut nested_map = HashMap::new();
        nested_map.insert(
            "owner".to_string(),
            Box::new(AttributeValue::String(format!("Owner {}", i % 5))),
        );
        nested_map.insert(
            "value".to_string(),
            Box::new(AttributeValue::Float(100000.0 + (i as f64) * 1000.0)),
        );
        attrs.insert("details".to_string(), AttributeValue::Map(nested_map));

        // Add an array attribute
        let array_values = vec![
            Box::new(AttributeValue::Integer(i as i64)),
            Box::new(AttributeValue::Integer((i * 2) as i64)),
            Box::new(AttributeValue::Integer((i * 3) as i64)),
        ];
        attrs.insert("values".to_string(), AttributeValue::Vec(array_values));

        // Set geographical extent
        let offset = (i as f64) * 100.0;
        cityobject.set_geographical_extent(Some(BBox::new(
            offset,
            offset,
            0.0,
            offset + 50.0,
            offset + 50.0,
            20.0,
        )));

        // Build geometry if requested
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

        // Add CityObject to the model
        let co_ref = model.cityobjects_mut().add(cityobject);
        cityobject_refs.push(co_ref);
    }

    Ok(cityobject_refs)
}

fn bench_build_cityobjects_without_geometry(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder");

    let nr_cityobjects = 10_000_usize;
    // Set throughput for better reporting
    group.throughput(Throughput::Elements(nr_cityobjects as u64));

    group.bench_function("build_10000_cityobjects_without_geometry", |b| {
        b.iter(|| build_cityobjects(black_box((nr_cityobjects, false))));
    });

    group.finish();
}

fn bench_build_cityobjects_with_geometry(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder");

    let nr_cityobjects = 10_000_usize;
    // Set throughput for better reporting
    group.throughput(Throughput::Elements(nr_cityobjects as u64));

    group.bench_function("build_10000_cityobjects_with_geometry", |b| {
        b.iter(|| build_cityobjects(black_box((nr_cityobjects, true))));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_build_cityobjects_without_geometry,
    bench_build_cityobjects_with_geometry
);
criterion_main!(benches);
