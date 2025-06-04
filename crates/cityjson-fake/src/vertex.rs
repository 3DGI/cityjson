use crate::cli::CJFakeConfig;
use crate::get_nr_items;
use cityjson::prelude::{QuantizedCoordinate, VertexRef, Vertices};
use fake::{Dummy, Fake};
use rand::Rng;

/// Fake [Vertices] with [QuantizedCoordinate]s.
///
/// # Examples
/// ```rust
/// use cjfake::prelude::*;
/// use fake::{Fake, Faker};
/// use cityjson::prelude::{Vertices, QuantizedCoordinate, VertexRef};
/// use rand;
///
/// // Example CJFakeConfig with arbitrary values
/// let cjfake = CJFakeConfig {
///     min_coordinate: 0,
///     max_coordinate: 100,
///     min_vertices: 3,
///     max_vertices: 5,
///     ..Default::default()
/// };
/// let mut rng = rand::rng();
/// let vertices: Vertices<u32, QuantizedCoordinate> = VerticesFaker::new(&cjfake).fake_with_rng(&mut rng);
/// assert!(vertices.len() >= 3 && vertices.len() <= 5);
/// ```
pub struct VerticesFaker<'cmbuild> {
    cjfake: &'cmbuild CJFakeConfig,
}

impl<'cmbuild> VerticesFaker<'cmbuild> {
    pub fn new(cjfake_config: &'cmbuild CJFakeConfig) -> Self {
        Self {
            cjfake: cjfake_config,
        }
    }
}
impl<'cmbuild, VR: VertexRef> Dummy<VerticesFaker<'cmbuild>> for Vertices<VR, QuantizedCoordinate> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &VerticesFaker, rng: &mut R) -> Self {
        let cf = QuantizedCoordinateFaker {
            min: config.cjfake.min_coordinate,
            max: config.cjfake.max_coordinate,
        };
        let nr_vertices =
            get_nr_items(config.cjfake.min_vertices..=config.cjfake.max_vertices, rng);
        let coords: Vec<QuantizedCoordinate> = (cf, nr_vertices).fake_with_rng(rng);
        Vertices::from(coords)
    }
}

pub struct QuantizedCoordinateFaker {
    min: i64,
    max: i64,
}

impl Dummy<QuantizedCoordinateFaker> for QuantizedCoordinate {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &QuantizedCoordinateFaker, rng: &mut R) -> Self {
        QuantizedCoordinate::new(
            rng.random_range(config.min..=config.max),
            rng.random_range(config.min..=config.max),
            rng.random_range(config.min..=config.max),
        )
    }
}
