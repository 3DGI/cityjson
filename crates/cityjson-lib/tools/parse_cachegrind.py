#!/usr/bin/env python3
import argparse
import json
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
    if numer is None or denom is None or denom == 0:
        return None
    return numer / denom


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("path")
    args = parser.parse_args()

    values = parse_cachegrind_summary(Path(args.path))
    if values is None:
        raise SystemExit("failed to parse cachegrind summary")

    output = {}
    for metric, (numer_key, denom_key) in RATIO_METRICS.items():
        value = ratio(values, numer_key, denom_key)
        if value is not None:
            output[metric] = value

    if not output:
        raise SystemExit("no cachegrind ratios found")

    print(json.dumps(output, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
