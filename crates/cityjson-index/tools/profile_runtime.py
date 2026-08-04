#!/usr/bin/env python3
"""Shared systemd/cgroup execution primitives for contained profiles."""

from __future__ import annotations

import json
import subprocess
import time
import uuid
from pathlib import Path


SAMPLE_INTERVAL_SECONDS = 0.1


def run_checked(args: list[str], *, capture: bool = False) -> subprocess.CompletedProcess[str]:
    """Run one explicit argv vector and fail on a non-zero exit status."""
    return subprocess.run(
        args,
        check=True,
        text=True,
        capture_output=capture,
    )


def systemctl_properties(unit: str, *names: str) -> dict[str, str]:
    """Read selected properties from a user systemd unit."""
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
    """Read integer-valued cgroup files such as memory.events."""
    values: dict[str, int] = {}
    if not path.exists():
        return values
    for line in path.read_text(encoding="utf-8").splitlines():
        key, separator, value = line.partition(" ")
        if separator and value.isdigit():
            values[key] = int(value)
    return values


def read_integer(path: Path) -> int | None:
    """Read one integer cgroup value, returning None when unavailable."""
    if not path.exists():
        return None
    value = path.read_text(encoding="utf-8").strip()
    return int(value) if value.isdigit() else None


def sample_cgroup(cgroup: Path) -> dict[str, object]:
    """Capture one cgroup memory sample."""
    pressure = cgroup / "memory.pressure"
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


def sample_has_measurements(sample: dict[str, object]) -> bool:
    """Return whether a sample contains the cgroup peak measurement."""
    return sample.get("memory_peak_bytes") is not None


def classify_outcome(properties: dict[str, str], memory_events: dict[str, int]) -> str:
    """Classify terminal status, giving authoritative OOM events priority."""
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
    """Append and flush one JSONL value."""
    with path.open("a", encoding="utf-8") as handle:
        json.dump(value, handle, sort_keys=True)
        handle.write("\n")
        handle.flush()


def run_contained(
    command: list[str],
    *,
    output_dir: Path,
    run_id: str,
    memory_max: str,
    unit_prefix: str,
) -> dict[str, object]:
    """Run an explicit command in a transient user-systemd cgroup."""
    samples = output_dir / "cgroup-memory.jsonl"
    unit = f"{unit_prefix}-{uuid.uuid4().hex[:12]}.service"
    systemd_command = [
        "systemd-run",
        "--user",
        f"--unit={unit}",
        "--no-block",
        "--property=Type=exec",
        "--property=RemainAfterExit=yes",
        "--property=MemoryAccounting=yes",
        f"--property=MemoryMax={memory_max or '8G'}",
        "--property=MemorySwapMax=0",
        "--property=RuntimeMaxSec=3600",
        f"--property=StandardOutput=append:{output_dir / 'stdout.log'}",
        f"--property=StandardError=append:{output_dir / 'stderr.log'}",
        *command,
    ]
    run_checked(systemd_command)

    cgroup: Path | None = None
    final_properties: dict[str, str] = {}
    last_sample: dict[str, object] | None = None
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
            current_sample = sample_cgroup(cgroup)
            append_json_line(samples, current_sample)
            if sample_has_measurements(current_sample):
                last_sample = current_sample
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
        terminal_sample = sample_cgroup(cgroup)
        append_json_line(samples, terminal_sample)
        if sample_has_measurements(terminal_sample):
            last_sample = terminal_sample
    finally:
        subprocess.run(["systemctl", "--user", "stop", unit], check=False)
        subprocess.run(["systemctl", "--user", "reset-failed", unit], check=False)

    if last_sample is None:
        raise RuntimeError(f"cgroup {cgroup} disappeared before yielding a memory sample")
    events_data = last_sample.get("memory_events", {})
    typed_events = events_data if isinstance(events_data, dict) else {}
    return {
        "run_id": run_id,
        "outcome": classify_outcome(final_properties, typed_events),
        "systemd": final_properties,
        "memory_peak_bytes": last_sample.get("memory_peak_bytes"),
        "memory_events": events_data,
    }
