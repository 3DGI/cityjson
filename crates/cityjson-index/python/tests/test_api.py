from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from cjindex import OpenedIndex
from cjlib import ModelType


REPO_ROOT = Path(__file__).resolve().parents[2]
CITYJSON_DATASET = REPO_ROOT / "tests" / "data" / "cityjson"


class OpenedIndexApiTests(unittest.TestCase):
    def test_cityjson_get_and_read_feature_return_actionable_feature_payloads(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            index_path = Path(tmpdir) / ".cjindex.sqlite"
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


if __name__ == "__main__":
    unittest.main()
