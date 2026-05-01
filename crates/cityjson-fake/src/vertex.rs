//! Vertex generation helpers.
//!
//! ```rust
//! use cityjson_fake::vertex::VerticesFaker;
//! use cityjson_fake::prelude::*;
//! use fake::{Fake, Faker};
//! use rand::SeedableRng;
//! use cityjson_types::v2_0::{RealWorldCoordinate, Vertices};
//!
//! let config = CJFakeConfig::default();
//! let mut rng = rand::prelude::SmallRng::seed_from_u64(4);
//! let vertices: Vertices<u32, RealWorldCoordinate> =
//!     VerticesFaker::new(&config).fake_with_rng(&mut rng);
//! assert!(!vertices.is_empty());
//! ```

use crate::cli::CJFakeConfig;
use crate::get_nr_items;
use cityjson_types::v2_0::{RealWorldCoordinate, VertexRef, Vertices};
use fake::RngExt;
use fake::{Dummy, Fake};
use rand::Rng;

/// Fake [`Vertices`] with [`RealWorldCoordinate`]s.
///
/// # Examples
/// ```rust
/// use cityjson_fake::prelude::*;
/// use fake::{Fake, Faker};
/// use cityjson_types::v2_0::{Vertices, RealWorldCoordinate, VertexRef};
/// use rand;
///
/// // Example CJFakeConfig with arbitrary values
/// let config = CJFakeConfig {
///     vertices: VertexConfig {
///         min_coordinate: 0.0,
///         max_coordinate: 100.0,
///         min_vertices: 3,
///         max_vertices: 5,
///         ..Default::default()
///     },
///     ..Default::default()
/// };
/// let mut rng = rand::rng();
/// let vertices: Vertices<u32, RealWorldCoordinate> = VerticesFaker::new(&config).fake_with_rng(&mut rng);
/// assert!(vertices.len() >= 3 && vertices.len() <= 5);
/// ```
pub struct VerticesFaker<'cmbuild> {
    config: &'cmbuild CJFakeConfig,
}

impl<'cmbuild> VerticesFaker<'cmbuild> {
    #[must_use]
    pub fn new(config: &'cmbuild CJFakeConfig) -> Self {
        Self { config }
    }
}

/// Faker for individual `RealWorldCoordinate` values.
///
/// # Examples
///
/// ```rust
/// use cityjson_fake::vertex::RealWorldCoordinateFaker;
/// use fake::Dummy;
/// use rand::SeedableRng;
///
/// let faker = RealWorldCoordinateFaker::new(0.0, 1.0);
/// let mut rng = rand::prelude::SmallRng::seed_from_u64(5);
/// let _coord =
///     <cityjson_types::v2_0::RealWorldCoordinate as Dummy<RealWorldCoordinateFaker>>::dummy_with_rng(
///         &faker,
///         &mut rng,
///     );
/// ```
pub struct RealWorldCoordinateFaker {
    min: f64,
    max: f64,
}

impl RealWorldCoordinateFaker {
    #[must_use]
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }
}

impl<VR: VertexRef> Dummy<VerticesFaker<'_>> for Vertices<VR, RealWorldCoordinate> {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &VerticesFaker, rng: &mut R) -> Self {
        let cf = RealWorldCoordinateFaker {
            min: config.config.vertices.min_coordinate,
            max: config.config.vertices.max_coordinate,
        };
        let nr_vertices = get_nr_items(
            config.config.vertices.min_vertices..=config.config.vertices.max_vertices,
            rng,
        );
        let coords: Vec<RealWorldCoordinate> = (cf, nr_vertices).fake_with_rng(rng);
        Vertices::from(coords)
    }
}

impl Dummy<RealWorldCoordinateFaker> for RealWorldCoordinate {
    fn dummy_with_rng<R: Rng + ?Sized>(config: &RealWorldCoordinateFaker, rng: &mut R) -> Self {
        RealWorldCoordinate::new(
            rng.random_range(config.min..=config.max),
            rng.random_range(config.min..=config.max),
            rng.random_range(config.min..=config.max),
        )
    }
}
