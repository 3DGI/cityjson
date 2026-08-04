#!/usr/bin/env python3
"""Profile the JSON-offset vertex-store bake-off candidate in isolated cgroups."""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import shutil
import sqlite3
import subprocess
import uuid
from datetime import UTC, datetime
from pathlib import Path

from analyze_profile import analyze as analyze_heaptrack
from profile_runtime import run_contained


WORKERS = (1, 4, 24)
FULL_TILES = 182
REDUCED_TILES = 24
SINGLE_TILES = 1
REPETITIONS = 3
GATE_RATIO = 0.9
FULL_PACKAGE_COUNT = 707_239
PERF_EVENTS = (
    "task-clock,cycles,instructions,branches,branch-misses,"
    "cache-references,cache-misses,page-faults,major-faults,context-switches"
)
TOOLS = ("native", "perf-stat", "heaptrack", "perf-record", "cachegrind")


def validate_tools() -> None:
    required = ["systemd-run", "systemctl", "perf", "heaptrack", "heaptrack_print", "valgrind"]
    missing = [name for name in required if shutil.which(name) is None]
    if missing:
        raise SystemExit("missing required profiling tools: " + ", ".join(missing))


def parse_memory_size(value: str) -> int:
    normalized = value.strip().upper()
    suffixes = {"K": 1, "M": 2, "G": 3, "T": 4}
    power = suffixes.get(normalized[-1], 0) if normalized else 0
    number = normalized[:-1] if power else normalized
    if not number.isdigit() or int(number) < 1:
        raise ValueError(f"invalid memory size: {value}")
    return int(number) * 1024**power


def gate_allows(projected_peak: int, memory_max_bytes: int) -> bool:
    return projected_peak <= int(memory_max_bytes * GATE_RATIO)


def projected_four_worker_peak(one_worker_peak: int) -> int:
    return one_worker_peak * 4


def projected_twenty_four_worker_peak(one_worker_peak: int, four_worker_peak: int) -> int:
    duplicated_slope = max(0, four_worker_peak - one_worker_peak)
    return one_worker_peak + (duplicated_slope * 23) // 3


def run_cargo_build(worktree: Path, target_dir: Path) -> None:
    command = [
        "cargo", "build", "--release", "--features", "vertex-store-bakeoff",
        "--bin", "vertex-store-bakeoff", "--target-dir", str(target_dir),
    ]
    subprocess.run(command, check=True, text=True, cwd=worktree)


def candidate_binary(args: argparse.Namespace) -> tuple[Path, Path | None]:
    binary = args.candidate_binary
    if binary is not None:
        binary = binary.resolve()
        if not binary.is_file():
            raise SystemExit(f"candidate binary does not exist: {binary}")
        return binary, args.candidate_worktree.resolve() if args.candidate_worktree else None
    if args.candidate_worktree is None:
        raise SystemExit("provide --candidate-binary or --candidate-worktree")
    worktree = args.candidate_worktree.resolve()
    if not worktree.is_dir():
        raise SystemExit(f"candidate worktree does not exist: {worktree}")
    target_dir = worktree / "target"
    binary = target_dir / "release" / "vertex-store-bakeoff"
    if not binary.is_file():
        run_cargo_build(worktree, target_dir)
    if not binary.is_file():
        raise SystemExit(f"candidate build did not produce {binary}")
    return binary, worktree


def command_version(binary: Path) -> str:
    version = subprocess.run(
        [str(binary), "--version"], check=False, text=True, capture_output=True
    )
    version_line = (version.stdout or version.stderr).splitlines()
    if version.returncode == 0 and version_line:
        return version_line[0]
    help_result = subprocess.run(
        [str(binary), "--help"], check=True, text=True, capture_output=True
    )
    help_line = (help_result.stdout or help_result.stderr).splitlines()
    detail = help_line[0] if help_line else "unknown"
    return f"unavailable (--version unsupported); help: {detail}"


def git_value(worktree: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args], check=True, text=True, capture_output=True, cwd=worktree
    )
    return result.stdout.strip()


def working_tree_provenance(worktree: Path) -> dict[str, object]:
    status = git_value(worktree, "status", "--porcelain=v1", "--untracked-files=all")
    diff = git_value(worktree, "diff", "--binary", "HEAD")
    digest = hashlib.sha256()
    digest.update(status.encode("utf-8"))
    digest.update(b"\0")
    digest.update(diff.encode("utf-8"))
    return {
        "dirty": bool(status),
        "status": status.splitlines(),
        "diff_sha256": digest.hexdigest(),
    }


def corpus_files(corpus_root: Path) -> list[Path]:
    files = sorted(path for path in corpus_root.rglob("*.city.json") if path.is_file())
    if not files:
        raise SystemExit(f"corpus contains no .city.json files: {corpus_root}")
    return files


def prepare_corpus(corpus_root: Path, prepared_root: Path, tiles: int) -> dict[str, object]:
    sources = corpus_files(corpus_root)
    if len(sources) < tiles:
        raise SystemExit(f"corpus has {len(sources)} tiles, expected at least {tiles}")
    dataset = prepared_root / f"cityjson-{tiles}"
    if dataset.exists():
        shutil.rmtree(dataset)
    dataset.mkdir(parents=True)
    selected = sources[:tiles]
    for source in selected:
        shutil.copy2(source, dataset / source.name)
    return {
        "tiles": tiles,
        "dataset_root": str(dataset),
        "source_names": [path.name for path in selected],
        "source_bytes": sum(path.stat().st_size for path in selected),
    }


def sqlite_scalar(connection: sqlite3.Connection, query: str) -> int:
    row = connection.execute(query).fetchone()
    if row is None or not isinstance(row[0], int):
        raise RuntimeError(f"sidecar query did not return an integer: {query}")
    return row[0]


def validate_sidecar(
    sidecar: Path,
    dataset_root: Path,
    expected_source_count: int,
    expected_package_count: int | None,
) -> dict[str, int | str]:
    if not sidecar.is_file():
        raise RuntimeError(f"candidate sidecar is missing: {sidecar}")
    uri = f"{sidecar.as_uri()}?mode=ro"
    connection = sqlite3.connect(uri, uri=True)
    try:
        marker = connection.execute(
            "SELECT schema_version, strategy FROM vertex_store_bakeoff_state WHERE id = 1"
        ).fetchone()
        if marker != (3, "json-offsets"):
            raise RuntimeError(f"invalid JSON-offset marker in {sidecar}: {marker!r}")
        state = connection.execute(
            "SELECT schema_version, needs_reindex FROM schema_state WHERE id = 1"
        ).fetchone()
        if state != (2, 0):
            raise RuntimeError(f"sidecar is stale or has an invalid schema state: {state!r}")
        source_count = sqlite_scalar(connection, "SELECT COUNT(*) FROM sources")
        package_count = sqlite_scalar(connection, "SELECT COUNT(*) FROM packages")
        if source_count != expected_source_count:
            raise RuntimeError(
                f"sidecar source count {source_count} != expected {expected_source_count}"
            )
        if expected_package_count is not None and package_count != expected_package_count:
            raise RuntimeError(
                f"sidecar package count {package_count} != expected {expected_package_count}"
            )
        payload_bytes = sqlite_scalar(
            connection,
            "SELECT COALESCE(SUM(payload_bytes), 0) FROM vertex_store_source_state",
        )
        rows = connection.execute(
            "SELECT path, source_size, source_mtime_ns FROM sources ORDER BY path"
        ).fetchall()
        for raw_path, source_size, source_mtime_ns in rows:
            source = Path(raw_path)
            if not source.is_file():
                raise RuntimeError(f"sidecar source is missing: {source}")
            stat = source.stat()
            if (source_size, source_mtime_ns) != (stat.st_size, stat.st_mtime_ns):
                raise RuntimeError(f"sidecar is stale for source {source}")
            if source.parent != dataset_root:
                raise RuntimeError(f"sidecar source escaped prepared dataset: {source}")
        return {
            "schema_version": int(marker[0]),
            "strategy": str(marker[1]),
            "source_count": source_count,
            "package_count": package_count,
            "candidate_payload_bytes": payload_bytes,
        }
    finally:
        connection.close()


def build_sidecar(
    binary: Path,
    preparation: dict[str, object],
    sidecar: Path,
    expected_package_count: int | None,
) -> dict[str, object]:
    dataset_root = Path(str(preparation["dataset_root"]))
    command = [
        str(binary), "build", "--dataset-root", str(dataset_root),
        "--sidecar", str(sidecar),
    ]
    subprocess.run(command, check=True, text=True)
    validation = validate_sidecar(
        sidecar, dataset_root, int(preparation["tiles"]), expected_package_count
    )
    return {
        "dataset_root": str(dataset_root),
        "sidecar": str(sidecar),
        "source_count": validation["source_count"],
        "package_count": validation["package_count"],
        "candidate_payload_bytes": validation["candidate_payload_bytes"],
        "sidecar_bytes": sidecar.stat().st_size,
        "read_only_validation": True,
        "fresh": True,
    }


def bakeoff_command(
    binary: Path,
    *,
    dataset_root: Path,
    sidecar: Path,
    result: Path,
    profile_output: Path,
    candidate_commit: str,
    harness_commit: str,
    corpus_identity: str,
    workers: int,
    repetition: int,
    tool: str,
    campaign_id: str,
) -> list[str]:
    return [
        str(binary), "tyler-materialization",
        "--dataset-root", str(dataset_root),
        "--sidecar", str(sidecar),
        "--result", str(result),
        "--profile-output", str(profile_output),
        "--candidate-commit", candidate_commit,
        "--harness-commit", harness_commit,
        "--corpus-identity", corpus_identity,
        "--workers", str(workers),
        "--repetition", str(repetition),
        "--runtime", f"campaign={campaign_id}",
        "--runtime", f"profiler={tool}",
        "--runtime", "memory-sampling-ms=100",
    ]


def profiler_command(tool: str, candidate: list[str], output_dir: Path) -> list[str]:
    if tool == "native":
        return candidate
    if tool == "perf-stat":
        return [
            "perf", "stat", "-x", ";", "-o", str(output_dir / "perf-stat.csv"),
            "-e", PERF_EVENTS, "--", *candidate,
        ]
    if tool == "heaptrack":
        return ["heaptrack", "--record-only", "-o", str(output_dir / "heaptrack"), *candidate]
    if tool == "perf-record":
        return [
            "perf", "record", "-F", "99", "--call-graph", "dwarf",
            "-o", str(output_dir / "perf.data"), "--", *candidate,
        ]
    if tool == "cachegrind":
        return [
            "valgrind", "--tool=cachegrind", "--cache-sim=yes", "--branch-sim=yes",
            f"--cachegrind-out-file={output_dir / 'cachegrind.out'}", *candidate,
        ]
    raise ValueError(f"unknown profiler: {tool}")


def parse_perf_stat(path: Path) -> dict[str, object]:
    if not path.is_file():
        return {}
    counters: dict[str, object] = {}
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        fields = line.split(";")
        if len(fields) < 3:
            continue
        value, event = fields[0].strip(), fields[2].strip()
        if value and event and value != "<not counted>":
            counters[event] = value
    return counters


def parse_candidate_result(
    result_path: Path,
    profile_path: Path,
    *,
    expected_workers: int,
    expected_package_count: int,
    expected_provenance: dict[str, object] | None = None,
) -> dict[str, object]:
    if not result_path.is_file():
        raise RuntimeError(f"candidate result is missing: {result_path}")
    if not profile_path.is_file():
        raise RuntimeError(f"candidate profile output is missing: {profile_path}")
    result = json.loads(result_path.read_text(encoding="utf-8"))
    if not isinstance(result, dict):
        raise RuntimeError(f"candidate result is not a JSON object: {result_path}")
    if result.get("schema_version") != 2:
        raise RuntimeError(f"unexpected candidate result schema: {result.get('schema_version')}")
    if result.get("experiment") != "tyler-materialization":
        raise RuntimeError(f"unexpected candidate experiment: {result.get('experiment')}")
    provenance, telemetry, payload = (
        result.get("provenance"), result.get("telemetry"), result.get("result")
    )
    profile = json.loads(profile_path.read_text(encoding="utf-8"))
    if not isinstance(provenance, dict) or not isinstance(telemetry, dict):
        raise RuntimeError("candidate result is missing provenance or telemetry")
    if expected_provenance is not None:
        for field, expected in expected_provenance.items():
            if provenance.get(field) != expected:
                raise RuntimeError(
                    f"candidate provenance {field}={provenance.get(field)!r} != {expected!r}"
                )
    if not isinstance(payload, dict) or not isinstance(profile, dict):
        raise RuntimeError("candidate result or profile output is malformed")
    if payload.get("configured_workers") != expected_workers:
        raise RuntimeError("candidate result worker count disagrees with command")
    if payload.get("package_count") != expected_package_count:
        raise RuntimeError("candidate package count disagrees with sidecar preparation")
    for field in ("elapsed_ns", "total_pipeline_elapsed_ns", "package_count", "configured_workers"):
        if not isinstance(payload.get(field), int):
            raise RuntimeError(f"candidate result is missing integer {field}")
    for field in ("current_rss_bytes", "process_peak_rss_bytes", "peak_rss_bytes"):
        if not isinstance(profile.get(field), int):
            raise RuntimeError(f"candidate profile is missing {field}")
    for field in ("requested_vertex_count", "unique_vertex_count", "returned_vertex_count",
                  "persistent_bytes_read", "source_json_bytes_read", "touched_units",
                  "retained_decoded_bytes"):
        if not isinstance(telemetry.get(field), int):
            raise RuntimeError(f"candidate telemetry is missing integer {field}")
    return {
        "candidate_result": result,
        "candidate_memory": profile,
        "materialization_elapsed_ns": payload["elapsed_ns"],
        "total_pipeline_elapsed_ns": payload["total_pipeline_elapsed_ns"],
        "package_count": payload["package_count"],
        "configured_workers": payload["configured_workers"],
        "model_digest": payload["model_digest"],
        "telemetry": telemetry,
    }


def command_version_for_metadata(binary: Path) -> str:
    return command_version(binary)


def run_measurement(
    args: argparse.Namespace,
    *,
    binary: Path,
    campaign_id: str,
    preparation: dict[str, object],
    sidecar: dict[str, object],
    tool: str,
    workers: int,
    repetition: int,
    run_dir: Path,
) -> dict[str, object]:
    run_dir.mkdir(parents=True)
    dataset_root = Path(str(preparation["dataset_root"]))
    candidate_result = run_dir / "tyler-result.json"
    profile_output = run_dir / "memory-snapshot.json"
    command = bakeoff_command(
        binary, dataset_root=dataset_root, sidecar=Path(str(sidecar["sidecar"])),
        result=candidate_result, profile_output=profile_output,
        candidate_commit=args.candidate_commit, harness_commit=args.harness_commit,
        corpus_identity=args.corpus_identity, workers=workers, repetition=repetition,
        tool=tool, campaign_id=campaign_id,
    )
    wrapped = profiler_command(tool, command, run_dir)
    metadata = {
        "schema_version": 2,
        "run_id": run_dir.name,
        "tool": tool,
        "workers": workers,
        "tiles": preparation["tiles"],
        "repetition": repetition,
        "binary": str(binary),
        "binary_version": command_version_for_metadata(binary),
        "candidate_commit": args.candidate_commit,
        "harness_commit": args.harness_commit,
        "corpus_identity": args.corpus_identity,
        "dataset_root": str(dataset_root),
        "sidecar": sidecar,
        "memory_max": args.memory_max,
        "memory_swap_max": "0",
        "sample_interval_seconds": 0.1,
        "command": wrapped,
        "candidate_command": command,
        "platform": platform.platform(),
    }
    if args.candidate_worktree is not None:
        worktree = args.candidate_worktree.resolve()
        metadata["candidate_worktree"] = str(worktree)
        metadata["working_tree"] = working_tree_provenance(worktree)
    (run_dir / "metadata.json").write_text(
        json.dumps(metadata, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    summary = run_contained(
        wrapped, output_dir=run_dir, run_id=run_dir.name,
        memory_max=args.memory_max, unit_prefix="vertex-store-bakeoff",
    )
    (run_dir / "summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    normalized = parse_candidate_result(
        candidate_result, profile_output, expected_workers=workers,
        expected_package_count=int(sidecar["package_count"]),
        expected_provenance={
            "candidate_commit": args.candidate_commit,
            "harness_commit": args.harness_commit,
            "corpus_identity": args.corpus_identity,
            "strategy": "json-offsets",
            "sidecar_path": str(Path(str(sidecar["sidecar"])).resolve()),
            "worker_count": workers,
            "repetition": repetition,
        },
    )
    heaptrack_summary = None
    if tool == "heaptrack":
        analysis = analyze_heaptrack(run_dir)
        heaptrack_summary = {
            "peak_requested_heap_bytes": analysis["peak_requested_heap_bytes"],
            "peak_allocation_categories_bytes": analysis["peak_allocation_categories_bytes"],
        }
    samples = [
        json.loads(line)
        for line in (run_dir / "cgroup-memory.jsonl").read_text(encoding="utf-8").splitlines()
    ]
    swap_samples = [
        sample.get("memory_stat", {})
        for sample in samples
        if isinstance(sample.get("memory_stat"), dict)
    ]
    swap_counters = {
        name: max((int(sample.get(name, 0)) for sample in swap_samples), default=0)
        for name in ("pswpin", "pswpout", "swapcached")
    }
    normalized.update({
        "run_id": run_dir.name,
        "tool": tool,
        "workers": workers,
        "tiles": preparation["tiles"],
        "repetition": repetition,
        "outcome": summary["outcome"],
        "cgroup_peak_bytes": summary["memory_peak_bytes"],
        "memory_events": summary["memory_events"],
        "systemd": summary["systemd"],
        "raw_artifacts": {
            "run_dir": str(run_dir),
            "metadata": str(run_dir / "metadata.json"),
            "summary": str(run_dir / "summary.json"),
            "cgroup_memory": str(run_dir / "cgroup-memory.jsonl"),
            "stdout": str(run_dir / "stdout.log"),
            "stderr": str(run_dir / "stderr.log"),
            "candidate_result": str(candidate_result),
            "memory_snapshot": str(profile_output),
            "perf_stat": str(run_dir / "perf-stat.csv") if tool == "perf-stat" else None,
            "perf_data": str(run_dir / "perf.data") if tool == "perf-record" else None,
            "heaptrack": str(run_dir / "heaptrack") if tool == "heaptrack" else None,
            "heaptrack_analysis": str(run_dir / "heaptrack-analysis.json") if tool == "heaptrack" else None,
            "cachegrind": str(run_dir / "cachegrind.out") if tool == "cachegrind" else None,
        },
        "perf_counters": parse_perf_stat(run_dir / "perf-stat.csv")
        if tool == "perf-stat" else {},
        "sidecar_bytes": sidecar["sidecar_bytes"],
        "candidate_payload_bytes": sidecar["candidate_payload_bytes"],
        "swap_counters": swap_counters,
        "heaptrack_summary": heaptrack_summary,
    })
    if summary["outcome"] != "completed":
        raise RuntimeError(f"candidate run {run_dir.name} ended as {summary['outcome']}")
    return normalized


def write_manifest(path: Path, manifest: dict[str, object]) -> None:
    path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def append_run(manifest_path: Path, manifest: dict[str, object], run: dict[str, object]) -> None:
    runs = manifest["runs"]
    if not isinstance(runs, list):
        raise RuntimeError("campaign manifest runs is not a list")
    runs.append(run)
    write_manifest(manifest_path, manifest)


def run_campaign(args: argparse.Namespace) -> Path:
    parse_memory_size(args.memory_max)
    validate_tools()
    corpus_root = args.corpus.resolve()
    work_root = args.work_root.resolve()
    output_root = args.output_root.resolve()
    binary, worktree = candidate_binary(args)
    if not corpus_root.is_dir():
        raise SystemExit(f"corpus root does not exist: {corpus_root}")
    timestamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
    campaign_id = f"{timestamp}-json-offsets-{uuid.uuid4().hex[:8]}"
    campaign_dir = output_root / "campaigns" / campaign_id
    campaign_dir.mkdir(parents=True)
    manifest_path = campaign_dir / "campaign.json"
    manifest = {
        "schema_version": 2, "campaign_id": campaign_id,
        "description": args.description, "candidate_commit": args.candidate_commit,
        "harness_commit": args.harness_commit, "candidate_binary": str(binary),
        "candidate_worktree": str(worktree) if worktree is not None else None,
        "candidate_binary_version": command_version(binary),
        "corpus_root": str(corpus_root), "corpus_identity": args.corpus_identity,
        "memory_max": args.memory_max, "memory_swap_max": "0",
        "sample_interval_seconds": 0.1, "runtime_max_seconds": 3600,
        "workers": list(WORKERS), "runs": [], "heaptrack_gates": [],
        "heaptrack_fallback": False, "preparations": {},
    }
    if worktree is not None:
        manifest["candidate_worktree_provenance"] = working_tree_provenance(worktree)
    write_manifest(manifest_path, manifest)

    prepared: dict[int, dict[str, object]] = {}
    for tiles in (FULL_TILES, REDUCED_TILES, SINGLE_TILES):
        prepared[tiles] = prepare_corpus(corpus_root, work_root, tiles)
        sidecar_path = {
            FULL_TILES: args.sidecar_182,
            REDUCED_TILES: args.sidecar_24,
            SINGLE_TILES: args.sidecar_1,
        }[tiles].resolve()
        expected = FULL_PACKAGE_COUNT if tiles == FULL_TILES else None
        sidecar = build_sidecar(binary, prepared[tiles], sidecar_path, expected)
        prepared[tiles]["sidecar_info"] = sidecar
        manifest["preparations"][str(tiles)] = prepared[tiles]
        write_manifest(manifest_path, manifest)

    digest_set: set[str] = set()
    for tool in ("native", "perf-stat"):
        for repetition in range(1, REPETITIONS + 1):
            for workers in WORKERS:
                run = run_measurement(
                    args, binary=binary, campaign_id=campaign_id,
                    preparation=prepared[FULL_TILES],
                    sidecar=prepared[FULL_TILES]["sidecar_info"],
                    tool=tool, workers=workers, repetition=repetition,
                    run_dir=campaign_dir / "runs" / f"{tool}-w{workers}-r{repetition}-t182",
                )
                digest_set.add(str(run["model_digest"]))
                append_run(manifest_path, manifest, run)
    write_manifest(manifest_path, manifest)

    peaks: dict[int, int] = {}
    full_heaptrack = True
    for workers in WORKERS:
        projected = None
        if workers == 4 and 1 in peaks:
            projected = projected_four_worker_peak(peaks[1])
        elif workers == 24 and 1 in peaks and 4 in peaks:
            projected = projected_twenty_four_worker_peak(peaks[1], peaks[4])
        allowed = projected is None or gate_allows(projected, parse_memory_size(args.memory_max))
        manifest["heaptrack_gates"].append({
            "workers": workers, "projected_peak_bytes": projected, "allowed": allowed,
            "reason": "initial measurement" if projected is None else (
                "below 90% safety threshold" if allowed
                else "projected cgroup peak exceeds 90% safety threshold"
            ),
        })
        write_manifest(manifest_path, manifest)
        if not allowed:
            full_heaptrack = False
            break
        run = run_measurement(
            args, binary=binary, campaign_id=campaign_id,
            preparation=prepared[FULL_TILES], sidecar=prepared[FULL_TILES]["sidecar_info"],
            tool="heaptrack", workers=workers, repetition=1,
            run_dir=campaign_dir / "runs" / f"heaptrack-w{workers}-r1-t182",
        )
        peaks[workers] = int(run["cgroup_peak_bytes"])
        digest_set.add(str(run["model_digest"]))
        append_run(manifest_path, manifest, run)
    if not full_heaptrack:
        manifest["heaptrack_fallback"] = True
        write_manifest(manifest_path, manifest)
        for workers in WORKERS:
            run = run_measurement(
                args, binary=binary, campaign_id=campaign_id,
                preparation=prepared[REDUCED_TILES], sidecar=prepared[REDUCED_TILES]["sidecar_info"],
                tool="heaptrack", workers=workers, repetition=1,
                run_dir=campaign_dir / "runs" / f"heaptrack-w{workers}-r1-t24",
            )
            digest_set.add(str(run["model_digest"]))
            append_run(manifest_path, manifest, run)

    for workers in WORKERS:
        run = run_measurement(
            args, binary=binary, campaign_id=campaign_id,
            preparation=prepared[REDUCED_TILES], sidecar=prepared[REDUCED_TILES]["sidecar_info"],
            tool="perf-record", workers=workers, repetition=1,
            run_dir=campaign_dir / "runs" / f"perf-record-w{workers}-r1-t24",
        )
        digest_set.add(str(run["model_digest"]))
        append_run(manifest_path, manifest, run)
    run = run_measurement(
        args, binary=binary, campaign_id=campaign_id,
        preparation=prepared[SINGLE_TILES], sidecar=prepared[SINGLE_TILES]["sidecar_info"],
        tool="cachegrind", workers=1, repetition=1,
        run_dir=campaign_dir / "runs" / "cachegrind-w1-r1-t1",
    )
    digest_set.add(str(run["model_digest"]))
    append_run(manifest_path, manifest, run)

    manifest["model_digests"] = sorted(digest_set)
    manifest["model_digest_count"] = len(digest_set)
    digests_by_tiles: dict[str, list[str]] = {}
    package_counts_by_tiles: dict[str, list[int]] = {}
    for run in manifest["runs"]:
        tiles = str(run["tiles"])
        digests_by_tiles.setdefault(tiles, [])
        package_counts_by_tiles.setdefault(tiles, [])
        if run["model_digest"] not in digests_by_tiles[tiles]:
            digests_by_tiles[tiles].append(run["model_digest"])
        if run["package_count"] not in package_counts_by_tiles[tiles]:
            package_counts_by_tiles[tiles].append(run["package_count"])
    manifest["model_digests_by_tiles"] = {
        tiles: sorted(digests) for tiles, digests in digests_by_tiles.items()
    }
    manifest["package_counts_by_tiles"] = {
        tiles: sorted(counts) for tiles, counts in package_counts_by_tiles.items()
    }
    manifest["expected_full_package_count"] = int(
        prepared[FULL_TILES]["sidecar_info"]["package_count"]
    )
    manifest["zero_oom_kills_required"] = True
    write_manifest(manifest_path, manifest)
    print(json.dumps({"campaign": str(campaign_dir)}, sort_keys=True))
    return campaign_dir


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    candidates = result.add_mutually_exclusive_group()
    candidates.add_argument("--candidate-worktree", type=Path)
    candidates.add_argument("--candidate-binary", type=Path)
    result.add_argument("--candidate-commit", required=True)
    result.add_argument("--harness-commit", required=True)
    result.add_argument("--corpus-identity", required=True)
    result.add_argument("--description", required=True)
    result.add_argument("--corpus", type=Path, required=True)
    result.add_argument("--work-root", type=Path, required=True)
    result.add_argument("--output-root", type=Path, required=True)
    result.add_argument("--sidecar-182", type=Path, required=True)
    result.add_argument("--sidecar-24", type=Path, required=True)
    result.add_argument("--sidecar-1", type=Path, required=True)
    result.add_argument("--memory-max", default="28G")
    return result


if __name__ == "__main__":
    raise SystemExit(0 if run_campaign(parser().parse_args()) else 1)
