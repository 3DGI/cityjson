//! Texture generation helpers.
//!
//! ```rust
//! use cityjson_fake::texture::TextureBuilder;
//!
//! let texture = TextureBuilder::default().build();
//! assert!(!texture.image().is_empty());
//! ```

use cityjson_types::prelude::OwnedStringStorage;
use cityjson_types::v2_0::{ImageType, Texture};
use fake::Fake;
use fake::RngExt;
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};
use std::path::PathBuf;

/// Builder for creating `Texture` with random properties.
///
/// # Examples
///
/// ```rust
/// use cityjson_fake::prelude::*;
///
/// let texture: cityjson_types::v2_0::Texture<OwnedStringStorage> = TextureBuilder::default()
///     .image()
///     .image_type()
///     .build();
///
/// assert!(!texture.image().is_empty());
/// ```
pub struct TextureBuilder {
    image_path: String,
    image_type: ImageType,
    rng: SmallRng,
}

impl Default for TextureBuilder {
    fn default() -> Self {
        let mut rng = SmallRng::seed_from_u64(0);
        Self::new(&mut rng)
    }
}

impl TextureBuilder {
    /// Creates a new `TextureBuilder` with random values. Derives a seeded RNG from `rng`.
    #[must_use]
    pub fn new(rng: &mut impl Rng) -> Self {
        use fake::faker::filesystem::raw::FilePath;
        use fake::locales::EN;

        let seed = rng.random::<u64>();
        let mut inner_rng = SmallRng::seed_from_u64(seed);

        let image_path: String = FilePath(EN).fake_with_rng(&mut inner_rng);
        let image_type = if inner_rng.random_bool(0.5) {
            ImageType::Png
        } else {
            ImageType::Jpg
        };

        Self {
            image_path,
            image_type,
            rng: inner_rng,
        }
    }

    /// Sets a random image type (PNG or JPG)
    #[must_use]
    pub fn image_type(mut self) -> Self {
        self.image_type = if self.rng.random_bool(0.5) {
            ImageType::Png
        } else {
            ImageType::Jpg
        };
        self
    }

    /// Sets a random image file path with appropriate extension
    #[must_use]
    pub fn image(mut self) -> Self {
        use fake::faker::filesystem::raw::FilePath;
        use fake::locales::EN;

        let fp: PathBuf = FilePath(EN).fake_with_rng(&mut self.rng);
        let path_with_ext = match self.image_type {
            ImageType::Png => fp.with_extension("png"),
            _ => fp.with_extension("jpg"),
        };
        self.image_path = path_with_ext.to_string_lossy().to_string();
        self
    }

    /// Builds and returns the Texture
    #[must_use]
    pub fn build(self) -> Texture<OwnedStringStorage> {
        Texture::new(self.image_path, self.image_type)
    }
}
