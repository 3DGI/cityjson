#![allow(unexpected_cfgs)]
#![allow(clippy::unnecessary_wraps)]

fn main() -> cjlib::Result<()> {
    #[cfg(feature = "arrow")]
    {
        let model = cjlib::CityModel::from_file("tests/data/v2_0/minimal.city.json")?;
        cjlib::arrow::to_file("tests/output/minimal.cjarrow", &model)?;
    }

    #[cfg(feature = "parquet")]
    {
        let model = cjlib::CityModel::from_file("tests/data/v2_0/minimal.city.json")?;
        cjlib::parquet::to_file("tests/output/minimal.cjparquet", &model)?;
    }

    Ok(())
}
