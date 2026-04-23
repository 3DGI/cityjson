#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import sys
from pathlib import Path
from typing import Any


type JsonValue = dict[str, Any] | list[Any] | str | int | float | bool | None


def load_json(path: Path) -> JsonValue:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def number_or_zero(value: object) -> float:
    if isinstance(value, int | float):
        return float(value)
    return 0.0


def find_key(data: JsonValue, keys: list[str]) -> float | None:
    if isinstance(data, dict):
        for key in keys:
            value = data.get(key)
            if isinstance(value, int | float):
                return float(value)
        for value in data.values():
            result = find_key(value, keys)
            if result is not None:
                return result
    if isinstance(data, list):
        for value in data:
            result = find_key(value, keys)
            if result is not None:
                return result
    return None


def totals_from_pps(data: JsonValue) -> tuple[float | None, float | None, float | None, float | None]:
    if not isinstance(data, dict):
        return None, None, None, None

    pps = data.get("pps")
    if not isinstance(pps, list):
        return None, None, None, None

    total_bytes = 0.0
    max_bytes = 0.0
    total_blocks = 0.0
    max_blocks = 0.0

    for entry in pps:
        if not isinstance(entry, dict):
            continue
        total_bytes += number_or_zero(entry.get("tb"))
        max_bytes += number_or_zero(entry.get("gb"))
        total_blocks += number_or_zero(entry.get("tbk"))
        max_blocks += number_or_zero(entry.get("gbk"))

    if total_bytes == 0.0 or max_bytes == 0.0:
        total_bytes = None
        max_bytes = None
    if total_blocks == 0.0 or max_blocks == 0.0:
        total_blocks = None
        max_blocks = None

    return max_bytes, total_bytes, max_blocks, total_blocks


def extract_metrics(path: Path) -> dict[str, int]:
    data = load_json(path)
    max_bytes = find_key(data, ["max_bytes", "max_bytes_live", "max_total_bytes", "peak_bytes"])
    total_bytes = find_key(data, ["total_bytes", "total_allocated_bytes", "total_bytes_allocated"])
    max_blocks = find_key(data, ["max_blocks", "max_blocks_live", "peak_blocks"])
    total_blocks = find_key(data, ["total_blocks", "total_allocations", "total_blocks_allocated"])

    if max_bytes is None or total_bytes is None or max_blocks is None or total_blocks is None:
        pps_max_bytes, pps_total_bytes, pps_max_blocks, pps_total_blocks = totals_from_pps(data)
        if max_bytes is None:
            max_bytes = pps_max_bytes
        if total_bytes is None:
            total_bytes = pps_total_bytes
        if max_blocks is None:
            max_blocks = pps_max_blocks
        if total_blocks is None:
            total_blocks = pps_total_blocks

    if max_bytes is None or total_bytes is None or max_blocks is None or total_blocks is None:
        raise SystemExit("failed to extract dhat heap totals")

    return {
        "heap_max_bytes": int(max_bytes),
        "heap_total_bytes": int(total_bytes),
        "heap_max_blocks": int(max_blocks),
        "heap_total_blocks": int(total_blocks),
    }


def append_csv_rows(args: argparse.Namespace, metrics: dict[str, int]) -> None:
    rows = []
    for metric, value in metrics.items():
        unit = "blocks" if metric.endswith("_blocks") else "bytes"
        rows.append(
            [
                args.timestamp,
                args.commit,
                args.description,
                args.mode,
                args.backend,
                args.bench,
                metric,
                str(value),
                unit,
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
    parser.add_argument("--dhat-json")
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
    path_arg = args.dhat_json or args.path
    if not path_arg:
        raise SystemExit("missing dhat json path")

    dhat_path = Path(path_arg)
    if not dhat_path.exists():
        raise SystemExit(f"dhat json not found: {dhat_path}")

    metrics = extract_metrics(dhat_path)

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
