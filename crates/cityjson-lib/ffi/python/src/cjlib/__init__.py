"""Python bindings for cjlib built on top of the shared C ABI."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Self

from cjlib._ffi import (
    CjlibError,
    FfiLibrary,
    GeometryType,
    ModelCapacitiesStruct,
    ModelType,
    RootKind,
    Status,
    Version,
)

__version__ = "0.1.0"

_ffi = FfiLibrary.load()


def _as_bytes(data: bytes | bytearray | memoryview) -> bytes:
    if isinstance(data, bytes):
        return data
    if isinstance(data, bytearray):
        return bytes(data)
    if isinstance(data, memoryview):
        return data.tobytes()
    raise TypeError("expected bytes-like data")


@dataclass(frozen=True)
class Probe:
    root_kind: RootKind
    version: Version
    has_version: bool


@dataclass(frozen=True)
class Vertex:
    x: float
    y: float
    z: float


@dataclass(frozen=True)
class UV:
    u: float
    v: float


@dataclass(frozen=True)
class ModelCapacities:
    cityobjects: int = 0
    vertices: int = 0
    semantics: int = 0
    materials: int = 0
    textures: int = 0
    geometries: int = 0
    template_vertices: int = 0
    template_geometries: int = 0
    uv_coordinates: int = 0

    def to_native(self) -> ModelCapacitiesStruct:
        return ModelCapacitiesStruct(
            cityobjects=self.cityobjects,
            vertices=self.vertices,
            semantics=self.semantics,
            materials=self.materials,
            textures=self.textures,
            geometries=self.geometries,
            template_vertices=self.template_vertices,
            template_geometries=self.template_geometries,
            uv_coordinates=self.uv_coordinates,
        )


@dataclass(frozen=True)
class ModelSummary:
    model_type: ModelType
    version: Version
    cityobject_count: int
    geometry_count: int
    geometry_template_count: int
    vertex_count: int
    template_vertex_count: int
    uv_coordinate_count: int
    semantic_count: int
    material_count: int
    texture_count: int
    extension_count: int
    has_metadata: bool
    has_transform: bool
    has_templates: bool
    has_appearance: bool


def probe_bytes(data: bytes | bytearray | memoryview) -> Probe:
    native = _ffi.probe(_as_bytes(data))
    return Probe(
        root_kind=RootKind(native.root_kind),
        version=Version(native.version),
        has_version=bool(native.has_version),
    )


class CityModel:
    def __init__(self, handle: int) -> None:
        self._handle = handle

    @classmethod
    def parse_document_bytes(cls, data: bytes | bytearray | memoryview) -> Self:
        return cls(_ffi.parse_document(_as_bytes(data)))

    @classmethod
    def parse_feature_bytes(cls, data: bytes | bytearray | memoryview) -> Self:
        return cls(_ffi.parse_feature(_as_bytes(data)))

    @classmethod
    def parse_feature_with_base_bytes(
        cls,
        feature_data: bytes | bytearray | memoryview,
        base_data: bytes | bytearray | memoryview,
    ) -> Self:
        return cls(_ffi.parse_feature_with_base(_as_bytes(feature_data), _as_bytes(base_data)))

    @classmethod
    def create(cls, *, model_type: ModelType) -> Self:
        return cls(_ffi.create(model_type))

    def close(self) -> None:
        if self._handle != 0:
            _ffi.free_model(self._handle)
            self._handle = 0

    def __del__(self) -> None:
        self.close()

    def summary(self) -> ModelSummary:
        native = _ffi.summary(self._handle)
        return ModelSummary(
            model_type=ModelType(native.model_type),
            version=Version(native.version),
            cityobject_count=native.cityobject_count,
            geometry_count=native.geometry_count,
            geometry_template_count=native.geometry_template_count,
            vertex_count=native.vertex_count,
            template_vertex_count=native.template_vertex_count,
            uv_coordinate_count=native.uv_coordinate_count,
            semantic_count=native.semantic_count,
            material_count=native.material_count,
            texture_count=native.texture_count,
            extension_count=native.extension_count,
            has_metadata=bool(native.has_metadata),
            has_transform=bool(native.has_transform),
            has_templates=bool(native.has_templates),
            has_appearance=bool(native.has_appearance),
        )

    def metadata_title(self) -> str:
        return _ffi.metadata_title(self._handle)

    def metadata_identifier(self) -> str:
        return _ffi.metadata_identifier(self._handle)

    def cityobject_ids(self) -> list[str]:
        count = self.summary().cityobject_count
        return [_ffi.cityobject_id(self._handle, index) for index in range(count)]

    def geometry_types(self) -> list[GeometryType]:
        count = self.summary().geometry_count
        return [_ffi.geometry_type(self._handle, index) for index in range(count)]

    def vertices(self) -> list[Vertex]:
        return [Vertex(x=item.x, y=item.y, z=item.z) for item in _ffi.vertices(self._handle)]

    def template_vertices(self) -> list[Vertex]:
        return [
            Vertex(x=item.x, y=item.y, z=item.z)
            for item in _ffi.template_vertices(self._handle)
        ]

    def uv_coordinates(self) -> list[UV]:
        return [UV(u=item.u, v=item.v) for item in _ffi.uv_coordinates(self._handle)]

    def serialize_document(self) -> str:
        return _ffi.serialize_document(self._handle).decode("utf-8")

    def serialize_feature(self) -> str:
        return _ffi.serialize_feature(self._handle).decode("utf-8")

    def reserve_import(self, capacities: ModelCapacities) -> None:
        _ffi.reserve_import(self._handle, capacities.to_native())

    def add_vertex(self, vertex: Vertex) -> int:
        return _ffi.add_vertex(self._handle, vertex.x, vertex.y, vertex.z)

    def add_template_vertex(self, vertex: Vertex) -> int:
        return _ffi.add_template_vertex(self._handle, vertex.x, vertex.y, vertex.z)

    def add_uv_coordinate(self, uv: UV) -> int:
        return _ffi.add_uv_coordinate(self._handle, uv.u, uv.v)
