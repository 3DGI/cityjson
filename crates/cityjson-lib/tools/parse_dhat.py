#!/usr/bin/env python3
import argparse
import json
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
        return None, None, None, None

    total_bytes = 0.0
    max_bytes = 0.0
    total_blocks = 0.0
    max_blocks = 0.0

    for entry in pps:
        if not isinstance(entry, dict):
            continue
        tb = entry.get("tb")
        gb = entry.get("gb")
        tbk = entry.get("tbk")
        gbk = entry.get("gbk")
        if isinstance(tb, (int, float)):
            total_bytes += float(tb)
        if isinstance(gb, (int, float)):
            max_bytes += float(gb)
        if isinstance(tbk, (int, float)):
            total_blocks += float(tbk)
        if isinstance(gbk, (int, float)):
            max_blocks += float(gbk)

    if total_bytes == 0 or max_bytes == 0:
        total_bytes = None
        max_bytes = None
    if total_blocks == 0 or max_blocks == 0:
        total_blocks = None
        max_blocks = None

    return max_bytes, total_bytes, max_blocks, total_blocks


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("path")
    args = parser.parse_args()

    data = load_json(Path(args.path))

    max_bytes = find_key(data, ["max_bytes", "max_bytes_live", "max_total_bytes", "peak_bytes"])
    total_bytes = find_key(
        data, ["total_bytes", "total_allocated_bytes", "total_bytes_allocated"]
    )
    max_blocks = find_key(data, ["max_blocks", "max_blocks_live", "peak_blocks"])
    total_blocks = find_key(
        data, ["total_blocks", "total_allocations", "total_blocks_allocated"]
    )

    if max_bytes is None or total_bytes is None or max_blocks is None or total_blocks is None:
        max_pps, total_pps, max_blocks_pps, total_blocks_pps = totals_from_pps(data)
        if max_bytes is None:
            max_bytes = max_pps
        if total_bytes is None:
            total_bytes = total_pps
        if max_blocks is None:
            max_blocks = max_blocks_pps
        if total_blocks is None:
            total_blocks = total_blocks_pps

    if max_bytes is None or total_bytes is None or max_blocks is None or total_blocks is None:
        raise SystemExit("failed to extract dhat heap totals")

    print(
        json.dumps(
            {
                "heap_max_bytes": int(max_bytes),
                "heap_total_bytes": int(total_bytes),
                "heap_max_blocks": int(max_blocks),
                "heap_total_blocks": int(total_blocks),
            },
            indent=2,
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
