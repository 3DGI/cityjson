## Run Metadata  
- Source: `bench_results/history.csv`  
- Timestamp: `2026-02-06T12:02:55Z`  
- Commit: `02f154e`  
- Description: `attribute-pool`  
- Mode: `full`  
- Bench version: `v2`  
- Seed: `12345`  
- Rust: `rustc 1.93.0 (254b59607 2026-01-19)`  

## High-Level Takeaways
- The default backend still leads in build-heavy and simple traversal workloads: `compute_mean_coordinates` is 58.7% faster, `builder/build_minimal_geometry` is 30.5% faster, and `builder/build_full_feature` is 30.7% faster than nested.  
- Nested now has a clearer lead in complex processing and end-to-end streaming: `compute_full_feature_stats` is 8.7% faster and `streaming/e2e` is 23.1% faster than default.  
- Default still has lower memory footprint in this run: `heap_max_bytes` is 28.1% lower, `heap_total_bytes` is 9.1% lower, `heap_max_blocks` is 23.4% lower, and `heap_total_blocks` is 5.3% lower than nested.  
- In `processor/compute_full_feature_stats`, default shows lower cache miss rates (D1: -28.3%, LL: -41.8%) but slightly higher branch miss rate (+5.5%) versus nested.  

## Change vs Inline-Attributes Baseline
- Baseline used here is the average of the three earlier `inline-attributes` full runs in `history.csv` (timestamps `2026-02-06T05:14:20Z`, `2026-02-06T05:34:42Z`, `2026-02-06T05:52:27Z`).  
- Default backend regressed in build throughput and time: `builder/build_minimal_geometry` time is +27.6% and `builder/build_full_feature` time is +28.0% (throughput down ~21.6% to 21.9%).  
- Default memory allocation activity increased: `heap_total_bytes` is +10.3%, `heap_max_blocks` is +22.5%, and `heap_total_blocks` is +32.5% (with `heap_max_bytes` essentially unchanged at +0.05%).  
- Nested backend remained close to baseline on build and memory metrics (roughly within +/-2% for builder timings; memory metrics unchanged).  
- Streaming improved for both backends by about 20% (`time_ms`: default -20.0%, nested -19.9%).  
- Cache miss rates improved significantly for both backends, especially default (`cache_d1_miss_rate`: -51.3%, `cache_ll_miss_rate`: -61.8% vs inline baseline averages).  

## Interpretation  
- The attribute-pool change appears to trade default-backend construction efficiency for better locality in processor-heavy paths and better streaming behavior.  
- Because nested remains mostly stable versus baseline while default shifts materially in builder and allocation-block metrics, the regressions are likely concentrated in default-specific construction/allocation paths rather than a global benchmark environment change.  
- The current profile is mixed: strong wins in streaming and cache locality, but meaningful default-backend build regressions. Prioritize default builder/allocation instrumentation before treating this as a net performance improvement.  
