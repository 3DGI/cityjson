use cityjson::prelude::*;
use cityjson::v2_0::*;
use criterion::{Criterion, criterion_group, criterion_main};
use rand::Rng;
use std::hint::black_box;

/// Generate a citymodel with n cityobjects, each with a solid geometry type.
/// The coordinate values are random.
fn generate_citymodel(n: usize) -> CityModel<u32, ResourceId32, OwnedStringStorage> {
    let mut model =
        CityModel::<u32, ResourceId32, OwnedStringStorage>::new(CityModelType::CityJSON);
    let mut rng = rand::rng();

    // Set basic metadata
    let metadata = model.metadata_mut();
    metadata.set_identifier(CityModelIdentifier::new("benchmark-model".to_string()));
    metadata.set_reference_system(CRS::new(
        "https://www.opengis.net/def/crs/EPSG/0/2355".to_string(),
    ));

    for i in 0..n {
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

        // Build a solid geometry using GeometryBuilder
        {
            let mut geometry_builder =
                GeometryBuilder::new(&mut model, GeometryType::Solid, BuilderMode::Regular)
                    .with_lod(LoD::LoD2);

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

            // Top face
            let ring_top = geometry_builder
                .add_ring(&[bv[4], bv[7], bv[6], bv[5]])
                .unwrap();
            let surface_top = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_top).unwrap();

            // Front face
            let ring_front = geometry_builder
                .add_ring(&[bv[0], bv[1], bv[5], bv[4]])
                .unwrap();
            let surface_front = geometry_builder.start_surface();
            geometry_builder.add_surface_outer_ring(ring_front).unwrap();

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

/// Compute the mean x,y,z coordinate for each geometry of each cityobject
fn compute_mean_coordinates(
    model: &CityModel<u32, ResourceId32, OwnedStringStorage>,
) -> Vec<(f64, f64, f64)> {
    let mut means = Vec::new();

    // Iterate through all cityobjects
    for (_id, cityobject) in model.cityobjects().iter() {
        // Iterate through all geometries of the cityobject
        if let Some(geometries) = cityobject.geometry() {
            for geometry_ref in geometries {
                if let Some(geometry) = model.get_geometry(*geometry_ref) {
                    // Get all vertices used by this geometry
                    if let Some(boundary) = geometry.boundaries() {
                        let vertex_indices = boundary.vertices();

                        if vertex_indices.is_empty() {
                            continue;
                        }

                        // Compute the sum of coordinates
                        let mut sum_x = 0i64;
                        let mut sum_y = 0i64;
                        let mut sum_z = 0i64;
                        let mut count = 0usize;

                        for vertex_idx in vertex_indices.iter() {
                            if let Some(vertex) = model.get_vertex(*vertex_idx) {
                                sum_x += vertex.x();
                                sum_y += vertex.y();
                                sum_z += vertex.z();
                                count += 1;
                            }
                        }

                        // Compute the mean
                        if count > 0 {
                            let mean_x = sum_x as f64 / count as f64;
                            let mean_y = sum_y as f64 / count as f64;
                            let mean_z = sum_z as f64 / count as f64;
                            means.push((mean_x, mean_y, mean_z));
                        }
                    }
                }
            }
        }
    }

    means
}

fn benchmark_mean_coordinates(c: &mut Criterion) {
    // Generate a citymodel with 10,000 cityobjects
    let model = generate_citymodel(10_000);

    c.bench_function("compute_mean_coordinates_10k", |b| {
        b.iter(|| {
            let means = compute_mean_coordinates(black_box(&model));
            black_box(means);
        })
    });
}

criterion_group!(benches, benchmark_mean_coordinates);
criterion_main!(benches);
