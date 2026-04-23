#!/usr/bin/env python3
import argparse
import csv
import re
import sys
from collections import defaultdict
from pathlib import Path


SPARKS = "▁▂▃▄▅▆▇█"
ABS_BAR_MIN = -200.0
ABS_BAR_MAX = 200.0
ABS_BAR_STEP = 5.0
ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
COLOR_RESET = "\x1b[0m"
DELTA_EPSILON_PCT = 1e-9


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

    print("Descriptions:")
    print_table(["description", "latest timestamp", "modes", "rows"], output_rows)
    print("")

    by_bench = defaultdict(list)
    for row in rows:
        by_bench[row["bench"]].append(row)

    bench_rows = []
    for bench, bench_data in sorted(
        by_bench.items(),
        key=lambda item: (
            bench_base_and_size(item[0])[0],
            bench_base_and_size(item[0])[1] is None,
            bench_base_and_size(item[0])[1] or 0,
            item[0],
        ),
    ):
        metric_count = len({r["metric"] for r in bench_data})
        bench_rows.append([bench, str(metric_count), str(len(bench_data))])
    print("Benches:")
    print_table(["bench", "metrics", "rows"], bench_rows)
    print("")

    by_metric = defaultdict(list)
    for row in rows:
        by_metric[row["metric"]].append(row)

    metric_rows = []
    for metric, metric_data in sorted(by_metric.items()):
        units = sorted({r["unit"] for r in metric_data})
        metric_rows.append([metric, ",".join(units), str(len(metric_data))])
    print("Metrics:")
    print_table(["metric", "units", "rows"], metric_rows)
    print("")


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


def format_pct(value):
    return f"{value:+.2f}%"


def metric_is_lower_better(metric, unit=""):
    metric_lc = (metric or "").lower()
    unit_lc = (unit or "").lower()

    if metric_lc.startswith("throughput_"):
        return False
    if "miss_rate" in metric_lc:
        return True
    if metric_lc.startswith("time_") or metric_lc.endswith("_ms"):
        return True
    if metric_lc.startswith("heap_") or metric_lc.startswith("memory_"):
        return True
    if unit_lc in {"bytes", "blocks"}:
        return True
    return False


def metric_effect(metric, delta_pct, unit=""):
    return -delta_pct if metric_is_lower_better(metric, unit) else delta_pct


def status_symbol(effect, threshold=0.0):
    floor = max(float(threshold), DELTA_EPSILON_PCT)
    if abs(effect) <= floor:
        return "="
    return "+" if effect > 0 else "-"



def compact_timestamp(timestamp):
    match = re.match(r"^(\d{4}-\d{2}-\d{2})T(\d{2}:\d{2})", timestamp)
    if not match:
        return timestamp
    return f"{match.group(1)} {match.group(2)}"


def summarize_run(run_rows):
    by_key = defaultdict(list)
    for row in run_rows:
        try:
            value = float(row["value"])
        except (TypeError, ValueError):
            continue
        key = (row["bench"], row["metric"], row["unit"])
        by_key[key].append(value)

    snapshot = {}
    for key, values in by_key.items():
        snapshot[key] = sum(values) / len(values)

    commits = sorted({r["commit"] for r in run_rows if r["commit"]})
    return {
        "snapshot": snapshot,
        "commit": ",".join(commits) if commits else "",
    }


def format_change_key(change_item):
    if change_item is None:
        return "-"
    key, delta_pct = change_item
    bench, metric, _unit = key
    effect = metric_effect(metric, delta_pct, _unit)
    return f"{bench}/{metric} ({format_pct(delta_pct)}, impact {format_pct(effect)})"


def colorize_row(row, color_code):
    return [f"\x1b[{color_code}m{cell}{COLOR_RESET}" for cell in row]


def summarize_snapshot_delta(prev_snapshot, curr_snapshot, threshold):
    prev_keys = set(prev_snapshot.keys())
    curr_keys = set(curr_snapshot.keys())
    shared_keys = sorted(prev_keys & curr_keys)
    added_count = len(curr_keys - prev_keys)
    removed_count = len(prev_keys - curr_keys)

    improved = 0
    regressed = 0
    unchanged = 0
    delta_values = []
    best_improvement = None
    worst_regression = None

    for key in shared_keys:
        prev_value = prev_snapshot[key]
        curr_value = curr_snapshot[key]
        if prev_value == 0:
            continue
        delta_pct = (curr_value / prev_value - 1.0) * 100.0
        delta_values.append(delta_pct)

        effect = metric_effect(key[1], delta_pct, key[2])
        status = status_symbol(effect, threshold)
        if status == "=":
            unchanged += 1
            continue

        if status == "+":
            improved += 1
            if best_improvement is None or effect > best_improvement[0]:
                best_improvement = (effect, key, delta_pct)
        else:
            regressed += 1
            if worst_regression is None or effect < worst_regression[0]:
                worst_regression = (effect, key, delta_pct)

    comparable = len(delta_values)
    net = improved - regressed
    if comparable > 0:
        mean_abs = sum(abs(v) for v in delta_values) / comparable
        mean_signed = sum(delta_values) / comparable
        mean_abs_str = f"{mean_abs:.2f}%"
        mean_signed_str = f"{mean_signed:+.2f}%"
    else:
        mean_abs_str = "-"
        mean_signed_str = "-"

    return {
        "comparable": comparable,
        "improved": improved,
        "regressed": regressed,
        "unchanged": unchanged,
        "added": added_count,
        "removed": removed_count,
        "net": net,
        "mean_abs": mean_abs_str,
        "mean_signed": mean_signed_str,
        "best": format_change_key((best_improvement[1], best_improvement[2]) if best_improvement else None),
        "worst": format_change_key((worst_regression[1], worst_regression[2]) if worst_regression else None),
    }


def metric_delta_rows(prev_snapshot, curr_snapshot, color, percent, threshold):
    shared_keys = sorted(
        set(prev_snapshot.keys()) & set(curr_snapshot.keys()),
        key=lambda key: (
            bench_base_and_size(key[0])[0],
            bench_base_and_size(key[0])[1] is None,
            bench_base_and_size(key[0])[1] or 0,
            key[0],
            key[1],
            key[2],
        ),
    )

    improvements = []
    regressions = []
    neutral_rows = []
    unchanged = 0
    skipped_zero = 0

    for key in shared_keys:
        bench, metric, unit = key
        prev_value = prev_snapshot[key]
        curr_value = curr_snapshot[key]
        if prev_value == 0:
            skipped_zero += 1
            continue

        delta_pct = (curr_value / prev_value - 1.0) * 100.0
        effect = metric_effect(metric, delta_pct, unit)
        status = status_symbol(effect, threshold)

        row = [
            bench,
            metric,
            display_unit(unit, percent),
            format_pct(delta_pct),
            format_pct(effect),
            status,
        ]
        if status == "=":
            unchanged += 1
            if threshold > 0:
                neutral_rows.append(row)
            continue

        if effect > 0:
            improvements.append((effect, row))
        else:
            regressions.append((effect, row))

    improvements.sort(key=lambda item: (-item[0], item[1][0], item[1][1]))
    regressions.sort(key=lambda item: (item[0], item[1][0], item[1][1]))

    rows = []
    for _effect, row in improvements:
        rows.append(colorize_row(row, "32") if color else row)
    for _effect, row in regressions:
        rows.append(colorize_row(row, "31") if color else row)
    rows.extend(neutral_rows)

    return rows, len(improvements), len(regressions), unchanged, skipped_zero


def print_metric_delta_table(prev_snapshot, curr_snapshot, color, label, percent, threshold):
    print("")
    print(f"Benchmark/metric changes ({label}):")
    rows, improved_count, regressed_count, unchanged_count, skipped_zero = metric_delta_rows(
        prev_snapshot,
        curr_snapshot,
        color,
        percent,
        threshold,
    )
    if not rows:
        print("No changed comparable benchmark/metric pairs.")
        return

    print_table(["bench", "metric", "unit", "Δ", "impact", "status"], rows)
    print("")
    print(
        "Summary: "
        f"improved={improved_count}, regressed={regressed_count}, unchanged={unchanged_count}, "
        f"zero-baseline-skipped={skipped_zero}"
    )
    if threshold > 0:
        print(f"Significance threshold: |impact| > {threshold:.2f}%")
    print("Note: metric direction is miss rates/time/memory lower-better, throughput higher-better.")
    print("Note: status symbols use metric direction (+ improved, - regressed, = unchanged).")


def average_snapshots(snapshots):
    by_key = defaultdict(list)
    for snapshot in snapshots:
        for key, value in snapshot.items():
            by_key[key].append(value)

    averaged = {}
    for key, values in by_key.items():
        averaged[key] = sum(values) / len(values)
    return averaged


def format_commit_run_window(first_timestamp, last_timestamp, run_count):
    first = compact_timestamp(first_timestamp)
    last = compact_timestamp(last_timestamp)
    if run_count <= 1 or first == last:
        return f"{first} (1 run)"
    return f"{first}..{last} ({run_count} runs)"


def resolve_compare_commit(filtered_rows, selector):
    selector = selector.strip()
    if not selector:
        raise SystemExit("--compare requires two non-empty commit selectors")

    matches = sorted({r["commit"] for r in filtered_rows if r["commit"] and r["commit"].startswith(selector)})
    if not matches:
        raise SystemExit(f'No commit matched selector "{selector}" with current filters')
    if len(matches) > 1:
        raise SystemExit(
            f'Ambiguous commit selector "{selector}" matched: {", ".join(matches)}. '
            "Use a longer prefix."
        )

    commit = matches[0]
    commit_rows = [r for r in filtered_rows if r["commit"] == commit]
    if not commit_rows:
        raise SystemExit(f'No rows matched commit selector "{selector}" with current filters')

    by_timestamp = defaultdict(list)
    for row in commit_rows:
        by_timestamp[row["timestamp"]].append(row)

    timestamps = sorted(by_timestamp.keys())
    snapshots = []
    for timestamp in timestamps:
        summary = summarize_run(by_timestamp[timestamp])
        snapshots.append(summary["snapshot"])

    return {
        "commit": commit,
        "snapshot": average_snapshots(snapshots),
        "run_count": len(timestamps),
        "first_timestamp": timestamps[0],
        "last_timestamp": timestamps[-1],
    }


def resolve_baseline_description(filtered_rows, description):
    matches = [r for r in filtered_rows if r["description"] == description]
    if not matches:
        raise SystemExit(f'No rows found for baseline description "{description}"')

    by_timestamp = defaultdict(list)
    for row in matches:
        by_timestamp[row["timestamp"]].append(row)

    timestamps = sorted(by_timestamp.keys())
    snapshots = [summarize_run(by_timestamp[ts])["snapshot"] for ts in timestamps]

    return {
        "description": description,
        "snapshot": average_snapshots(snapshots),
        "run_count": len(timestamps),
        "first_timestamp": timestamps[0],
        "last_timestamp": timestamps[-1],
    }


def parse_compare_arg(compare):
    parts = [part.strip() for part in compare.split(",")]
    parts = [part for part in parts if part]
    if len(parts) != 2:
        raise SystemExit("--compare expects exactly two commits: --compare commit1,commit2")
    return parts[0], parts[1]


def print_overview_legend(show_extremes, threshold):
    print("")
    print("Legend:")
    print("- pairs: comparable bench+metric pairs shared with the previous run")
    print("- + / - / = : improved / regressed / unchanged")
    print("- metric direction: miss rates/time/memory lower is better; throughput higher is better")
    print("- +new / -old: pairs only present in current/previous run")
    print("- net: improved - regressed")
    if threshold > 0:
        print(f"- significance threshold: |impact| > {threshold:.2f}%")
    if show_extremes:
        print("- best/worst: largest single-pair movement relative to previous run")


def backend_overview(rows, backend, mode, description, show_extremes, compare, baseline, color, percent, threshold):
    filtered = [r for r in rows if r["backend"] == backend]
    if mode and mode != "all":
        filtered = [r for r in filtered if r["mode"] == mode]

    if not filtered:
        print("No rows found for backend overview with the given filters.")
        return

    if compare and baseline:
        raise SystemExit("--compare and --baseline are mutually exclusive")

    if baseline:
        base_info = resolve_baseline_description(filtered, baseline)

        if description:
            curr_rows = [r for r in filtered if r["description"] == description]
            if not curr_rows:
                raise SystemExit(f'No rows found for description "{description}"')
            curr_label = f'"{description}"'
        else:
            non_baseline = [r for r in filtered if r["description"] != baseline]
            if not non_baseline:
                raise SystemExit("No non-baseline rows found for comparison")
            latest_ts = max(r["timestamp"] for r in non_baseline)
            curr_rows = [r for r in non_baseline if r["timestamp"] == latest_ts]
            curr_label = f"latest ({compact_timestamp(latest_ts)})"

        curr_by_ts = defaultdict(list)
        for row in curr_rows:
            curr_by_ts[row["timestamp"]].append(row)
        curr_timestamps = sorted(curr_by_ts.keys())
        curr_snapshots = [summarize_run(curr_by_ts[ts])["snapshot"] for ts in curr_timestamps]
        curr_avg_snapshot = average_snapshots(curr_snapshots)

        base_window = format_commit_run_window(
            base_info["first_timestamp"], base_info["last_timestamp"], base_info["run_count"]
        )
        base_label = f'"{baseline}" ({base_window})'
        curr_label_full = f"{curr_label} ({len(curr_timestamps)} run(s))"

        print_metric_delta_table(
            base_info["snapshot"],
            curr_avg_snapshot,
            color,
            f"{base_label} -> {curr_label_full}",
            percent,
            threshold,
        )
        print_overview_legend(show_extremes, threshold)
        return

    if description:
        filtered = [r for r in filtered if r["description"] == description]

    if not filtered:
        print("No rows found for backend overview with the given filters.")
        return

    by_timestamp = defaultdict(list)
    for row in filtered:
        by_timestamp[row["timestamp"]].append(row)

    run_summaries = []
    for timestamp in sorted(by_timestamp.keys()):
        summary = summarize_run(by_timestamp[timestamp])
        run_summaries.append(
            {
                "timestamp": timestamp,
                "ts": compact_timestamp(timestamp),
                "commit": summary["commit"],
                "snapshot": summary["snapshot"],
            }
        )

    if compare:
        left_selector, right_selector = parse_compare_arg(compare)
        left_run = resolve_compare_commit(filtered, left_selector)
        right_run = resolve_compare_commit(filtered, right_selector)

        delta = summarize_snapshot_delta(left_run["snapshot"], right_run["snapshot"], threshold)
        row = [
            f"{left_run['commit']} -> {right_run['commit']}",
            (
                f"{format_commit_run_window(left_run['first_timestamp'], left_run['last_timestamp'], left_run['run_count'])} "
                f"-> {format_commit_run_window(right_run['first_timestamp'], right_run['last_timestamp'], right_run['run_count'])}"
            ),
            str(delta["comparable"]),
            str(delta["improved"]),
            str(delta["regressed"]),
            str(delta["unchanged"]),
            str(delta["added"]),
            str(delta["removed"]),
            str(delta["net"]),
            delta["mean_abs"],
            delta["mean_signed"],
        ]
        if color and delta["net"] != 0:
            row = colorize_row(row, "32" if delta["net"] > 0 else "31")

        print(f"Backend compare: {backend}")
        print_table(
            [
                "commits",
                "runs",
                "pairs",
                "+",
                "-",
                "=",
                "+new",
                "-old",
                "net",
                "|Δ|",
                "Δ",
            ],
            [row],
        )
        if show_extremes:
            print("")
            print("Largest moves:")
            print_table(["best", "worst"], [[delta["best"], delta["worst"]]])
        print_metric_delta_table(
            left_run["snapshot"],
            right_run["snapshot"],
            color,
            (
                f"{left_run['commit']} ({left_run['run_count']} run(s), "
                f"{format_commit_run_window(left_run['first_timestamp'], left_run['last_timestamp'], left_run['run_count'])})"
                " -> "
                f"{right_run['commit']} ({right_run['run_count']} run(s), "
                f"{format_commit_run_window(right_run['first_timestamp'], right_run['last_timestamp'], right_run['run_count'])})"
            ),
            percent,
            threshold,
        )
        print("Note: commit compare uses per-commit averages across all matching runs after filters.")
        print_overview_legend(show_extremes, threshold)
        return

    run_rows = []
    extreme_rows = []
    prev_snapshot = None
    for summary in run_summaries:
        snapshot = summary["snapshot"]
        compact_ts = summary["ts"]
        commit = summary["commit"]

        if prev_snapshot is None:
            run_rows.append(
                [
                    compact_ts,
                    commit,
                    "0",
                    "0",
                    "0",
                    "0",
                    str(len(snapshot)),
                    "0",
                    "0",
                    "-",
                    "-",
                ]
            )
            extreme_rows.append([compact_ts, commit, "-", "-"])
            prev_snapshot = snapshot
            continue

        delta = summarize_snapshot_delta(prev_snapshot, snapshot, threshold)
        run_rows.append(
            [
                compact_ts,
                commit,
                str(delta["comparable"]),
                str(delta["improved"]),
                str(delta["regressed"]),
                str(delta["unchanged"]),
                str(delta["added"]),
                str(delta["removed"]),
                str(delta["net"]),
                delta["mean_abs"],
                delta["mean_signed"],
            ]
        )
        extreme_rows.append(
            [
                compact_ts,
                commit,
                delta["best"],
                delta["worst"],
            ]
        )
        prev_snapshot = snapshot

    print(f"Backend overview: {backend}")
    print_table(
        [
            "ts",
            "commit",
            "pairs",
            "+",
            "-",
            "=",
            "+new",
            "-old",
            "net",
            "|Δ|",
            "Δ",
        ],
        run_rows,
    )
    if show_extremes:
        print("")
        print("Largest moves per run:")
        print_table(["ts", "commit", "best", "worst"], extreme_rows)
    if len(run_summaries) >= 2:
        first_run = run_summaries[0]
        last_run = run_summaries[-1]
        print_metric_delta_table(
            first_run["snapshot"],
            last_run["snapshot"],
            color,
            f"{first_run['commit']} @ {first_run['ts']} -> {last_run['commit']} @ {last_run['ts']}",
            percent,
            threshold,
        )
    print_overview_legend(show_extremes, threshold)


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


def display_unit(unit, percent):
    if percent and unit == "ratio":
        return "%"
    if unit == "percent":
        return "%"
    return unit or "-"


def format_value(value, unit, percent):
    if percent and unit == "ratio":
        return value * 100.0, display_unit(unit, percent)
    return value, display_unit(unit, percent)


def series_rows(rows, bench, metric, mode, backend, plot, plot_width, raw, percent, color, threshold):
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
                value_display, unit_label = format_value(value, unit, percent)
                value_str = f"{value_display:.6g}"
            else:
                unit_label = display_unit(unit, percent)
                value_str = r["value"]
            output_rows.append(
                [
                    r["timestamp"],
                    r["backend"],
                    value_str,
                    unit_label,
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
            value_display, unit_label = format_value(r["value"], r["unit"], percent)
            output_rows.append(
                [
                    r["timestamp"],
                    r["backend"],
                    f"{value_display:.6g}",
                    unit_label,
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
            first_value = next((v for v in values if v is not None), None)
            latest_value = next((v for v in reversed(values) if v is not None), None)
            if first_value is None or latest_value is None:
                continue

            if first_value == 0:
                delta_str = "-"
                impact_str = "-"
                status = "n/a"
                effect = 0.0
            else:
                delta_pct = (latest_value / first_value - 1.0) * 100.0
                effect = metric_effect(metric, delta_pct, unit)
                delta_str = format_pct(delta_pct)
                impact_str = format_pct(effect)
                status = status_symbol(effect, threshold)

            row = [
                backend_name,
                plot_line,
                f"{first_value:.6g}",
                f"{latest_value:.6g}",
                delta_str,
                impact_str,
                status,
                unit,
            ]
            if color and status in {"+", "-"}:
                row = colorize_row(row, "32" if effect > 0 else "31")
            plot_rows.append(row)

        print_table(
            ["backend", "sparkline", "first", "latest", "Δ", "impact", "status", "unit"],
            plot_rows,
        )
        print("Note: sparklines are scaled per-backend (local min/max).")
        print("Note: metric direction is miss rates/time/memory lower-better, throughput higher-better.")
        print("Note: status symbols use metric direction (+ improved, - regressed, = unchanged).")
        if threshold > 0:
            print(f"Note: significance threshold for status/color is |impact| > {threshold:.2f}%.")
        if has_missing:
            print("Note: · in sparkline means missing data for that timestamp.")

    if not filtered:
        print("No rows found for the given series.")


def main():
    class HelpFormatter(
        argparse.ArgumentDefaultsHelpFormatter,
        argparse.RawDescriptionHelpFormatter,
    ):
        pass

    key_map = {
        "csv": "--csv",
        "description": "--description",
        "mode": "--mode",
        "timestamp": "--timestamp",
        "bench": "--bench",
        "metric": "--metric",
        "backend": "--backend",
        "list": "--list",
        "backend_overview": "--backend-overview",
        "backend-overview": "--backend-overview",
        "backend_overview_extremes": "--backend-overview-extremes",
        "backend-overview-extremes": "--backend-overview-extremes",
        "series": "--series",
        "series_raw": "--series-raw",
        "series-raw": "--series-raw",
        "compare": "--compare",
        "baseline": "--baseline",
        "plot": "--plot",
        "plot_width": "--plot-width",
        "plot-width": "--plot-width",
        "color": "--color",
        "threshold": "--threshold",
        "percent": "--percent",
    }
    bool_keys = {
        "list",
        "backend_overview",
        "backend-overview",
        "backend_overview_extremes",
        "backend-overview-extremes",
        "series",
        "series_raw",
        "series-raw",
        "plot",
        "color",
        "percent",
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
                if key in bool_keys:
                    if truthy(value):
                        mapped_args.append(flag)
                    elif key == "color":
                        mapped_args.append("--no-color")
                else:
                    mapped_args.extend([flag, value])
                continue
        mapped_args.append(arg)
    sys.argv[1:] = mapped_args

    parser = argparse.ArgumentParser(
        description=(
            "Analyze benchmark history CSV files. "
            "Four modes: run history overview (default), baseline comparison, "
            "commit-to-commit comparison, and per-benchmark time series."
        ),
        epilog=(
            "Modes:\n"
            "  (default)     Show run-by-run history for the default backend:\n"
            "                  just perf-analyze\n"
            "                  just perf-analyze --mode all\n"
            "                  just perf-analyze --backend-overview-extremes\n"
            "\n"
            "  --list        List all descriptions, benches, and metrics in the CSV:\n"
            "                  just perf-analyze --list\n"
            "\n"
            "  --baseline    Compare current/latest run against a named baseline description:\n"
            "                  just perf-analyze baseline=\"<saved description>\"\n"
            "                  just perf-analyze description=\"<current>\" baseline=\"<saved>\"\n"
            "                  just perf-analyze baseline=\"<saved>\" --percent --threshold 1\n"
            "\n"
            "  --compare     Compare two specific commits (prefix match):\n"
            "                  just perf-analyze --compare abc1234,def5678\n"
            "                  just perf-analyze --compare abc1234,def5678 --backend-overview-extremes\n"
            "\n"
            "  --series      Time series for one bench+metric pair:\n"
            "                  just perf-analyze --series bench=\"<bench>\" metric=\"<metric>\"\n"
            "                  just perf-analyze --series --plot bench=\"<bench>\" metric=\"<metric>\"\n"
            "                  just perf-analyze --series --series-raw bench=\"<bench>\" metric=\"<metric>\"\n"
        ),
        formatter_class=HelpFormatter,
    )
    parser.add_argument(
        "--csv",
        default="benches/results/history.csv",
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
        help="Exact benchmark name for series mode (required with --series).",
    )
    parser.add_argument(
        "--metric",
        default="",
        help="Exact metric name for series mode (required with --series).",
    )
    parser.add_argument(
        "--backend",
        default="default",
        help="Backend filter: 'default' (or 'nested' for historical queries).",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="List descriptions, benches, and metrics, then exit.",
    )
    parser.add_argument(
        "--series",
        action="store_true",
        help="Show historical rows for one bench+metric instead of backend comparison.",
    )
    parser.add_argument(
        "--backend-overview",
        action="store_true",
        help="Show whole-suite run-to-run deltas for one backend across timestamps.",
    )
    parser.add_argument(
        "--backend-overview-extremes",
        action="store_true",
        help="With --backend-overview, include per-run best/worst single-pair changes.",
    )
    parser.add_argument(
        "--compare",
        default="",
        help=(
            "Compare two commits using per-commit averages across all matching runs: "
            "--compare commit1,commit2. Mutually exclusive with --baseline."
        ),
    )
    parser.add_argument(
        "--baseline",
        default="",
        help=(
            "Compare current/latest run against a named baseline description. "
            "Use --description to select the current side; omit to use the latest non-baseline run. "
            "Mutually exclusive with --compare."
        ),
    )
    parser.add_argument(
        "--plot",
        action="store_true",
        help="Include an ASCII visualization (delta bar in compare mode, sparkline in series mode).",
    )
    parser.add_argument(
        "--plot-width",
        type=int,
        default=24,
        help="Target sparkline width (characters) in series mode.",
    )
    parser.add_argument(
        "--color",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Colorize rows by outcome (green improved, red regressed; use --no-color to disable).",
    )
    parser.add_argument(
        "--series-raw",
        action="store_true",
        help="In series mode, print raw rows instead of timestamp/backend averages.",
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=0.0,
        help=(
            "Significance threshold in percent for status/color. "
            "Only impacts with absolute value greater than this are marked +/- and colored."
        ),
    )
    parser.add_argument(
        "--percent",
        action="store_true",
        help="Display ratio units as percent (multiply by 100).",
    )
    args = parser.parse_args()
    if args.threshold < 0:
        raise SystemExit("--threshold must be >= 0")

    rows = load_rows(Path(args.csv))

    if args.list:
        list_descriptions(rows)
        return

    if args.series:
        series_rows(
            rows,
            args.bench,
            args.metric,
            args.mode,
            args.backend,
            args.plot,
            args.plot_width,
            args.series_raw,
            args.percent,
            args.color,
            args.threshold,
        )
        return

    backend_overview(
        rows,
        args.backend,
        args.mode,
        args.description,
        args.backend_overview_extremes,
        args.compare,
        args.baseline,
        args.color,
        args.percent,
        args.threshold,
    )


if __name__ == "__main__":
    main()
