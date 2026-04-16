#![allow(unexpected_cfgs)]
#![allow(clippy::unnecessary_wraps)]

fn main() -> cityjson_lib::Result<()> {
    #[cfg(feature = "arrow")]
    {
        let model = cityjson_lib::json::from_file("tests/data/v2_0/minimal.city.json")?;
        cityjson_lib::arrow::to_file("tests/output/minimal.cjarrow", &model)?;
    }

    #[cfg(feature = "parquet")]
    {
        let model = cityjson_lib::json::from_file("tests/data/v2_0/minimal.city.json")?;
        cityjson_lib::parquet::to_file("tests/output/minimal.cjparquet", &model)?;
    }

    Ok(())
}
