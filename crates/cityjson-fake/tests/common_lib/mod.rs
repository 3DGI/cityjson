use cjval::CJValidator;
use std::path::PathBuf;

#[allow(dead_code)]
pub fn invalids_dir() -> PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("invalids")
}

#[allow(dead_code)]
pub fn count_invalids(invalids_dir: &PathBuf) -> usize {
    let mut count: usize = 0;
    for entry in std::fs::read_dir(invalids_dir).unwrap().flatten() {
        let p = entry.path();
        if p.extension().is_some_and(|ext| ext == "json") {
            let name = p.file_name().unwrap();
            let d = name
                .to_string_lossy()
                .replace("cjfake_invalid_", "")
                .replace(".city.json", "");
            if let Ok(c) = d.parse::<usize>() {
                if c > count {
                    count = c;
                }
            }
        }
    }
    count
}

/// Validate a `CityJSON` str with [cjval]. If the `CityJSON` is invalid, serialize it for
/// later analysis.
#[allow(dead_code)]
pub fn validate(cityjson_str: &str, test_name: &str) {
    let val = CJValidator::from_str(cityjson_str);
    // assert!(val.validate().iter().all(|(c, s)| s.is_valid()));
    let invalids: Vec<(String, String)> = val
        .validate()
        .into_iter()
        .filter(|(_, summary)| !summary.is_valid())
        .map(|(criterion, summary)| (criterion, summary.to_string()))
        .collect();
    if !invalids.is_empty() {
        // Serialize invalid citymodels for later analysis
        let idir = invalids_dir();
        let invalids_count = count_invalids(&idir);
        let current_invalid_nr = invalids_count + 1;
        let fname = format!("{test_name}_{current_invalid_nr}.city.json");
        std::fs::write(idir.join(&fname), cityjson_str).unwrap();
        println!("Serialized invalid CityJSON to {fname}");
    }
    for (criterion, summary) in &val.validate() {
        assert!(
            summary.is_valid(),
            "{criterion} is not valid with {summary}"
        );
    }
}
