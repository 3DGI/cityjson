#![allow(unexpected_cfgs)]

fn main() -> cjlib::Result<()> {
    #[cfg(feature = "arrow")]
    {
        let model = cjlib::arrow::from_file("tests/data/v2_0/minimal.cjarrow")?;
        println!("arrow transport loaded {} CityObjects", model.cityobjects().len());
    }

    #[cfg(feature = "parquet")]
    {
        let model = cjlib::parquet::from_file("tests/data/v2_0/minimal.cjparquet")?;
        println!(
            "parquet transport loaded {} CityObjects",
            model.cityobjects().len()
        );
    }

    Ok(())
}
