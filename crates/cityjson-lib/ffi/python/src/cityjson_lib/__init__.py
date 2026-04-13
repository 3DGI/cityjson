"""Python bindings for cityjson_lib built on top of the shared C ABI."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Self

from cityjson_lib._ffi import (
    CjlibError,
    CityJSONSeqAutoTransformOptionsPayload,
    CityJSONSeqWriteOptionsPayload,
    FfiLibrary,
    GeometryType,
    GeometryBoundaryPayload,
    ModelCapacitiesStruct,
    ModelType,
    RootKind,
    Status,
    StringViewStruct,
    TransformStruct,
    Version,
    WriteOptionsStruct,
    WriteOptionsPayload,
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
class GeometryBoundary:
    geometry_type: GeometryType
    has_boundaries: bool
    vertex_indices: list[int]
    ring_offsets: list[int]
    surface_offsets: list[int]
    shell_offsets: list[int]
    solid_offsets: list[int]

    def to_native_payload(self) -> GeometryBoundaryPayload:
        return GeometryBoundaryPayload(
            geometry_type=self.geometry_type,
            has_boundaries=self.has_boundaries,
            vertex_indices=self.vertex_indices,
            ring_offsets=self.ring_offsets,
            surface_offsets=self.surface_offsets,
            shell_offsets=self.shell_offsets,
            solid_offsets=self.solid_offsets,
        )


@dataclass(frozen=True)
class WriteOptions:
    pretty: bool = False
    validate_default_themes: bool = True

    def to_native(self) -> WriteOptionsStruct:
        return _ffi.write_options(
            WriteOptionsPayload(
                pretty=self.pretty,
                validate_default_themes=self.validate_default_themes,
            )
        )


@dataclass(frozen=True)
class CityJSONSeqWriteOptions:
    validate_default_themes: bool = True
    trailing_newline: bool = True
    update_metadata_geographical_extent: bool = True

    def to_native(self):
        return _ffi.cityjsonseq_write_options(
            CityJSONSeqWriteOptionsPayload(
                validate_default_themes=self.validate_default_themes,
                trailing_newline=self.trailing_newline,
                update_metadata_geographical_extent=self.update_metadata_geographical_extent,
            )
        )


@dataclass(frozen=True)
class AutoTransformOptions:
    scale: tuple[float, float, float] = (0.001, 0.001, 0.001)
    validate_default_themes: bool = True
    trailing_newline: bool = True
    update_metadata_geographical_extent: bool = True

    def to_native(self):
        return _ffi.cityjsonseq_auto_transform_options(
            CityJSONSeqAutoTransformOptionsPayload(
                scale=self.scale,
                validate_default_themes=self.validate_default_themes,
                trailing_newline=self.trailing_newline,
                update_metadata_geographical_extent=self.update_metadata_geographical_extent,
            )
        )


@dataclass(frozen=True)
class Transform:
    scale: tuple[float, float, float]
    translate: tuple[float, float, float]

    def to_native(self) -> TransformStruct:
        return TransformStruct(
            scale_x=self.scale[0],
            scale_y=self.scale[1],
            scale_z=self.scale[2],
            translate_x=self.translate[0],
            translate_y=self.translate[1],
            translate_z=self.translate[2],
        )


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

    def geometry_boundary(self, index: int) -> GeometryBoundary:
        payload = _ffi.geometry_boundary(self._handle, index)
        return GeometryBoundary(
            geometry_type=payload.geometry_type,
            has_boundaries=payload.has_boundaries,
            vertex_indices=payload.vertex_indices,
            ring_offsets=payload.ring_offsets,
            surface_offsets=payload.surface_offsets,
            shell_offsets=payload.shell_offsets,
            solid_offsets=payload.solid_offsets,
        )

    def geometry_boundary_coordinates(self, index: int) -> list[Vertex]:
        return [
            Vertex(x=item.x, y=item.y, z=item.z)
            for item in _ffi.geometry_boundary_coordinates(self._handle, index)
        ]

    def vertices(self) -> list[Vertex]:
        return [Vertex(x=item.x, y=item.y, z=item.z) for item in _ffi.vertices(self._handle)]

    def template_vertices(self) -> list[Vertex]:
        return [
            Vertex(x=item.x, y=item.y, z=item.z)
            for item in _ffi.template_vertices(self._handle)
        ]

    def uv_coordinates(self) -> list[UV]:
        return [UV(u=item.u, v=item.v) for item in _ffi.uv_coordinates(self._handle)]

    def set_metadata_title(self, title: str) -> None:
        _ffi.set_metadata_title(self._handle, title)

    def set_metadata_identifier(self, identifier: str) -> None:
        _ffi.set_metadata_identifier(self._handle, identifier)

    def set_transform(self, transform: Transform) -> None:
        _ffi.set_transform(self._handle, transform.to_native())

    def clear_transform(self) -> None:
        _ffi.clear_transform(self._handle)

    def add_cityobject(self, cityobject_id: str, cityobject_type: str) -> None:
        _ffi.add_cityobject(self._handle, cityobject_id, cityobject_type)

    def remove_cityobject(self, cityobject_id: str) -> None:
        _ffi.remove_cityobject(self._handle, cityobject_id)

    def attach_geometry_to_cityobject(self, cityobject_id: str, geometry_index: int) -> None:
        _ffi.attach_geometry_to_cityobject(self._handle, cityobject_id, geometry_index)

    def clear_cityobject_geometry(self, cityobject_id: str) -> None:
        _ffi.clear_cityobject_geometry(self._handle, cityobject_id)

    def add_geometry_from_boundary(self, boundary: GeometryBoundary, lod: str | None = None) -> int:
        return _ffi.add_geometry_from_boundary(self._handle, boundary.to_native_payload(), lod)

    def append_model(self, other: Self) -> None:
        _ffi.append_model(self._handle, other._handle)

    def extract_cityobjects(self, cityobject_ids: list[str]) -> Self:
        return type(self)(_ffi.extract_cityobjects(self._handle, cityobject_ids))

    def cleanup(self) -> None:
        _ffi.cleanup(self._handle)

    def serialize_document(self, options: WriteOptions | None = None) -> str:
        return self.serialize_document_bytes(options).decode("utf-8")

    def serialize_document_bytes(self, options: WriteOptions | None = None) -> bytes:
        payload = options.to_native() if options is not None else WriteOptions().to_native()
        return _ffi.serialize_document_with_options(self._handle, payload)

    def serialize_feature(self, options: WriteOptions | None = None) -> str:
        return self.serialize_feature_bytes(options).decode("utf-8")

    def serialize_feature_bytes(self, options: WriteOptions | None = None) -> bytes:
        payload = options.to_native() if options is not None else WriteOptions().to_native()
        return _ffi.serialize_feature_with_options(self._handle, payload)

    def reserve_import(self, capacities: ModelCapacities) -> None:
        _ffi.reserve_import(self._handle, capacities.to_native())

    def add_vertex(self, vertex: Vertex) -> int:
        return _ffi.add_vertex(self._handle, vertex.x, vertex.y, vertex.z)

    def add_template_vertex(self, vertex: Vertex) -> int:
        return _ffi.add_template_vertex(self._handle, vertex.x, vertex.y, vertex.z)

    def add_uv_coordinate(self, uv: UV) -> int:
        return _ffi.add_uv_coordinate(self._handle, uv.u, uv.v)


def merge_feature_stream_bytes(data: bytes | bytearray | memoryview) -> CityModel:
    return CityModel(_ffi.parse_feature_stream_merge(_as_bytes(data)))


def serialize_feature_stream(
    models: list[CityModel],
    options: WriteOptions | None = None,
) -> str:
    return serialize_feature_stream_bytes(models, options).decode("utf-8")


def serialize_feature_stream_bytes(
    models: list[CityModel],
    options: WriteOptions | None = None,
) -> bytes:
    payload = options.to_native() if options is not None else WriteOptions().to_native()
    handles = [model._handle for model in models]
    return _ffi.serialize_feature_stream(handles, payload)


def write_cityjsonseq_with_transform(
    base_root: CityModel,
    features: list[CityModel],
    transform: Transform,
    options: CityJSONSeqWriteOptions | None = None,
) -> str:
    return write_cityjsonseq_with_transform_bytes(
        base_root,
        features,
        transform,
        options,
    ).decode("utf-8")


def write_cityjsonseq_with_transform_bytes(
    base_root: CityModel,
    features: list[CityModel],
    transform: Transform,
    options: CityJSONSeqWriteOptions | None = None,
) -> bytes:
    payload = (
        options.to_native()
        if options is not None
        else CityJSONSeqWriteOptions().to_native()
    )
    handles = [model._handle for model in features]
    return _ffi.serialize_cityjsonseq_with_transform(
        base_root._handle,
        handles,
        transform.to_native(),
        payload,
    )


def write_cityjsonseq_auto_transform(
    base_root: CityModel,
    features: list[CityModel],
    options: AutoTransformOptions | None = None,
) -> str:
    return write_cityjsonseq_auto_transform_bytes(
        base_root,
        features,
        options,
    ).decode("utf-8")


def write_cityjsonseq_auto_transform_bytes(
    base_root: CityModel,
    features: list[CityModel],
    options: AutoTransformOptions | None = None,
) -> bytes:
    payload = (
        options.to_native()
        if options is not None
        else AutoTransformOptions().to_native()
    )
    handles = [model._handle for model in features]
    return _ffi.serialize_cityjsonseq_auto_transform(
        base_root._handle,
        handles,
        payload,
    )
