#!/usr/bin/env python3
"""Convert a retained Heaptrack profiling artifact into stable JSON evidence."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
from pathlib import Path


CATEGORY_PATTERNS: tuple[tuple[str, tuple[str, ...]], ...] = (
    ("vertex_cache", ("parse_vertices_fragment", "load_shared_vertices")),
    (
        "package_reconstruction",
        (
            "read_package_members_from_file",
            "parse_cityobject_entry",
            "build_feature_parts",
            "from_feature_assembly",
        ),
    ),
    ("reference_preload", ("package_ref_page_after_record_id",)),
    ("sqlite", ("sqlite", "rusqlite")),
    ("rayon_runtime", ("rayon", "thread_pool")),
)


def run_checked(args: list[str], *, capture: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, check=True, text=True, capture_output=capture)


def read_json(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"expected a JSON object in {path}")
    return value


def read_json_lines(path: Path) -> list[dict[str, object]]:
    if not path.exists():
        return []
    values: list[dict[str, object]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        value = json.loads(line)
        if isinstance(value, dict):
            values.append(value)
    return values


def heaptrack_trace(artifact: Path) -> Path:
    candidates = sorted(
        path
        for path in artifact.glob("heaptrack*")
        if path.is_file() and path.suffix in {".zst", ".gz"}
    )
    if len(candidates) != 1:
        raise ValueError(f"expected one Heaptrack trace in {artifact}, found {len(candidates)}")
    return candidates[0]


def parse_folded_stacks(path: Path) -> tuple[int, dict[str, int]]:
    categories = {name: 0 for name, _ in CATEGORY_PATTERNS}
    categories["other"] = 0
    total = 0
    for line in path.read_text(encoding="utf-8").splitlines():
        stack, separator, raw_cost = line.rpartition(" ")
        if not separator or not raw_cost.isdigit():
            continue
        cost = int(raw_cost)
        total += cost
        category = "other"
        lowered = stack.lower()
        for name, patterns in CATEGORY_PATTERNS:
            if any(pattern.lower() in lowered for pattern in patterns):
                category = name
                break
        categories[category] += cost
    return total, categories


def cache_checkpoint(
    events: list[dict[str, object]], event_name: str
) -> dict[str, object] | None:
    return next((event for event in events if event.get("event") == event_name), None)


def peak_cgroup_sample(samples: list[dict[str, object]]) -> dict[str, object] | None:
    measured = [
        sample
        for sample in samples
        if isinstance(sample.get("memory_current_bytes"), int)
    ]
    if not measured:
        return None
    return max(measured, key=lambda sample: int(sample["memory_current_bytes"]))


def benchmark_record(stdout_path: Path) -> dict[str, object] | None:
    if not stdout_path.exists():
        return None
    text = stdout_path.read_text(encoding="utf-8")
    marker = '{\n  "schema_version"'
    start = text.find(marker)
    if start < 0:
        return None
    value, _ = json.JSONDecoder().raw_decode(text[start:])
    if not isinstance(value, dict):
        return None
    runs = value.get("runs")
    if not isinstance(runs, list) or not runs or not isinstance(runs[0], dict):
        return None
    return runs[0]


def analyze(artifact: Path) -> dict[str, object]:
    metadata = read_json(artifact / "metadata.json")
    if metadata.get("tool") != "heaptrack":
        raise ValueError(f"{artifact} is not a Heaptrack artifact")
    trace = heaptrack_trace(artifact)
    executable = shutil.which("heaptrack_print")
    if executable is None:
        raise RuntimeError("heaptrack_print is required to analyze Heaptrack traces")

    report_path = artifact / "heaptrack-report.txt"
    report = run_checked(
        [executable, "-f", str(trace), "-p", "-a", "-T", "-n", "40", "-s", "10"],
        capture=True,
    )
    report_path.write_text(report.stdout, encoding="utf-8")

    folded_path = artifact / "heaptrack-peak.stacks"
    run_checked(
        [
            executable,
            "-f",
            str(trace),
            "--print-flamegraph",
            str(folded_path),
            "--flamegraph-cost-type",
            "peak",
        ],
        capture=True,
    )
    massif_path = artifact / "heaptrack.massif"
    run_checked(
        [
            executable,
            "-f",
            str(trace),
            "--print-massif",
            str(massif_path),
            "--massif-threshold",
            "0.1",
        ],
        capture=True,
    )

    peak_heap_bytes, categories = parse_folded_stacks(folded_path)
    events = read_json_lines(artifact / "stage-events.jsonl")
    before_drop = cache_checkpoint(events, "cache_before_drop")
    after_drop = cache_checkpoint(events, "cache_after_drop")
    record = benchmark_record(artifact / "stdout.log")
    process_peak_rss_bytes = (
        record.get("process_peak_rss_bytes") if record is not None else None
    )
    residual_upper_bound_bytes = None
    requested_heap_exceeds_rss_bytes = None
    if isinstance(process_peak_rss_bytes, int):
        if process_peak_rss_bytes >= peak_heap_bytes:
            residual_upper_bound_bytes = process_peak_rss_bytes - peak_heap_bytes
        else:
            requested_heap_exceeds_rss_bytes = peak_heap_bytes - process_peak_rss_bytes

    rss_drop_bytes = None
    if before_drop is not None and after_drop is not None:
        before_rss = before_drop.get("current_rss_bytes")
        after_rss = after_drop.get("current_rss_bytes")
        if isinstance(before_rss, int) and isinstance(after_rss, int):
            rss_drop_bytes = max(0, before_rss - after_rss)

    peak_sample = peak_cgroup_sample(read_json_lines(artifact / "cgroup-memory.jsonl"))
    result: dict[str, object] = {
        "schema_version": 1,
        "run_id": metadata.get("run_id"),
        "workers": metadata.get("workers"),
        "tiles": metadata.get("tiles"),
        "trace": trace.name,
        "peak_requested_heap_bytes": peak_heap_bytes,
        "peak_allocation_categories_bytes": categories,
        "process_peak_rss_bytes": process_peak_rss_bytes,
        "rss_minus_requested_heap_upper_bound_bytes": residual_upper_bound_bytes,
        "requested_heap_exceeds_rss_bytes": requested_heap_exceeds_rss_bytes,
        "cache_before_drop": before_drop,
        "cache_after_drop": after_drop,
        "rss_drop_after_cache_clear_bytes": rss_drop_bytes,
        "peak_cgroup_sample": peak_sample,
        "interpretation": {
            "vertex_capacity_bytes": "Exact allocated capacity of retained [i64; 3] vertex buffers reported by the benchmark.",
            "peak_requested_heap_bytes": "Heaptrack live requested allocation bytes at its global heap peak.",
            "rss_minus_requested_heap_upper_bound_bytes": "Upper bound containing allocator slack and metadata, stacks, anonymous mappings, and profiler overhead; not pure fragmentation.",
            "requested_heap_exceeds_rss_bytes": "Requested live allocation capacity that was not resident at the process RSS peak; no fragmentation bound is derived in this case.",
        },
    }
    output = artifact / "heaptrack-analysis.json"
    output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return result


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("artifact", type=Path)
    return result


def main() -> int:
    args = parser().parse_args()
    artifact = args.artifact
    if not artifact.exists() or not artifact.is_dir():
        raise SystemExit(f"profiling artifact does not exist: {artifact}")
    result = analyze(artifact.resolve())
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
