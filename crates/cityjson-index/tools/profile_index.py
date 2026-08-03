#!/usr/bin/env python3
"""Run one contained cityjson-index profiling target and retain OOM evidence."""

from __future__ import annotations

import argparse
import json
import platform
import shutil
import subprocess
import sys
import time
import uuid
from datetime import UTC, datetime
from pathlib import Path


TOOLS = {"native", "perf-stat", "perf-record", "heaptrack", "cachegrind", "massif"}
SAMPLE_INTERVAL_SECONDS = 0.1


def run_checked(args: list[str], *, capture: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        check=True,
        text=True,
        capture_output=capture,
    )


def systemctl_properties(unit: str, *names: str) -> dict[str, str]:
    command = ["systemctl", "--user", "show", unit]
    for name in names:
        command.extend(["--property", name])
    result = run_checked(command, capture=True)
    properties: dict[str, str] = {}
    for line in result.stdout.splitlines():
        key, separator, value = line.partition("=")
        if separator:
            properties[key] = value
    return properties


def read_key_values(path: Path) -> dict[str, int]:
    values: dict[str, int] = {}
    if not path.exists():
        return values
    for line in path.read_text(encoding="utf-8").splitlines():
        key, separator, value = line.partition(" ")
        if separator and value.isdigit():
            values[key] = int(value)
    return values


def read_integer(path: Path) -> int | None:
    try:
        return int(path.read_text(encoding="utf-8").strip())
    except (FileNotFoundError, ValueError):
        return None


def sample_cgroup(cgroup: Path) -> dict[str, object]:
    pressure = cgroup.joinpath("memory.pressure")
    return {
        "timestamp_ns": time.time_ns(),
        "memory_current_bytes": read_integer(cgroup / "memory.current"),
        "memory_peak_bytes": read_integer(cgroup / "memory.peak"),
        "memory_events": read_key_values(cgroup / "memory.events"),
        "memory_stat": read_key_values(cgroup / "memory.stat"),
        "memory_pressure": pressure.read_text(encoding="utf-8").splitlines()
        if pressure.exists()
        else [],
    }


def classify_outcome(properties: dict[str, str], memory_events: dict[str, int]) -> str:
    """Distinguish a cgroup OOM from ordinary failures and external termination."""
    if memory_events.get("oom_kill", 0) > 0:
        return "cgroup_oom"
    result = properties.get("Result", "unknown")
    status = int(properties.get("ExecMainStatus", "0") or 0)
    if result == "success" and status == 0:
        return "completed"
    if result in {"timeout", "watchdog"}:
        return "killed"
    return "failed"


def append_json_line(path: Path, value: object) -> None:
    with path.open("a", encoding="utf-8") as handle:
        json.dump(value, handle, sort_keys=True)
        handle.write("\n")
        handle.flush()


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
    if args.tool in {"perf-record", "heaptrack"} and args.tiles > 24:
        raise SystemExit(f"{args.tool} is restricted to at most 24 tiles")
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
        return ["heaptrack", "-o", str(output_dir / "heaptrack"), *benchmark]
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


def metadata(args: argparse.Namespace, run_id: str, command: list[str]) -> dict[str, object]:
    commit = run_checked(["git", "rev-parse", "HEAD"], capture=True).stdout.strip()
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
    prepare(args, binary)

    timestamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
    run_id = f"{timestamp}-{args.tool}-w{args.workers}-t{args.tiles}-{uuid.uuid4().hex[:8]}"
    output_dir = args.output_root.resolve() / run_id
    output_dir.mkdir(parents=True)
    events = output_dir / "stage-events.jsonl"
    samples = output_dir / "cgroup-memory.jsonl"
    command = profiler_command(args, binary, events, output_dir)
    (output_dir / "metadata.json").write_text(
        json.dumps(metadata(args, run_id, command), indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    unit = f"cityjson-index-profile-{uuid.uuid4().hex[:12]}.service"
    memory_max = args.memory_max or "8G"
    systemd_command = [
        "systemd-run",
        "--user",
        f"--unit={unit}",
        "--no-block",
        "--property=Type=exec",
        "--property=RemainAfterExit=yes",
        "--property=MemoryAccounting=yes",
        f"--property=MemoryMax={memory_max}",
        "--property=MemorySwapMax=0",
        "--property=RuntimeMaxSec=3600",
        f"--property=StandardOutput=append:{output_dir / 'stdout.log'}",
        f"--property=StandardError=append:{output_dir / 'stderr.log'}",
        *command,
    ]
    run_checked(systemd_command)

    cgroup: Path | None = None
    final_properties: dict[str, str] = {}
    try:
        for _ in range(100):
            properties = systemctl_properties(unit, "ControlGroup", "SubState")
            control_group = properties.get("ControlGroup", "")
            if control_group:
                candidate = Path("/sys/fs/cgroup") / control_group.removeprefix("/")
                if candidate.exists():
                    cgroup = candidate
                    break
            time.sleep(SAMPLE_INTERVAL_SECONDS)
        if cgroup is None:
            raise RuntimeError(f"systemd unit {unit} did not expose its cgroup")

        while True:
            append_json_line(samples, sample_cgroup(cgroup))
            final_properties = systemctl_properties(
                unit,
                "SubState",
                "Result",
                "ExecMainCode",
                "ExecMainStatus",
            )
            if final_properties.get("SubState") in {"exited", "failed", "dead"}:
                break
            time.sleep(SAMPLE_INTERVAL_SECONDS)
        final_sample = sample_cgroup(cgroup)
        append_json_line(samples, final_sample)
    finally:
        subprocess.run(["systemctl", "--user", "stop", unit], check=False)
        subprocess.run(["systemctl", "--user", "reset-failed", unit], check=False)

    events_data = final_sample.get("memory_events", {})
    typed_events = events_data if isinstance(events_data, dict) else {}
    outcome = classify_outcome(final_properties, typed_events)
    summary = {
        "run_id": run_id,
        "outcome": outcome,
        "systemd": final_properties,
        "memory_peak_bytes": final_sample.get("memory_peak_bytes"),
        "memory_events": events_data,
    }
    (output_dir / "summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps({"output": str(output_dir), **summary}, sort_keys=True))
    return 0 if outcome in {"completed", "cgroup_oom"} else 1


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
    return result


if __name__ == "__main__":
    raise SystemExit(run_profile(parser().parse_args()))
