use rand::SeedableRng;
use rand::rngs::StdRng;
use std::env;

pub const BENCH_VERSION: &str = "v2";

pub const DEFAULT_SEED: u64 = 12345;

pub const DEFAULT_SIZE_BUILDER: usize = 10_000;
pub const FAST_SIZE_BUILDER: usize = 1_000;

pub const DEFAULT_SIZE_MEMORY: usize = 7_000;
pub const FAST_SIZE_MEMORY: usize = 1_000;

pub const DEFAULT_SIZE_PROCESSOR: usize = 10_000;
pub const FAST_SIZE_PROCESSOR: usize = 1_000;

pub const CUBE_VERTICES: [(f64, f64, f64); 8] = [
    (0.0, 0.0, 0.0),
    (1000.0, 0.0, 0.0),
    (1000.0, 1000.0, 0.0),
    (0.0, 1000.0, 0.0),
    (0.0, 0.0, 500.0),
    (1000.0, 0.0, 500.0),
    (1000.0, 1000.0, 500.0),
    (0.0, 1000.0, 500.0),
];

#[derive(Clone, Copy, Debug)]
pub struct BenchParams {
    pub size: usize,
    pub seed: u64,
}

#[must_use]
pub fn params_from_env(default_size: usize, fast_size: usize) -> BenchParams {
    let mode = env::var("BENCH_MODE").unwrap_or_else(|_| "full".to_string());

    let size = env::var("BENCH_SIZE")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(|| {
            if mode == "fast" {
                fast_size
            } else {
                default_size
            }
        });

    let seed = env::var("BENCH_SEED")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_SEED);

    BenchParams { size, seed }
}

#[must_use]
pub fn rng_from_seed(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}
