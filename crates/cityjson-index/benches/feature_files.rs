use criterion::{Criterion, criterion_group, criterion_main};

mod support;

use support::{LayoutKind, bench_layout};

fn bench_feature_files(c: &mut Criterion) {
    bench_layout(c, LayoutKind::FeatureFiles);
}

criterion_group!(benches, bench_feature_files);
criterion_main!(benches);
