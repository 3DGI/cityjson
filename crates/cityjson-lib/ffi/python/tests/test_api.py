from __future__ import annotations

from pathlib import Path
import unittest

from cityjson_lib import (
    AutoTransformOptions,
    CityModel,
    CityJSONSeqWriteOptions,
    GeometryBoundary,
    GeometryType,
    ModelCapacities,
    ModelType,
    RootKind,
    Transform,
    UV,
    WriteOptions,
    Version,
    Vertex,
    merge_feature_stream_bytes,
    probe_bytes,
    serialize_feature_stream,
    serialize_feature_stream_bytes,
    write_cityjsonseq_auto_transform_bytes,
    write_cityjsonseq_with_transform_bytes,
)


FIXTURE_PATH = Path(__file__).resolve().parents[3] / "tests" / "data" / "v2_0" / "minimal.city.json"


class PythonBindingSmokeTest(unittest.TestCase):
    def test_arrow_round_trip(self) -> None:
        payload = FIXTURE_PATH.read_bytes()

        model = CityModel.parse_document_bytes(payload)
        self.addCleanup(model.close)
        arrow_bytes = model.serialize_arrow_bytes()
        self.assertTrue(arrow_bytes)

        round_trip = CityModel.parse_arrow_bytes(arrow_bytes)
        self.addCleanup(round_trip.close)
        self.assertEqual(round_trip.summary().cityobject_count, 2)
        self.assertEqual(round_trip.cityobject_ids(), ["building-1", "building-part-1"])
        self.assertEqual(
            round_trip.geometry_types(),
            [GeometryType.MULTI_SURFACE, GeometryType.MULTI_POINT],
        )

    def test_parse_edit_extract_and_serialize_document(self) -> None:
        payload = FIXTURE_PATH.read_bytes()

        probe = probe_bytes(payload)
        self.assertEqual(probe.root_kind, RootKind.CITY_JSON)
        self.assertEqual(probe.version, Version.V2_0)
        self.assertTrue(probe.has_version)

        model = CityModel.parse_document_bytes(payload)
        self.addCleanup(model.close)

        summary = model.summary()
        self.assertEqual(summary.model_type, ModelType.CITY_JSON)
        self.assertEqual(summary.cityobject_count, 2)
        self.assertEqual(summary.geometry_count, 2)
        self.assertEqual(summary.vertex_count, 5)
        self.assertEqual(summary.uv_coordinate_count, 4)
        self.assertEqual(summary.material_count, 1)
        self.assertEqual(summary.texture_count, 1)
        self.assertTrue(summary.has_metadata)
        self.assertTrue(summary.has_transform)

        self.assertEqual(model.metadata_title(), "Facade Fixture")
        self.assertEqual(model.metadata_identifier(), "fixture-1")
        self.assertEqual(model.cityobject_ids(), ["building-1", "building-part-1"])
        self.assertEqual(
            model.geometry_types(),
            [GeometryType.MULTI_SURFACE, GeometryType.MULTI_POINT],
        )
        self.assertEqual(
            model.geometry_boundary(0),
            GeometryBoundary(
                geometry_type=GeometryType.MULTI_SURFACE,
                has_boundaries=True,
                vertex_indices=[0, 1, 2, 3, 0],
                ring_offsets=[0],
                surface_offsets=[0],
                shell_offsets=[],
                solid_offsets=[],
            ),
        )
        self.assertEqual(
            model.geometry_boundary_coordinates(0),
            [
                Vertex(10.0, 20.0, 0.0),
                Vertex(11.0, 20.0, 0.0),
                Vertex(11.0, 21.0, 0.0),
                Vertex(10.0, 21.0, 0.0),
                Vertex(10.0, 20.0, 0.0),
            ],
        )
        self.assertEqual(
            model.geometry_boundary(1),
            GeometryBoundary(
                geometry_type=GeometryType.MULTI_POINT,
                has_boundaries=True,
                vertex_indices=[4],
                ring_offsets=[],
                surface_offsets=[],
                shell_offsets=[],
                solid_offsets=[],
            ),
        )
        self.assertEqual(
            model.geometry_boundary_coordinates(1),
            [Vertex(12.0, 22.0, 0.0)],
        )

        model.set_metadata_title("Updated Facade Fixture")
        model.set_metadata_identifier("fixture-1-updated")
        model.set_transform(Transform(scale=(0.5, 0.5, 1.0), translate=(10.0, 20.0, 0.0)))
        model.clear_transform()
        model.add_cityobject("annex-1", "Building")
        annex_vertex = model.add_vertex(Vertex(13.0, 23.0, 0.0))
        annex_geometry = model.add_geometry_from_boundary(
            GeometryBoundary(
                geometry_type=GeometryType.MULTI_POINT,
                has_boundaries=True,
                vertex_indices=[annex_vertex],
                ring_offsets=[],
                surface_offsets=[],
                shell_offsets=[],
                solid_offsets=[],
            )
        )
        model.attach_geometry_to_cityobject("annex-1", annex_geometry)
        model.clear_cityobject_geometry("annex-1")
        model.attach_geometry_to_cityobject("annex-1", annex_geometry)

        extracted = model.extract_cityobjects(["annex-1"])
        self.addCleanup(extracted.close)

        self.assertEqual(extracted.cityobject_ids(), ["annex-1"])
        self.assertEqual(extracted.geometry_types(), [GeometryType.MULTI_POINT])
        self.assertIn("Updated Facade Fixture", extracted.serialize_document(WriteOptions()))

        pretty_document = extracted.serialize_document(WriteOptions(pretty=True))
        self.assertIn("\n", pretty_document)
        self.assertIn("Updated Facade Fixture", pretty_document)

        self.assertIn("fixture-1-updated", model.serialize_document())
        self.assertIn(b"fixture-1-updated", model.serialize_document_bytes())
        self.assertEqual(len(model.uv_coordinates()), 4)
        self.assertIn('"type":"CityJSON"', model.serialize_document())

    def test_append_and_cleanup_workflows(self) -> None:
        model = CityModel.parse_feature_bytes(
            b'{"type":"CityJSONFeature","id":"feature-a","CityObjects":{"feature-a":{"type":"Building"}},"vertices":[]}'
        )
        self.addCleanup(model.close)

        other = CityModel.parse_feature_bytes(
            b'{"type":"CityJSONFeature","id":"feature-b","CityObjects":{"feature-b":{"type":"BuildingPart"}},"vertices":[]}'
        )
        self.addCleanup(other.close)

        removal = CityModel.parse_feature_bytes(
            b'{"type":"CityJSONFeature","id":"remove-me","CityObjects":{"remove-me":{"type":"Building"}},"vertices":[]}'
        )
        self.addCleanup(removal.close)
        removal.add_cityobject("remove-me", "Building")
        self.assertEqual(removal.summary().cityobject_count, 2)
        removal.remove_cityobject("remove-me")
        self.assertEqual(removal.summary().cityobject_count, 1)

        model.set_transform(Transform(scale=(1.0, 1.0, 1.0), translate=(0.0, 0.0, 0.0)))
        other.set_transform(Transform(scale=(1.0, 1.0, 1.0), translate=(0.0, 0.0, 0.0)))

        first_vertex = model.add_vertex(Vertex(1.0, 2.0, 3.0))
        first_geometry = model.add_geometry_from_boundary(
            GeometryBoundary(
                geometry_type=GeometryType.MULTI_POINT,
                has_boundaries=True,
                vertex_indices=[first_vertex],
                ring_offsets=[],
                surface_offsets=[],
                shell_offsets=[],
                solid_offsets=[],
            )
        )
        model.attach_geometry_to_cityobject("feature-a", first_geometry)

        second_vertex = other.add_vertex(Vertex(4.0, 5.0, 6.0))
        second_geometry = other.add_geometry_from_boundary(
            GeometryBoundary(
                geometry_type=GeometryType.MULTI_POINT,
                has_boundaries=True,
                vertex_indices=[second_vertex],
                ring_offsets=[],
                surface_offsets=[],
                shell_offsets=[],
                solid_offsets=[],
            )
        )
        other.attach_geometry_to_cityobject("feature-b", second_geometry)

        model.append_model(other)
        model.cleanup()

        summary = model.summary()
        self.assertEqual(summary.model_type, ModelType.CITY_JSON_FEATURE)
        self.assertEqual(summary.cityobject_count, 2)
        self.assertEqual(summary.geometry_count, 2)
        self.assertEqual(summary.vertex_count, 2)
        self.assertEqual(model.cityobject_ids(), ["feature-a", "feature-b"])
        self.assertIn("feature-a", model.serialize_feature(WriteOptions(pretty=True)))
        self.assertIn(b"feature-a", model.serialize_feature_bytes())

    def test_feature_stream_helpers_round_trip(self) -> None:
        payload = FIXTURE_PATH.read_bytes()
        feature_payload = (
            b'{"type":"CityJSONFeature","id":"feature-1","CityObjects":{"feature-1":{"type":"Building"}},"vertices":[]}'
        )

        feature_model = CityModel.parse_feature_with_base_bytes(feature_payload, payload)
        self.addCleanup(feature_model.close)

        stream = serialize_feature_stream([feature_model], WriteOptions())
        self.assertIn('"type":"CityJSONFeature"', stream)
        stream_bytes = serialize_feature_stream_bytes([feature_model], WriteOptions())
        self.assertIn(b'"type":"CityJSONFeature"', stream_bytes)

        merged = merge_feature_stream_bytes(payload + b"\n" + stream_bytes)
        self.addCleanup(merged.close)
        self.assertIn("feature-1", merged.cityobject_ids())
        self.assertEqual(merged.summary().cityobject_count, 3)

    def test_strict_cityjsonseq_writer_helpers(self) -> None:
        base_root = CityModel.parse_document_bytes(
            b'{"type":"CityJSON","version":"2.0","metadata":{"title":"base-root"},"CityObjects":{},"vertices":[]}'
        )
        self.addCleanup(base_root.close)

        feature_a = CityModel.parse_feature_bytes(
            b'{"type":"CityJSONFeature","id":"feature-a","metadata":{"title":"base-root"},"CityObjects":{"feature-a":{"type":"Building","geometry":[{"type":"MultiPoint","boundaries":[0,1]}]}},"vertices":[[10,20,30],[12,22,31]]}'
        )
        self.addCleanup(feature_a.close)
        feature_b = CityModel.parse_feature_bytes(
            b'{"type":"CityJSONFeature","id":"feature-b","metadata":{"title":"base-root"},"CityObjects":{"feature-b":{"type":"BuildingPart","geometry":[{"type":"MultiPoint","boundaries":[0]}]}},"vertices":[[9,21,40]]}'
        )
        self.addCleanup(feature_b.close)

        explicit = write_cityjsonseq_with_transform_bytes(
            base_root,
            [feature_a],
            Transform(scale=(0.5, 0.5, 1.0), translate=(10.0, 20.0, 30.0)),
            CityJSONSeqWriteOptions(),
        )
        self.assertIn(b'"type":"CityJSON"', explicit)
        self.assertIn(b'"type":"CityJSONFeature"', explicit)
        self.assertIn(b'"geographicalExtent":[10.0,20.0,30.0,12.0,22.0,31.0]', explicit)

        auto = write_cityjsonseq_auto_transform_bytes(
            base_root,
            [feature_a, feature_b],
            AutoTransformOptions(scale=(0.5, 1.0, 5.0)),
        )
        self.assertIn(b'"translate":[9.0,20.0,30.0]', auto)
        self.assertIn(b'"type":"CityJSONFeature"', auto)
