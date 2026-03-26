//! Benchmark the execution speed with criterion.rs.
//! Run 'just download' and 'just download-legacy' first to download the data files.
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, SamplingMode};

fn read_file<P: AsRef<Path>>(path: P) -> String {
    let mut s = String::new();
    File::open(path.as_ref())
        .unwrap()
        .read_to_string(&mut s)
        .unwrap();
    s
}

/// The measurement time needs to be large enough so that all samples can complete execution and a
/// bit more. However, [linear sampling](https://bheisler.github.io/criterion.rs/book/user_guide/advanced_configuration.html#sampling-mode)
/// requires a lot more time than flat sampling, but it produces much more reliable results and a
/// regression chart.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn calculate_measurement_time(expected_time_per_test: Duration, sample_size: u32) -> Duration {
    expected_time_per_test * (sample_size as f32 * 7.0).floor() as u32
}

/// Benchmark with real data. Run 'just download' and 'just download-legacy' first.
fn real_data(c: &mut Criterion) {
    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("downloaded");
    let legacy_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("downloaded_legacy");
    let warm_up_multiplier: u32 = 8;
    // Need to find a good balance for the sample size because one test takes long.
    let sample_size: u32 = 10;
    let sampling_mode = SamplingMode::Flat;

    let mut group_3dbag = c.benchmark_group("3DBAG");
    let content_3dbag = read_file(data_dir.join("10-356-724.city.json"));
    // Measured about 140ms runtime per test on a laptop.
    let expected_time_per_test = Duration::from_millis(140);
    group_3dbag.sample_size(sample_size as usize);
    group_3dbag.warm_up_time(expected_time_per_test * warm_up_multiplier);
    group_3dbag.measurement_time(calculate_measurement_time(
        expected_time_per_test,
        sample_size,
    ));
    group_3dbag.sampling_mode(sampling_mode);
    group_3dbag.bench_function("serde_cityjson/owned", |b| {
        b.iter_with_large_drop(|| {
            serde_cityjson::from_str_owned(black_box(&content_3dbag)).unwrap()
        });
    });
    group_3dbag.bench_function("serde_cityjson/borrowed", |b| {
        b.iter_with_large_drop(|| {
            serde_cityjson::from_str_borrowed(black_box(&content_3dbag)).unwrap()
        });
    });
    group_3dbag.bench_function("serde_json::Value", |b| {
        b.iter_with_large_drop(|| {
            serde_json::from_str::<serde_json::Value>(black_box(&content_3dbag)).unwrap()
        });
    });
    group_3dbag.finish();

    let mut group_3dbvz = c.benchmark_group("3D Basisvoorziening");
    let content_3dbvz = read_file(data_dir.join("30gz1_04.city.json"));
    // Measured about 7-8s of runtime per test on a laptop.
    let _expected_time_per_test = Duration::new(5, 0);
    group_3dbvz.sample_size(sample_size as usize);
    // group_3dbvz.warm_up_time(expected_time_per_test * warm_up_multiplier);
    // group_3dbvz.measurement_time(calculate_measurement_time(
    //     expected_time_per_test,
    //     sample_size,
    // ));
    group_3dbvz.sampling_mode(sampling_mode);
    group_3dbvz.bench_function("serde_cityjson/owned", |b| {
        b.iter_with_large_drop(|| {
            serde_cityjson::from_str_owned(black_box(&content_3dbvz)).unwrap()
        });
    });
    // serde_cityjson/borrowed is omitted: this file contains JSON-escaped strings
    // (Dutch special characters), which BorrowedStringStorage rejects with an error.
    group_3dbvz.bench_function("serde_json::Value", |b| {
        b.iter_with_large_drop(|| {
            serde_json::from_str::<serde_json::Value>(black_box(&content_3dbvz)).unwrap()
        });
    });
    group_3dbvz.finish();

    // --- v0.4.5 legacy groups (v1.1 data, pre-cityjson-rs refactor) ---
    // Run 'just download-legacy' to populate tests/data/downloaded_legacy/.
    // These groups are skipped gracefully if the legacy data is not present.

    let legacy_3dbag = legacy_dir.join("10-356-724.city.json");
    if legacy_3dbag.exists() {
        let content_3dbag_v11 = read_file(&legacy_3dbag);
        let mut group = c.benchmark_group("3DBAG (v0.4.5)");
        group.sample_size(sample_size as usize);
        group.warm_up_time(expected_time_per_test * warm_up_multiplier);
        group.measurement_time(calculate_measurement_time(
            expected_time_per_test,
            sample_size,
        ));
        group.sampling_mode(sampling_mode);
        group.bench_function("serde_cityjson_legacy/from_str", |b| {
            b.iter_with_large_drop(|| {
                serde_cityjson_legacy::from_str(black_box(&content_3dbag_v11)).unwrap()
            });
        });
        group.bench_function("serde_json::Value", |b| {
            b.iter_with_large_drop(|| {
                serde_json::from_str::<serde_json::Value>(black_box(&content_3dbag_v11)).unwrap()
            });
        });
        group.finish();
    }

    let legacy_3dbvz = legacy_dir.join("30gz1_04.city.json");
    if legacy_3dbvz.exists() {
        let content_3dbvz_v11 = read_file(&legacy_3dbvz);
        let mut group = c.benchmark_group("3D Basisvoorziening (v0.4.5)");
        group.sample_size(sample_size as usize);
        group.sampling_mode(sampling_mode);
        group.bench_function("serde_cityjson_legacy/from_str", |b| {
            b.iter_with_large_drop(|| {
                serde_cityjson_legacy::from_str(black_box(&content_3dbvz_v11)).unwrap()
            });
        });
        group.bench_function("serde_json::Value", |b| {
            b.iter_with_large_drop(|| {
                serde_json::from_str::<serde_json::Value>(black_box(&content_3dbvz_v11)).unwrap()
            });
        });
        group.finish();
    }
}

criterion_group!(benches, real_data);
criterion_main!(benches);
