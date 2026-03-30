# Performance Testing Guide for cjindex

This guide explains how to use the simple performance testing script to benchmark cjindex on large datasets.

## Overview

The `perf-test` binary provides simple performance measurements for:
- **GET operations**: Random feature lookups by ID
- **QUERY operations**: Bounding box (BBox) based spatial queries

It measures:
- Warmup time (to account for caching and JIT)
- Total operation time
- Per-operation latency (milliseconds)
- Throughput (operations per second)

## Building the Binary

```bash
cargo build --release --bin perf-test
```

The binary will be available at `target/release/perf-test`.

## Usage

```bash
./target/release/perf-test <DATASET_DIR>
```

### Arguments

- `DATASET_DIR`: Path to a cjindex dataset directory
  - The script auto-detects the storage layout (NDJSON, CityJSON, or feature-files)
  - Creates/uses a sidecar SQLite index at `<DATASET_DIR>/.cjindex.sqlite`

## How It Works

1. **Dataset Resolution**: Auto-detects the storage layout under the dataset directory
2. **Index Building**: Creates or rebuilds the SQLite index (reindexing)
3. **Test Workload Selection**:
   - Tries to use the realistic workload from feature-files if available
   - Falls back to sampling from the SQLite index if not available
4. **Performance Measurement**:
   - Runs warmup rounds to stabilize cache behavior
   - Runs measured rounds and reports statistics
   - Includes both aggregate and per-operation metrics

## Output Format

The script outputs:

### Initial Setup
```
Loading dataset from: <path>
Storage layout: ndjson|cityjson|feature-files
Index path: <path>

Reindex took: X.XXs
```

### GET Performance Test
```
Testing N get operations...
Warmup (10 ops): X.XXXXs (X.XXXXms/op)
Measured (N ops): X.XXXXs
  - Total: X.XXs
  - Per operation: X.XXXXms
  - Throughput: N ops/sec
```

### QUERY (BBox) Performance Test
```
Testing M query operations...
Warmup (3 ops): X.XXXXs (X.XXXXms/op)
Measured (M ops): X.XXXXs
  - Total: X.XXs
  - Per operation: X.XXXXms
  - Throughput: M ops/sec
  - Total results returned: K
  - Avg results per query: K/M
```

### Summary
```
GET:   X.XXXXms/op (N ops/sec)
QUERY: X.XXXXms/op (M ops/sec)
```

## Example: Testing a Large Dataset

### Scenario: Testing NDJSON layout with 270k CityObjects

```bash
# Assuming the dataset is at /data/cityobjects

./target/release/perf-test /data/cityobjects
```

Output might look like:
```
Loading dataset from: /data/cityobjects
Storage layout: ndjson
Index path: /data/cityobjects/.cjindex.sqlite

Reindex took: 45.32s
Building test workload...
Using realistic workload from feature-files layout
Test workload has 1000 get IDs and 191 bbox queries

--- GET Performance Test ---
Testing 1000 get operations...
Warmup (10 ops): 0.0523s (5.2314ms/op)
Measured (1000 ops): 4.8932s
  - Total: 4.89s
  - Per operation: 4.8932ms
  - Throughput: 204 ops/sec

--- QUERY (BBox) Performance Test ---
Testing 191 query operations...
Warmup (3 ops): 0.1842s (61.4011ms/op)
Measured (191 ops): 12.4567s
  - Total: 12.46s
  - Per operation: 65.1809ms
  - Throughput: 15 ops/sec
  - Total results returned: 15886
  - Avg results per query: 83.2
```

## Notes on Large Datasets

- **First Run (Reindexing)**: The first time you run the test on a dataset, it will reindex. This can take a while on very large datasets (10s to minutes).
- **Subsequent Runs**: Subsequent runs are much faster since the index already exists.
- **Disk I/O**: Performance is heavily influenced by disk I/O characteristics. SSDs will significantly outperform HDDs.
- **Caching**: The warmup phase helps mitigate CPU cache effects. Results after warmup are more representative of sustained performance.
- **Memory**: Ensure sufficient RAM; the index is loaded into memory.

## Test Workload Selection

The script prefers using a realistic workload:

1. **Feature-files layout**: Uses `realistic_workload::build_realistic_workload()`
   - Provides 1000 deterministic GET IDs
   - Provides ~191 deterministic BBOX queries
   - Best if your dataset is organized as feature-files

2. **Other layouts (NDJSON, CityJSON)**: Falls back to SQLite sampling
   - Samples up to 1000 feature IDs from the index
   - Generates 100 spatial bbox queries
   - Works on any layout

## Comparing Performance Across Layouts

To compare performance across all three supported layouts:

```bash
# Prepare data in all three layouts (if available in tests/data)
./target/release/perf-test tests/data/ndjson
./target/release/perf-test tests/data/cityjson
./target/release/perf-test tests/data/feature-files
```

## Troubleshooting

### "No features found in dataset"
- The dataset directory might not be recognized as a valid cjindex dataset
- Check that it contains valid CityJSON files in one of the supported layouts
- Verify the path is correct

### Timeout on reindex
- Large datasets (1M+ CityObjects) may take a long time to index
- Consider running in the background: `nohup ./target/release/perf-test /data/cityobjects > perf-test.log 2>&1 &`

### Very slow query performance
- Check available disk space and I/O throughput
- Monitor CPU and disk usage during the test
- Consider closing other applications that might compete for resources

## Future Enhancements

Possible additions to the performance test script:
- Command-line options to specify custom test workload sizes
- Output formats (CSV, JSON) for easy parsing
- Profiling and detailed breakdown by operation type
- Statistical analysis (percentiles, standard deviation)
- Comparison mode to track performance over time
