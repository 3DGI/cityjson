from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from cityjson_index import FeatureFilter, FeatureFilterSummary, LodSelection, OpenedIndex
from cityjson_lib import ModelType


REPO_ROOT = Path(__file__).resolve().parents[3]
CITYJSON_DATASET = REPO_ROOT / "tests" / "data" / "cityjson"


class OpenedIndexApiTests(unittest.TestCase):
    def test_cityjson_get_and_read_feature_return_actionable_feature_payloads(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            index_path = Path(tmpdir) / ".cityjson_index.sqlite"
            with OpenedIndex.open(CITYJSON_DATASET, index_path) as index:
                index.reindex()
                refs = index.feature_ref_page(0, 1)

                self.assertTrue(refs)
                ref = refs[0]

                by_id = index.get(ref.feature_id)
                self.assertIsNotNone(by_id)
                by_ref = index.read_feature(ref)
                self.assertEqual(by_id.summary().model_type, ModelType.CITY_JSON_FEATURE)
                self.assertEqual(by_ref.summary().model_type, ModelType.CITY_JSON_FEATURE)
                self.assertTrue(by_id.summary().has_transform)
                self.assertTrue(by_ref.summary().has_transform)
                self.assertIn(ref.feature_id, by_id.cityobject_ids())

                by_id_payload = index.get_json(ref.feature_id)
                by_ref_payload = index.read_feature_json(ref)

                self.assertEqual(by_id_payload["type"], "CityJSONFeature")
                self.assertEqual(by_ref_payload["type"], "CityJSONFeature")
                self.assertIn("transform", by_id_payload)
                self.assertIn("transform", by_ref_payload)
                self.assertIn("metadata", by_id_payload)
                self.assertIn("metadata", by_ref_payload)
                self.assertIn(ref.feature_id, by_id_payload["CityObjects"])
                self.assertEqual(by_id_payload, by_ref_payload)

    def test_read_filtered_features_reports_diagnostics(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            index_path = Path(tmpdir) / ".cityjson_index.sqlite"
            with OpenedIndex.open(CITYJSON_DATASET, index_path) as index:
                index.reindex()
                refs = index.feature_ref_page(0, 2)

                filter = FeatureFilter(
                    cityobject_types={"Building"},
                    default_lod=LodSelection.HIGHEST,
                )
                filtered = index.read_filtered_features(refs, filter)

                self.assertEqual(len(filtered), len(refs))
                self.assertTrue(filtered)
                self.assertEqual(filtered[0].model.summary().model_type, ModelType.CITY_JSON_FEATURE)
                self.assertIn("Building", filtered[0].diagnostics.available_types)
                self.assertIn("Building", filtered[0].diagnostics.retained_types)
                self.assertEqual(filtered[0].diagnostics.retained_lods["Building"], {"1.0"})

                single = index.read_filtered_feature(refs[0], filter)
                self.assertEqual(single.model.cityobject_ids(), filtered[0].model.cityobject_ids())
                self.assertEqual(single.diagnostics, filtered[0].diagnostics)

    def test_filter_summary_reports_missing_requested_lods(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            index_path = Path(tmpdir) / ".cityjson_index.sqlite"
            with OpenedIndex.open(CITYJSON_DATASET, index_path) as index:
                index.reindex()
                refs = index.feature_ref_page(0, 2)

                filter = FeatureFilter(
                    cityobject_types={"Building"},
                    default_lod=LodSelection.HIGHEST,
                    lods_by_type={"Building": LodSelection.Exact("2.0")},
                )
                filtered = index.read_filtered_features(refs, filter)
                summary = FeatureFilterSummary()
                for feature in filtered:
                    summary.add(feature.diagnostics)

                self.assertEqual(summary.available_lods["Building"], {"1.0"})
                self.assertEqual(summary.retained_feature_count, 0)
                self.assertEqual(summary.ignored_feature_count, len(refs))

                failures = summary.requested_lod_failures(filter)
                self.assertEqual(len(failures), 1)
                self.assertEqual(failures[0].cityobject_type, "Building")
                self.assertEqual(failures[0].requested_lod, "2.0")
                self.assertEqual(failures[0].available_lods, {"1.0"})
                self.assertEqual(filtered[0].diagnostics.missing_lods, failures)

                with self.assertRaisesRegex(RuntimeError, "requested LoD selector matched no geometry"):
                    summary.ensure_requested_lods_available(filter)


if __name__ == "__main__":
    unittest.main()
