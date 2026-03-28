use std::path::PathBuf;

pub fn data_root() -> PathBuf {
    PathBuf::from("/home/balazs/Data/3DBAG_3dtiles_test/cjindex")
}

pub fn feature_files_root() -> PathBuf {
    data_root().join("feature-files")
}

pub fn cityjson_root() -> PathBuf {
    data_root().join("cityjson")
}

pub fn ndjson_root() -> PathBuf {
    data_root().join("ndjson")
}
