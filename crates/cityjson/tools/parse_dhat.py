#!/usr/bin/env python3
import argparse
import csv
import json
import sys
from pathlib import Path


def load_json(path: Path):
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def find_key(data, keys):
    if isinstance(data, dict):
        for key in keys:
            value = data.get(key)
            if isinstance(value, (int, float)):
                return float(value)
        for value in data.values():
            result = find_key(value, keys)
            if result is not None:
                return result
    elif isinstance(data, list):
        for value in data:
            result = find_key(value, keys)
            if result is not None:
                return result
    return None


def totals_from_pps(data):
    pps = data.get("pps")
    if not isinstance(pps, list):
        return None, None
    total_bytes = 0
    max_bytes = 0
    for entry in pps:
        if not isinstance(entry, dict):
            continue
        tb = entry.get("tb")
        gb = entry.get("gb")
        if isinstance(tb, (int, float)):
            total_bytes += float(tb)
        if isinstance(gb, (int, float)):
            max_bytes += float(gb)
    if total_bytes == 0 or max_bytes == 0:
        return None, None
    return max_bytes, total_bytes


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--dhat-json", required=True)
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

    dhat_path = Path(args.dhat_json)
    if not dhat_path.exists():
        print(f"dhat json not found: {dhat_path}", file=sys.stderr)
        sys.exit(1)

    data = load_json(dhat_path)

    max_bytes = find_key(data, ["max_bytes", "max_bytes_live", "max_total_bytes", "peak_bytes"])
    total_bytes = find_key(data, ["total_bytes", "total_allocated_bytes", "total_bytes_allocated"])

    if max_bytes is None or total_bytes is None:
        max_from_pps, total_from_pps = totals_from_pps(data)
        if max_bytes is None:
            max_bytes = max_from_pps
        if total_bytes is None:
            total_bytes = total_from_pps

    if max_bytes is None:
        print("Failed to locate max bytes in dhat output", file=sys.stderr)
        sys.exit(1)
    if total_bytes is None:
        print("Failed to locate total bytes in dhat output", file=sys.stderr)
        sys.exit(1)

    rows = [
        [
            args.timestamp,
            args.commit,
            args.description,
            args.mode,
            args.backend,
            args.bench,
            "heap_max_bytes",
            f"{max_bytes:.0f}",
            "bytes",
            args.seed,
            args.bench_version,
            args.rustc,
        ],
        [
            args.timestamp,
            args.commit,
            args.description,
            args.mode,
            args.backend,
            args.bench,
            "heap_total_bytes",
            f"{total_bytes:.0f}",
            "bytes",
            args.seed,
            args.bench_version,
            args.rustc,
        ],
    ]

    with open(args.out, "a", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerows(rows)


if __name__ == "__main__":
    main()
