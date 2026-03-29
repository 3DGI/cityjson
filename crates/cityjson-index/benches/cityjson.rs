mod support;

use criterion::{Criterion, criterion_group, criterion_main};
use support::{LayoutKind, bench_layout};

fn bench_cityjson(c: &mut Criterion) {
    bench_layout(c, LayoutKind::CityJson);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_cityjson
}
criterion_main!(benches);
