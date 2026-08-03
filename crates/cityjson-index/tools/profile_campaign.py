#!/usr/bin/env python3
"""Run the reproducible Tyler worker and allocation profiling campaign."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import uuid
from datetime import UTC, datetime
from pathlib import Path


WORKERS = (1, 4, 24)
FULL_TILE_COUNT = 182
FALLBACK_TILE_COUNT = 24
TIMING_REPETITIONS = 3
GATE_RATIO = 0.9


def parse_memory_size(value: str) -> int:
    normalized = value.strip().upper()
    suffixes = {"K": 1, "M": 2, "G": 3, "T": 4}
    if normalized and normalized[-1] in suffixes:
        power = suffixes[normalized[-1]]
        number = normalized[:-1]
    else:
        power = 0
        number = normalized
    if not number.isdigit() or int(number) < 1:
        raise ValueError(f"invalid memory size: {value}")
    return int(number) * (1024**power)


def projected_four_worker_peak(one_worker_peak: int) -> int:
    return one_worker_peak * 4


def projected_twenty_four_worker_peak(one_worker_peak: int, four_worker_peak: int) -> int:
    duplicated_slope = max(0, four_worker_peak - one_worker_peak)
    return one_worker_peak + (duplicated_slope * 23) // 3


def gate_allows(projected_peak: int, memory_max_bytes: int) -> bool:
    return projected_peak <= int(memory_max_bytes * GATE_RATIO)


def write_manifest(path: Path, manifest: dict[str, object]) -> None:
    path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run_profile(
    args: argparse.Namespace,
    *,
    tool: str,
    workers: int,
    tiles: int,
    description: str,
    skip_prepare: bool = True,
) -> dict[str, object]:
    command = [
        sys.executable,
        str(Path(__file__).with_name("profile_index.py")),
        "--tool",
        tool,
        "--workers",
        str(workers),
        "--tiles",
        str(tiles),
        "--description",
        description,
        "--memory-max",
        args.memory_max,
        "--corpus",
        str(args.corpus),
        "--work-root",
        str(args.work_root),
        "--output-root",
        str(args.output_root),
    ]
    if skip_prepare:
        command.append("--skip-prepare")
    result = subprocess.run(command, check=True, text=True, capture_output=True)
    if result.stderr:
        print(result.stderr, file=sys.stderr, end="")
    lines = [line for line in result.stdout.splitlines() if line.strip()]
    if not lines:
        raise RuntimeError(f"profile command produced no summary: {' '.join(command)}")
    summary = json.loads(lines[-1])
    if not isinstance(summary, dict):
        raise RuntimeError("profile command summary was not a JSON object")
    print(json.dumps(summary, sort_keys=True))
    return summary


def prepare_worker_sidecar(args: argparse.Namespace, workers: int) -> None:
    repo = Path(__file__).resolve().parents[3]
    binary = repo / "target" / "profiling" / "bench-index"
    command = [
        str(binary),
        "--prepare-only",
        "--case",
        "tyler-pipeline",
        "--layout",
        "city-json",
        "--workers",
        str(workers),
        "--tyler-tile-count",
        str(FULL_TILE_COUNT),
        "--work-root",
        str(args.work_root),
        "--groningen-corpus",
        str(args.corpus),
    ]
    subprocess.run(command, check=True, text=True)


def record_run(
    manifest_path: Path,
    manifest: dict[str, object],
    summary: dict[str, object],
) -> None:
    runs = manifest["runs"]
    if not isinstance(runs, list):
        raise RuntimeError("campaign manifest runs field is not a list")
    runs.append(summary)
    write_manifest(manifest_path, manifest)


def record_gate(
    manifest_path: Path,
    manifest: dict[str, object],
    *,
    workers: int,
    projected_peak: int | None,
    allowed: bool,
    reason: str,
) -> None:
    gates = manifest["heaptrack_gates"]
    if not isinstance(gates, list):
        raise RuntimeError("campaign manifest heaptrack_gates field is not a list")
    gates.append(
        {
            "workers": workers,
            "projected_peak_bytes": projected_peak,
            "allowed": allowed,
            "reason": reason,
        }
    )
    write_manifest(manifest_path, manifest)


def completed_peak(summary: dict[str, object]) -> int | None:
    peak = summary.get("memory_peak_bytes")
    if summary.get("outcome") != "completed" or not isinstance(peak, int):
        return None
    return peak


def run_heaptrack_matrix(
    args: argparse.Namespace,
    manifest_path: Path,
    manifest: dict[str, object],
) -> list[Path]:
    artifacts: list[Path] = []
    peaks: dict[int, int] = {}
    full_matrix_complete = True
    for workers in WORKERS:
        projected_peak = None
        if workers == 4 and 1 in peaks:
            projected_peak = projected_four_worker_peak(peaks[1])
        elif workers == 24 and 1 in peaks and 4 in peaks:
            projected_peak = projected_twenty_four_worker_peak(peaks[1], peaks[4])
        allowed = projected_peak is None or gate_allows(
            projected_peak, parse_memory_size(args.memory_max)
        )
        reason = "initial measurement" if projected_peak is None else "below 90% safety threshold"
        if not allowed:
            reason = "projected cgroup peak exceeds 90% safety threshold"
        record_gate(
            manifest_path,
            manifest,
            workers=workers,
            projected_peak=projected_peak,
            allowed=allowed,
            reason=reason,
        )
        if not allowed:
            full_matrix_complete = False
            break
        summary = run_profile(
            args,
            tool="heaptrack",
            workers=workers,
            tiles=FULL_TILE_COUNT,
            description=f"{args.description} heaptrack full corpus",
        )
        record_run(manifest_path, manifest, summary)
        output = summary.get("output")
        if isinstance(output, str):
            artifacts.append(Path(output))
        peak = completed_peak(summary)
        if peak is None:
            full_matrix_complete = False
            break
        peaks[workers] = peak

    if not full_matrix_complete:
        manifest["heaptrack_fallback"] = True
        write_manifest(manifest_path, manifest)
        for workers in WORKERS:
            summary = run_profile(
                args,
                tool="heaptrack",
                workers=workers,
                tiles=FALLBACK_TILE_COUNT,
                description=f"{args.description} heaptrack reduced fallback",
                skip_prepare=False,
            )
            record_run(manifest_path, manifest, summary)
            output = summary.get("output")
            if isinstance(output, str):
                artifacts.append(Path(output))
    return artifacts


def analyze_heaptrack_artifacts(artifacts: list[Path], manifest: dict[str, object]) -> None:
    analyses: list[dict[str, object]] = []
    analyzer = Path(__file__).with_name("analyze_profile.py")
    for artifact in artifacts:
        traces = list(artifact.glob("heaptrack*.zst")) + list(artifact.glob("heaptrack*.gz"))
        if not traces:
            analyses.append({"artifact": str(artifact), "outcome": "trace_missing"})
            continue
        result = subprocess.run(
            [sys.executable, str(analyzer), str(artifact)],
            check=False,
            text=True,
            capture_output=True,
        )
        analyses.append(
            {
                "artifact": str(artifact),
                "outcome": "completed" if result.returncode == 0 else "failed",
                "error": result.stderr.strip() if result.returncode != 0 else None,
            }
        )
    manifest["heaptrack_analyses"] = analyses


def run_campaign(args: argparse.Namespace) -> Path:
    args.corpus = args.corpus.resolve()
    args.work_root = args.work_root.resolve()
    args.output_root = args.output_root.resolve()
    timestamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
    campaign_id = f"{timestamp}-tyler-matrix-{uuid.uuid4().hex[:8]}"
    campaign_dir = args.output_root / "campaigns" / campaign_id
    campaign_dir.mkdir(parents=True)
    manifest_path = campaign_dir / "campaign.json"
    manifest: dict[str, object] = {
        "schema_version": 1,
        "campaign_id": campaign_id,
        "description": args.description,
        "memory_max": args.memory_max,
        "workers": list(WORKERS),
        "full_tile_count": FULL_TILE_COUNT,
        "runs": [],
        "heaptrack_gates": [],
        "heaptrack_fallback": False,
    }
    write_manifest(manifest_path, manifest)

    for workers in WORKERS:
        prepare_worker_sidecar(args, workers)

    for tool in ("native", "perf-stat"):
        for repetition in range(1, TIMING_REPETITIONS + 1):
            for workers in WORKERS:
                summary = run_profile(
                    args,
                    tool=tool,
                    workers=workers,
                    tiles=FULL_TILE_COUNT,
                    description=f"{args.description} {tool} repetition {repetition}",
                )
                summary["repetition"] = repetition
                record_run(manifest_path, manifest, summary)

    heaptrack_artifacts = run_heaptrack_matrix(args, manifest_path, manifest)
    for workers in WORKERS:
        summary = run_profile(
            args,
            tool="perf-record",
            workers=workers,
            tiles=FALLBACK_TILE_COUNT,
            description=f"{args.description} perf-record",
            skip_prepare=False,
        )
        record_run(manifest_path, manifest, summary)
    summary = run_profile(
        args,
        tool="cachegrind",
        workers=1,
        tiles=1,
        description=f"{args.description} cachegrind",
        skip_prepare=False,
    )
    record_run(manifest_path, manifest, summary)

    analyze_heaptrack_artifacts(heaptrack_artifacts, manifest)
    write_manifest(manifest_path, manifest)
    print(json.dumps({"campaign": str(campaign_dir)}, sort_keys=True))
    return campaign_dir


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("--description", required=True)
    result.add_argument("--memory-max", default="28G")
    result.add_argument("--corpus", type=Path, required=True)
    result.add_argument("--work-root", type=Path, required=True)
    result.add_argument("--output-root", type=Path, required=True)
    return result


if __name__ == "__main__":
    raise SystemExit(0 if run_campaign(parser().parse_args()) else 1)
