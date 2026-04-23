use cityjson_index::benchmark::{BenchmarkCli, print_report, run};
use clap::Parser;

fn main() -> cityjson_lib::Result<()> {
    let cli = BenchmarkCli::parse();
    let json = cli.json;
    let report = run(&cli)?;
    print_report(&report, json)
}
