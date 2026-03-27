"""Generate benchmark plots and README-ready tables for serde_cityjson."""
# /// script
# requires-python = ">=3.12"
# dependencies = ["matplotlib"]
# ///

from __future__ import annotations

import argparse
import json
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

import matplotlib.lines as mlines
import matplotlib.pyplot as plt

BENCHMARK_IDS = {
    "read": (
        "serde_cityjson/owned",
        "serde_cityjson/borrowed",
        "serde_json::Value",
    ),
    "write": (
        "serde_cityjson/to_string",
        "serde_cityjson/to_string_validated",
        "serde_json::to_string",
    ),
}

BASELINE_IDS = {
    "read": "serde_json::Value",
    "write": "serde_json::to_string",
}

OUTPUT_DIR = Path("benches") / "results"
OUTPUT_DIR.mkdir(exist_ok=True, parents=True)
MARKER = "d"
MARKERSIZE = 60


@dataclass(frozen=True)
class CaseMeta:
    case_id: str
    description: str
    borrowed: bool
    input_bytes: int
    output_bytes: int


def cargo_target_directory() -> Path:
    res = subprocess.run(
        ["cargo", "metadata", "--format-version", "1"],
        capture_output=True,
        check=True,
    )
    metadata = json.loads(res.stdout.decode("utf-8"))
    return Path(metadata["target_directory"])


def sanitize_for_path(name: str) -> str:
    return re.sub(r"[^a-zA-Z0-9 \n.]", "_", name)


def load_suite_metadata(suite: str) -> dict[str, CaseMeta]:
    path = OUTPUT_DIR / f"suite_metadata_{suite}.json"
    if not path.exists():
        return {}
    with path.open("r", encoding="utf-8") as handle:
        payload = json.load(handle)
    result: dict[str, CaseMeta] = {}
    for case in payload.get("cases", []):
        result[case["id"]] = CaseMeta(
            case_id=case["id"],
            description=case.get("description", ""),
            borrowed=case.get("borrowed", False),
            input_bytes=int(case.get("input_bytes", 0)),
            output_bytes=int(case.get("output_bytes", 0)),
        )
    return result


def load_estimate(path: Path) -> float:
    with path.open("r", encoding="utf-8") as handle:
        estimates = json.load(handle)
    return float(estimates["mean"]["point_estimate"])


def format_duration(ns: float) -> str:
    if ns >= 1_000_000_000:
        return f"{ns / 1_000_000_000:.3f} s"
    if ns >= 1_000_000:
        return f"{ns / 1_000_000:.3f} ms"
    if ns >= 1_000:
        return f"{ns / 1_000:.3f} us"
    return f"{ns:.0f} ns"


def format_throughput(bytes_count: int, ns: float) -> str:
    if ns <= 0:
        return "-"
    mib_per_s = (bytes_count / (ns / 1_000_000_000.0)) / (1024.0 * 1024.0)
    return f"{mib_per_s:.1f} MiB/s"


def benchmark_file(case_id: str, bench_id: str) -> Path:
    criterion_dir = cargo_target_directory().joinpath("criterion")
    return (
        criterion_dir
        .joinpath(case_id)
        .joinpath(sanitize_for_path(bench_id))
        .joinpath("new")
        .joinpath("estimates.json")
    )


def collect_suite_results(suite: str, case_meta: dict[str, CaseMeta]) -> dict[str, dict[str, float]]:
    results: dict[str, dict[str, float]] = {}
    for case_id, _meta in case_meta.items():
        for bench_id in BENCHMARK_IDS[suite]:
            estimate_path = benchmark_file(case_id, bench_id)
            if estimate_path.exists():
                results.setdefault(case_id, {})[bench_id] = load_estimate(estimate_path)
    return results


def case_order(results: dict[str, dict[str, float]]) -> list[str]:
    return sorted(results.keys())


def plot_suite(suite: str, results: dict[str, dict[str, float]]) -> None:
    if not results:
        return
    plt.style.use("seaborn-v0_8-muted")
    fig, ax = plt.subplots(figsize=(10, max(3, 0.35 * len(results))))
    order = case_order(results)
    y_positions = list(range(len(order)))

    for bench_id in BENCHMARK_IDS[suite]:
        points = []
        for idx, case_id in enumerate(order):
            suite_case = results[case_id]
            baseline = suite_case.get(BASELINE_IDS[suite])
            estimate = suite_case.get(bench_id)
            if baseline is None or estimate is None:
                continue
            points.append((estimate / baseline, idx))
        if points:
            xs, ys = zip(*points)
            ax.scatter(xs, ys, marker=MARKER, s=MARKERSIZE, label=bench_id)

    ax.vlines(x=1, ymin=0, ymax=1, transform=ax.get_xaxis_transform(), colors="red")
    red_line = mlines.Line2D([], [], color="red", label="serde_json::Value")
    series_handles, _ = ax.get_legend_handles_labels()
    ax.legend(handles=[red_line] + series_handles)
    ax.set_yticks(y_positions, order)
    ax.set_xlim(left=0.0)
    ax.grid(visible=True, which="major", axis="x")
    ax.set_title(f"Relative execution time of serde_cityjson compared to serde_json::Value ({suite})")
    ax.set_xlabel("Factor of execution time relative to serde_json::Value (>1 = slower)")
    plt.tight_layout()
    filepath = OUTPUT_DIR / f"speed_relative_{suite}.png"
    plt.savefig(filepath)
    plt.close(fig)
    print(f"Saved {filepath}")


def render_suite_table(suite: str, results: dict[str, dict[str, float]], case_meta: dict[str, CaseMeta]) -> str:
    if not results:
        return ""
    rows = [
        f"### {suite.capitalize()} Benchmarks",
        "",
        "| Case | Description | serde_cityjson | serde_json::Value | Factor |",
        "| --- | --- | --- | --- | --- |",
    ]
    main_bench = "serde_cityjson/owned" if suite == "read" else "serde_cityjson/to_string"
    extra_bench = "serde_cityjson/borrowed" if suite == "read" else "serde_cityjson/to_string_validated"
    baseline_bench = BASELINE_IDS[suite]

    for case_id in case_order(results):
        suite_case = results[case_id]
        meta = case_meta.get(case_id, CaseMeta(case_id, "", False, 0, 0))
        baseline = suite_case.get(baseline_bench)
        main = suite_case.get(main_bench)
        if baseline is None or main is None:
            continue
        factor = main / baseline
        size_bytes = meta.input_bytes if suite == "read" else meta.output_bytes
        summary = (
            f"{main_bench.split('/')[-1]} {format_duration(main)} "
            f"({format_throughput(size_bytes, main)})"
        )
        if extra_bench in suite_case:
            extra = suite_case[extra_bench]
            summary += f"; {extra_bench.split('/')[-1]} {format_duration(extra)} ({format_throughput(size_bytes, extra)})"
        rows.append(
            "| {case} | {desc} | {main} | {base} | {factor:.2f}x |".format(
                case=meta.case_id,
                desc=meta.description or "",
                main=summary,
                base=f"{format_duration(baseline)} ({format_throughput(size_bytes, baseline)})",
                factor=factor,
            )
        )
    return "\n".join(rows) + "\n"


def markdown_fragment() -> str:
    fragments = [
        "# Benchmark Summary",
        "",
        "Generated from Criterion results.",
        "",
    ]
    for suite in ("read", "write"):
        metadata = load_suite_metadata(suite)
        results = collect_suite_results(suite, metadata)
        if not results:
            continue
        fragments.append(render_suite_table(suite, results, metadata))
    return "\n".join(fragments).strip() + "\n"


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--markdown", action="store_true", help="Write a Markdown summary file")
    args = parser.parse_args()

    for suite in ("read", "write"):
        metadata = load_suite_metadata(suite)
        plot_suite(suite, collect_suite_results(suite, metadata))

    if args.markdown:
        output = OUTPUT_DIR / "benchmark_summary.md"
        output.write_text(markdown_fragment(), encoding="utf-8")
        print(f"Saved {output}")


if __name__ == "__main__":
    main()
