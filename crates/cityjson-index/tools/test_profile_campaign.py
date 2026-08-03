"""Unit tests for the staged Tyler profiling campaign."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("profile_campaign.py")
SPEC = importlib.util.spec_from_file_location("profile_campaign", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
profile_campaign = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(profile_campaign)


class MemoryGateTests(unittest.TestCase):
    def test_parses_binary_systemd_memory_sizes(self) -> None:
        self.assertEqual(profile_campaign.parse_memory_size("28G"), 28 * 1024**3)
        self.assertEqual(profile_campaign.parse_memory_size("1024"), 1024)

    def test_projects_worker_duplication_from_observed_slope(self) -> None:
        one_gib = 1024**3
        self.assertEqual(
            profile_campaign.projected_twenty_four_worker_peak(2 * one_gib, 5 * one_gib),
            25 * one_gib,
        )

    def test_gate_reserves_ten_percent_of_the_cgroup_limit(self) -> None:
        limit = profile_campaign.parse_memory_size("28G")
        self.assertTrue(profile_campaign.gate_allows(25 * 1024**3, limit))
        self.assertFalse(profile_campaign.gate_allows(26 * 1024**3, limit))


if __name__ == "__main__":
    unittest.main()
