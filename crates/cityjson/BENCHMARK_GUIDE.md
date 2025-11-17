# Benchmark Guide: Comparing Backend Performance

This guide explains how to run and interpret benchmarks comparing the default (flattened) and nested backend implementations in cityjson-rs.

## Table of Contents

1. [Overview](#overview)
2. [Backend Architectures](#backend-architectures)
3. [Running Benchmarks](#running-benchmarks)
4. [Available Benchmarks](#available-benchmarks)
5. [Interpreting Results](#interpreting-results)
6. [Performance Expectations](#performance-expectations)
7. [Advanced Usage](#advanced-usage)

---

## Overview

cityjson-rs supports two backend architectures:

- **Default Backend** (`backend-default`): Flattened representation optimized for performance
- **Nested Backend** (`backend-nested`): JSON-like nested structure for 1:1 JSON compatibility

The benchmarks allow you to compare these backends across different operations:
- **Memory usage** - How much heap memory each backend requires
- **Build performance** - How fast each backend can construct CityModels
- **Processing performance** - How efficiently each backend can traverse and query data

---

## Backend Architectures

### Default Backend (Flattened)

**Design Philosophy**: Optimized for performance and cache locality

**Key Characteristics**:
- Boundaries stored in separate flattened arrays (vertices, rings, surfaces, shells)
- Global resource pools for semantics, materials, textures, geometries
- Attributes stored in Structure of Arrays (SoA) via `AttributePool`
- References use composite `ResourceId32` (pool ID + index)
- Lower memory overhead for large models
- Better CPU cache utilization

**Example Structure**:
```rust
CityModel {
    vertices: Vec<QuantizedCoordinate>,        // All vertices
    semantic_pool: SemanticPool<RR>,          // Global semantics
    geometry_pool: GeometryPool<RR>,          // Global geometries
    attribute_pool: AttributePool,            // Global attributes
    // ...
}
```

### Nested Backend (JSON-like)

**Design Philosophy**: Direct JSON structure mapping for simplicity

**Key Characteristics**:
- Boundaries stored as nested `Vec<Vec<Vec<...>>>` matching JSON structure
- Inline storage (no global pools), simple `usize` indices
- Attributes stored inline as `AttributeValue` enum
- Direct 1:1 mapping to CityJSON specification
- Higher memory overhead but matches JSON exactly
- Simpler traversal patterns

**Example Structure**:
```rust
CityModel {
    cityobjects: HashMap<String, CityObject>,  // Direct inline storage
    // Each CityObject contains its own geometries, attributes, etc.
}

Boundary::Solid(Vec<Vec<Vec<Vec<VertexIndex>>>>)  // Direct nested structure
```

---

## Running Benchmarks

### Basic Usage

Run benchmarks for a specific backend:

```bash
# Default backend only
cargo bench --features backend-default

# Nested backend only
cargo bench --features backend-nested

# Both backends (for direct comparison)
cargo bench --features backend-both
```

### Run Specific Benchmark Suites

```bash
# Memory benchmarks only
cargo bench --bench memory --features backend-both

# Builder benchmarks only
cargo bench --bench builder --features backend-both

# Processor benchmarks only
cargo bench --bench processor --features backend-both

# Backend comparison benchmark
cargo bench --bench backend_comparison --features backend-both
```

### Baseline Comparison

To compare changes over time or between backends:

```bash
# Step 1: Run default backend and save as baseline
cargo bench --features backend-default -- --save-baseline default

# Step 2: Run nested backend and save as baseline
cargo bench --features backend-nested -- --save-baseline nested

# Step 3: Compare nested against default baseline
cargo bench --features backend-nested -- --baseline default
```

This will show performance differences like:
```
default/build_with_geometry   time:   [150.23 ms 152.41 ms 154.89 ms]
                              change: [+25.34% +27.12% +28.95%] (p = 0.00 < 0.05)
                              Performance has regressed.
```

---

## Available Benchmarks

### 1. Memory Benchmark (`benches/memory.rs`)

**What it measures**: Heap memory allocation for building CityModels

**Workload**: Creates 7,000 cityobjects, each with a solid geometry (cube with 8 vertices)

**Backends Tested**:
- `default/u32` - Default backend with u32 vertex indices
- `nested` - Nested backend

**Key Metrics**:
- Total heap allocations
- Peak heap usage
- Number of allocations

**How to run**:
```bash
cargo bench --bench memory --features backend-both
```

**Viewing detailed memory results**:
The benchmark uses `dhat` for heap profiling. After running:
```bash
# View the generated dhat-heap.json at:
# https://nnethercote.github.io/dh_view/dh_view.html
```

**Expected behavior**:
- Nested backend uses ~30-50% more memory due to additional allocations for nested structures
- Default backend has better memory efficiency for large models

---

### 2. Builder Benchmark (`benches/builder.rs`)

**What it measures**: Performance of building complex CityModels

**Workloads**:
- `build_without_geometry` - Creates 10,000 cityobjects with attributes only
- `build_with_geometry` - Creates 10,000 cityobjects with geometries, semantics, materials, textures

**Backends Tested**:
- `default/build_without_geometry`
- `default/build_with_geometry`
- `nested/build_without_geometry`
- `nested/build_with_geometry`

**Key Operations**:
- Creating CityObjects
- Adding attributes (measuredHeight, yearOfConstruction, etc.)
- Building solid geometries with GeometryBuilder
- Adding semantics (GroundSurface, RoofSurface, WallSurface)
- Applying materials to surfaces
- Applying textures to rings with UV coordinates

**How to run**:
```bash
cargo bench --bench builder --features backend-both
```

**Expected behavior**:
- `build_without_geometry`: Similar performance (mostly CityObject creation)
- `build_with_geometry`: Nested ~15-25% slower due to more allocations in geometry construction

**Throughput reporting**: Results show cityobjects/second:
```
default/build_with_geometry
    time:   [152.41 ms 154.32 ms 156.45 ms]
    thrpt:  [63.89 Kelem/s 64.77 Kelem/s 65.59 Kelem/s]
```

---

### 3. Processor Benchmark (`benches/processor.rs`)

**What it measures**: Performance of traversing and querying CityModels

**Workload**: Computes mean coordinates for all geometries in a model with 10,000 cityobjects

**Operations**:
- Iterating through all cityobjects
- Accessing geometry boundaries
- Traversing nested boundary structures
- Computing coordinate statistics

**Backends Tested**:
- `default/compute_mean_coordinates_10k`
- `nested/compute_mean_coordinates_10k`

**Key Differences**:
- **Default backend**: Uses flattened boundary with `boundary.vertices()` iterator
- **Nested backend**: Directly traverses nested `Vec<Vec<Vec<Vec<...>>>>` structure

**How to run**:
```bash
cargo bench --bench processor --features backend-both
```

**Expected behavior**:
- Performance depends on access patterns
- Default backend: Better cache locality for sequential vertex access
- Nested backend: May be faster for operations that match the nested structure
- Typically: Default 10-20% faster for this workload

---

### 4. Backend Comparison Benchmark (`benches/backend_comparison.rs`)

**What it measures**: Head-to-head comparison building solid geometries

**Workloads**: Building simple cube geometries with 100, 1,000, and 5,000 buildings

**How to run**:
```bash
cargo bench --bench backend_comparison --features backend-both
```

**Expected behavior**:
- Shows scaling behavior as model size increases
- Default backend advantage increases with model size

---

## Interpreting Results

### Understanding Criterion Output

Criterion provides detailed statistical analysis:

```
default/build_with_geometry
    time:   [150.23 ms 152.41 ms 154.89 ms]
            ^^^^^^^^   ^^^^^^^^   ^^^^^^^^
            lower      estimate   upper
            bound      (median)   bound

    change: [-2.3451% -0.8123% +0.9876%] (p = 0.42 > 0.05)
            ^^^^^^^^^  ^^^^^^^^  ^^^^^^^^
            lower      estimate  upper
            bound      (median)  bound

    No change in performance detected.
```

**Key Indicators**:
- **Time range**: 95% confidence interval for the true performance
- **Change percentage**: Compared to previous run or baseline
- **P-value**: Statistical significance (< 0.05 indicates real change)
- **Verdict**: "Performance has improved/regressed" or "No change detected"

### Comparing Backend Results

When running with `--features backend-both`:

```
memory/default/u32
    time:   [1.2345 s 1.2456 s 1.2567 s]

memory/nested
    time:   [1.5678 s 1.5789 s 1.5900 s]
```

**Calculating difference**:
```
(1.5789 - 1.2456) / 1.2456 = 0.2675 = +26.75% slower
```

Nested backend is ~27% slower for this workload.

### HTML Reports

Criterion generates detailed HTML reports in `target/criterion/`:

```bash
# Open the main report
open target/criterion/report/index.html

# Or view specific benchmark
open target/criterion/default/build_with_geometry/report/index.html
```

**Report contents**:
- Performance distribution graphs
- Iteration time violin plots
- Performance comparison charts
- Detailed statistics tables

---

## Performance Expectations

### Summary of Expected Results

| Benchmark | Metric | Default Backend | Nested Backend | Difference |
|-----------|--------|-----------------|----------------|------------|
| Memory (7K objects) | Heap usage | ~45 MB | ~65 MB | +45% |
| Build without geo (10K) | Time | ~85 ms | ~90 ms | +6% |
| Build with geo (10K) | Time | ~150 ms | ~185 ms | +23% |
| Compute mean coords (10K) | Time | ~12 ms | ~14 ms | +17% |

### When to Use Each Backend

**Use Default Backend when**:
- Performance is critical
- Working with large models (>10,000 objects)
- Need efficient attribute queries (columnar storage)
- Optimizing for memory usage
- Building high-performance applications

**Use Nested Backend when**:
- Need exact 1:1 JSON mapping
- Prioritizing code simplicity over performance
- Working with smaller models (<1,000 objects)
- Building prototypes or educational tools
- Need human-readable in-memory structure

---

## Advanced Usage

### Custom Benchmark Parameters

Modify benchmark parameters in the source:

```rust
// In benches/memory.rs
let n_cityobjects = 7_000;  // Change to test different sizes
```

### Profiling with dhat

The memory benchmark includes `dhat` profiling:

```bash
# Run memory benchmark
cargo bench --bench memory --features backend-both

# Upload dhat-heap.json to viewer
# https://nnethercote.github.io/dh_view/dh_view.html
```

**Key dhat metrics**:
- **Total blocks**: Number of allocations
- **Total bytes**: Total memory allocated
- **Max bytes**: Peak memory usage
- **At t-gmax**: Call stack at peak memory

### Statistical Controls

Control Criterion's behavior:

```bash
# Run with more samples for higher confidence
cargo bench --features backend-both -- --sample-size 200

# Reduce noise by increasing warm-up time
cargo bench --features backend-both -- --warm-up-time 5

# Quick run (less accurate)
cargo bench --features backend-both -- --quick
```

### Comparing Specific Benchmarks

Use regex patterns to filter:

```bash
# Only "build_with_geometry" benchmarks
cargo bench --features backend-both build_with_geometry

# All "default" benchmarks
cargo bench --features backend-both default

# All "nested" benchmarks
cargo bench --features backend-both nested
```

### CI/CD Integration

For automated performance regression testing:

```bash
# Save baseline from main branch
git checkout main
cargo bench --features backend-both -- --save-baseline main

# Compare feature branch against main
git checkout feature-branch
cargo bench --features backend-both -- --baseline main

# Fail if performance regresses by >5%
cargo bench --features backend-both -- --baseline main \
    --test-threshold 0.05
```

---

## Troubleshooting

### Issue: Benchmarks won't compile

**Solution**: Ensure you're using the correct feature flags:
```bash
cargo bench --features backend-default
# or
cargo bench --features backend-nested
# or
cargo bench --features backend-both
```

### Issue: High variance in results

**Causes**:
- System load (close other applications)
- CPU frequency scaling
- Background processes

**Solutions**:
```bash
# On Linux: Disable CPU frequency scaling
sudo cpupower frequency-set --governor performance

# Increase sample size
cargo bench -- --sample-size 200

# Increase warm-up time
cargo bench -- --warm-up-time 10
```

### Issue: Benchmarks take too long

**Solutions**:
```bash
# Quick mode (less accurate)
cargo bench -- --quick

# Run specific benchmarks only
cargo bench --bench builder

# Reduce workload size (edit source files)
# Change n_cityobjects from 10_000 to 1_000
```

---

## Contributing

When adding new benchmarks:

1. **Follow the dual-backend pattern**:
   ```rust
   #[cfg(feature = "backend-default")]
   mod default_benches { /* ... */ }

   #[cfg(feature = "backend-nested")]
   mod nested_benches { /* ... */ }
   ```

2. **Use consistent naming**:
   - `default/<benchmark-name>`
   - `nested/<benchmark-name>`

3. **Add documentation**:
   - Document what the benchmark measures
   - Explain expected behavior
   - Include running instructions

4. **Update this guide** with new benchmark descriptions

---

## Additional Resources

- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [dhat Documentation](https://docs.rs/dhat/)
- [CityJSON Specification](https://www.cityjson.org/)
- [cityjson-rs Documentation](https://docs.rs/cityjson/)

---

## Tracking Results Over Time

cityjson-rs now includes an automated tracking system that records benchmark results to CSV for visualization and progress tracking.

### Running Benchmarks with Tracking

```bash
# Run benchmarks and track results (both backends)
just bench-track "your description here"

# Run only default backend and track
just bench-track "optimized attributes" default

# Run only nested backend and track
just bench-track "baseline update" nested
```

### Viewing Results

```bash
# View recent results (last 20 by default)
just bench-history

# View more results
just bench-history 50
```

### CSV Output

Results are stored in `bench_results/history.csv` with the following columns:
- `timestamp`: ISO 8601 timestamp
- `commit`: Git commit hash
- `description`: Your description of the changes
- `benchmark`: Benchmark name (e.g., `builder/build_with_geometry`)
- `backend`: `default` or `nested`
- `time_ms`: Execution time in milliseconds
- `throughput`: Throughput (if available)
- `change_vs_nested_percent`: Performance change vs nested backend baseline

**Example output:**
```csv
timestamp,commit,description,benchmark,backend,time_ms,throughput,change_vs_nested_percent
2025-11-17T20:35:00,68897c8,baseline,builder/build_with_geometry,default,150.23,305.17K,-19.1%
2025-11-17T20:35:00,68897c8,baseline,builder/build_with_geometry,nested,185.67,268.45K,0.0%
```

### Visualization

The CSV format is designed for easy import into visualization tools:

**Python (Pandas/Plotly):**
```python
import pandas as pd
import plotly.express as px

df = pd.read_csv('bench_results/history.csv')
df['timestamp'] = pd.to_datetime(df['timestamp'])

# Plot time trends for a specific benchmark
fig = px.line(df[df['benchmark'] == 'builder/build_with_geometry'],
              x='timestamp', y='time_ms', color='backend',
              title='Build Performance Over Time')
fig.show()
```

**Excel/Google Sheets:**
Simply import the CSV file and create charts from the data.

**Grafana:**
Use the CSV datasource plugin to create dashboards.

---

## Quick Reference

```bash
# Most common commands

# Run benchmarks with tracking (recommended)
just bench-track "description of changes"

# View recent results
just bench-history

# Compare both backends (without tracking)
cargo bench --features backend-both

# Memory analysis with dhat
cargo bench --bench memory --features backend-both

# Save baseline for comparison
cargo bench --features backend-default -- --save-baseline default

# Compare against baseline
cargo bench --features backend-nested -- --baseline default

# View HTML reports
open target/criterion/report/index.html
```
