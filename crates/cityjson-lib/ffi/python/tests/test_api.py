from __future__ import annotations

import json
from pathlib import Path
import unittest

from cityjson_lib import (
    AffineTransform4x4,
    AutoTransformOptions,
    BBox,
    CityModel,
    CityJSONSeqWriteOptions,
    CityObjectDraft,
    Contact,
    ContactRole,
    ContactType,
    GeometryBoundary,
    GeometryDraft,
    GeometryTemplateId,
    GeometryType,
    ImageType,
    RGBA,
    RGB,
    RingDraft,
    ModelCapacities,
    ModelType,
    RootKind,
    SemanticId,
    ShellDraft,
    SurfaceDraft,
    TextureType,
    Transform,
    UV,
    Value,
    WrapMode,
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
from cityjson_lib._fake_complete import build_fake_complete_model
from cityjson_lib._ffi import CjlibError, Status


FIXTURE_PATH = Path(__file__).resolve().parents[3] / "tests" / "data" / "v2_0" / "minimal.city.json"
FAKE_COMPLETE_FIXTURE_PATH = (
    Path(__file__).resolve().parents[3]
    / "tests"
    / "data"
    / "v2_0"
    / "cityjson_fake_complete.city.json"
)


class PythonBindingSmokeTest(unittest.TestCase):
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
        extracted = model.extract_cityobjects(["building-1"])
        self.addCleanup(extracted.close)

        self.assertEqual(extracted.cityobject_ids(), ["building-1"])
        self.assertEqual(extracted.geometry_types(), [GeometryType.MULTI_SURFACE])
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
            b'{"type":"CityJSONFeature","id":"keep","CityObjects":{"keep":{"type":"Building"},"remove-me":{"type":"Building"}},"vertices":[]}'
        )
        self.addCleanup(removal.close)
        self.assertEqual(removal.summary().cityobject_count, 2)
        removal.remove_cityobject("remove-me")
        self.assertEqual(removal.summary().cityobject_count, 1)

        model.set_transform(Transform(scale=(1.0, 1.0, 1.0), translate=(0.0, 0.0, 0.0)))
        other.set_transform(Transform(scale=(1.0, 1.0, 1.0), translate=(0.0, 0.0, 0.0)))

        model.append_model(other)
        model.cleanup()

        with self.assertRaises(CjlibError) as error:
            model.append_model(model)
        self.assertEqual(error.exception.status, Status.INVALID_ARGUMENT)

        summary = model.summary()
        self.assertEqual(summary.model_type, ModelType.CITY_JSON_FEATURE)
        self.assertEqual(summary.cityobject_count, 2)
        self.assertEqual(summary.geometry_count, 0)
        self.assertEqual(summary.vertex_count, 0)
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

    def test_typed_authoring_api_and_consumption_guards(self) -> None:
        model = CityModel.create(model_type=ModelType.CITY_JSON)
        self.addCleanup(model.close)

        model.reserve_import(
            ModelCapacities(
                cityobjects=2,
                vertices=4,
                semantics=1,
                materials=1,
                textures=1,
                geometries=2,
                template_vertices=4,
                template_geometries=1,
            )
        )
        model.set_metadata_title("Typed Fixture")
        model.set_metadata_identifier("typed-fixture")
        model.set_metadata_geographical_extent(
            BBox(min_x=0.0, min_y=0.0, min_z=0.0, max_x=10.0, max_y=20.0, max_z=30.0)
        )
        model.set_metadata_reference_date("2026-04-18")
        model.set_metadata_reference_system("EPSG:7415")
        contact = (
            Contact()
            .set_name("Author")
            .set_email("author@example.com")
            .set_role(ContactRole.AUTHOR)
            .set_type(ContactType.INDIVIDUAL)
            .set_address(Value.object().insert("city", Value.string("Delft")))
        )
        model.set_metadata_contact(contact)
        model.set_metadata_extra("note", Value.string("typed"))
        model.set_root_extra("+stats", Value.object().insert("count", Value.integer(1)))
        model.add_extension("Noise", "https://example.com/noise.ext.json", "0.1")

        roof = model.add_semantic("RoofSurface")
        self.assertIsInstance(roof, SemanticId)
        model.set_semantic_extra(roof, "surfaceAttribute", Value.boolean(True))

        material = model.add_material("irradiation")
        model.set_material_ambient_intensity(material, 0.2)
        model.set_material_diffuse_color(material, RGB(r=0.2, g=0.3, b=0.4))
        model.set_material_emissive_color(material, RGB(r=0.2, g=0.3, b=0.4))
        model.set_material_specular_color(material, RGB(r=0.2, g=0.3, b=0.4))
        model.set_material_shininess(material, 0.1)
        model.set_material_transparency(material, 0.25)
        model.set_material_is_smooth(material, True)

        texture = model.add_texture("https://example.com/texture.png", ImageType.PNG)
        model.set_texture_wrap_mode(texture, WrapMode.WRAP)
        model.set_texture_type(texture, TextureType.SPECIFIC)
        model.set_texture_border_color(texture, RGBA(r=1.0, g=1.0, b=1.0, a=1.0))
        model.set_default_material_theme("irradiation")
        model.set_default_texture_theme("winter-textures")

        v0 = model.add_vertex(Vertex(x=0.0, y=0.0, z=0.0))
        v1 = model.add_vertex(Vertex(x=1.0, y=0.0, z=0.0))
        v2 = model.add_vertex(Vertex(x=1.0, y=1.0, z=0.0))
        v3 = model.add_vertex(Vertex(x=0.0, y=1.0, z=0.0))

        ring = RingDraft().push_vertex_index(v0).push_vertex_index(v1).push_vertex_index(v2).push_vertex_index(v3)
        surface = SurfaceDraft(ring).set_semantic(roof).add_material("irradiation", material)
        with self.assertRaises(RuntimeError):
            ring.push_vertex_index(v0)

        geometry = GeometryDraft.multi_surface("2.2").add_surface(surface)
        geometry_id = model.add_geometry(geometry)

        location = GeometryDraft.multi_point("1").add_point(v0, roof)
        location_id = model.add_geometry(location)

        template = GeometryDraft.multi_surface("2.1").add_surface(
            SurfaceDraft(
                RingDraft()
                .push_vertex(Vertex(x=0.0, y=0.0, z=0.0))
                .push_vertex(Vertex(x=1.0, y=0.0, z=0.0))
                .push_vertex(Vertex(x=1.0, y=1.0, z=0.0))
                .push_vertex(Vertex(x=0.0, y=1.0, z=0.0))
            )
        )
        template_id = model.add_geometry_template(template)
        self.assertIsInstance(template_id, GeometryTemplateId)

        instance = GeometryDraft.instance(
            template_id,
            v0,
            AffineTransform4x4(
                elements=(
                    1.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                )
            ),
        )
        instance_id = model.add_geometry(instance)

        solid_outer = ShellDraft().add_surface(
            SurfaceDraft(
                RingDraft().push_vertex_index(v0).push_vertex_index(v1).push_vertex_index(v2).push_vertex_index(v3)
            )
        )
        solid = GeometryDraft.solid("1.0").add_solid(solid_outer)
        solid_id = model.add_geometry(solid)

        child_value = Value.string("typed")
        payload = Value.object().insert("name", child_value)
        with self.assertRaises(RuntimeError):
            payload.insert("again", child_value)

        building = CityObjectDraft("building-typed", "Building")
        building.set_attribute("name", Value.string("Typed Building"))
        building.set_extra("location", Value.geometry(location_id))
        building_id = model.add_cityobject(building)
        model.add_cityobject_geometry(building_id, geometry_id)
        model.add_cityobject_geometry(building_id, instance_id)
        model.add_cityobject_geometry(building_id, solid_id)

        parent = model.add_cityobject(CityObjectDraft("parent-typed", "CityObjectGroup"))
        model.add_cityobject_parent(building_id, parent)

        summary = model.summary()
        self.assertEqual(summary.cityobject_count, 2)
        self.assertEqual(summary.geometry_count, 4)
        self.assertEqual(summary.geometry_template_count, 1)
        self.assertEqual(summary.semantic_count, 1)
        self.assertEqual(summary.material_count, 1)
        self.assertEqual(summary.texture_count, 1)
        self.assertEqual(summary.extension_count, 1)

    def test_fake_complete_python_authoring_matches_fixture_structurally(self) -> None:
        model = build_fake_complete_model()
        self.addCleanup(model.close)

        actual = json.loads(
            model.serialize_document(
                WriteOptions(pretty=True, validate_default_themes=False)
            )
        )
        expected = json.loads(FAKE_COMPLETE_FIXTURE_PATH.read_text(encoding="utf-8"))
        self.assertEqual(actual, expected)
