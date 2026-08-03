"""Unit tests for stable Heaptrack artifact analysis."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("analyze_profile.py")
SPEC = importlib.util.spec_from_file_location("analyze_profile", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
analyze_profile = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(analyze_profile)


class FoldedStackTests(unittest.TestCase):
    def test_peak_costs_are_partitioned_without_double_counting(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            stacks = Path(directory) / "peak.stacks"
            stacks.write_text(
                "main;read_package;load_shared_vertices;parse_vertices_fragment 100\n"
                "main;read_package;parse_cityobject_entry 40\n"
                "main;package_ref_page_after_record_id;sqlite3_step 20\n"
                "main;rayon_core 10\n"
                "main;unclassified 5\n",
                encoding="utf-8",
            )

            total, categories = analyze_profile.parse_folded_stacks(stacks)

            self.assertEqual(total, 175)
            self.assertEqual(categories["vertex_cache"], 100)
            self.assertEqual(categories["package_reconstruction"], 40)
            self.assertEqual(categories["reference_preload"], 20)
            self.assertEqual(categories["rayon_runtime"], 10)
            self.assertEqual(categories["other"], 5)
            self.assertEqual(sum(categories.values()), total)

    def test_peak_cgroup_sample_uses_current_memory(self) -> None:
        samples = [
            {"memory_current_bytes": 100, "memory_peak_bytes": 500},
            {"memory_current_bytes": 300, "memory_peak_bytes": 400},
        ]
        self.assertEqual(
            analyze_profile.peak_cgroup_sample(samples),
            samples[1],
        )


if __name__ == "__main__":
    unittest.main()
