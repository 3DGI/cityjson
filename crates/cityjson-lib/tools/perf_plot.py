#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["matplotlib"]
# ///

from __future__ import annotations

import argparse
import csv
import shutil
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
import re

import matplotlib.lines as mlines
import matplotlib.pyplot as plt


BASELINE_FORMAT = "serde_json::Value"
LOGICAL_METRIC = "logical_throughput_bytes_s"
TIME_METRIC = "time_ms"
PHYSICAL_METRIC = "throughput_bytes_s"
SUITE_NAMES = {
    "deserialize": "read",
    "serialize": "write",
}
CASE_LABELS = {
    "io_3dbag_cityjson": "3DBAG tile",
    "io_3dbag_cityjson_cluster_4x": "3DBAG cluster 4x",
}
FORMAT_LABELS = {
    "serde_json::Value": "serde_json::Value",
    "cityjson_lib": "cityjson_lib",
    "cityjson_lib::json": "cityjson_lib::json",
    "cityarrow": "cityarrow",
    "cityparquet": "cityparquet",
}
FORMAT_ORDER = (
    "cityjson_lib",
    "cityjson_lib::json",
    "cityarrow",
    "cityparquet",
)
FORMAT_COLORS = {
    "cityjson_lib": "#1b9e77",
    "cityjson_lib::json": "#7570b3",
    "cityarrow": "#d95f02",
    "cityparquet": "#e7298a",
}
MARKER = "D"
MARKER_SIZE = 70


@dataclass(frozen=True)
class ParsedBench:
    suite_key: str
    case_id: str
    format_id: str
    operation: str

    @property
    def suite_name(self) -> str:
        return SUITE_NAMES[self.suite_key]


@dataclass(frozen=True)
class PerfPoint:
    score: float
    display_value: float
    display_unit: str


def slugify(value: str) -> str:
    normalized = re.sub(r"[^a-zA-Z0-9._-]+", "-", value.strip())
    compact = normalized.strip("-")
    if compact:
        return compact
    return "snapshot"


def load_rows(path: Path) -> list[dict[str, str]]:
    if not path.exists():
        raise SystemExit(f"csv not found: {path}")
    with path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


def select_rows(
    rows: list[dict[str, str]],
    description: str | None,
    mode: str,
    timestamp: str | None,
) -> tuple[list[dict[str, str]], str, str]:
    filtered = rows
    if description:
        filtered = [row for row in filtered if row["description"] == description]
    if mode != "all":
        filtered = [row for row in filtered if row["mode"] == mode]
    if not filtered:
        raise SystemExit("no rows found for the selected filters")

    if timestamp:
        selected = [row for row in filtered if row["timestamp"] == timestamp]
        if not selected:
            raise SystemExit("no rows found for the selected timestamp")
        selected_description = description or selected[0]["description"]
        return selected, timestamp, selected_description

    latest_timestamp = max(row["timestamp"] for row in filtered)
    selected = [row for row in filtered if row["timestamp"] == latest_timestamp]
    selected_description = description or selected[0]["description"]
    return selected, latest_timestamp, selected_description


def parse_bench(bench: str) -> ParsedBench | None:
    parts = bench.split("/")
    if len(parts) != 4:
        return None
    suite_key, case_id, format_id, operation = parts
    if suite_key not in SUITE_NAMES:
        return None
    format_aliases = {
        "serde_cityjson": "cityjson_lib",
    }
    return ParsedBench(
        suite_key=suite_key,
        case_id=case_id,
        format_id=format_aliases.get(format_id, format_id),
        operation=operation,
    )


def select_metric_source(rows: list[dict[str, str]]) -> str:
    metrics = {row["metric"] for row in rows}
    if LOGICAL_METRIC in metrics:
        return LOGICAL_METRIC
    if TIME_METRIC in metrics:
        return TIME_METRIC
    if PHYSICAL_METRIC in metrics:
        return PHYSICAL_METRIC
    raise SystemExit("no supported performance metric rows found for the selected snapshot")


def collect_performance(
    rows: list[dict[str, str]],
    metric_source: str,
) -> dict[str, dict[str, dict[str, PerfPoint]]]:
    collected: dict[str, dict[str, dict[str, PerfPoint]]] = defaultdict(
        lambda: defaultdict(dict)
    )
    for row in rows:
        if row["metric"] != metric_source:
            continue
        parsed = parse_bench(row["bench"])
        if parsed is None:
            continue
        if parsed.operation not in {"read", "write"}:
            continue
        value = float(row["value"])
        if metric_source == TIME_METRIC:
            if value <= 0.0:
                continue
            point = PerfPoint(
                score=1.0 / value,
                display_value=value,
                display_unit="ms",
            )
        else:
            if value <= 0.0:
                continue
            point = PerfPoint(
                score=value,
                display_value=value,
                display_unit="bytes_s",
            )
        collected[parsed.suite_name][parsed.case_id][parsed.format_id] = point
    return collected


def case_sort_key(case_id: str) -> tuple[int, str]:
    if case_id == "io_3dbag_cityjson":
        return (0, case_id)
    if case_id == "io_3dbag_cityjson_cluster_4x":
        return (1, case_id)
    return (2, case_id)


def case_label(case_id: str) -> str:
    return CASE_LABELS.get(case_id, case_id)


def format_label(format_id: str) -> str:
    return FORMAT_LABELS.get(format_id, format_id)


def format_mib_per_s(value_bytes_s: float) -> str:
    return f"{value_bytes_s / (1024.0 * 1024.0):.1f} MiB/s"


def metric_label(metric_source: str) -> str:
    if metric_source == LOGICAL_METRIC:
        return "relative speed using a common logical dataset-size denominator"
    if metric_source == TIME_METRIC:
        return "relative speed derived from wall-clock time"
    return "relative encoded-byte throughput"


def axis_label(metric_source: str) -> str:
    if metric_source == LOGICAL_METRIC:
        return "Relative speed (>1 = faster than serde_json::Value, common logical bytes)"
    if metric_source == TIME_METRIC:
        return "Relative speed (>1 = faster than serde_json::Value, inverse wall-clock time)"
    return "Relative throughput (>1 = faster than serde_json::Value)"


def format_point(point: PerfPoint) -> str:
    if point.display_unit == "ms":
        return f"{point.display_value:.1f} ms"
    return format_mib_per_s(point.display_value)


def render_cell(value: PerfPoint | None, baseline: PerfPoint | None) -> str:
    if value is None:
        return "-"
    if baseline is None or baseline.score <= 0.0:
        return format_point(value)
    ratio = value.score / baseline.score
    return f"{ratio:.2f}x ({format_point(value)})"


def plot_suite(
    output_path: Path,
    suite_name: str,
    suite_data: dict[str, dict[str, PerfPoint]],
    metric_source: str,
) -> bool:
    case_ids = sorted(
        [
            case_id
            for case_id, values in suite_data.items()
            if BASELINE_FORMAT in values
        ],
        key=case_sort_key,
    )
    if not case_ids:
        return False

    plt.style.use("seaborn-v0_8-muted")
    figure, axis = plt.subplots(figsize=(10, max(3.2, 0.8 * len(case_ids))))

    max_ratio = 1.0
    plotted_any = False
    for format_id in FORMAT_ORDER:
        xs: list[float] = []
        ys: list[int] = []
        for index, case_id in enumerate(case_ids):
            baseline = suite_data[case_id].get(BASELINE_FORMAT)
            value = suite_data[case_id].get(format_id)
            if baseline is None or baseline.score <= 0.0 or value is None:
                continue
            xs.append(value.score / baseline.score)
            ys.append(index)
        if not xs:
            continue
        plotted_any = True
        max_ratio = max(max_ratio, max(xs))
        axis.scatter(
            xs,
            ys,
            marker=MARKER,
            s=MARKER_SIZE,
            color=FORMAT_COLORS.get(format_id),
            label=format_label(format_id),
        )

    if not plotted_any:
        plt.close(figure)
        return False

    axis.vlines(x=1.0, ymin=0, ymax=1, transform=axis.get_xaxis_transform(), colors="black", linewidth=1.2)
    baseline_handle = mlines.Line2D([], [], color="black", linewidth=1.2, label="serde_json::Value baseline")
    handles, labels = axis.get_legend_handles_labels()
    axis.legend(handles=[baseline_handle] + handles, labels=["serde_json::Value baseline"] + labels)
    axis.set_yticks(list(range(len(case_ids))), [case_label(case_id) for case_id in case_ids])
    axis.set_xlim(left=0.0, right=max(1.2, max_ratio * 1.15))
    axis.grid(visible=True, which="major", axis="x")
    axis.set_title(f"{suite_name.capitalize()} speed relative to serde_json::Value")
    axis.set_xlabel(axis_label(metric_source))
    plt.tight_layout()
    figure.savefig(output_path, dpi=160)
    plt.close(figure)
    return True


def markdown_summary(
    description: str,
    timestamp: str,
    mode: str,
    throughput: dict[str, dict[str, dict[str, PerfPoint]]],
    metric_source: str,
) -> str:
    lines = [
        "# Benchmark Plot Summary",
        "",
        f"- Description: `{description}`",
        f"- Timestamp: `{timestamp}`",
        f"- Mode: `{mode}`",
        f"- Metric: {metric_label(metric_source)} relative to `serde_json::Value` (`>1` means faster)",
        "",
    ]

    for suite_name in ("read", "write"):
        suite_data = throughput.get(suite_name, {})
        case_ids = sorted(
            [case_id for case_id, values in suite_data.items() if BASELINE_FORMAT in values],
            key=case_sort_key,
        )
        if not case_ids:
            continue
        lines.extend(
            [
                f"## {suite_name.capitalize()}",
                "",
                "| Case | Baseline | cityjson_lib | cityjson_lib::json | cityarrow | cityparquet |",
                "| --- | --- | --- | --- | --- | --- |",
            ]
        )
        for case_id in case_ids:
            baseline = suite_data[case_id].get(BASELINE_FORMAT)
            row = [
                case_label(case_id),
                format_point(baseline) if baseline is not None else "-",
            ]
            for format_id in FORMAT_ORDER:
                row.append(render_cell(suite_data[case_id].get(format_id), baseline))
            lines.append("| " + " | ".join(row) + " |")
        lines.append("")
    return "\n".join(lines)


def copy_latest(files: list[Path], latest_dir: Path) -> None:
    latest_dir.mkdir(parents=True, exist_ok=True)
    for path in files:
        destination = latest_dir / path.name
        shutil.copy2(path, destination)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--csv", default="bench_results/history.csv")
    parser.add_argument("--description")
    parser.add_argument("--mode", default="all")
    parser.add_argument("--timestamp")
    parser.add_argument("--out-dir", default="bench_results/plots")
    args = parser.parse_args()

    rows = load_rows(Path(args.csv))
    if not rows:
        raise SystemExit("no rows found")

    selected_rows, selected_timestamp, selected_description = select_rows(
        rows,
        args.description,
        args.mode,
        args.timestamp,
    )
    metric_source = select_metric_source(selected_rows)
    throughput = collect_performance(selected_rows, metric_source)
    if not throughput:
        raise SystemExit("no performance rows found for the selected snapshot")

    root_dir = Path(args.out_dir)
    description_dir = root_dir / slugify(selected_description)
    snapshot_dir = description_dir / selected_timestamp
    latest_dir = description_dir / "latest"
    snapshot_dir.mkdir(parents=True, exist_ok=True)

    created_files: list[Path] = []
    for suite_name in ("read", "write"):
        output_path = snapshot_dir / f"throughput_relative_{suite_name}.png"
        if plot_suite(output_path, suite_name, throughput.get(suite_name, {}), metric_source):
            created_files.append(output_path)

    summary_path = snapshot_dir / "benchmark_summary.md"
    summary_path.write_text(
        markdown_summary(
            selected_description,
            selected_timestamp,
            args.mode,
            throughput,
            metric_source,
        ),
        encoding="utf-8",
    )
    created_files.append(summary_path)

    copy_latest(created_files, latest_dir)

    print(f"Snapshot: {selected_description} @ {selected_timestamp}")
    for path in created_files:
        print(f"Saved {path}")
    print(f"Updated {latest_dir}")


if __name__ == "__main__":
    main()
