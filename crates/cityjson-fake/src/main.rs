use clap::Parser;

fn main() {
    let cli = cityjson_fake::cli::Cli::parse();
    if let Err(err) = cityjson_fake::cli::run(cli) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
