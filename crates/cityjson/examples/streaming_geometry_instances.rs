//! Streaming example with `GeometryInstance` templates (default backend).
//!
//! This is not a benchmark. It demonstrates how to stream template geometries
//! and instance them on the consumer side using the default backend.

use std::sync::mpsc;
use std::thread;

use cityjson::backend::default::geometry::GeometryBuilder;
use cityjson::prelude::*;
use cityjson::resources::pool::ResourceId32;
use cityjson::v2_0::{CityModel, CityObject, CityObjectType};

#[derive(Debug, Clone)]
struct WireTemplateGeometry {
    template_vertices: Vec<(f64, f64, f64)>,
}

#[derive(Debug, Clone)]
struct WireGeometry {
    template_ref: usize,
    transformation_matrix: [f64; 16],
}

#[derive(Debug, Clone)]
struct WireGlobalProperties {
    metadata_identifier: String,
    crs: String,
    geometry_templates: Vec<WireTemplateGeometry>,
}

#[derive(Debug, Clone)]
struct WireCityObjectData {
    id: String,
    vertices: Vec<(i64, i64, i64)>,
    geometries: Vec<WireGeometry>,
}

#[derive(Debug, Clone)]
enum StreamMessage {
    GlobalProperties(WireGlobalProperties),
    CityObject(WireCityObjectData),
    Done,
}

fn producer(tx: &mpsc::SyncSender<StreamMessage>) {
    let global_props = WireGlobalProperties {
        metadata_identifier: "streaming-instance-example".to_string(),
        crs: "https://www.opengis.net/def/crs/EPSG/0/7415".to_string(),
        geometry_templates: vec![WireTemplateGeometry {
            template_vertices: vec![
                (0.0, 0.0, 0.0),
                (5.0, 0.0, 0.0),
                (5.0, 5.0, 0.0),
                (0.0, 5.0, 0.0),
            ],
        }],
    };

    tx.send(StreamMessage::GlobalProperties(global_props))
        .expect("Failed to send global properties");

    let cityobject = WireCityObjectData {
        id: "building-0".to_string(),
        vertices: vec![(0, 0, 0)],
        geometries: vec![WireGeometry {
            template_ref: 0,
            transformation_matrix: [
                1.0, 0.0, 0.0, 0.0, //
                0.0, 1.0, 0.0, 0.0, //
                0.0, 0.0, 1.0, 0.0, //
                10.0, 20.0, 0.0, 1.0,
            ],
        }],
    };

    tx.send(StreamMessage::CityObject(cityobject))
        .expect("Failed to send CityObject");

    tx.send(StreamMessage::Done)
        .expect("Failed to send completion signal");
}

fn build_template_from_wire(
    model: &mut CityModel<u32, OwnedStringStorage>,
    wire_template: &WireTemplateGeometry,
) -> Result<ResourceId32> {
    let mut builder = GeometryBuilder::new(model, GeometryType::MultiPoint, BuilderMode::Template)
        .with_lod(LoD::LoD1);

    for (x, y, z) in &wire_template.template_vertices {
        builder.add_template_point(RealWorldCoordinate::new(*x, *y, *z));
    }

    builder.build()
}

fn consumer(rx: &mpsc::Receiver<StreamMessage>) -> Result<CityModel<u32, OwnedStringStorage>> {
    let Ok(StreamMessage::GlobalProperties(global_props)) = rx.recv() else {
        return Err(Error::InvalidGeometry(
            "Expected GlobalProperties as first message".to_string(),
        ));
    };

    let mut model = CityModel::<u32, OwnedStringStorage>::new(CityModelType::CityJSON);
    model
        .metadata_mut()
        .set_identifier(CityModelIdentifier::new(global_props.metadata_identifier));
    model
        .metadata_mut()
        .set_reference_system(CRS::new(global_props.crs));

    let mut template_refs = Vec::new();
    for wire_template in &global_props.geometry_templates {
        let template_ref = build_template_from_wire(&mut model, wire_template)?;
        template_refs.push(template_ref);
    }

    while let Ok(message) = rx.recv() {
        match message {
            StreamMessage::CityObject(wire_co) => {
                let mut cityobject = CityObject::new(
                    CityObjectIdentifier::new(wire_co.id.clone()),
                    CityObjectType::Building,
                );

                let vertex_refs: Vec<VertexIndex<u32>> = wire_co
                    .vertices
                    .iter()
                    .map(|(x, y, z)| {
                        model
                            .add_vertex(QuantizedCoordinate::new(*x, *y, *z))
                            .expect("Failed to add vertex")
                    })
                    .collect();

                for wire_geom in wire_co.geometries {
                    let template_ref =
                        template_refs.get(wire_geom.template_ref).ok_or_else(|| {
                            Error::InvalidGeometry(format!(
                                "Invalid template reference: {}",
                                wire_geom.template_ref
                            ))
                        })?;

                    let geom_ref = GeometryBuilder::new(
                        &mut model,
                        GeometryType::GeometryInstance,
                        BuilderMode::Regular,
                    )
                    .with_template(*template_ref)?
                    .with_transformation_matrix(wire_geom.transformation_matrix)?
                    .with_reference_vertex(vertex_refs[0])
                    .build()?;

                    cityobject.add_geometry(GeometryRef::from_parts(
                        geom_ref.index(),
                        geom_ref.generation(),
                    ));
                }

                model.cityobjects_mut().add(cityobject)?;
            }
            StreamMessage::Done => break,
            StreamMessage::GlobalProperties(_) => {
                return Err(Error::InvalidGeometry(
                    "Unexpected GlobalProperties message after stream started".to_string(),
                ));
            }
        }
    }

    Ok(model)
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::sync_channel::<StreamMessage>(4);

    let result = thread::scope(|s| {
        let producer_handle = s.spawn(move || producer(&tx));
        let consumer_handle = s.spawn(move || consumer(&rx));

        producer_handle.join().expect("Producer thread panicked");
        consumer_handle.join().expect("Consumer thread panicked")
    });

    let model = result?;
    println!(
        "streaming example done: {} cityobjects, {} geometries",
        model.cityobjects().len(),
        model.iter_geometries().count()
    );

    Ok(())
}
