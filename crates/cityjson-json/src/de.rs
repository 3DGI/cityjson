mod attributes;
mod build;
mod cityobjects;
mod geometry;
mod parse;
mod profiling;
mod root;
mod sections;
mod validation;

pub(crate) use parse::from_str_owned;

#[cfg(test)]
mod perf_probe {
    use std::fs;
    use std::path::PathBuf;
    use std::time::Instant;

    use cityjson::prelude::OwnedStringStorage;

    use super::build::build_model;
    use super::profiling;
    use super::root::parse_root;

    fn data_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("downloaded")
            .join(name)
    }

    fn measure<F, T>(label: &str, f: F) -> std::time::Duration
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let _ = f();
        let elapsed = start.elapsed();
        eprintln!("{label:<18}: {elapsed:?}");
        elapsed
    }

    fn print_profile() {
        let mut entries = profiling::snapshot();
        entries.sort_by(|a, b| b.1.total.cmp(&a.1.total).then_with(|| a.0.cmp(b.0)));
        for (label, record) in entries {
            eprintln!(
                "  {label:<34} total={:?} count={}",
                record.total, record.count
            );
        }
    }

    #[test]
    #[ignore = "manual timing probe"]
    fn probe_deser_breakdown_3dbag() {
        let input = fs::read_to_string(data_path("10-356-724.city.json")).unwrap();

        measure("serde_json::Value", || {
            serde_json::from_str::<serde_json::Value>(&input).unwrap()
        });
        measure("parse_root", || parse_root(&input).unwrap());
        profiling::reset();
        measure("build_model", || {
            let raw = parse_root(&input).unwrap();
            build_model::<OwnedStringStorage>(raw).unwrap()
        });
        print_profile();
        measure("from_str_owned", || super::from_str_owned(&input).unwrap());
    }

    #[test]
    #[ignore = "manual timing probe"]
    fn probe_deser_breakdown_3dbvz() {
        let input = fs::read_to_string(data_path("30gz1_04.city.json")).unwrap();

        measure("serde_json::Value", || {
            serde_json::from_str::<serde_json::Value>(&input).unwrap()
        });
        measure("parse_root", || parse_root(&input).unwrap());
        profiling::reset();
        measure("build_model", || {
            let raw = parse_root(&input).unwrap();
            build_model::<OwnedStringStorage>(raw).unwrap()
        });
        print_profile();
        measure("from_str_owned", || super::from_str_owned(&input).unwrap());
    }
}
