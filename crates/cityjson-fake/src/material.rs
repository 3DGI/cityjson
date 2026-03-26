use cityjson::prelude::StringStorage;
use cityjson::v2_0::{Material, RGB};
use fake::Dummy;
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};

/// Faker for RGB color values [0.0..1.0]
pub struct RgbFaker;

impl Dummy<RgbFaker> for RGB {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &RgbFaker, rng: &mut R) -> Self {
        RGB::new(
            rng.random_range(0.0..=1.0),
            rng.random_range(0.0..=1.0),
            rng.random_range(0.0..=1.0),
        )
    }
}

/// Builder for creating `Material` with random properties
///
/// # Examples
///
/// ```rust
/// use cjfake::prelude::*;
///
/// let material: cityjson::v2_0::Material<OwnedStringStorage> = MaterialBuilder::default()
///     .name()
///     .diffuse_color()
///     .shininess()
///     .build();
///
/// assert!(!material.name().is_empty());
/// ```
pub struct MaterialBuilder<SS: StringStorage> {
    material: Material<SS>,
    rng: SmallRng,
}

impl<SS: StringStorage<String = String>> Default for MaterialBuilder<SS> {
    fn default() -> Self {
        let mut rng = SmallRng::seed_from_u64(0);
        Self::new(&mut rng)
    }
}

impl<SS: StringStorage<String = String>> MaterialBuilder<SS> {
    /// Creates a new `MaterialBuilder`. Derives a seeded RNG from the provided `rng`.
    #[must_use]
    pub fn new(rng: &mut impl Rng) -> Self {
        let seed = rng.random::<u64>();
        Self {
            material: Material::new("material".to_string()),
            rng: SmallRng::seed_from_u64(seed),
        }
    }

    /// Sets a random name for the material
    #[must_use]
    pub fn name(mut self) -> Self {
        use fake::faker::lorem::raw::Word;
        use fake::locales::EN;
        use fake::Fake;
        self.material = Material::new(Word(EN).fake_with_rng(&mut self.rng));
        self
    }

    /// Sets a random ambient intensity value (0.0..1.0)
    #[must_use]
    pub fn ambient_intensity(mut self) -> Self {
        self.material
            .set_ambient_intensity(Some(self.rng.random_range(0.0..=1.0)));
        self
    }

    /// Sets a random diffuse color
    #[must_use]
    pub fn diffuse_color(mut self) -> Self {
        let rgb = RGB::new(
            self.rng.random_range(0.0..=1.0),
            self.rng.random_range(0.0..=1.0),
            self.rng.random_range(0.0..=1.0),
        );
        self.material.set_diffuse_color(Some(rgb));
        self
    }

    /// Sets a random emissive color
    #[must_use]
    pub fn emissive_color(mut self) -> Self {
        let rgb = RGB::new(
            self.rng.random_range(0.0..=1.0),
            self.rng.random_range(0.0..=1.0),
            self.rng.random_range(0.0..=1.0),
        );
        self.material.set_emissive_color(Some(rgb));
        self
    }

    /// Sets a random specular color
    #[must_use]
    pub fn specular_color(mut self) -> Self {
        let rgb = RGB::new(
            self.rng.random_range(0.0..=1.0),
            self.rng.random_range(0.0..=1.0),
            self.rng.random_range(0.0..=1.0),
        );
        self.material.set_specular_color(Some(rgb));
        self
    }

    /// Sets a random shininess value (0.0..1.0)
    #[must_use]
    pub fn shininess(mut self) -> Self {
        self.material
            .set_shininess(Some(self.rng.random_range(0.0..=1.0)));
        self
    }

    /// Sets a random transparency value (0.0..1.0)
    #[must_use]
    pub fn transparency(mut self) -> Self {
        self.material
            .set_transparency(Some(self.rng.random_range(0.0..=1.0)));
        self
    }

    /// Builds and returns the Material
    #[must_use]
    pub fn build(self) -> Material<SS> {
        self.material
    }
}
