//! Minimal streaming example.
//!
//! This shows the shape of a producer/consumer flow without pretending to be a
//! benchmark or a full streaming framework.

use std::sync::mpsc;
use std::thread;

use cityjson::error::Result;
use cityjson::resources::storage::OwnedStringStorage;
use cityjson::v2_0::{
    AffineTransform3D, CityModel, CityModelType, CityObject, CityObjectIdentifier, CityObjectType,
    GeometryDraft, LoD, PointDraft, RealWorldCoordinate,
};

type Model = CityModel<u32, OwnedStringStorage>;

#[derive(Debug, Clone)]
struct StreamGeometry {
    template_vertices: Vec<(f64, f64, f64)>,
}

#[derive(Debug, Clone)]
enum Message {
    Template(StreamGeometry),
    Object(String),
    Done,
}

fn producer(tx: &mpsc::SyncSender<Message>) {
    tx.send(Message::Template(StreamGeometry {
        template_vertices: vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
    }))
    .expect("failed to send template");

    tx.send(Message::Object("building-1".to_string()))
        .expect("failed to send object");
    tx.send(Message::Done).expect("failed to finish stream");
}

fn consumer(rx: &mpsc::Receiver<Message>) -> Result<Model> {
    let mut model = Model::new(CityModelType::CityJSON);

    let Message::Template(template) = rx.recv().expect("stream ended early") else {
        unreachable!();
    };
    let template_ref = GeometryDraft::multi_point(
        Some(LoD::LoD1),
        template
            .template_vertices
            .into_iter()
            .map(|(x, y, z)| PointDraft::new(RealWorldCoordinate::new(x, y, z))),
    )
    .insert_template_into(&mut model)?;

    while let Message::Object(id) = rx.recv().expect("stream ended early") {
        let mut cityobject =
            CityObject::new(CityObjectIdentifier::new(id), CityObjectType::Building);
        let geometry_ref = GeometryDraft::instance(
            template_ref,
            RealWorldCoordinate::new(0.0, 0.0, 0.0),
            AffineTransform3D::default(),
        )
        .insert_into(&mut model)?;
        cityobject.add_geometry(geometry_ref);
        model.cityobjects_mut().add(cityobject)?;
    }

    Ok(model)
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::sync_channel(1);
    let producer_handle = thread::spawn(move || producer(&tx));
    let model = consumer(&rx)?;
    producer_handle.join().expect("producer panicked");
    println!(
        "streaming example built {} cityobject(s) and {} geometry template(s)",
        model.cityobjects().len(),
        model.geometry_template_count()
    );
    Ok(())
}
