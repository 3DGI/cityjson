#!/usr/bin/env python3
import argparse
import csv
import sys
from pathlib import Path


RATIO_METRICS = {
    "cache_d1_miss_rate": ("D1mr", "Dr"),
    "cache_ll_miss_rate": ("DLmr", "Dr"),
    "branch_miss_rate": ("Bcm", "Bc"),
}


def parse_cachegrind_summary(path: Path):
    events = None
    summary = None
    with path.open("r", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if line.startswith("events:"):
                events = line.split()[1:]
            elif line.startswith("summary:"):
                summary = line.split()[1:]
    if not events or not summary:
        return None
    values = {}
    for name, value in zip(events, summary):
        cleaned = value.replace(",", "")
        try:
            values[name] = float(cleaned)
        except ValueError:
            continue
    return values


def ratio(values, numer_key, denom_key):
    numer = values.get(numer_key)
    denom = values.get(denom_key)
    if numer is None or denom is None:
        return None, f"missing {numer_key} or {denom_key}"
    if denom == 0:
        return None, f"zero denominator {denom_key}"
    return numer / denom, None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cachegrind-out", required=True)
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

    cachegrind_path = Path(args.cachegrind_out)
    if not cachegrind_path.exists():
        print(f"cachegrind output not found: {cachegrind_path}", file=sys.stderr)
        sys.exit(1)

    values = parse_cachegrind_summary(cachegrind_path)
    if values is None:
        print("Failed to parse cachegrind summary/events", file=sys.stderr)
        sys.exit(1)

    rows = []
    for metric, (numer_key, denom_key) in RATIO_METRICS.items():
        value, warning = ratio(values, numer_key, denom_key)
        if value is None:
            print(f"Skipping {metric}: {warning}", file=sys.stderr)
            continue
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

    if not rows:
        print("No cachegrind ratios emitted", file=sys.stderr)
        sys.exit(1)

    with open(args.out, "a", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerows(rows)


if __name__ == "__main__":
    main()
