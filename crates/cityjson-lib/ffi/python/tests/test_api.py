from __future__ import annotations

from pathlib import Path
import unittest

from cjlib import (
    CityModel,
    GeometryBoundary,
    GeometryType,
    ModelCapacities,
    ModelType,
    RootKind,
    UV,
    Version,
    Vertex,
    probe_bytes,
)


FIXTURE_PATH = Path(__file__).resolve().parents[3] / "tests" / "data" / "v2_0" / "minimal.city.json"


class PythonBindingSmokeTest(unittest.TestCase):
    def test_parse_inspect_and_serialize_document(self) -> None:
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
        self.assertEqual(len(model.vertices()), 5)
        self.assertEqual(model.vertices()[0].x, 10.0)
        self.assertEqual(model.vertices()[4].y, 22.0)
        self.assertEqual(len(model.uv_coordinates()), 4)
        self.assertIn('"type":"CityJSON"', model.serialize_document())

    def test_create_and_add_vertices(self) -> None:
        model = CityModel.create(model_type=ModelType.CITY_JSON_FEATURE)
        self.addCleanup(model.close)

        capacities = ModelCapacities(vertices=2, template_vertices=1, uv_coordinates=1)
        model.reserve_import(capacities)

        self.assertEqual(model.add_vertex(Vertex(1.0, 2.0, 3.0)), 0)
        self.assertEqual(model.add_template_vertex(Vertex(4.0, 5.0, 6.0)), 0)
        self.assertEqual(model.add_uv_coordinate(UV(0.25, 0.75)), 0)

        summary = model.summary()
        self.assertEqual(summary.model_type, ModelType.CITY_JSON_FEATURE)
        self.assertEqual(summary.vertex_count, 1)
        self.assertEqual(summary.template_vertex_count, 1)
        self.assertEqual(summary.uv_coordinate_count, 1)
