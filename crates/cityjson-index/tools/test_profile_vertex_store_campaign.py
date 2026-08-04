"""Tests for the JSON-offset candidate profiling campaign."""

from __future__ import annotations

import argparse
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


TOOLS_ROOT = Path(__file__).parent
if str(TOOLS_ROOT) not in sys.path:
    sys.path.insert(0, str(TOOLS_ROOT))
MODULE_PATH = TOOLS_ROOT / "profile_vertex_store_campaign.py"
SPEC = importlib.util.spec_from_file_location("profile_vertex_store_campaign", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
campaign = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(campaign)


def arguments(**overrides: object) -> argparse.Namespace:
    values: dict[str, object] = {
        "candidate_binary": Path("/candidate/vertex-store-bakeoff"),
        "candidate_worktree": None,
        "candidate_commit": "1f33165e5481455074ff007f7dff4b8d948e4287",
        "harness_commit": "bc23d7e",
        "corpus_identity": "groningen-182-local-2026-08-04",
        "description": "test campaign",
        "corpus": Path("/corpus"),
        "work_root": Path("/work"),
        "output_root": Path("/output"),
        "sidecar_182": Path("/work/offsets-182.sqlite"),
        "sidecar_24": Path("/work/offsets-24.sqlite"),
        "sidecar_1": Path("/work/offsets-1.sqlite"),
        "memory_max": "28G",
    }
    values.update(overrides)
    return argparse.Namespace(**values)


class CommandTests(unittest.TestCase):
    def test_candidate_command_is_explicit_and_records_provenance(self) -> None:
        command = campaign.bakeoff_command(
            Path("/candidate/vertex-store-bakeoff"),
            dataset_root=Path("/prepared"),
            sidecar=Path("/prepared/offsets.sqlite"),
            result=Path("/run/result.json"),
            profile_output=Path("/run/memory.json"),
            candidate_commit="candidate-sha",
            harness_commit="bc23d7e",
            corpus_identity="groningen",
            workers=4,
            repetition=2,
            tool="native",
            campaign_id="campaign-id",
        )
        self.assertEqual(command[0:2], ["/candidate/vertex-store-bakeoff", "tyler-materialization"])
        self.assertIn("--profile-output", command)
        self.assertIn("candidate-sha", command)
        self.assertIn("bc23d7e", command)
        self.assertIn("groningen", command)
        self.assertNotIn("&&", command)
        self.assertNotIn(" ", command[0])

    def test_profiler_wraps_the_candidate_without_shell_expansion(self) -> None:
        command = campaign.profiler_command(
            "perf-stat",
            ["/candidate", "tyler-materialization", "--workers", "24"],
            Path("/run"),
        )
        self.assertEqual(command[0:2], ["perf", "stat"])
        self.assertIn("--", command)
        self.assertEqual(command[-2:], ["--workers", "24"])

    def test_memory_gate_matches_baseline_policy(self) -> None:
        limit = campaign.parse_memory_size("28G")
        self.assertTrue(campaign.gate_allows(25 * 1024**3, limit))
        self.assertFalse(campaign.gate_allows(26 * 1024**3, limit))


class MetadataTests(unittest.TestCase):
    def test_worktree_provenance_contains_dirty_state_and_diff_digest(self) -> None:
        provenance = campaign.working_tree_provenance(Path(__file__).parents[3])
        self.assertIn("dirty", provenance)
        self.assertIn("status", provenance)
        self.assertEqual(len(provenance["diff_sha256"]), 64)


class CgroupTests(unittest.TestCase):
    def test_oom_kill_event_is_authoritative(self) -> None:
        from profile_runtime import classify_outcome

        self.assertEqual(
            classify_outcome({"Result": "failed", "ExecMainStatus": "9"}, {"oom_kill": 1}),
            "cgroup_oom",
        )

    def test_timeout_is_not_an_oom(self) -> None:
        from profile_runtime import classify_outcome

        self.assertEqual(
            classify_outcome({"Result": "timeout", "ExecMainStatus": "9"}, {}),
            "killed",
        )


class ResultTests(unittest.TestCase):
    def test_candidate_result_is_normalized(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result_path = root / "result.json"
            profile_path = root / "memory.json"
            result_path.write_text(
                json.dumps(
                    {
                        "schema_version": 2,
                        "experiment": "tyler-materialization",
                        "provenance": {"strategy": "json-offsets"},
                        "telemetry": {
                            "requested_vertex_count": 10,
                            "unique_vertex_count": 8,
                            "returned_vertex_count": 8,
                            "persistent_bytes_read": 20,
                            "source_json_bytes_read": 30,
                            "touched_units": 2,
                            "retained_decoded_bytes": 0,
                        },
                        "result": {
                            "package_count": 7,
                            "configured_workers": 4,
                            "model_digest": "sha256:digest",
                            "elapsed_ns": 100,
                            "total_pipeline_elapsed_ns": 200,
                        },
                    }
                ),
                encoding="utf-8",
            )
            profile_path.write_text(
                json.dumps(
                    {
                        "current_rss_bytes": 101,
                        "process_peak_rss_bytes": 202,
                        "peak_rss_bytes": 202,
                    }
                ),
                encoding="utf-8",
            )
            normalized = campaign.parse_candidate_result(
                result_path,
                profile_path,
                expected_workers=4,
                expected_package_count=7,
                expected_provenance={"strategy": "json-offsets"},
            )
            self.assertEqual(normalized["materialization_elapsed_ns"], 100)
            self.assertEqual(normalized["candidate_memory"]["process_peak_rss_bytes"], 202)

    def test_missing_result_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(RuntimeError, "result is missing"):
                campaign.parse_candidate_result(
                    root / "missing.json",
                    root / "memory.json",
                    expected_workers=1,
                    expected_package_count=1,
                )


if __name__ == "__main__":
    unittest.main()
