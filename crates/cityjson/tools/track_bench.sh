#!/usr/bin/env bash
#
# track_bench.sh - Extract Criterion benchmark results and append to CSV
#
# Usage: ./tools/track_bench.sh "description of changes"
#

set -euo pipefail

DESCRIPTION="${1:-benchmark run}"
TIMESTAMP=$(date -Iseconds)
COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
CSV_FILE="bench_results/history.csv"
CRITERION_DIR="target/criterion"

# Export variables for Python script
export DESCRIPTION
export TIMESTAMP
export COMMIT

# Ensure CSV file exists with headers
if [ ! -f "$CSV_FILE" ]; then
    echo "timestamp,commit,description,benchmark,backend,time_ms,throughput,change_vs_nested_percent" > "$CSV_FILE"
fi

# Check if criterion output exists
if [ ! -d "$CRITERION_DIR" ]; then
    echo "Error: Criterion output directory not found at $CRITERION_DIR"
    echo "Please run benchmarks first with: cargo bench"
    exit 1
fi

echo "Extracting benchmark results..."
echo "Description: $DESCRIPTION"
echo "Commit: $COMMIT"
echo "Timestamp: $TIMESTAMP"
echo ""

# Use Python to parse JSON and extract results
python3 << 'PYTHON_SCRIPT'
import json
import sys
import os
from pathlib import Path
from datetime import datetime

criterion_dir = Path("target/criterion")
csv_file = "bench_results/history.csv"
timestamp = os.environ.get("TIMESTAMP", datetime.now().isoformat())
commit = os.environ.get("COMMIT", "unknown")
description = os.environ.get("DESCRIPTION", "benchmark run")

# Store baseline values (from nested backend)
nested_baselines = {}
results = []

# Find all benchmark directories (may be nested under group directories)
for group_dir in criterion_dir.iterdir():
    if not group_dir.is_dir() or group_dir.name == "report":
        continue

    # Check if this is a group directory (contains benchmark subdirectories)
    # or a direct benchmark directory
    base_dir = group_dir / "base"

    bench_dirs = []
    if base_dir.exists():
        # Direct benchmark directory
        bench_dirs = [group_dir]
    else:
        # Group directory containing benchmarks
        bench_dirs = [d for d in group_dir.iterdir() if d.is_dir()]

    for bench_dir in bench_dirs:
        base_dir = bench_dir / "base"
        if not base_dir.exists():
            continue

        estimates_file = base_dir / "estimates.json"
        benchmark_file = base_dir / "benchmark.json"

        if not estimates_file.exists() or not benchmark_file.exists():
            continue

        # Parse JSON files
        with open(estimates_file) as f:
            estimates = json.load(f)

        with open(benchmark_file) as f:
            benchmark = json.load(f)

        # Extract benchmark name and backend
        full_id = benchmark.get("full_id", "")
        title = benchmark.get("title", "")

        # Parse backend from the ID (e.g., "builder/default/build_with_geometry")
        parts = full_id.split("/")
        if len(parts) >= 3:
            suite = parts[0]
            backend = parts[1]
            bench_name = "/".join(parts[2:])
            benchmark_name = f"{suite}/{bench_name}"
        elif len(parts) == 2:
            # No backend specified, assume default
            suite = parts[0]
            backend = "default"
            bench_name = parts[1]
            benchmark_name = f"{suite}/{bench_name}"
        else:
            continue

        # Extract time in nanoseconds and convert to milliseconds
        time_ns = estimates.get("mean", {}).get("point_estimate", 0)
        time_ms = time_ns / 1_000_000

        # Calculate throughput if available
        throughput_info = benchmark.get("throughput")
        throughput_str = ""
        if throughput_info:
            # Get the number of elements
            elements = throughput_info.get("Elements") or throughput_info.get("Bytes")
            if elements and time_ns > 0:
                # Calculate elements per second
                throughput = (elements / time_ns) * 1_000_000_000
                # Format as K elements/s for readability
                throughput_k = throughput / 1000
                throughput_str = f"{throughput_k:.2f}K"

        # Store result
        result = {
            "benchmark": benchmark_name,
            "backend": backend,
            "time_ms": time_ms,
            "throughput": throughput_str
        }

        # Track nested baselines
        if backend == "nested":
            nested_baselines[benchmark_name] = time_ms

        results.append(result)

# Second pass: calculate changes vs nested baseline
for result in results:
    benchmark_name = result["benchmark"]
    backend = result["backend"]
    time_ms = result["time_ms"]

    change_percent = ""
    if backend == "nested":
        change_percent = "0.00"
    elif benchmark_name in nested_baselines:
        baseline = nested_baselines[benchmark_name]
        if baseline > 0:
            change = ((time_ms - baseline) / baseline) * 100
            change_percent = f"{change:.2f}"

    result["change_percent"] = change_percent

# Write results to CSV
with open(csv_file, "a") as f:
    for result in results:
        row = f"{timestamp},{commit},{description},{result['benchmark']},{result['backend']},{result['time_ms']:.2f},{result['throughput']},{result['change_percent']}"
        f.write(row + "\n")

        # Display result
        change_display = ""
        if result['change_percent']:
            change_display = f" ({result['change_percent']}% vs nested)"
        print(f"  {result['benchmark']} [{result['backend']}]: {result['time_ms']:.2f}ms{change_display}")

print(f"\n✅ Saved {len(results)} results to {csv_file}")

PYTHON_SCRIPT

echo ""
echo "View results with:"
echo "  just bench-history"
echo "  or: cat $CSV_FILE"
