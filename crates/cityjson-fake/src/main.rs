use clap::Parser;

fn main() {
    let cli = cjfake::cli::Cli::parse();
    if let Err(err) = cjfake::cli::run(cli) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
