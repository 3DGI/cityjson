#!/usr/bin/env python3
import argparse
import csv
import json
import sys
from pathlib import Path


def load_json(path: Path):
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def find_benchmarks(criterion_dir: Path):
    benchmarks = list(criterion_dir.rglob("new/benchmark.json"))
    if benchmarks:
        return benchmarks
    return list(criterion_dir.rglob("base/benchmark.json"))


def bench_id_from(benchmark_json: dict) -> str:
    for key in ("full_id", "id", "title"):
        value = benchmark_json.get(key)
        if value:
            return str(value)
    raise ValueError("benchmark.json missing full_id/id/title")


def throughput_from(benchmark_json: dict, mean_ns: float):
    throughput = benchmark_json.get("throughput")
    if not isinstance(throughput, dict) or not throughput:
        return []

    results = []
    for kind, amount in throughput.items():
        if isinstance(amount, dict):
            amount = amount.get("value")
        try:
            amount = float(amount)
        except (TypeError, ValueError):
            continue
        seconds = mean_ns / 1_000_000_000.0
        if seconds <= 0:
            continue
        if kind == "Elements":
            results.append(("throughput_elem_s", amount / seconds, "elem_s"))
        elif kind == "Bytes":
            results.append(("throughput_bytes_s", amount / seconds, "bytes_s"))
    return results


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--criterion-dir", required=True)
    parser.add_argument("--timestamp", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--description", required=True)
    parser.add_argument("--mode", required=True)
    parser.add_argument("--backend", required=True)
    parser.add_argument("--seed", required=True)
    parser.add_argument("--bench-version", required=True)
    parser.add_argument("--rustc", required=True)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    criterion_dir = Path(args.criterion_dir)
    if not criterion_dir.exists():
        print(f"Criterion directory not found: {criterion_dir}", file=sys.stderr)
        sys.exit(1)

    benchmark_files = find_benchmarks(criterion_dir)
    if not benchmark_files:
        print(f"No benchmark.json files found in {criterion_dir}", file=sys.stderr)
        sys.exit(1)

    bench_entries = []
    for bench_file in benchmark_files:
        bench_json = load_json(bench_file)
        bench_id_raw = bench_id_from(bench_json)
        bench_entries.append((bench_file, bench_json, bench_id_raw))

    has_duplicates = any(bench_id.endswith(" #2") for _, _, bench_id in bench_entries)

    rows = []
    for bench_file, bench_json, bench_id_raw in bench_entries:
        is_duplicate = bench_id_raw.endswith(" #2")
        if has_duplicates:
            if args.backend == "nested":
                if not is_duplicate:
                    continue
            else:
                if is_duplicate:
                    continue

        bench_id = bench_id_raw[:-3] if is_duplicate else bench_id_raw
        if bench_id.startswith("memory/"):
            continue

        base_dir = bench_file.parent
        estimates_file = base_dir / "estimates.json"
        if not estimates_file.exists():
            print(f"Missing estimates.json for {bench_file}", file=sys.stderr)
            sys.exit(1)

        estimates_json = load_json(estimates_file)
        mean = estimates_json.get("mean", {})
        point_estimate = mean.get("point_estimate")
        if point_estimate is None:
            print(f"Missing mean.point_estimate in {estimates_file}", file=sys.stderr)
            sys.exit(1)

        try:
            mean_ns = float(point_estimate)
        except (TypeError, ValueError):
            print(f"Invalid mean.point_estimate in {estimates_file}", file=sys.stderr)
            sys.exit(1)

        time_ms = mean_ns / 1_000_000.0
        rows.append(
            [
                args.timestamp,
                args.commit,
                args.description,
                args.mode,
                args.backend,
                bench_id,
                "time_ms",
                f"{time_ms:.6f}",
                "ms",
                args.seed,
                args.bench_version,
                args.rustc,
            ]
        )

        for metric, value, unit in throughput_from(bench_json, mean_ns):
            rows.append(
                [
                args.timestamp,
                args.commit,
                args.description,
                args.mode,
                args.backend,
                bench_id,
                metric,
                    f"{value:.6f}",
                    unit,
                    args.seed,
                    args.bench_version,
                    args.rustc,
                ]
            )

    with open(args.out, "a", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerows(rows)


if __name__ == "__main__":
    main()
