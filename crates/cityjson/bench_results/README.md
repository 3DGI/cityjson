# Benchmark Results Tracking

This directory contains benchmark results tracked over time for the cityjson-rs project.

## Files

- **`history.csv`** - Main CSV file containing all benchmark results with timestamps, commit hashes, and performance metrics
- **`history/`** - Legacy markdown-based benchmark reports (for reference)

## CSV Format

The `history.csv` file contains the following columns:

| Column | Description |
|--------|-------------|
| `timestamp` | ISO 8601 timestamp of when the benchmark was run |
| `commit` | Git commit hash (short form) |
| `description` | User-provided description of the changes being benchmarked |
| `benchmark` | Name of the benchmark (e.g., `builder/build_with_geometry`) |
| `backend` | Backend type: `default` (flattened) or `nested` (JSON-like) |
| `time_ms` | Execution time in milliseconds |
| `throughput` | Throughput in K elements/second (if applicable) |
| `change_vs_nested_percent` | Performance change compared to nested backend baseline (negative = faster, positive = slower) |

## Usage

### Running Benchmarks with Tracking

```bash
# Run both backends and track results
just bench-track "your description here"

# Run only default backend
just bench-track "optimized attributes" default

# Run only nested backend
just bench-track "baseline update" nested
```

### Viewing Results

```bash
# View recent results
just bench-history

# View more results
just bench-history 50
```

### Manual Tracking

If you've already run benchmarks, you can manually track the results:

```bash
./tools/track_bench.sh "description of changes"
```

## Visualization

The CSV format is designed for easy import into visualization tools:

**Python (Pandas/Plotly):**
```python
import pandas as pd
import plotly.express as px

df = pd.read_csv('bench_results/history.csv')
df['timestamp'] = pd.to_datetime(df['timestamp'])

# Plot time trends
fig = px.line(df[df['benchmark'] == 'builder/build_with_geometry'],
              x='timestamp', y='time_ms', color='backend',
              title='Build Performance Over Time')
fig.show()

# Plot performance improvements
fig = px.bar(df[df['backend'] == 'default'].groupby('benchmark').last().reset_index(),
             x='benchmark', y='change_vs_nested_percent',
             title='Performance vs Nested Backend')
fig.show()
```

**Excel/Google Sheets:**
Simply import the CSV file and create charts.

**Grafana:**
Use the CSV datasource plugin to create dashboards.

## Baseline Comparison

The nested backend serves as the baseline for comparison. All default backend results show the percentage change compared to the nested backend:

- **Negative values** indicate the default backend is faster (improvement)
- **Positive values** indicate the default backend is slower (regression)
- **0.00** for nested backend (it's the baseline)

## Example Output

```csv
timestamp,commit,description,benchmark,backend,time_ms,throughput,change_vs_nested_percent
2025-11-17T20:52:40+00:00,68897c8,baseline test,builder/build_with_geometry,default,74.19,134.78K,-12.35
2025-11-17T20:52:40+00:00,68897c8,baseline test,builder/build_with_geometry,nested,84.64,118.14K,0.00
```

This shows that the default backend is 12.35% faster than the nested backend for building geometries.
