fn main() {
    if let Err(error) = cityjson_export::run(std::env::args()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
