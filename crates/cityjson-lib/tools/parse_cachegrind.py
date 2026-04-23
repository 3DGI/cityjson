#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import sys
from pathlib import Path


RATIO_METRICS = {
    "cache_d1_miss_rate": ("D1mr", "Dr"),
    "cache_ll_miss_rate": ("DLmr", "Dr"),
    "branch_miss_rate": ("Bcm", "Bc"),
}


def parse_cachegrind_summary(path: Path) -> dict[str, float] | None:
    events: list[str] | None = None
    summary: list[str] | None = None
    with path.open("r", encoding="utf-8") as handle:
        for raw_line in handle:
            line = raw_line.strip()
            if line.startswith("events:"):
                events = line.split()[1:]
            if line.startswith("summary:"):
                summary = line.split()[1:]
    if not events or not summary:
        return None

    values: dict[str, float] = {}
    for name, value in zip(events, summary, strict=False):
        cleaned = value.replace(",", "")
        try:
            values[name] = float(cleaned)
        except ValueError:
            continue
    return values


def ratio(values: dict[str, float], numer_key: str, denom_key: str) -> float | None:
    numer = values.get(numer_key)
    denom = values.get(denom_key)
    if numer is None or denom is None or denom == 0.0:
        return None
    return numer / denom


def extract_metrics(path: Path) -> dict[str, float]:
    values = parse_cachegrind_summary(path)
    if values is None:
        raise SystemExit("failed to parse cachegrind summary")

    metrics: dict[str, float] = {}
    for metric, (numer_key, denom_key) in RATIO_METRICS.items():
        metric_value = ratio(values, numer_key, denom_key)
        if metric_value is not None:
            metrics[metric] = metric_value

    if not metrics:
        raise SystemExit("no cachegrind ratios found")
    return metrics


def append_csv_rows(args: argparse.Namespace, metrics: dict[str, float]) -> None:
    rows = []
    for metric, value in metrics.items():
        rows.append(
            [
                args.timestamp,
                args.commit,
                args.description,
                args.mode,
                args.backend,
                args.bench,
                metric,
                f"{value:.8f}",
                "ratio",
                args.seed,
                args.bench_version,
                args.rustc,
            ]
        )

    with Path(args.out).open("a", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerows(rows)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("path", nargs="?")
    parser.add_argument("--cachegrind-out")
    parser.add_argument("--timestamp")
    parser.add_argument("--commit")
    parser.add_argument("--description")
    parser.add_argument("--mode")
    parser.add_argument("--backend")
    parser.add_argument("--bench")
    parser.add_argument("--seed")
    parser.add_argument("--bench-version")
    parser.add_argument("--rustc")
    parser.add_argument("--out")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    path_arg = args.cachegrind_out or args.path
    if not path_arg:
        raise SystemExit("missing cachegrind output path")

    cachegrind_path = Path(path_arg)
    if not cachegrind_path.exists():
        raise SystemExit(f"cachegrind output not found: {cachegrind_path}")

    metrics = extract_metrics(cachegrind_path)

    if args.out:
        required = [
            "timestamp",
            "commit",
            "description",
            "mode",
            "backend",
            "bench",
            "seed",
            "bench_version",
            "rustc",
        ]
        missing = [name for name in required if getattr(args, name) in {None, ""}]
        if missing:
            missing_list = ", ".join(sorted(missing))
            print(f"missing required csv arguments: {missing_list}", file=sys.stderr)
            raise SystemExit(1)
        append_csv_rows(args, metrics)
        return

    print(json.dumps(metrics, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
