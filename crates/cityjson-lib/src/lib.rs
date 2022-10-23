use std::fmt;

///```rust
///let cm = CityModel::new();
///let cm2 = CityModel::default();
/// ```
struct CityModel {
    version: String,
}

impl CityModel {
    pub fn new() -> Self {
        Self {
            version: String::from(""),
        }
    }
}

impl Default for CityModel {
    fn default() -> Self {
        Self {
            version: String::from(""),
        }
    }
}

impl fmt::Debug for CityModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CityModel")
            .field("version", &self.version)
            .finish()
    }
}

impl fmt::Display for CityModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(version: {})", &self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instantiate_citymodel() {
        let _cm = CityModel::new();
        let _cm2 = CityModel::default();
    }

    #[test]
    fn debug_citymodel() {
        let cm = CityModel::new();
        println!("{:?}", cm);
    }

    #[test]
    fn display_citymodel() {
        let cm = CityModel::new();
        println!("{}", cm);
    }
}
