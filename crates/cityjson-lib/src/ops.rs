use crate::{CityModel, Result};

#[derive(Debug, Clone, Default)]
pub struct Selection {
    ids: Vec<String>,
}

impl Selection {
    pub fn from_ids<I, S>(ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            ids: ids.into_iter().map(Into::into).collect(),
        }
    }
}

pub fn subset(model: &CityModel, selection: Selection) -> Result<CityModel> {
    let _ = selection.ids;
    Ok(model.clone())
}

pub fn merge<I>(models: I) -> Result<CityModel>
where
    I: IntoIterator<Item = CityModel>,
{
    models
        .into_iter()
        .next()
        .ok_or_else(|| crate::Error::UnsupportedFeature("merge requires at least one model".into()))
}

pub fn upgrade(model: CityModel) -> Result<CityModel> {
    Ok(model)
}

pub mod geometry {
    use crate::{CityModel, Result};

    pub fn surface_area(_model: &CityModel, _feature_id: &str) -> Result<f64> {
        Ok(0.0)
    }

    pub fn volume(_model: &CityModel, _feature_id: &str) -> Result<f64> {
        Ok(0.0)
    }
}

pub mod vertices {
    use crate::{CityModel, Result};

    #[derive(Debug, Clone, Copy, Default)]
    pub struct CleanReport {
        pub duplicates_removed: usize,
        pub orphans_removed: usize,
    }

    pub fn clean(_model: &mut CityModel) -> Result<CleanReport> {
        Ok(CleanReport::default())
    }
}

pub mod lod {
    use crate::{CityModel, Result};

    pub fn filter(model: &CityModel, _lod: &str) -> Result<CityModel> {
        Ok(model.clone())
    }
}
