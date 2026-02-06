## Run Metadata  
- Source: `bench_results/history.csv`  
- Timestamp: `2026-02-06T05:14:20Z`  
- Commit: `782f331`  
- Mode: `full`  
- Bench version: `v2`  
- Seed: `12345`  
- Rust: `rustc 1.93.0 (254b59607 2026-01-19)`  
  
## High-Level Takeaways
- The default (flattened) backend is clearly faster for build-heavy workloads and simple coordinate reductions.  
- Memory usage is materially lower with the default backend across all heap metrics.  
- Two workloads are near parity or slightly favor nested: `compute_full_feature_stats` and `streaming/e2e`. The deltas are small enough to treat as parity unless they persist across multiple runs.  
- Cache-miss metrics in `processor/compute_full_feature_stats` slightly favor nested, consistent with the near-parity timing.

## Interpretation  
- The default backend’s flattened layout shows strong advantages for construction and simple per-vertex reductions, which are typical of data processing pipelines that build and scan models.  
- The near-parity results in `compute_full_feature_stats` suggest that this workload is either not layout-sensitive or is dominated by work that does not benefit strongly from the columnar structure.  
- The small nested advantage in streaming is modest and could be noise or a sign of avoidable overhead in the default streaming path (e.g., resource lookups or batching). It’s not large enough to contradict the overall trend.
