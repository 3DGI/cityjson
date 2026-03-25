#![allow(unexpected_cfgs)]

fn main() -> cjlib::Result<()> {
    #[cfg(feature = "arrow")]
    {
        let model = cjlib::arrow::from_file("tests/data/v2_0/minimal.cjarrow")?;
        cjlib::arrow::to_file("tests/output/minimal.cjarrow", &model)?;
        println!(
            "arrow transport loaded {} CityObjects",
            model.as_inner().cityobjects().len()
        );
    }

    #[cfg(feature = "parquet")]
    {
        let model = cjlib::parquet::from_file("tests/data/v2_0/minimal.cjparquet")?;
        cjlib::parquet::to_file("tests/output/minimal.cjparquet", &model)?;
        println!(
            "parquet transport loaded {} CityObjects",
            model.as_inner().cityobjects().len()
        );
    }

    Ok(())
}
