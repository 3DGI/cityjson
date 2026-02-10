#!/usr/bin/env python3
import argparse
import csv
import re
import sys
from collections import defaultdict
from pathlib import Path


LOWER_BETTER = {
    "time_ms",
    "time_producer_ms",
    "time_consumer_ms",
    "heap_max_bytes",
    "heap_total_bytes",
    "heap_max_blocks",
    "heap_total_blocks",
    "cache_d1_miss_rate",
    "cache_ll_miss_rate",
    "branch_miss_rate",
    "stream_total_s",
    "stream_producer_s",
    "stream_consumer_s",
}
CACHE_METRICS = {"cache_d1_miss_rate", "cache_ll_miss_rate", "branch_miss_rate"}
HEAP_BYTES_METRICS = {"heap_max_bytes", "heap_total_bytes"}
HEAP_BLOCK_METRICS = {"heap_max_blocks", "heap_total_blocks"}
HEAP_METRICS = HEAP_BYTES_METRICS | HEAP_BLOCK_METRICS
SPARKS = "▁▂▃▄▅▆▇█"
ABS_BAR_MIN = -200.0
ABS_BAR_MAX = 200.0
ABS_BAR_STEP = 5.0
ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
COLOR_RESET = "\x1b[0m"


def visible_len(text):
    return len(ANSI_RE.sub("", text))


def print_table(headers, rows):
    if not rows:
        print(" | ".join(headers))
        return
    widths = [len(h) for h in headers]
    for row in rows:
        for idx, cell in enumerate(row):
            widths[idx] = max(widths[idx], visible_len(cell))

    def fmt_line(values):
        parts = []
        for idx, cell in enumerate(values):
            pad = widths[idx] - visible_len(cell)
            parts.append(cell + (" " * max(0, pad)))
        return " | ".join(parts)

    print(fmt_line(headers))
    for row in rows:
        print(fmt_line(row))


def truthy(value: str) -> bool:
    return value.strip().lower() in {"1", "true", "yes", "y", "on"}


def color_metric(metric, enabled):
    if not enabled:
        return metric
    if metric == "time_ms":
        color = "33"
    elif metric in HEAP_METRICS:
        color = "36"
    elif metric.startswith("throughput_"):
        color = "32"
    else:
        color = "35"
    return f"\x1b[{color}m{metric}{COLOR_RESET}"


def bench_base_and_size(bench_name: str):
    match = re.match(r"^(.*)/(\d+)$", bench_name)
    if not match:
        return bench_name, None
    base = match.group(1)
    try:
        return base, int(match.group(2))
    except ValueError:
        return bench_name, None


def load_rows(csv_path: Path):
    if not csv_path.exists():
        raise SystemExit(f"csv not found: {csv_path}")
    with csv_path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


def list_descriptions(rows):
    by_desc = defaultdict(list)
    for row in rows:
        by_desc[row["description"]].append(row)

    if not by_desc:
        print("No rows found.")
        return

    output_rows = []
    for desc, desc_rows in sorted(by_desc.items()):
        latest = max(r["timestamp"] for r in desc_rows)
        modes = sorted({r["mode"] for r in desc_rows})
        output_rows.append([desc, latest, ",".join(modes), str(len(desc_rows))])

    print_table(["description", "latest timestamp", "modes", "rows"], output_rows)


def select_rows(rows, description, mode, timestamp):
    if description:
        rows = [r for r in rows if r["description"] == description]

    if mode and mode != "all":
        rows = [r for r in rows if r["mode"] == mode]

    if not rows:
        return [], None

    if timestamp:
        rows = [r for r in rows if r["timestamp"] == timestamp]
        return rows, timestamp

    latest = max(r["timestamp"] for r in rows)
    rows = [r for r in rows if r["timestamp"] == latest]
    return rows, latest


def sparkline(values, width, missing_char="·"):
    if not values:
        return ""
    if width <= 0:
        return ""
    if len(values) > width:
        # downsample by bucket
        step = len(values) / width
        buckets = []
        for i in range(width):
            start = int(i * step)
            end = max(start + 1, int((i + 1) * step))
            chunk = values[start:end]
            numeric = [v for v in chunk if v is not None]
            if numeric:
                buckets.append(sum(numeric) / len(numeric))
            else:
                buckets.append(None)
        values = buckets

    numeric_values = [v for v in values if v is not None]
    if not numeric_values:
        return missing_char * len(values)

    lo = min(numeric_values)
    hi = max(numeric_values)
    if hi == lo:
        return "".join(SPARKS[-1] if v is not None else missing_char for v in values)
    out = []
    for v in values:
        if v is None:
            out.append(missing_char)
            continue
        idx = int((v - lo) / (hi - lo) * (len(SPARKS) - 1))
        out.append(SPARKS[idx])
    return "".join(out)


def delta_bar(delta_pct):
    steps_per_side = int(ABS_BAR_MAX // ABS_BAR_STEP)
    center = steps_per_side
    total_len = steps_per_side * 2 + 1
    bar = ["░"] * total_len

    clamped = max(ABS_BAR_MIN, min(ABS_BAR_MAX, delta_pct))
    steps = int(round(abs(clamped) / ABS_BAR_STEP))
    if steps > 0:
        if clamped > 0:
            for i in range(1, steps + 1):
                idx = center + i
                if idx < total_len:
                    bar[idx] = "█"
        else:
            for i in range(1, steps + 1):
                idx = center - i
                if idx >= 0:
                    bar[idx] = "█"

    bar[center] = "│"
    tick_step = int(100 // ABS_BAR_STEP)
    for offset in (-tick_step, tick_step):
        idx = center + offset
        if 0 <= idx < total_len:
            bar[idx] = "┆"

    prefix = ""
    suffix = ""
    if delta_pct < ABS_BAR_MIN:
        prefix = f"({delta_pct:+.2f}%) "
    elif delta_pct > ABS_BAR_MAX:
        suffix = f" ({delta_pct:+.2f}%)"

    return f"{prefix}{''.join(bar)}{suffix}"


def compare_rows(rows, top, plot, color, percent):
    for row in rows:
        try:
            row["value"] = float(row["value"])
        except (TypeError, ValueError):
            row["value"] = None

    grouped = defaultdict(dict)
    for row in rows:
        if row["value"] is None:
            continue
        key = (row["bench"], row["metric"], row["unit"])
        grouped[key][row["backend"]] = row["value"]

    comparisons = []
    for (bench, metric, unit), values in grouped.items():
        if "default" not in values or "nested" not in values:
            continue
        default = values["default"]
        nested = values["nested"]
        if default == 0:
            continue
        delta_pct = (nested / default - 1.0) * 100.0
        comparisons.append((bench, metric, unit, default, nested, delta_pct))

    if top > 0:
        comparisons.sort(key=lambda r: abs(r[5]), reverse=True)
        comparisons = comparisons[:top]
    comparisons.sort(
        key=lambda r: (
            r[1],
            bench_base_and_size(r[0])[0],
            bench_base_and_size(r[0])[1] is None,
            bench_base_and_size(r[0])[1] or 0,
            r[0],
        )
    )

    lower_rows = []
    higher_rows = []
    for bench, metric, unit, default, nested, delta_pct in comparisons:
        metric_cell = color_metric(metric, color)
        default_disp, unit_disp = format_value(default, unit, percent)
        nested_disp, unit_disp = format_value(nested, unit, percent)
        if metric == "time_ms":
            if delta_pct < 0:
                meaning = "nested faster"
            elif delta_pct > 0:
                meaning = "nested slower"
            else:
                meaning = "no change"
        elif metric in HEAP_BYTES_METRICS:
            if delta_pct < 0:
                meaning = "nested uses less memory"
            elif delta_pct > 0:
                meaning = "nested uses more memory"
            else:
                meaning = "no change"
        elif metric in HEAP_BLOCK_METRICS:
            if delta_pct < 0:
                meaning = "nested uses fewer allocations"
            elif delta_pct > 0:
                meaning = "nested uses more allocations"
            else:
                meaning = "no change"
        elif metric in CACHE_METRICS:
            if delta_pct < 0:
                meaning = "nested lower miss rate"
            elif delta_pct > 0:
                meaning = "nested higher miss rate"
            else:
                meaning = "no change"
        elif metric.startswith("throughput_"):
            if delta_pct > 0:
                meaning = "nested higher throughput"
            elif delta_pct < 0:
                meaning = "nested lower throughput"
            else:
                meaning = "no change"
        else:
            if metric in LOWER_BETTER:
                if delta_pct < 0:
                    meaning = "nested lower value"
                elif delta_pct > 0:
                    meaning = "nested higher value"
                else:
                    meaning = "no change"
            else:
                if delta_pct > 0:
                    meaning = "nested higher value"
                elif delta_pct < 0:
                    meaning = "nested lower value"
                else:
                    meaning = "no change"
        base_row = [
            bench,
            metric_cell,
            unit_disp,
            f"{default_disp:.6g}",
            f"{nested_disp:.6g}",
            f"{delta_pct:+.2f}%",
            meaning,
        ]
        if plot:
            bar = delta_bar(delta_pct)
            base_row.append(bar)
        if metric in LOWER_BETTER:
            lower_rows.append(base_row)
        else:
            higher_rows.append(base_row)

    headers = [
        "bench",
        "metric",
        "unit",
        "default",
        "nested",
        "nested vs default",
        "meaning",
    ]
    if plot:
        headers.append("nested change bar")

    if lower_rows:
        print("Lower is better:")
        print_table(headers, lower_rows)
        print("")
    if higher_rows:
        print("Higher is better:")
        print_table(headers, higher_rows)
        print("")

    if not comparisons:
        print("No comparable default/nested pairs found.")
        return

    print("")
    print("Legend:")
    print("- nested vs default = percent change using default as baseline")
    print("- meaning = plain-language interpretation for that metric")
    print("- nested change bar = fixed scale from -200% to +200% in 5% steps")
    print("- bar markers: ┆ = ±100%, │ = 0%")
    print("- values beyond ±200% show the exact percent before/after the bar")
    print("- cache ratios: lower is better (use --percent=1 to show as percent)")
    if color:
        print("- metric colors: time_ms=yellow, heap_*=cyan, throughput_*=green, other=magenta")


def aggregate_series(filtered):
    by_key = defaultdict(list)
    meta = {}
    for r in filtered:
        key = (r["timestamp"], r["backend"])
        try:
            value = float(r["value"])
        except (TypeError, ValueError):
            continue
        by_key[key].append(value)
        meta.setdefault(key, {
            "unit": r["unit"],
            "description": r["description"],
            "mode": r["mode"],
        })
    aggregated = []
    for (timestamp, backend), values in by_key.items():
        info = meta.get((timestamp, backend), {})
        aggregated.append(
            {
                "timestamp": timestamp,
                "backend": backend,
                "value": sum(values) / len(values),
                "samples": len(values),
                "unit": info.get("unit", ""),
                "description": info.get("description", ""),
                "mode": info.get("mode", ""),
            }
        )
    aggregated.sort(key=lambda r: (r["timestamp"], r["backend"]))
    return aggregated


def format_value(value, unit, percent):
    if percent and unit == "ratio":
        return value * 100.0, "percent"
    return value, unit


def series_rows(rows, bench, metric, mode, backend, plot, plot_width, raw, percent):
    if not bench or not metric:
        raise SystemExit("series mode requires --bench and --metric")

    filtered = [
        r for r in rows
        if r["bench"] == bench and r["metric"] == metric
    ]

    if mode and mode != "all":
        filtered = [r for r in filtered if r["mode"] == mode]

    if backend != "both":
        filtered = [r for r in filtered if r["backend"] == backend]

    filtered.sort(key=lambda r: (r["timestamp"], r["backend"]))

    if raw:
        output_rows = []
        for r in filtered:
            try:
                value = float(r["value"])
            except (TypeError, ValueError):
                value = None
            unit = r["unit"]
            if value is not None:
                display_value, display_unit = format_value(value, unit, percent)
                value_str = f"{display_value:.6g}"
            else:
                display_unit = unit
                value_str = r["value"]
            output_rows.append(
                [
                    r["timestamp"],
                    r["backend"],
                    value_str,
                    display_unit,
                    r["description"],
                    r["mode"],
                ]
            )
        print_table(
            ["timestamp", "backend", "value", "unit", "description", "mode"],
            output_rows,
        )
    else:
        aggregated = aggregate_series(filtered)
        output_rows = []
        for r in aggregated:
            display_value, display_unit = format_value(r["value"], r["unit"], percent)
            output_rows.append(
                [
                    r["timestamp"],
                    r["backend"],
                    f"{display_value:.6g}",
                    display_unit,
                    r["description"],
                    r["mode"],
                    str(r["samples"]),
                ]
            )
        print_table(
            ["timestamp", "backend", "value(avg)", "unit", "description", "mode", "samples"],
            output_rows,
        )

    if plot:
        if raw:
            source = filtered
        else:
            source = aggregate_series(filtered)

        timestamps = sorted({r["timestamp"] for r in source})
        by_backend = defaultdict(dict)
        unit = ""
        for r in source:
            try:
                value = float(r["value"]) if raw else float(r["value"])
            except (TypeError, ValueError):
                continue
            value, unit = format_value(value, r.get("unit", ""), percent)
            by_backend[r["backend"]][r["timestamp"]] = value
            unit = unit or r.get("unit", unit)

        print("")
        plot_rows = []
        has_missing = False
        for backend_name in sorted(by_backend.keys()):
            values = []
            for ts in timestamps:
                value = by_backend[backend_name].get(ts)
                if value is None:
                    has_missing = True
                values.append(value)
            numeric_values = [v for v in values if v is not None]
            if not numeric_values:
                continue
            plot_line = sparkline(values, plot_width)
            plot_rows.append(
                [
                    backend_name,
                    plot_line,
                    f"{min(numeric_values):.6g}",
                    f"{max(numeric_values):.6g}",
                    f"{numeric_values[-1]:.6g}",
                    unit,
                ]
            )

        print_table(
            ["backend", "sparkline", "min", "max", "latest", "unit"],
            plot_rows,
        )
        print("Note: sparklines are scaled per-backend (local min/max).")
        if has_missing:
            print("Note: · in sparkline means missing data for that timestamp.")

    if not filtered:
        print("No rows found for the given series.")


def main():
    key_map = {
        "csv": "--csv",
        "description": "--description",
        "mode": "--mode",
        "timestamp": "--timestamp",
        "bench": "--bench",
        "metric": "--metric",
        "backend": "--backend",
        "list": "--list",
        "series": "--series",
        "series_raw": "--series-raw",
        "series-raw": "--series-raw",
        "plot": "--plot",
        "plot_width": "--plot-width",
        "plot-width": "--plot-width",
        "color": "--color",
        "top": "--top",
        "percent": "--percent",
    }
    raw_args = sys.argv[1:]
    mapped_args = []
    for arg in raw_args:
        if arg.startswith("-"):
            mapped_args.append(arg)
            continue
        if "=" in arg:
            key, value = arg.split("=", 1)
            flag = key_map.get(key)
            if flag:
                mapped_args.extend([flag, value])
                continue
        mapped_args.append(arg)
    sys.argv[1:] = mapped_args

    parser = argparse.ArgumentParser(
        description=(
            "Analyze benchmark history CSV files. Compare default vs nested backends, "
            "list available descriptions, or inspect per-benchmark time series."
        ),
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "--csv",
        default="bench_results/history.csv",
        help="Path to the benchmark history CSV file.",
    )
    parser.add_argument(
        "--description",
        default="",
        help="Filter rows by exact benchmark run description.",
    )
    parser.add_argument(
        "--mode",
        default="full",
        help="Filter by benchmark mode (set to 'all' to disable mode filtering).",
    )
    parser.add_argument(
        "--timestamp",
        default="",
        help="Exact timestamp to analyze; when omitted, the latest matching timestamp is used.",
    )
    parser.add_argument(
        "--bench",
        default="",
        help="Exact benchmark name for series mode (required with --series=1).",
    )
    parser.add_argument(
        "--metric",
        default="",
        help="Exact metric name for series mode (required with --series=1).",
    )
    parser.add_argument(
        "--backend",
        default="both",
        help="Backend filter: 'default', 'nested', or 'both'.",
    )
    parser.add_argument(
        "--list",
        default="0",
        help="List descriptions and exit. Truthy values: 1,true,yes,y,on.",
    )
    parser.add_argument(
        "--series",
        default="0",
        help="Show historical rows for one bench+metric instead of backend comparison.",
    )
    parser.add_argument(
        "--plot",
        default="0",
        help="Include an ASCII visualization (delta bar in compare mode, sparkline in series mode).",
    )
    parser.add_argument(
        "--plot-width",
        default="24",
        help="Target sparkline width (characters) in series mode.",
    )
    parser.add_argument(
        "--color",
        default="1",
        help="Colorize metric names in compare mode. Truthy values: 1,true,yes,y,on.",
    )
    parser.add_argument(
        "--series-raw",
        default="0",
        help="In series mode, print raw rows instead of timestamp/backend averages.",
    )
    parser.add_argument(
        "--top",
        default="0",
        help="If >0, keep only the N largest absolute nested-vs-default deltas.",
    )
    parser.add_argument(
        "--percent",
        default="0",
        help="Display ratio units as percent (multiply by 100).",
    )
    args = parser.parse_args()

    rows = load_rows(Path(args.csv))

    if truthy(args.list):
        list_descriptions(rows)
        return

    if truthy(args.series):
        series_rows(
            rows,
            args.bench,
            args.metric,
            args.mode,
            args.backend,
            truthy(args.plot),
            int(args.plot_width),
            truthy(args.series_raw),
            truthy(args.percent),
        )
        return

    selected, timestamp = select_rows(
        rows,
        args.description,
        args.mode,
        args.timestamp,
    )
    if not selected:
        print("No rows found for the given filters.")
        if args.description:
            print("Try --mode all or a different description.")
        return

    print(f"timestamp: {timestamp}")
    compare_rows(
        selected,
        int(args.top),
        truthy(args.plot),
        truthy(args.color),
        truthy(args.percent),
    )


if __name__ == "__main__":
    main()
