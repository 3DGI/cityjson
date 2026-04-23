#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
from collections import defaultdict
from pathlib import Path


SPARKS = "▁▂▃▄▅▆▇█"


def load_rows(path: Path) -> list[dict[str, str]]:
    if not path.exists():
        raise SystemExit(f"csv not found: {path}")
    with path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


def print_table(headers: list[str], rows: list[list[str]]) -> None:
    widths = [len(header) for header in headers]
    for row in rows:
        for index, cell in enumerate(row):
            widths[index] = max(widths[index], len(cell))
    print(" | ".join(header.ljust(widths[index]) for index, header in enumerate(headers)))
    for row in rows:
        print(" | ".join(cell.ljust(widths[index]) for index, cell in enumerate(row)))


def sparkline(values: list[float]) -> str:
    if not values:
        return ""
    low = min(values)
    high = max(values)
    if high == low:
        return SPARKS[-1] * len(values)
    parts: list[str] = []
    for value in values:
        position = int((value - low) / (high - low) * (len(SPARKS) - 1))
        parts.append(SPARKS[position])
    return "".join(parts)


def select_rows(
    rows: list[dict[str, str]],
    description: str | None,
    mode: str,
    timestamp: str | None,
) -> tuple[list[dict[str, str]], str | None]:
    filtered = rows
    if description:
        filtered = [row for row in filtered if row["description"] == description]
    if mode != "all":
        filtered = [row for row in filtered if row["mode"] == mode]
    if not filtered:
        return [], None
    if timestamp:
        filtered = [row for row in filtered if row["timestamp"] == timestamp]
        return filtered, timestamp
    latest = max(row["timestamp"] for row in filtered)
    return [row for row in filtered if row["timestamp"] == latest], latest


def latest_and_previous_timestamps(
    rows: list[dict[str, str]],
    description: str,
    mode: str,
) -> tuple[str | None, str | None]:
    filtered = [row for row in rows if row["description"] == description]
    if mode != "all":
        filtered = [row for row in filtered if row["mode"] == mode]
    timestamps = sorted({row["timestamp"] for row in filtered}, reverse=True)
    if not timestamps:
        return None, None
    latest = timestamps[0]
    previous = timestamps[1] if len(timestamps) > 1 else None
    return latest, previous


def summarize_listing(rows: list[dict[str, str]]) -> None:
    descriptions = defaultdict(list)
    benches = defaultdict(set)
    metrics = defaultdict(set)
    for row in rows:
        descriptions[row["description"]].append(row)
        benches[row["bench"]].add(row["metric"])
        metrics[row["metric"]].add(row["unit"])

    desc_rows: list[list[str]] = []
    for description, desc_rows_data in sorted(descriptions.items()):
        latest = max(row["timestamp"] for row in desc_rows_data)
        modes = ",".join(sorted({row["mode"] for row in desc_rows_data}))
        desc_rows.append([description, latest, modes, str(len(desc_rows_data))])

    bench_rows: list[list[str]] = []
    for bench, metric_names in sorted(benches.items()):
        bench_rows.append([bench, str(len(metric_names))])

    metric_rows: list[list[str]] = []
    for metric, units in sorted(metrics.items()):
        metric_rows.append([metric, ",".join(sorted(units))])

    print("Descriptions:")
    print_table(["description", "latest timestamp", "modes", "rows"], desc_rows)
    print("")
    print("Benches:")
    print_table(["bench", "metrics"], bench_rows)
    print("")
    print("Metrics:")
    print_table(["metric", "units"], metric_rows)


def snapshot(rows: list[dict[str, str]], previous_rows: list[dict[str, str]] | None) -> list[list[str]]:
    current_values: dict[tuple[str, str, str], float] = {}
    previous_values: dict[tuple[str, str, str], float] = {}

    for row in rows:
        current_values[(row["bench"], row["metric"], row["unit"])] = float(row["value"])
    if previous_rows:
        for row in previous_rows:
            previous_values[(row["bench"], row["metric"], row["unit"])] = float(row["value"])

    output_rows: list[list[str]] = []
    for key in sorted(current_values):
        bench, metric, unit = key
        value = current_values[key]
        delta_text = "-"
        previous_value = previous_values.get(key)
        if previous_value not in {None, 0.0}:
            delta_pct = ((value / previous_value) - 1.0) * 100.0
            delta_text = f"{delta_pct:+.2f}%"
        output_rows.append([bench, metric, f"{value:.6g}", unit, delta_text])
    return output_rows


def show_series(rows: list[dict[str, str]], description: str, mode: str, bench: str, metric: str) -> None:
    filtered = [
        row
        for row in rows
        if row["description"] == description and row["bench"] == bench and row["metric"] == metric
    ]
    if mode != "all":
        filtered = [row for row in filtered if row["mode"] == mode]
    if not filtered:
        raise SystemExit("no rows found for the selected series")

    by_timestamp: dict[str, list[dict[str, str]]] = defaultdict(list)
    for row in filtered:
        by_timestamp[row["timestamp"]].append(row)

    values: list[float] = []
    table_rows: list[list[str]] = []
    for timestamp in sorted(by_timestamp):
        rows_for_timestamp = by_timestamp[timestamp]
        value = sum(float(row["value"]) for row in rows_for_timestamp) / len(rows_for_timestamp)
        unit = rows_for_timestamp[0]["unit"]
        commit = rows_for_timestamp[0]["commit"]
        values.append(value)
        table_rows.append([timestamp, f"{value:.6g}", unit, commit])

    print(f"Series: {description} / {bench} / {metric}")
    print(f"Sparkline: {sparkline(values)}")
    print_table(["timestamp", "value", "unit", "commit"], table_rows)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--csv", default="benches/results/history.csv")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--description")
    parser.add_argument("--mode", default="all")
    parser.add_argument("--timestamp")
    parser.add_argument("--series", action="store_true")
    parser.add_argument("--bench")
    parser.add_argument("--metric")
    args = parser.parse_args()

    rows = load_rows(Path(args.csv))
    if not rows:
        raise SystemExit("no rows found")

    if args.list or not args.description:
        summarize_listing(rows)
        return

    if args.series:
        if not args.bench or not args.metric:
            raise SystemExit("--series requires --bench and --metric")
        show_series(rows, args.description, args.mode, args.bench, args.metric)
        return

    selected_rows, selected_timestamp = select_rows(rows, args.description, args.mode, args.timestamp)
    if not selected_rows:
        raise SystemExit("no rows found for the selected snapshot")

    _latest, previous_timestamp = latest_and_previous_timestamps(rows, args.description, args.mode)
    previous_rows: list[dict[str, str]] | None = None
    if previous_timestamp and previous_timestamp != selected_timestamp:
        previous_rows = [
            row
            for row in rows
            if row["description"] == args.description
            and row["timestamp"] == previous_timestamp
            and (args.mode == "all" or row["mode"] == args.mode)
        ]

    print(f"Snapshot: {args.description} @ {selected_timestamp}")
    if previous_timestamp and previous_timestamp != selected_timestamp:
        print(f"Delta baseline: {previous_timestamp}")
    print_table(["bench", "metric", "value", "unit", "delta"], snapshot(selected_rows, previous_rows))


if __name__ == "__main__":
    main()
