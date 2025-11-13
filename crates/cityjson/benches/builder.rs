//! Benchmarks that build objects
use cityjson::cityjson::core::attributes::{AttributeOwnerType, OwnedAttributePool};
use cityjson::prelude::*;
use cityjson::v2_0::*;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::collections::HashMap;
use std::hint::black_box;

/// Helper function to build a geometry with semantics, materials, and textures.
/// Tests the realistic case where each surface has unique attributes (e.g., azimuth, slope, area).
fn build_geometry_with_semantics_materials_textures(
    model: &mut CityModel<u32, ResourceId32, OwnedStringStorage>,
    pool: &mut OwnedAttributePool,
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

    // Add semantic: GroundSurface with unique attributes
    let mut ground_semantic = Semantic::new(SemanticType::GroundSurface);
    let ground_attrs = ground_semantic.attributes_mut();
    let area_id = pool.add_float(
        "area".to_string(),
        true,
        100.0 + (index as f64) * 0.5,  // Unique area per surface
        AttributeOwnerType::Semantic,
        None,
    );
    ground_attrs.insert("area".to_string(), area_id);
    geometry_builder.set_semantic_surface(None, ground_semantic, false)?;

    // Top surface (Roof)
    let ring_top = geometry_builder.add_ring(&[bv4, bv5, bv6, bv7])?;
    let surface_top = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_top)?;

    // Add semantic: RoofSurface with unique attributes (azimuth, slope, area)
    let mut roof_semantic = Semantic::new(SemanticType::RoofSurface);
    let roof_attrs = roof_semantic.attributes_mut();
    let azimuth_id = pool.add_float(
        "azimuth".to_string(),
        true,
        (index % 360) as f64,  // Unique azimuth per roof
        AttributeOwnerType::Semantic,
        None,
    );
    let slope_id = pool.add_float(
        "slope".to_string(),
        true,
        15.0 + ((index % 30) as f64),  // Unique slope per roof
        AttributeOwnerType::Semantic,
        None,
    );
    let roof_area_id = pool.add_float(
        "area".to_string(),
        true,
        200.0 + (index as f64) * 1.2,  // Unique area per roof
        AttributeOwnerType::Semantic,
        None,
    );
    roof_attrs.insert("azimuth".to_string(), azimuth_id);
    roof_attrs.insert("slope".to_string(), slope_id);
    roof_attrs.insert("area".to_string(), roof_area_id);
    geometry_builder.set_semantic_surface(None, roof_semantic, false)?;

    // Add material to roof if available
    if let Some((material, _mat_ref)) = material_data {
        geometry_builder.set_material_surface(None, material.clone(), "default".to_string(), true)?;
    }

    // Front wall (WallSurface)
    let ring_front = geometry_builder.add_ring(&[bv0, bv1, bv5, bv4])?;
    let surface_front = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_front)?;

    // Add semantic: WallSurface (north) with unique attributes
    let mut wall_north = Semantic::new(SemanticType::WallSurface);
    let wall_north_attrs = wall_north.attributes_mut();
    let orientation_n_id = pool.add_string(
        "orientation".to_string(),
        true,
        "north".to_string(),
        AttributeOwnerType::Semantic,
        None,
    );
    let wall_area_n_id = pool.add_float(
        "area".to_string(),
        true,
        50.0 + (index as f64) * 0.3,
        AttributeOwnerType::Semantic,
        None,
    );
    wall_north_attrs.insert("orientation".to_string(), orientation_n_id);
    wall_north_attrs.insert("area".to_string(), wall_area_n_id);
    geometry_builder.set_semantic_surface(None, wall_north, false)?;

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
        geometry_builder.set_texture_ring(None, texture.clone(), "default".to_string(), true)?;
    }

    // Back wall
    let ring_back = geometry_builder.add_ring(&[bv2, bv3, bv7, bv6])?;
    let surface_back = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_back)?;
    let mut wall_south = Semantic::new(SemanticType::WallSurface);
    let wall_south_attrs = wall_south.attributes_mut();
    let orientation_s_id = pool.add_string(
        "orientation".to_string(),
        true,
        "south".to_string(),
        AttributeOwnerType::Semantic,
        None,
    );
    let wall_area_s_id = pool.add_float(
        "area".to_string(),
        true,
        50.0 + (index as f64) * 0.3 + 0.1,
        AttributeOwnerType::Semantic,
        None,
    );
    wall_south_attrs.insert("orientation".to_string(), orientation_s_id);
    wall_south_attrs.insert("area".to_string(), wall_area_s_id);
    geometry_builder.set_semantic_surface(None, wall_south, false)?;

    // Left wall
    let ring_left = geometry_builder.add_ring(&[bv0, bv4, bv7, bv3])?;
    let surface_left = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_left)?;
    let mut wall_west = Semantic::new(SemanticType::WallSurface);
    let wall_west_attrs = wall_west.attributes_mut();
    let orientation_w_id = pool.add_string(
        "orientation".to_string(),
        true,
        "west".to_string(),
        AttributeOwnerType::Semantic,
        None,
    );
    let wall_area_w_id = pool.add_float(
        "area".to_string(),
        true,
        50.0 + (index as f64) * 0.3 + 0.2,
        AttributeOwnerType::Semantic,
        None,
    );
    wall_west_attrs.insert("orientation".to_string(), orientation_w_id);
    wall_west_attrs.insert("area".to_string(), wall_area_w_id);
    geometry_builder.set_semantic_surface(None, wall_west, false)?;

    // Right wall
    let ring_right = geometry_builder.add_ring(&[bv1, bv2, bv6, bv5])?;
    let surface_right = geometry_builder.start_surface();
    geometry_builder.add_surface_outer_ring(ring_right)?;
    let mut wall_east = Semantic::new(SemanticType::WallSurface);
    let wall_east_attrs = wall_east.attributes_mut();
    let orientation_e_id = pool.add_string(
        "orientation".to_string(),
        true,
        "east".to_string(),
        AttributeOwnerType::Semantic,
        None,
    );
    let wall_area_e_id = pool.add_float(
        "area".to_string(),
        true,
        50.0 + (index as f64) * 0.3 + 0.3,
        AttributeOwnerType::Semantic,
        None,
    );
    wall_east_attrs.insert("orientation".to_string(), orientation_e_id);
    wall_east_attrs.insert("area".to_string(), wall_area_e_id);
    geometry_builder.set_semantic_surface(None, wall_east, false)?;

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

    // Create attribute pool for all attributes
    let mut pool = OwnedAttributePool::new();

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
        let measured_height_id = pool.add_float(
            "measuredHeight".to_string(),
            true,
            10.0 + (i as f64) * 0.5,
            AttributeOwnerType::CityObject,
            None,
        );
        let year_of_construction_id = pool.add_integer(
            "yearOfConstruction".to_string(),
            true,
            2000 + (i as i64 % 24),
            AttributeOwnerType::CityObject,
            None,
        );
        let function_id = pool.add_string(
            "function".to_string(),
            true,
            format!("function_{}", i % 10),
            AttributeOwnerType::CityObject,
            None,
        );
        let active_id = pool.add_bool(
            "active".to_string(),
            true,
            i % 2 == 0,
            AttributeOwnerType::CityObject,
            None,
        );
        attrs.insert("measuredHeight".to_string(), measured_height_id);
        attrs.insert("yearOfConstruction".to_string(), year_of_construction_id);
        attrs.insert("function".to_string(), function_id);
        attrs.insert("active".to_string(), active_id);

        // Add complex nested attributes
        let owner_id = pool.add_string(
            "owner".to_string(),
            true,
            format!("Owner {}", i % 5),
            AttributeOwnerType::Element,
            None,
        );
        let value_id = pool.add_float(
            "value".to_string(),
            true,
            100000.0 + (i as f64) * 1000.0,
            AttributeOwnerType::Element,
            None,
        );
        let mut nested_map = HashMap::new();
        nested_map.insert("owner".to_string(), owner_id);
        nested_map.insert("value".to_string(), value_id);
        let details_id = pool.add_map(
            "details".to_string(),
            true,
            nested_map,
            AttributeOwnerType::CityObject,
            None,
        );
        attrs.insert("details".to_string(), details_id);

        // Add an array attribute
        let val1_id = pool.add_integer(
            "".to_string(),
            false,
            i as i64,
            AttributeOwnerType::Element,
            None,
        );
        let val2_id = pool.add_integer(
            "".to_string(),
            false,
            (i * 2) as i64,
            AttributeOwnerType::Element,
            None,
        );
        let val3_id = pool.add_integer(
            "".to_string(),
            false,
            (i * 3) as i64,
            AttributeOwnerType::Element,
            None,
        );
        let array_values = vec![val1_id, val2_id, val3_id];
        let values_id = pool.add_vector(
            "values".to_string(),
            true,
            array_values,
            AttributeOwnerType::CityObject,
            None,
        );
        attrs.insert("values".to_string(), values_id);

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
                &mut pool,
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

    group.bench_function("build_cityobjects_without_geometry", |b| {
        b.iter(|| {
            let refs = build_cityobjects(black_box((nr_cityobjects, false)))
                .expect("cityobjects builder failed");
            black_box(refs);
        });
    });

    group.finish();
}

fn bench_build_cityobjects_with_geometry(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder");

    let nr_cityobjects = 10_000_usize;
    // Set throughput for better reporting
    group.throughput(Throughput::Elements(nr_cityobjects as u64));

    group.bench_function("build_cityobjects_with_geometry", |b| {
        b.iter(|| {
            let refs = build_cityobjects(black_box((nr_cityobjects, true)))
                .expect("cityobjects builder failed");
            black_box(refs);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_build_cityobjects_without_geometry,
    bench_build_cityobjects_with_geometry
);
criterion_main!(benches);
