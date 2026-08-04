#!/usr/bin/env python3
"""Run one contained cityjson-index profiling target and retain OOM evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import shutil
import sys
import uuid
from datetime import UTC, datetime
from pathlib import Path

from profile_runtime import (
    SAMPLE_INTERVAL_SECONDS,
    append_json_line,
    classify_outcome,
    read_integer,
    read_key_values,
    run_checked,
    run_contained,
    sample_cgroup,
    sample_has_measurements,
    systemctl_properties,
)


TOOLS = {"native", "perf-stat", "perf-record", "heaptrack", "cachegrind", "massif"}


def executable_for(tool: str) -> str | None:
    return {
        "native": None,
        "perf-stat": "perf",
        "perf-record": "perf",
        "heaptrack": "heaptrack",
        "cachegrind": "valgrind",
        "massif": "valgrind",
    }[tool]


def validate(args: argparse.Namespace) -> None:
    if args.tool not in TOOLS:
        raise SystemExit(f"unknown tool {args.tool!r}; expected one of {', '.join(sorted(TOOLS))}")
    if args.workers < 1 or args.tiles < 1:
        raise SystemExit("workers and tiles must both be positive")
    if args.tiles == 182 and not args.memory_max:
        raise SystemExit("the 182-tile profile requires an explicit --memory-max")
    if args.tool in {"cachegrind", "massif"} and (args.workers != 1 or args.tiles != 1):
        raise SystemExit(f"{args.tool} is restricted to --workers 1 --tiles 1")
    if args.tool == "perf-record" and args.tiles > 24:
        raise SystemExit("perf-record is restricted to at most 24 tiles")
    required = ["systemd-run", "systemctl"]
    tool_executable = executable_for(args.tool)
    if tool_executable:
        required.append(tool_executable)
    missing = [name for name in required if shutil.which(name) is None]
    if missing:
        raise SystemExit(f"missing required profiling tools: {', '.join(missing)}")


def profiler_command(
    args: argparse.Namespace,
    binary: Path,
    events: Path,
    output_dir: Path,
) -> list[str]:
    target = "tyler-pipeline" if args.tool in {"native", "perf-stat"} else "tyler-feature-materialization"
    benchmark = [
        str(binary),
        "--profile-target",
        target,
        "--reuse-prepared",
        "--case",
        "tyler-pipeline",
        "--layout",
        "city-json",
        "--workers",
        str(args.workers),
        "--tyler-tile-count",
        str(args.tiles),
        "--work-root",
        str(args.work_root),
        "--groningen-corpus",
        str(args.corpus),
        "--profile-events",
        str(events),
        "--json",
    ]
    if args.tool == "native":
        return benchmark
    if args.tool == "perf-stat":
        return [
            "perf",
            "stat",
            "-x",
            ";",
            "-o",
            str(output_dir / "perf-stat.csv"),
            "-e",
            "task-clock,cycles,instructions,branches,branch-misses,cache-references,cache-misses,page-faults,major-faults,context-switches",
            "--",
            *benchmark,
        ]
    if args.tool == "perf-record":
        return [
            "perf",
            "record",
            "-F",
            "99",
            "--call-graph",
            "dwarf",
            "-o",
            str(output_dir / "perf.data"),
            "--",
            *benchmark,
        ]
    if args.tool == "heaptrack":
        return [
            "heaptrack",
            "--record-only",
            "-o",
            str(output_dir / "heaptrack"),
            *benchmark,
        ]
    if args.tool == "cachegrind":
        return [
            "valgrind",
            "--tool=cachegrind",
            "--cache-sim=yes",
            "--branch-sim=yes",
            f"--cachegrind-out-file={output_dir / 'cachegrind.out'}",
            *benchmark,
        ]
    return [
        "valgrind",
        "--tool=massif",
        "--time-unit=B",
        "--stacks=no",
        f"--massif-out-file={output_dir / 'massif.out'}",
        *benchmark,
    ]


def prepare(args: argparse.Namespace, binary: Path) -> None:
    command = [
        str(binary),
        "--prepare-only",
        "--case",
        "tyler-pipeline",
        "--layout",
        "city-json",
        "--workers",
        str(args.workers),
        "--tyler-tile-count",
        str(args.tiles),
        "--work-root",
        str(args.work_root),
        "--groningen-corpus",
        str(args.corpus),
    ]
    run_checked(command)


def command_version(executable: str) -> str:
    result = run_checked([executable, "--version"], capture=True)
    return (result.stdout or result.stderr).strip().splitlines()[0]


def working_tree_provenance(repo: Path) -> dict[str, object]:
    status = run_checked(
        ["git", "status", "--porcelain=v1", "--untracked-files=all"], capture=True
    ).stdout
    diff = run_checked(["git", "diff", "--binary", "HEAD"], capture=True).stdout
    digest = hashlib.sha256()
    digest.update(status.encode("utf-8"))
    digest.update(b"\0")
    digest.update(diff.encode("utf-8"))
    return {
        "dirty": bool(status.strip()),
        "status": status.splitlines(),
        "diff_sha256": digest.hexdigest(),
    }


def metadata(
    args: argparse.Namespace, run_id: str, command: list[str], repo: Path
) -> dict[str, object]:
    commit = run_checked(["git", "rev-parse", "HEAD"], capture=True).stdout.strip()
    tool_executable = executable_for(args.tool)
    return {
        "schema_version": 1,
        "run_id": run_id,
        "description": args.description,
        "tool": args.tool,
        "workers": args.workers,
        "tiles": args.tiles,
        "memory_max": args.memory_max,
        "memory_swap_max": "0",
        "commit": commit,
        "working_tree": working_tree_provenance(repo),
        "tool_version": command_version(tool_executable) if tool_executable else None,
        "platform": platform.platform(),
        "python": sys.version,
        "command": command,
    }


def run_profile(args: argparse.Namespace) -> int:
    validate(args)
    repo = Path(__file__).resolve().parents[3]
    binary = repo / "target" / "profiling" / "bench-index"
    if not binary.exists():
        raise SystemExit(f"profiling binary not found: {binary}; run the just recipe instead")
    args.work_root = args.work_root.resolve()
    args.corpus = args.corpus.resolve()
    if not args.skip_prepare:
        prepare(args, binary)

    timestamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
    run_id = f"{timestamp}-{args.tool}-w{args.workers}-t{args.tiles}-{uuid.uuid4().hex[:8]}"
    output_dir = args.output_root.resolve() / run_id
    output_dir.mkdir(parents=True)
    events = output_dir / "stage-events.jsonl"
    samples = output_dir / "cgroup-memory.jsonl"
    command = profiler_command(args, binary, events, output_dir)
    (output_dir / "metadata.json").write_text(
        json.dumps(metadata(args, run_id, command, repo), indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    summary = run_contained(
        command,
        output_dir=output_dir,
        run_id=run_id,
        memory_max=args.memory_max,
        unit_prefix="cityjson-index-profile",
    )
    (output_dir / "summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps({"output": str(output_dir), **summary}, sort_keys=True))
    return 0 if summary["outcome"] in {"completed", "cgroup_oom"} else 1


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("--tool", required=True, choices=sorted(TOOLS))
    result.add_argument("--workers", required=True, type=int)
    result.add_argument("--tiles", required=True, type=int)
    result.add_argument("--memory-max", default="")
    result.add_argument("--description", required=True)
    result.add_argument("--corpus", type=Path, required=True)
    result.add_argument("--work-root", type=Path, required=True)
    result.add_argument("--output-root", type=Path, required=True)
    result.add_argument("--skip-prepare", action="store_true")
    return result


if __name__ == "__main__":
    raise SystemExit(run_profile(parser().parse_args()))
