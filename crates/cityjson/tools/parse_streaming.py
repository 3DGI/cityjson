#!/usr/bin/env python3
import argparse
import csv
import json
import sys
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--stream-json", required=True)
    parser.add_argument("--timestamp", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--description", required=True)
    parser.add_argument("--mode", required=True)
    parser.add_argument("--backend", required=True)
    parser.add_argument("--bench", required=True)
    parser.add_argument("--seed", required=True)
    parser.add_argument("--bench-version", required=True)
    parser.add_argument("--rustc", required=True)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    metrics_path = Path(args.stream_json)
    if not metrics_path.exists():
        print(f"streaming metrics not found: {metrics_path}", file=sys.stderr)
        sys.exit(1)

    try:
        data = json.loads(metrics_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        print(f"failed to parse streaming metrics json: {exc}", file=sys.stderr)
        sys.exit(1)

    metrics = data.get("metrics")
    if not isinstance(metrics, dict):
        print("streaming metrics json missing 'metrics' object", file=sys.stderr)
        sys.exit(1)

    rows = []
    for metric, value in metrics.items():
        if value is None:
            continue
        try:
            numeric = float(value)
        except (TypeError, ValueError):
            print(f"invalid metric value for {metric}: {value}", file=sys.stderr)
            continue
        unit = "buildings_s" if "throughput" in metric else "s"
        rows.append(
            [
                args.timestamp,
                args.commit,
                args.description,
                args.mode,
                args.backend,
                args.bench,
                metric,
                f"{numeric:.6f}",
                unit,
                args.seed,
                args.bench_version,
                args.rustc,
            ]
        )

    if not rows:
        print("no streaming metrics emitted", file=sys.stderr)
        sys.exit(1)

    with open(args.out, "a", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerows(rows)


if __name__ == "__main__":
    main()
