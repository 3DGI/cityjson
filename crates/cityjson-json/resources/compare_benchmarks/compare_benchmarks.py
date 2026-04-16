"""Generate benchmark plots and README-ready tables for cityjson-json."""
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
        "cityjson-json/owned",
        "cityjson-json/borrowed",
        "serde_json::Value",
    ),
    "write": (
        "cityjson-json/as_json_to_value",
        "cityjson-json/to_string",
        "cityjson-json/to_string_validated",
        "serde_json::to_string",
    ),
}

BASELINE_IDS = {
    "read": "serde_json::Value",
    "write": "serde_json::to_string",
}

MAIN_BENCH_IDS = {
    "read": "cityjson-json/owned",
    "write": "cityjson-json/to_string",
}

REPO_ROOT = Path(__file__).resolve().parents[2]
OUTPUT_DIR = REPO_ROOT / "benches" / "results"
OUTPUT_DIR.mkdir(exist_ok=True, parents=True)
README_PATH = REPO_ROOT / "README.md"
README_MARKER_START = "<!-- benchmark-summary:start -->"
README_MARKER_END = "<!-- benchmark-summary:end -->"
README_CASE_LIMIT = 3
MARKER = "d"
MARKERSIZE = 60


@dataclass(frozen=True)
class CaseMeta:
    case_id: str
    description: str
    borrowed: bool
    input_bytes: int
    benchmark_bytes: dict[str, int]


def cargo_target_directory() -> Path:
    res = subprocess.run(
        ["cargo", "metadata", "--format-version", "1"],
        cwd=REPO_ROOT,
        capture_output=True,
        check=True,
    )
    metadata = json.loads(res.stdout.decode("utf-8"))
    return Path(metadata["target_directory"])


def sanitize_for_path(name: str) -> str:
    return re.sub(r"[^a-zA-Z0-9 \n.\-]", "_", name)


def load_suite_metadata(suite: str) -> dict[str, CaseMeta]:
    path = OUTPUT_DIR / f"suite_metadata_{suite}.json"
    if not path.exists():
        return {}
    with path.open("r", encoding="utf-8") as handle:
        payload = json.load(handle)
    result: dict[str, CaseMeta] = {}
    for case in payload.get("cases", []):
        benchmark_bytes = {
            bench_id: int(bytes_count)
            for bench_id, bytes_count in case.get("benchmark_bytes", {}).items()
        }
        legacy_output_bytes = case.get("output_bytes")
        if suite == "write" and not benchmark_bytes and legacy_output_bytes is not None:
            output_bytes = int(legacy_output_bytes)
            benchmark_bytes = {
                "cityjson-json/to_string": output_bytes,
                "cityjson-json/to_string_validated": output_bytes,
                "serde_json::to_string": output_bytes,
            }
        result[case["id"]] = CaseMeta(
            case_id=case["id"],
            description=case.get("description", ""),
            borrowed=case.get("borrowed", False),
            input_bytes=int(case.get("input_bytes", 0)),
            benchmark_bytes=benchmark_bytes,
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


def format_speedup(estimate_ns: float, baseline_ns: float) -> str:
    if estimate_ns <= 0:
        return "-"
    return f"{baseline_ns / estimate_ns:.2f}x"


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


def benchmark_bytes_for(suite: str, meta: CaseMeta, bench_id: str) -> int:
    if suite == "read":
        return meta.input_bytes
    return meta.benchmark_bytes.get(bench_id, 0)


def bench_display_name(bench_id: str) -> str:
    return bench_id.split("/")[-1]


def plot_suite(suite: str, results: dict[str, dict[str, float]]) -> None:
    if not results:
        return
    plt.style.use("seaborn-v0_8-muted")
    fig, ax = plt.subplots(figsize=(10, max(3, 0.35 * len(results))))
    order = case_order(results)
    y_positions = list(range(len(order)))

    baseline_label = BASELINE_IDS[suite]
    for bench_id in BENCHMARK_IDS[suite]:
        if bench_id == baseline_label:
            continue
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
    red_line = mlines.Line2D([], [], color="red", label=baseline_label)
    series_handles, _ = ax.get_legend_handles_labels()
    ax.legend(handles=[red_line] + series_handles)
    ax.set_yticks(y_positions, order)
    ax.set_xlim(left=0.0)
    ax.grid(visible=True, which="major", axis="x")
    ax.set_title(f"Relative execution time compared to {baseline_label} ({suite})")
    ax.set_xlabel(f"Factor of execution time relative to {baseline_label} (>1 = slower)")
    plt.tight_layout()
    filepath = OUTPUT_DIR / f"speed_relative_{suite}.png"
    plt.savefig(filepath)
    plt.close(fig)
    print(f"Saved {filepath}")


def render_suite_table(suite: str, results: dict[str, dict[str, float]], case_meta: dict[str, CaseMeta]) -> str:
    if not results:
        return ""
    baseline_bench = BASELINE_IDS[suite]
    rows = [
        f"### {suite.capitalize()} Benchmarks",
        "",
        f"| Case | Description | cityjson-json | {baseline_bench} | Factor |",
        "| --- | --- | --- | --- | --- |",
    ]
    main_bench = MAIN_BENCH_IDS[suite]

    for case_id in case_order(results):
        suite_case = results[case_id]
        meta = case_meta.get(case_id, CaseMeta(case_id, "", False, 0, {}))
        baseline = suite_case.get(baseline_bench)
        main = suite_case.get(main_bench)
        if baseline is None or main is None:
            continue
        factor = main / baseline
        summaries: list[str] = []
        for bench_id in BENCHMARK_IDS[suite]:
            if bench_id == baseline_bench:
                continue
            estimate = suite_case.get(bench_id)
            if estimate is None:
                continue
            summaries.append(
                f"{bench_display_name(bench_id)} {format_duration(estimate)} "
                f"({format_throughput(benchmark_bytes_for(suite, meta, bench_id), estimate)})"
            )
        rows.append(
            "| {case} | {desc} | {main} | {base} | {factor:.2f}x |".format(
                case=meta.case_id,
                desc=meta.description or "",
                main="; ".join(summaries),
                base=(
                    f"{format_duration(baseline)} "
                    f"({format_throughput(benchmark_bytes_for(suite, meta, baseline_bench), baseline)})"
                ),
                factor=factor,
            )
        )
    return "\n".join(rows) + "\n"


def readme_acquired_case_order(
    results: dict[str, dict[str, float]], case_meta: dict[str, CaseMeta]
) -> list[str]:
    required_benchmarks = (
        "cityjson-json/owned",
        "cityjson-json/borrowed",
        "serde_json::Value",
    )
    eligible_cases: list[tuple[int, str]] = []
    for case_id, suite_case in results.items():
        if not case_id.startswith("io_"):
            continue
        if not all(bench_id in suite_case for bench_id in required_benchmarks):
            continue
        meta = case_meta.get(case_id, CaseMeta(case_id, "", False, 0, {}))
        eligible_cases.append((meta.input_bytes, case_id))
    eligible_cases.sort(key=lambda item: (-item[0], item[1]))
    return [case_id for _, case_id in eligible_cases[:README_CASE_LIMIT]]


def readme_stress_case_order(
    results: dict[str, dict[str, float]], case_meta: dict[str, CaseMeta]
) -> list[str]:
    required_benchmarks = (
        "cityjson-json/owned",
        "cityjson-json/borrowed",
        "serde_json::Value",
    )
    return sorted(
        case_id for case_id, suite_case in results.items()
        if case_id.startswith("stress_")
        and all(bench_id in suite_case for bench_id in required_benchmarks)
    )


def readme_fragment(results: dict[str, dict[str, float]], case_meta: dict[str, CaseMeta]) -> str:
    header = "| Case | Owned | Borrowed | `serde_json::Value` | Owned vs Value | Borrowed vs Value |"
    sep = "| --- | --- | --- | --- | --- | --- |"

    def render_rows(case_ids: list[str]) -> list[str]:
        rows = []
        for case_id in case_ids:
            suite_case = results[case_id]
            meta = case_meta.get(case_id, CaseMeta(case_id, "", False, 0, {}))
            owned = suite_case["cityjson-json/owned"]
            borrowed = suite_case["cityjson-json/borrowed"]
            baseline = suite_case["serde_json::Value"]
            rows.append(
                "| {case} | {owned_tp} | {borrowed_tp} | {baseline_tp} | {owned_speed} | {borrowed_speed} |".format(
                    case=f"`{meta.case_id}`",
                    owned_tp=format_throughput(meta.input_bytes, owned),
                    borrowed_tp=format_throughput(meta.input_bytes, borrowed),
                    baseline_tp=format_throughput(meta.input_bytes, baseline),
                    owned_speed=format_speedup(owned, baseline),
                    borrowed_speed=format_speedup(borrowed, baseline),
                )
            )
        return rows

    acquired_ids = readme_acquired_case_order(results, case_meta)
    stress_ids = readme_stress_case_order(results, case_meta)

    if not acquired_ids and not stress_ids:
        return ""

    lines: list[str] = []
    if acquired_ids:
        lines += ["**Acquired data**", "", header, sep] + render_rows(acquired_ids) + [""]
    if stress_ids:
        lines += ["**Stress cases**", "", header, sep] + render_rows(stress_ids) + [""]

    return "\n".join(lines)


def update_readme(fragment: str) -> None:
    if not fragment:
        raise SystemExit("no read benchmark results available to update README")

    readme = README_PATH.read_text(encoding="utf-8")
    start = readme.find(README_MARKER_START)
    end = readme.find(README_MARKER_END)
    if start < 0 or end < 0 or end < start:
        raise SystemExit(
            f"README markers not found in {README_PATH}. Expected {README_MARKER_START} / {README_MARKER_END}."
        )

    start += len(README_MARKER_START)
    replacement = f"\n{fragment}"
    updated = readme[:start] + replacement + readme[end:]
    README_PATH.write_text(updated, encoding="utf-8")
    print(f"Updated {README_PATH}")


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
    parser.add_argument(
        "--readme",
        action="store_true",
        help="Update the README benchmark snippet from the current read benchmark results",
    )
    args = parser.parse_args()

    suite_results: dict[str, dict[str, dict[str, float]]] = {}
    suite_metadata: dict[str, dict[str, CaseMeta]] = {}
    for suite in ("read", "write"):
        metadata = load_suite_metadata(suite)
        results = collect_suite_results(suite, metadata)
        suite_metadata[suite] = metadata
        suite_results[suite] = results
        plot_suite(suite, results)

    if args.markdown:
        output = OUTPUT_DIR / "benchmark_summary.md"
        output.write_text(markdown_fragment(), encoding="utf-8")
        print(f"Saved {output}")

    if args.readme:
        update_readme(readme_fragment(suite_results["read"], suite_metadata["read"]))


if __name__ == "__main__":
    main()
