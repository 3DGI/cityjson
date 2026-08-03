"""Unit tests for the profiling supervisor's safety and command policy."""

from __future__ import annotations

import argparse
import importlib.util
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("profile_index.py")
SPEC = importlib.util.spec_from_file_location("profile_index", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
profile_index = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(profile_index)


def arguments(**overrides: object) -> argparse.Namespace:
    values: dict[str, object] = {
        "tool": "native",
        "workers": 1,
        "tiles": 24,
        "memory_max": "",
        "work_root": Path("work"),
        "corpus": Path("corpus"),
        "skip_prepare": False,
    }
    values.update(overrides)
    return argparse.Namespace(**values)


class ValidationTests(unittest.TestCase):
    def test_full_corpus_requires_explicit_memory_limit(self) -> None:
        with self.assertRaisesRegex(SystemExit, "explicit --memory-max"):
            profile_index.validate(arguments(tiles=182))

    def test_valgrind_is_restricted_to_reduced_single_worker_run(self) -> None:
        with self.assertRaisesRegex(SystemExit, "restricted"):
            profile_index.validate(arguments(tool="cachegrind", workers=2, tiles=1))

    def test_allocation_profilers_target_materialization_only(self) -> None:
        command = profile_index.profiler_command(
            arguments(tool="heaptrack"), Path("bench-index"), Path("events"), Path("out")
        )
        self.assertIn("tyler-feature-materialization", command)
        self.assertIn("--record-only", command)

    def test_heaptrack_allows_the_full_corpus_with_a_memory_limit(self) -> None:
        profile_index.validate(arguments(tool="heaptrack", tiles=182, memory_max="28G"))

    def test_native_runs_target_the_complete_pipeline(self) -> None:
        command = profile_index.profiler_command(
            arguments(), Path("bench-index"), Path("events"), Path("out")
        )
        self.assertIn("tyler-pipeline", command)
        self.assertIn("corpus", command)


class ParsingTests(unittest.TestCase):
    def test_reads_cgroup_key_value_file(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "memory.events"
            path.write_text("oom 2\noom_kill 1\ninvalid value\n", encoding="utf-8")
            self.assertEqual(profile_index.read_key_values(path), {"oom": 2, "oom_kill": 1})

    def test_oom_event_has_priority_over_exit_status(self) -> None:
        properties = {"Result": "failed", "ExecMainStatus": "9"}
        self.assertEqual(profile_index.classify_outcome(properties, {"oom_kill": 1}), "cgroup_oom")

    def test_timeout_is_not_misreported_as_oom(self) -> None:
        properties = {"Result": "timeout", "ExecMainStatus": "9"}
        self.assertEqual(profile_index.classify_outcome(properties, {}), "killed")

    def test_empty_terminal_sample_does_not_contain_measurements(self) -> None:
        self.assertFalse(profile_index.sample_has_measurements({"memory_peak_bytes": None}))
        self.assertTrue(profile_index.sample_has_measurements({"memory_peak_bytes": 1024}))


if __name__ == "__main__":
    unittest.main()
