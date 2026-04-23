#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import sys
from pathlib import Path
from typing import Any

CASE_LOGICAL_BYTES = {
    "io_3dbag_cityjson": 6_008_573.0,
    "io_3dbag_cityjson_cluster_4x": 21_040_983.0,
}


def load_json(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def find_benchmarks(criterion_dir: Path) -> list[Path]:
    benchmarks = list(criterion_dir.rglob("new/benchmark.json"))
    if benchmarks:
        return benchmarks
    return list(criterion_dir.rglob("base/benchmark.json"))


def bench_id_from(benchmark_json: dict[str, Any]) -> str:
    for key in ("full_id", "id", "title"):
        value = benchmark_json.get(key)
        if isinstance(value, str) and value:
            return value
    raise ValueError("benchmark.json missing full_id/id/title")


def throughput_rows(benchmark_json: dict[str, Any], mean_ns: float) -> list[tuple[str, float, str]]:
    throughput = benchmark_json.get("throughput")
    if not isinstance(throughput, dict) or not throughput:
        return []

    rows: list[tuple[str, float, str]] = []
    seconds = mean_ns / 1_000_000_000.0
    if seconds <= 0.0:
        return []

    for kind, amount in throughput.items():
        value = amount
        if isinstance(amount, dict):
            value = amount.get("value")
        if not isinstance(value, int | float):
            continue
        if kind == "Elements":
            rows.append(("throughput_elem_s", float(value) / seconds, "elem_s"))
        if kind == "Bytes":
            rows.append(("throughput_bytes_s", float(value) / seconds, "bytes_s"))
    return rows


def logical_throughput_row(bench_id: str, mean_ns: float) -> tuple[str, float, str] | None:
    parts = bench_id.split("/")
    if len(parts) != 4:
        return None

    case_id = parts[1]
    logical_bytes = CASE_LOGICAL_BYTES.get(case_id)
    if logical_bytes is None:
        return None

    seconds = mean_ns / 1_000_000_000.0
    if seconds <= 0.0:
        return None

    return ("logical_throughput_bytes_s", logical_bytes / seconds, "bytes_s")


def main() -> None:
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
        print(f"criterion directory not found: {criterion_dir}", file=sys.stderr)
        raise SystemExit(1)

    benchmark_files = find_benchmarks(criterion_dir)
    if not benchmark_files:
        print(f"no benchmark.json files found in {criterion_dir}", file=sys.stderr)
        raise SystemExit(1)

    rows: list[list[str]] = []
    for bench_file in benchmark_files:
        bench_json = load_json(bench_file)
        bench_id = bench_id_from(bench_json)

        estimates_file = bench_file.parent / "estimates.json"
        if not estimates_file.exists():
            print(f"missing estimates.json for {bench_file}", file=sys.stderr)
            raise SystemExit(1)
        estimates_json = load_json(estimates_file)
        mean = estimates_json.get("mean")
        if not isinstance(mean, dict):
            print(f"missing mean estimate in {estimates_file}", file=sys.stderr)
            raise SystemExit(1)
        point_estimate = mean.get("point_estimate")
        if not isinstance(point_estimate, int | float):
            print(f"missing mean.point_estimate in {estimates_file}", file=sys.stderr)
            raise SystemExit(1)

        mean_ns = float(point_estimate)
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

        for metric, value, unit in throughput_rows(bench_json, mean_ns):
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

        logical_row = logical_throughput_row(bench_id, mean_ns)
        if logical_row is not None:
            metric, value, unit = logical_row
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

    with Path(args.out).open("a", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerows(rows)


if __name__ == "__main__":
    main()
