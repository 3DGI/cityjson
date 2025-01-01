use std::path::PathBuf;
use cjval::CJValidator;

pub fn invalids_dir() -> PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("invalids")
}

pub fn count_invalids(invalids_dir: &PathBuf) -> usize {
    let mut count: usize = 0;
    for entry in std::fs::read_dir(invalids_dir).unwrap() {
        if entry.is_ok() {
            let p = entry.unwrap().path();
            if p.extension().is_some_and(|ext| ext == "json") {
                let name = p.file_name().unwrap();
                let d = name
                    .to_string_lossy()
                    .replace("cjfake_invalid_", "")
                    .replace(".city.json", "");
                let c = d.parse::<usize>().unwrap();
                if c > count {
                    count = c;
                }
            }
        }
    }
    count
}

/// Validate a CityJSON str with [cjval]. If the CityJSON is invalid, serialize it for
/// later analysis.
pub fn validate(cityjson_str: &str, test_name: &str) {
    let val = CJValidator::from_str(&cityjson_str);
    // assert!(val.validate().iter().all(|(c, s)| s.is_valid()));
    let invalids: Vec<(String, String)> = val
        .validate()
        .into_iter()
        .filter(|(_, summary)| !summary.is_valid())
        .map(|(criterion, summary)| (criterion, summary.to_string()))
        .collect();
    if invalids.len() > 0 {
        // Serialize invalid citymodels for later analysis
        let idir = invalids_dir();
        let invalids_count = count_invalids(&idir);
        let current_invalid_nr = invalids_count + 1;
        let fname = format!("{}_{}.city.json", test_name, current_invalid_nr);
        std::fs::write(idir.join(&fname), cityjson_str).unwrap();
        println!("Serialized invalid CityJSON to {}", &fname);
    }
    for (criterion, summary) in val.validate().iter() {
        assert!(
            summary.is_valid(),
            "{} is not valid with {}",
            criterion,
            summary
        )
    }
}