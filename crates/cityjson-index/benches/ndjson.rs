mod support;

use criterion::{Criterion, criterion_group, criterion_main};
use support::{LayoutKind, bench_layout};

fn bench_ndjson(c: &mut Criterion) {
    bench_layout(c, LayoutKind::Ndjson);
}

criterion_group!(benches, bench_ndjson);
criterion_main!(benches);
