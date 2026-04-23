#[path = "support/mod.rs"]
mod support;

use std::env;
use std::time::Instant;

use support::{Workload, load_case, prepare_workload, run_workload, throughput_bytes};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProfileMode {
    None,
    Dhat,
}

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    let args = parse_args(&raw_args);
    let case = load_case(&args.case_id);
    let prepared = prepare_workload(&case, args.workload);

    let _profiler = match args.profile_mode {
        ProfileMode::None => None,
        ProfileMode::Dhat => Some(dhat::Profiler::new_heap()),
    };

    let started = Instant::now();
    for _ in 0..args.iterations {
        run_workload(&prepared);
    }
    let elapsed = started.elapsed();

    let elapsed_ns = elapsed.as_nanos();
    let throughput_bytes = throughput_bytes(&case, args.workload);

    println!("{{");
    println!("  \"case\": \"{}\",", case.id);
    println!("  \"workload\": \"{}\",", args.workload.label());
    println!("  \"profile\": \"{}\",", args.profile_mode.label());
    println!("  \"iterations\": {},", args.iterations);
    println!("  \"bytes_per_iteration\": {throughput_bytes},");
    println!("  \"elapsed_ns\": {elapsed_ns},");
    println!(
        "  \"elapsed_per_iteration_ns\": {}",
        elapsed_ns / u128::from(args.iterations)
    );
    println!("}}");
}

#[derive(Debug)]
struct Args {
    case_id: String,
    workload: Workload,
    iterations: u64,
    profile_mode: ProfileMode,
}

fn parse_args(args: &[String]) -> Args {
    let mut case_id = "io_3dbag_cityjson".to_string();
    let mut workload = None;
    let mut iterations = 1_u64;
    let mut profile_mode = ProfileMode::None;

    let mut index = 0_usize;
    while index < args.len() {
        match args[index].as_str() {
            "--case" => {
                index += 1;
                case_id = value(args, index, "--case").to_string();
            }
            "--workload" => {
                index += 1;
                workload = Some(
                    value(args, index, "--workload")
                        .parse()
                        .unwrap_or_else(|error: String| panic!("{error}")),
                );
            }
            "--iterations" => {
                index += 1;
                iterations = value(args, index, "--iterations")
                    .parse()
                    .unwrap_or_else(|error| panic!("invalid iterations: {error}"));
                assert!(iterations > 0, "iterations must be greater than zero");
            }
            "--profile" => {
                index += 1;
                profile_mode = match value(args, index, "--profile") {
                    "none" => ProfileMode::None,
                    "dhat" => ProfileMode::Dhat,
                    other => panic!("unknown profile mode '{other}'"),
                };
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => panic!("unknown argument '{other}'"),
        }
        index += 1;
    }

    Args {
        case_id,
        workload: workload.unwrap_or_else(|| {
            panic!("missing required --workload argument; run with --help for valid values")
        }),
        iterations,
        profile_mode,
    }
}

fn value<'a>(args: &'a [String], index: usize, flag: &str) -> &'a str {
    args.get(index)
        .map_or_else(|| panic!("missing value for {flag}"), String::as_str)
}

fn print_usage() {
    println!("Usage:");
    println!(
        "  cargo bench --bench profile -- --workload <name> [--case <id>] [--iterations <n>] [--profile none|dhat]"
    );
    println!();
    println!("Cases:");
    println!("  io_3dbag_cityjson");
    println!("  io_3dbag_cityjson_cluster_4x");
    println!();
    println!("Workloads:");
    println!("  serde_json-read");
    println!("  cityjson_lib-read");
    println!("  cityjson-lib-json-read");
    println!("  serde_json-write");
    println!("  cityjson_lib-write");
    println!("  cityjson-lib-json-write");
    println!("  cityarrow-read");
    println!("  cityarrow-write");
}

impl ProfileMode {
    fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Dhat => "dhat",
        }
    }
}
