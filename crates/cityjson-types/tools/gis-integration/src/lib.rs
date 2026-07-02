pub fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://cityjson:cityjson@localhost:5432/cityjson_gis".to_owned())
}
