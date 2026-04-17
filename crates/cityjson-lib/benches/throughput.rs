#[path = "support/mod.rs"]
mod support;

use std::time::Duration;

use criterion::{Criterion, SamplingMode, Throughput, criterion_group, criterion_main};
use support::{
    READ_WORKLOADS, WRITE_WORKLOADS, benchmark_cases, prepare_workload, run_workload,
    throughput_bytes,
};

fn configure_group(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    group.sample_size(10);
    group.warm_up_time(Duration::from_secs(3));
    group.measurement_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
}

fn bench_throughput(c: &mut Criterion) {
    let cases = benchmark_cases();

    for case in &cases {
        let mut read_group = c.benchmark_group(format!("deserialize/{}", case.id));
        read_group.plot_config(
            criterion::PlotConfiguration::default()
                .summary_scale(criterion::AxisScale::Logarithmic),
        );
        configure_group(&mut read_group);

        for workload in READ_WORKLOADS {
            let prepared = prepare_workload(case, workload);
            read_group.throughput(Throughput::Bytes(throughput_bytes(case, workload)));
            read_group.bench_function(workload.label(), |b| {
                b.iter_with_large_drop(|| run_workload(&prepared));
            });
        }
        read_group.finish();

        let mut write_group = c.benchmark_group(format!("serialize/{}", case.id));
        write_group.plot_config(
            criterion::PlotConfiguration::default()
                .summary_scale(criterion::AxisScale::Logarithmic),
        );
        configure_group(&mut write_group);

        for workload in WRITE_WORKLOADS {
            let prepared = prepare_workload(case, workload);
            write_group.throughput(Throughput::Bytes(throughput_bytes(case, workload)));
            write_group.bench_function(workload.label(), |b| {
                b.iter_with_large_drop(|| run_workload(&prepared));
            });
        }
        write_group.finish();
    }
}

criterion_group!(benches, bench_throughput);
criterion_main!(benches);
