use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};

use serde_cityjson::deserialize_from_path;

/// Deserialize into a serde_json::Value.
fn serde_value<P: AsRef<Path>>(path: P) -> serde_json::Result<serde_json::Value> {
    let file = File::open(path.as_ref()).unwrap();
    let reader = BufReader::new(&file);
    let cm: serde_json::Value = serde_json::from_reader(reader)?;
    Ok(cm)
}

/// The measurement time needs to be large enough so that all samples can complete execution and a
/// bit more. However, [linear sampling](https://bheisler.github.io/criterion.rs/book/user_guide/advanced_configuration.html#sampling-mode)
/// requires a lot more time than flat sampling, but it produces much more reliable results and a
/// regression chart.
fn calculate_measurement_time(expected_time_per_test: Duration, sample_size: u32) -> Duration {
    expected_time_per_test * (sample_size as f32 * 7.0).floor() as u32
}

/// Benchmark with real data. Run 'just download' first to download the data files.
fn real_data(c: &mut Criterion) {
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("data")
        .join("downloaded");
    let warm_up_multiplier: u32 = 8;
    // Need to find a good balance for the sample size because one test takes long.
    let sample_size: u32 = 12;
    let sampling_mode = SamplingMode::Linear;

    let mut group_3dbag = c.benchmark_group("3DBAG");
    let p_json = data_dir.join("10-356-724.city.json");
    // Measured about 140ms runtime per test on a laptop.
    let expected_time_per_test = Duration::from_millis(140);
    group_3dbag.sample_size(sample_size as usize);
    group_3dbag.warm_up_time(expected_time_per_test * warm_up_multiplier);
    group_3dbag.measurement_time(calculate_measurement_time(
        expected_time_per_test,
        sample_size,
    ));
    group_3dbag.sampling_mode(sampling_mode);
    group_3dbag.bench_function("serde_cityjson", |b| {
        b.iter_with_large_drop(|| {
            let cm = deserialize_from_path(black_box(&p_json)).unwrap();
            black_box(&cm);
        })
    });
    group_3dbag.bench_function("serde_json::Value", |b| {
        b.iter_with_large_drop(|| {
            let cm = serde_value(black_box(&p_json)).unwrap();
            black_box(&cm);
        })
    });
    group_3dbag.finish();

    let mut group_3dbvz = c.benchmark_group("3D Basisvoorziening");
    let p_json = data_dir.join("30gz1_04.json");
    // Measured about 7-8s of runtime per test on a laptop.
    let expected_time_per_test = Duration::new(9, 0);
    group_3dbvz.sample_size(sample_size as usize);
    group_3dbvz.warm_up_time(expected_time_per_test * warm_up_multiplier);
    group_3dbvz.measurement_time(calculate_measurement_time(
        expected_time_per_test,
        sample_size,
    ));
    group_3dbvz.sampling_mode(sampling_mode);
    group_3dbvz.bench_function("serde_cityjson", |b| {
        b.iter_with_large_drop(|| {
            let cm = deserialize_from_path(black_box(&p_json)).unwrap();
            black_box(&cm);
        })
    });
    group_3dbvz.bench_function("serde_json::Value", |b| {
        b.iter_with_large_drop(|| {
            let cm = serde_value(black_box(&p_json)).unwrap();
            black_box(&cm);
        })
    });
    group_3dbvz.finish();
}

criterion_group!(benches, real_data);
criterion_main!(benches);
