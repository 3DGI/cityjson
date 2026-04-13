"""Low-level ctypes bridge for the shared cityjson_lib C ABI."""

from __future__ import annotations

from dataclasses import dataclass
from ctypes import (
    CDLL,
    POINTER,
    Structure,
    c_bool,
    c_double,
    c_float,
    c_int,
    c_size_t,
    c_ubyte,
    c_void_p,
    pointer,
    string_at,
)
from enum import IntEnum
import os
from pathlib import Path


class Status(IntEnum):
    SUCCESS = 0
    INVALID_ARGUMENT = 1
    IO = 2
    SYNTAX = 3
    VERSION = 4
    SHAPE = 5
    UNSUPPORTED = 6
    MODEL = 7
    INTERNAL = 8


class ErrorKind(IntEnum):
    NONE = 0
    INVALID_ARGUMENT = 1
    IO = 2
    SYNTAX = 3
    VERSION = 4
    SHAPE = 5
    UNSUPPORTED = 6
    MODEL = 7
    INTERNAL = 8


class RootKind(IntEnum):
    CITY_JSON = 0
    CITY_JSON_FEATURE = 1


class Version(IntEnum):
    UNKNOWN = 0
    V1_0 = 1
    V1_1 = 2
    V2_0 = 3


class ModelType(IntEnum):
    CITY_JSON = 0
    CITY_JSON_FEATURE = 1


class GeometryType(IntEnum):
    MULTI_POINT = 0
    MULTI_LINE_STRING = 1
    MULTI_SURFACE = 2
    COMPOSITE_SURFACE = 3
    SOLID = 4
    MULTI_SOLID = 5
    COMPOSITE_SOLID = 6
    GEOMETRY_INSTANCE = 7


class CjlibError(RuntimeError):
    def __init__(self, status: Status, kind: ErrorKind, message: str) -> None:
        super().__init__(message)
        self.status = status
        self.kind = kind


class BytesStruct(Structure):
    _fields_ = [("data", POINTER(c_ubyte)), ("len", c_size_t)]


class VertexStruct(Structure):
    _fields_ = [("x", c_double), ("y", c_double), ("z", c_double)]


class UVStruct(Structure):
    _fields_ = [("u", c_float), ("v", c_float)]


class VerticesStruct(Structure):
    _fields_ = [("data", POINTER(VertexStruct)), ("len", c_size_t)]


class UVsStruct(Structure):
    _fields_ = [("data", POINTER(UVStruct)), ("len", c_size_t)]


class IndicesStruct(Structure):
    _fields_ = [("data", POINTER(c_size_t)), ("len", c_size_t)]


class GeometryBoundaryStruct(Structure):
    _fields_ = [
        ("geometry_type", c_int),
        ("has_boundaries", c_bool),
        ("vertex_indices", IndicesStruct),
        ("ring_offsets", IndicesStruct),
        ("surface_offsets", IndicesStruct),
        ("shell_offsets", IndicesStruct),
        ("solid_offsets", IndicesStruct),
    ]


class StringViewStruct(Structure):
    _fields_ = [("data", POINTER(c_ubyte)), ("len", c_size_t)]


class IndicesViewStruct(Structure):
    _fields_ = [("data", POINTER(c_size_t)), ("len", c_size_t)]


class GeometryBoundaryViewStruct(Structure):
    _fields_ = [
        ("geometry_type", c_int),
        ("vertex_indices", IndicesViewStruct),
        ("ring_offsets", IndicesViewStruct),
        ("surface_offsets", IndicesViewStruct),
        ("shell_offsets", IndicesViewStruct),
        ("solid_offsets", IndicesViewStruct),
    ]


class WriteOptionsStruct(Structure):
    _fields_ = [("pretty", c_bool), ("validate_default_themes", c_bool)]


class CityJSONSeqWriteOptionsStruct(Structure):
    _fields_ = [
        ("validate_default_themes", c_bool),
        ("trailing_newline", c_bool),
        ("update_metadata_geographical_extent", c_bool),
    ]


class CityJSONSeqAutoTransformOptionsStruct(Structure):
    _fields_ = [
        ("scale_x", c_double),
        ("scale_y", c_double),
        ("scale_z", c_double),
        ("validate_default_themes", c_bool),
        ("trailing_newline", c_bool),
        ("update_metadata_geographical_extent", c_bool),
    ]


class TransformStruct(Structure):
    _fields_ = [
        ("scale_x", c_double),
        ("scale_y", c_double),
        ("scale_z", c_double),
        ("translate_x", c_double),
        ("translate_y", c_double),
        ("translate_z", c_double),
    ]


class ProbeStruct(Structure):
    _fields_ = [("root_kind", c_int), ("version", c_int), ("has_version", c_bool)]


class ModelSummaryStruct(Structure):
    _fields_ = [
        ("model_type", c_int),
        ("version", c_int),
        ("cityobject_count", c_size_t),
        ("geometry_count", c_size_t),
        ("geometry_template_count", c_size_t),
        ("vertex_count", c_size_t),
        ("template_vertex_count", c_size_t),
        ("uv_coordinate_count", c_size_t),
        ("semantic_count", c_size_t),
        ("material_count", c_size_t),
        ("texture_count", c_size_t),
        ("extension_count", c_size_t),
        ("has_metadata", c_bool),
        ("has_transform", c_bool),
        ("has_templates", c_bool),
        ("has_appearance", c_bool),
    ]


class ModelCapacitiesStruct(Structure):
    _fields_ = [
        ("cityobjects", c_size_t),
        ("vertices", c_size_t),
        ("semantics", c_size_t),
        ("materials", c_size_t),
        ("textures", c_size_t),
        ("geometries", c_size_t),
        ("template_vertices", c_size_t),
        ("template_geometries", c_size_t),
        ("uv_coordinates", c_size_t),
    ]


@dataclass(frozen=True)
class GeometryBoundaryPayload:
    geometry_type: GeometryType
    has_boundaries: bool
    vertex_indices: list[int]
    ring_offsets: list[int]
    surface_offsets: list[int]
    shell_offsets: list[int]
    solid_offsets: list[int]


@dataclass(frozen=True)
class WriteOptionsPayload:
    pretty: bool = False
    validate_default_themes: bool = False


@dataclass(frozen=True)
class CityJSONSeqWriteOptionsPayload:
    validate_default_themes: bool = True
    trailing_newline: bool = True
    update_metadata_geographical_extent: bool = True


@dataclass(frozen=True)
class CityJSONSeqAutoTransformOptionsPayload:
    scale: tuple[float, float, float] = (0.001, 0.001, 0.001)
    validate_default_themes: bool = True
    trailing_newline: bool = True
    update_metadata_geographical_extent: bool = True


def _candidate_library_paths() -> list[Path]:
    package_dir = Path(__file__).resolve().parent
    names = ["libcityjson_lib_ffi_core.so", "libcityjson_lib_ffi_core.dylib", "cityjson_lib_ffi_core.dll"]

    candidates: list[Path] = []
    if "CITYJSON_LIB_FFI_CORE_LIB" in os.environ:
        candidates.append(Path(os.environ["CITYJSON_LIB_FFI_CORE_LIB"]))

    for name in names:
        candidates.append(package_dir / name)
        if len(package_dir.parents) > 3:
            repo_root = package_dir.parents[3]
            candidates.append(repo_root / "target" / "release" / name)
            candidates.append(repo_root / "target" / "debug" / name)

    unique_candidates: list[Path] = []
    for candidate in candidates:
        if candidate not in unique_candidates:
            unique_candidates.append(candidate)

    return unique_candidates


def _load_cdll() -> CDLL:
    for candidate in _candidate_library_paths():
        if candidate.exists():
            return CDLL(str(candidate))

    searched = ", ".join(str(candidate) for candidate in _candidate_library_paths())
    raise FileNotFoundError(f"could not locate cityjson_lib ffi shared library; searched: {searched}")


class FfiLibrary:
    def __init__(self, library: CDLL) -> None:
        self._lib = library
        self._configure()

    @classmethod
    def load(cls) -> "FfiLibrary":
        return cls(_load_cdll())

    def _configure(self) -> None:
        self._lib.cj_last_error_kind.argtypes = []
        self._lib.cj_last_error_kind.restype = c_int
        self._lib.cj_last_error_message_len.argtypes = []
        self._lib.cj_last_error_message_len.restype = c_size_t
        self._lib.cj_last_error_message_copy.argtypes = [POINTER(c_ubyte), c_size_t, POINTER(c_size_t)]
        self._lib.cj_last_error_message_copy.restype = c_int
        self._lib.cj_clear_error.argtypes = []
        self._lib.cj_clear_error.restype = c_int

        self._lib.cj_probe_bytes.argtypes = [POINTER(c_ubyte), c_size_t, POINTER(ProbeStruct)]
        self._lib.cj_probe_bytes.restype = c_int

        self._lib.cj_model_parse_document_bytes.argtypes = [POINTER(c_ubyte), c_size_t, POINTER(c_void_p)]
        self._lib.cj_model_parse_document_bytes.restype = c_int
        self._lib.cj_model_parse_feature_bytes.argtypes = [POINTER(c_ubyte), c_size_t, POINTER(c_void_p)]
        self._lib.cj_model_parse_feature_bytes.restype = c_int
        self._lib.cj_model_parse_feature_with_base_bytes.argtypes = [
            POINTER(c_ubyte),
            c_size_t,
            POINTER(c_ubyte),
            c_size_t,
            POINTER(c_void_p),
        ]
        self._lib.cj_model_parse_feature_with_base_bytes.restype = c_int
        self._lib.cj_model_create.argtypes = [c_int, POINTER(c_void_p)]
        self._lib.cj_model_create.restype = c_int
        self._lib.cj_model_free.argtypes = [c_void_p]
        self._lib.cj_model_free.restype = c_int

        self._lib.cj_model_serialize_document.argtypes = [c_void_p, POINTER(BytesStruct)]
        self._lib.cj_model_serialize_document.restype = c_int
        self._lib.cj_model_serialize_feature.argtypes = [c_void_p, POINTER(BytesStruct)]
        self._lib.cj_model_serialize_feature.restype = c_int
        self._lib.cj_bytes_free.argtypes = [BytesStruct]
        self._lib.cj_bytes_free.restype = c_int

        self._lib.cj_model_get_summary.argtypes = [c_void_p, POINTER(ModelSummaryStruct)]
        self._lib.cj_model_get_summary.restype = c_int
        self._lib.cj_model_get_metadata_title.argtypes = [c_void_p, POINTER(BytesStruct)]
        self._lib.cj_model_get_metadata_title.restype = c_int
        self._lib.cj_model_get_metadata_identifier.argtypes = [c_void_p, POINTER(BytesStruct)]
        self._lib.cj_model_get_metadata_identifier.restype = c_int
        self._lib.cj_model_get_cityobject_id.argtypes = [c_void_p, c_size_t, POINTER(BytesStruct)]
        self._lib.cj_model_get_cityobject_id.restype = c_int
        self._lib.cj_model_get_geometry_type.argtypes = [c_void_p, c_size_t, POINTER(c_int)]
        self._lib.cj_model_get_geometry_type.restype = c_int

        self._lib.cj_model_copy_geometry_boundary.argtypes = [
            c_void_p,
            c_size_t,
            POINTER(GeometryBoundaryStruct),
        ]
        self._lib.cj_model_copy_geometry_boundary.restype = c_int
        self._lib.cj_model_copy_geometry_boundary_coordinates.argtypes = [
            c_void_p,
            c_size_t,
            POINTER(VerticesStruct),
        ]
        self._lib.cj_model_copy_geometry_boundary_coordinates.restype = c_int

        self._lib.cj_model_copy_vertices.argtypes = [c_void_p, POINTER(VerticesStruct)]
        self._lib.cj_model_copy_vertices.restype = c_int
        self._lib.cj_model_copy_template_vertices.argtypes = [c_void_p, POINTER(VerticesStruct)]
        self._lib.cj_model_copy_template_vertices.restype = c_int
        self._lib.cj_vertices_free.argtypes = [VerticesStruct]
        self._lib.cj_vertices_free.restype = c_int

        self._lib.cj_model_copy_uv_coordinates.argtypes = [c_void_p, POINTER(UVsStruct)]
        self._lib.cj_model_copy_uv_coordinates.restype = c_int
        self._lib.cj_uvs_free.argtypes = [UVsStruct]
        self._lib.cj_uvs_free.restype = c_int

        self._lib.cj_indices_free.argtypes = [IndicesStruct]
        self._lib.cj_indices_free.restype = c_int
        self._lib.cj_geometry_boundary_free.argtypes = [GeometryBoundaryStruct]
        self._lib.cj_geometry_boundary_free.restype = c_int

        self._lib.cj_model_reserve_import.argtypes = [c_void_p, ModelCapacitiesStruct]
        self._lib.cj_model_reserve_import.restype = c_int
        self._lib.cj_model_add_vertex.argtypes = [c_void_p, VertexStruct, POINTER(c_size_t)]
        self._lib.cj_model_add_vertex.restype = c_int
        self._lib.cj_model_add_template_vertex.argtypes = [c_void_p, VertexStruct, POINTER(c_size_t)]
        self._lib.cj_model_add_template_vertex.restype = c_int
        self._lib.cj_model_add_uv_coordinate.argtypes = [c_void_p, UVStruct, POINTER(c_size_t)]
        self._lib.cj_model_add_uv_coordinate.restype = c_int

        self._lib.cj_model_set_metadata_title.argtypes = [c_void_p, StringViewStruct]
        self._lib.cj_model_set_metadata_title.restype = c_int
        self._lib.cj_model_set_metadata_identifier.argtypes = [c_void_p, StringViewStruct]
        self._lib.cj_model_set_metadata_identifier.restype = c_int
        self._lib.cj_model_set_transform.argtypes = [c_void_p, TransformStruct]
        self._lib.cj_model_set_transform.restype = c_int
        self._lib.cj_model_clear_transform.argtypes = [c_void_p]
        self._lib.cj_model_clear_transform.restype = c_int

        self._lib.cj_model_add_cityobject.argtypes = [c_void_p, StringViewStruct, StringViewStruct]
        self._lib.cj_model_add_cityobject.restype = c_int
        self._lib.cj_model_remove_cityobject.argtypes = [c_void_p, StringViewStruct]
        self._lib.cj_model_remove_cityobject.restype = c_int
        self._lib.cj_model_attach_geometry_to_cityobject.argtypes = [
            c_void_p,
            StringViewStruct,
            c_size_t,
        ]
        self._lib.cj_model_attach_geometry_to_cityobject.restype = c_int
        self._lib.cj_model_clear_cityobject_geometry.argtypes = [c_void_p, StringViewStruct]
        self._lib.cj_model_clear_cityobject_geometry.restype = c_int

        self._lib.cj_model_add_geometry_from_boundary.argtypes = [
            c_void_p,
            GeometryBoundaryViewStruct,
            StringViewStruct,
            POINTER(c_size_t),
        ]
        self._lib.cj_model_add_geometry_from_boundary.restype = c_int
        self._lib.cj_model_cleanup.argtypes = [c_void_p]
        self._lib.cj_model_cleanup.restype = c_int
        self._lib.cj_model_append_model.argtypes = [c_void_p, c_void_p]
        self._lib.cj_model_append_model.restype = c_int
        self._lib.cj_model_extract_cityobjects.argtypes = [
            c_void_p,
            POINTER(StringViewStruct),
            c_size_t,
            POINTER(c_void_p),
        ]
        self._lib.cj_model_extract_cityobjects.restype = c_int

        self._lib.cj_model_serialize_document_with_options.argtypes = [
            c_void_p,
            WriteOptionsStruct,
            POINTER(BytesStruct),
        ]
        self._lib.cj_model_serialize_document_with_options.restype = c_int
        self._lib.cj_model_serialize_feature_with_options.argtypes = [
            c_void_p,
            WriteOptionsStruct,
            POINTER(BytesStruct),
        ]
        self._lib.cj_model_serialize_feature_with_options.restype = c_int
        self._lib.cj_model_parse_feature_stream_merge_bytes.argtypes = [
            POINTER(c_ubyte),
            c_size_t,
            POINTER(c_void_p),
        ]
        self._lib.cj_model_parse_feature_stream_merge_bytes.restype = c_int
        self._lib.cj_model_serialize_feature_stream.argtypes = [
            POINTER(c_void_p),
            c_size_t,
            WriteOptionsStruct,
            POINTER(BytesStruct),
        ]
        self._lib.cj_model_serialize_feature_stream.restype = c_int
        self._lib.cj_model_serialize_cityjsonseq_with_transform.argtypes = [
            c_void_p,
            POINTER(c_void_p),
            c_size_t,
            TransformStruct,
            CityJSONSeqWriteOptionsStruct,
            POINTER(BytesStruct),
        ]
        self._lib.cj_model_serialize_cityjsonseq_with_transform.restype = c_int
        self._lib.cj_model_serialize_cityjsonseq_auto_transform.argtypes = [
            c_void_p,
            POINTER(c_void_p),
            c_size_t,
            CityJSONSeqAutoTransformOptionsStruct,
            POINTER(BytesStruct),
        ]
        self._lib.cj_model_serialize_cityjsonseq_auto_transform.restype = c_int

    def _raise_if_error(self, raw_status: int) -> None:
        status = Status(raw_status)
        if status is Status.SUCCESS:
            return

        length = self._lib.cj_last_error_message_len()
        message = ""
        if length > 0:
            buffer = (c_ubyte * (length + 1))()
            copied = c_size_t(0)
            copy_status = Status(
                self._lib.cj_last_error_message_copy(buffer, len(buffer), pointer(copied))
            )
            if copy_status is Status.SUCCESS:
                message = bytes(buffer[: copied.value]).decode("utf-8")

        raise CjlibError(status, ErrorKind(self._lib.cj_last_error_kind()), message)

    def _data_pointer(self, data: bytes) -> POINTER(c_ubyte):
        if not data:
            return POINTER(c_ubyte)()

        array_type = c_ubyte * len(data)
        buffer = array_type.from_buffer_copy(data)
        return buffer

    def _string_view(self, data: str) -> tuple[StringViewStruct, object]:
        encoded = data.encode("utf-8")
        if not encoded:
            return StringViewStruct(), b""

        array_type = c_ubyte * len(encoded)
        buffer = array_type.from_buffer_copy(encoded)
        return StringViewStruct(buffer, len(encoded)), buffer

    def _indices_view(self, values: list[int]) -> tuple[IndicesViewStruct, object]:
        if not values:
            return IndicesViewStruct(), ()

        array_type = c_size_t * len(values)
        buffer = array_type(*values)
        return IndicesViewStruct(buffer, len(values)), buffer

    def _geometry_boundary_view(
        self, payload: GeometryBoundaryPayload
    ) -> tuple[GeometryBoundaryViewStruct, list[object]]:
        vertex_indices, vertex_buffer = self._indices_view(payload.vertex_indices)
        ring_offsets, ring_buffer = self._indices_view(payload.ring_offsets)
        surface_offsets, surface_buffer = self._indices_view(payload.surface_offsets)
        shell_offsets, shell_buffer = self._indices_view(payload.shell_offsets)
        solid_offsets, solid_buffer = self._indices_view(payload.solid_offsets)
        return (
            GeometryBoundaryViewStruct(
                geometry_type=int(payload.geometry_type),
                vertex_indices=vertex_indices,
                ring_offsets=ring_offsets,
                surface_offsets=surface_offsets,
                shell_offsets=shell_offsets,
                solid_offsets=solid_offsets,
            ),
            [vertex_buffer, ring_buffer, surface_buffer, shell_buffer, solid_buffer],
        )

    def _write_options(self, options: WriteOptionsPayload) -> WriteOptionsStruct:
        return WriteOptionsStruct(
            pretty=options.pretty,
            validate_default_themes=options.validate_default_themes,
        )

    def _cityjsonseq_write_options(
        self, options: CityJSONSeqWriteOptionsPayload
    ) -> CityJSONSeqWriteOptionsStruct:
        return CityJSONSeqWriteOptionsStruct(
            validate_default_themes=options.validate_default_themes,
            trailing_newline=options.trailing_newline,
            update_metadata_geographical_extent=options.update_metadata_geographical_extent,
        )

    def _cityjsonseq_auto_transform_options(
        self, options: CityJSONSeqAutoTransformOptionsPayload
    ) -> CityJSONSeqAutoTransformOptionsStruct:
        return CityJSONSeqAutoTransformOptionsStruct(
            scale_x=options.scale[0],
            scale_y=options.scale[1],
            scale_z=options.scale[2],
            validate_default_themes=options.validate_default_themes,
            trailing_newline=options.trailing_newline,
            update_metadata_geographical_extent=options.update_metadata_geographical_extent,
        )

    def _take_bytes(self, payload: BytesStruct) -> bytes:
        if payload.len == 0:
            self._raise_if_error(self._lib.cj_bytes_free(payload))
            return b""

        data = string_at(payload.data, payload.len)
        self._raise_if_error(self._lib.cj_bytes_free(payload))
        return data

    def _take_vertices(self, payload: VerticesStruct) -> list[VertexStruct]:
        if payload.len == 0:
            self._raise_if_error(self._lib.cj_vertices_free(payload))
            return []

        values = [
            VertexStruct(
                x=payload.data[index].x,
                y=payload.data[index].y,
                z=payload.data[index].z,
            )
            for index in range(payload.len)
        ]
        self._raise_if_error(self._lib.cj_vertices_free(payload))
        return values

    def _take_uvs(self, payload: UVsStruct) -> list[UVStruct]:
        if payload.len == 0:
            self._raise_if_error(self._lib.cj_uvs_free(payload))
            return []

        values = [
            UVStruct(u=payload.data[index].u, v=payload.data[index].v)
            for index in range(payload.len)
        ]
        self._raise_if_error(self._lib.cj_uvs_free(payload))
        return values

    def _copy_indices(self, payload: IndicesStruct) -> list[int]:
        values = [payload.data[index] for index in range(payload.len)]
        return values

    def _take_geometry_boundary(self, payload: GeometryBoundaryStruct) -> GeometryBoundaryPayload:
        boundary = GeometryBoundaryPayload(
            geometry_type=GeometryType(payload.geometry_type),
            has_boundaries=bool(payload.has_boundaries),
            vertex_indices=self._copy_indices(payload.vertex_indices),
            ring_offsets=self._copy_indices(payload.ring_offsets),
            surface_offsets=self._copy_indices(payload.surface_offsets),
            shell_offsets=self._copy_indices(payload.shell_offsets),
            solid_offsets=self._copy_indices(payload.solid_offsets),
        )
        self._raise_if_error(self._lib.cj_geometry_boundary_free(payload))
        return boundary

    def probe(self, data: bytes) -> ProbeStruct:
        probe = ProbeStruct()
        pointer_data = self._data_pointer(data)
        self._raise_if_error(self._lib.cj_probe_bytes(pointer_data, len(data), pointer(probe)))
        return probe

    def parse_document(self, data: bytes) -> int:
        handle = c_void_p()
        pointer_data = self._data_pointer(data)
        self._raise_if_error(
            self._lib.cj_model_parse_document_bytes(pointer_data, len(data), pointer(handle))
        )
        return int(handle.value)

    def parse_feature(self, data: bytes) -> int:
        handle = c_void_p()
        pointer_data = self._data_pointer(data)
        self._raise_if_error(
            self._lib.cj_model_parse_feature_bytes(pointer_data, len(data), pointer(handle))
        )
        return int(handle.value)

    def parse_feature_with_base(self, feature_data: bytes, base_data: bytes) -> int:
        handle = c_void_p()
        feature_pointer = self._data_pointer(feature_data)
        base_pointer = self._data_pointer(base_data)
        self._raise_if_error(
            self._lib.cj_model_parse_feature_with_base_bytes(
                feature_pointer,
                len(feature_data),
                base_pointer,
                len(base_data),
                pointer(handle),
            )
        )
        return int(handle.value)

    def create(self, model_type: ModelType) -> int:
        handle = c_void_p()
        self._raise_if_error(self._lib.cj_model_create(int(model_type), pointer(handle)))
        return int(handle.value)

    def free_model(self, handle: int) -> None:
        self._raise_if_error(self._lib.cj_model_free(c_void_p(handle)))

    def serialize_document(self, handle: int) -> bytes:
        payload = BytesStruct()
        self._raise_if_error(self._lib.cj_model_serialize_document(c_void_p(handle), pointer(payload)))
        return self._take_bytes(payload)

    def serialize_feature(self, handle: int) -> bytes:
        payload = BytesStruct()
        self._raise_if_error(self._lib.cj_model_serialize_feature(c_void_p(handle), pointer(payload)))
        return self._take_bytes(payload)

    def summary(self, handle: int) -> ModelSummaryStruct:
        summary = ModelSummaryStruct()
        self._raise_if_error(self._lib.cj_model_get_summary(c_void_p(handle), pointer(summary)))
        return summary

    def metadata_title(self, handle: int) -> str:
        payload = BytesStruct()
        self._raise_if_error(self._lib.cj_model_get_metadata_title(c_void_p(handle), pointer(payload)))
        return self._take_bytes(payload).decode("utf-8")

    def metadata_identifier(self, handle: int) -> str:
        payload = BytesStruct()
        self._raise_if_error(
            self._lib.cj_model_get_metadata_identifier(c_void_p(handle), pointer(payload))
        )
        return self._take_bytes(payload).decode("utf-8")

    def cityobject_id(self, handle: int, index: int) -> str:
        payload = BytesStruct()
        self._raise_if_error(
            self._lib.cj_model_get_cityobject_id(c_void_p(handle), index, pointer(payload))
        )
        return self._take_bytes(payload).decode("utf-8")

    def geometry_type(self, handle: int, index: int) -> GeometryType:
        geometry_type = c_int()
        self._raise_if_error(
            self._lib.cj_model_get_geometry_type(c_void_p(handle), index, pointer(geometry_type))
        )
        return GeometryType(geometry_type.value)

    def geometry_boundary(self, handle: int, index: int) -> dict[str, object]:
        payload = GeometryBoundaryStruct()
        self._raise_if_error(
            self._lib.cj_model_copy_geometry_boundary(c_void_p(handle), index, pointer(payload))
        )
        return self._take_geometry_boundary(payload)

    def geometry_boundary_coordinates(self, handle: int, index: int) -> list[VertexStruct]:
        payload = VerticesStruct()
        self._raise_if_error(
            self._lib.cj_model_copy_geometry_boundary_coordinates(
                c_void_p(handle), index, pointer(payload)
            )
        )
        return self._take_vertices(payload)

    def vertices(self, handle: int) -> list[VertexStruct]:
        payload = VerticesStruct()
        self._raise_if_error(self._lib.cj_model_copy_vertices(c_void_p(handle), pointer(payload)))
        return self._take_vertices(payload)

    def template_vertices(self, handle: int) -> list[VertexStruct]:
        payload = VerticesStruct()
        self._raise_if_error(
            self._lib.cj_model_copy_template_vertices(c_void_p(handle), pointer(payload))
        )
        return self._take_vertices(payload)

    def uv_coordinates(self, handle: int) -> list[UVStruct]:
        payload = UVsStruct()
        self._raise_if_error(
            self._lib.cj_model_copy_uv_coordinates(c_void_p(handle), pointer(payload))
        )
        return self._take_uvs(payload)

    def reserve_import(self, handle: int, capacities: ModelCapacitiesStruct) -> None:
        self._raise_if_error(self._lib.cj_model_reserve_import(c_void_p(handle), capacities))

    def add_vertex(self, handle: int, x: float, y: float, z: float) -> int:
        index = c_size_t(0)
        self._raise_if_error(
            self._lib.cj_model_add_vertex(
                c_void_p(handle), VertexStruct(x=x, y=y, z=z), pointer(index)
            )
        )
        return int(index.value)

    def add_template_vertex(self, handle: int, x: float, y: float, z: float) -> int:
        index = c_size_t(0)
        self._raise_if_error(
            self._lib.cj_model_add_template_vertex(
                c_void_p(handle), VertexStruct(x=x, y=y, z=z), pointer(index)
            )
        )
        return int(index.value)

    def add_uv_coordinate(self, handle: int, u: float, v: float) -> int:
        index = c_size_t(0)
        self._raise_if_error(
            self._lib.cj_model_add_uv_coordinate(c_void_p(handle), UVStruct(u=u, v=v), pointer(index))
        )
        return int(index.value)

    def geometry_boundary_view(
        self, payload: GeometryBoundaryPayload
    ) -> tuple[GeometryBoundaryViewStruct, list[object]]:
        return self._geometry_boundary_view(payload)

    def write_options(self, payload: WriteOptionsPayload) -> WriteOptionsStruct:
        return self._write_options(payload)

    def cityjsonseq_write_options(
        self, payload: CityJSONSeqWriteOptionsPayload
    ) -> CityJSONSeqWriteOptionsStruct:
        return self._cityjsonseq_write_options(payload)

    def cityjsonseq_auto_transform_options(
        self, payload: CityJSONSeqAutoTransformOptionsPayload
    ) -> CityJSONSeqAutoTransformOptionsStruct:
        return self._cityjsonseq_auto_transform_options(payload)

    def set_metadata_title(self, handle: int, title: str) -> None:
        view, _buffer = self._string_view(title)
        self._raise_if_error(self._lib.cj_model_set_metadata_title(c_void_p(handle), view))

    def set_metadata_identifier(self, handle: int, identifier: str) -> None:
        view, _buffer = self._string_view(identifier)
        self._raise_if_error(self._lib.cj_model_set_metadata_identifier(c_void_p(handle), view))

    def set_transform(self, handle: int, transform: TransformStruct) -> None:
        self._raise_if_error(self._lib.cj_model_set_transform(c_void_p(handle), transform))

    def clear_transform(self, handle: int) -> None:
        self._raise_if_error(self._lib.cj_model_clear_transform(c_void_p(handle)))

    def add_cityobject(self, handle: int, cityobject_id: str, cityobject_type: str) -> None:
        cityobject_view, _cityobject_buffer = self._string_view(cityobject_id)
        type_view, _type_buffer = self._string_view(cityobject_type)
        self._raise_if_error(
            self._lib.cj_model_add_cityobject(c_void_p(handle), cityobject_view, type_view)
        )

    def remove_cityobject(self, handle: int, cityobject_id: str) -> None:
        view, _buffer = self._string_view(cityobject_id)
        self._raise_if_error(self._lib.cj_model_remove_cityobject(c_void_p(handle), view))

    def attach_geometry_to_cityobject(self, handle: int, cityobject_id: str, geometry_index: int) -> None:
        view, _buffer = self._string_view(cityobject_id)
        self._raise_if_error(
            self._lib.cj_model_attach_geometry_to_cityobject(
                c_void_p(handle), view, geometry_index
            )
        )

    def clear_cityobject_geometry(self, handle: int, cityobject_id: str) -> None:
        view, _buffer = self._string_view(cityobject_id)
        self._raise_if_error(
            self._lib.cj_model_clear_cityobject_geometry(c_void_p(handle), view)
        )

    def add_geometry_from_boundary(
        self, handle: int, boundary: GeometryBoundaryPayload, lod: str | None = None
    ) -> int:
        boundary_view, _buffers = self._geometry_boundary_view(boundary)
        lod_view, _lod_buffer = self._string_view(lod) if lod is not None else (StringViewStruct(), b"")
        index = c_size_t(0)
        self._raise_if_error(
            self._lib.cj_model_add_geometry_from_boundary(
                c_void_p(handle), boundary_view, lod_view, pointer(index)
            )
        )
        return int(index.value)

    def cleanup(self, handle: int) -> None:
        self._raise_if_error(self._lib.cj_model_cleanup(c_void_p(handle)))

    def append_model(self, target_handle: int, source_handle: int) -> None:
        self._raise_if_error(
            self._lib.cj_model_append_model(c_void_p(target_handle), c_void_p(source_handle))
        )

    def extract_cityobjects(self, handle: int, cityobject_ids: list[str]) -> int:
        if not cityobject_ids:
            raise ValueError("cityobject_ids must not be empty")

        buffers: list[object] = []
        views = []
        for cityobject_id in cityobject_ids:
            view, buffer = self._string_view(cityobject_id)
            views.append(view)
            buffers.append(buffer)

        array_type = StringViewStruct * len(views)
        array = array_type(*views)
        extracted = c_void_p()
        self._raise_if_error(
            self._lib.cj_model_extract_cityobjects(
                c_void_p(handle), array, len(views), pointer(extracted)
            )
        )
        return int(extracted.value)

    def serialize_document_with_options(self, handle: int, options: WriteOptionsStruct) -> bytes:
        payload = BytesStruct()
        self._raise_if_error(
            self._lib.cj_model_serialize_document_with_options(
                c_void_p(handle), options, pointer(payload)
            )
        )
        return self._take_bytes(payload)

    def serialize_feature_with_options(self, handle: int, options: WriteOptionsStruct) -> bytes:
        payload = BytesStruct()
        self._raise_if_error(
            self._lib.cj_model_serialize_feature_with_options(
                c_void_p(handle), options, pointer(payload)
            )
        )
        return self._take_bytes(payload)

    def parse_feature_stream_merge(self, data: bytes) -> int:
        handle = c_void_p()
        pointer_data = self._data_pointer(data)
        self._raise_if_error(
            self._lib.cj_model_parse_feature_stream_merge_bytes(
                pointer_data, len(data), pointer(handle)
            )
        )
        return int(handle.value)

    def serialize_feature_stream(
        self, handles: list[int], options: WriteOptionsStruct
    ) -> bytes:
        if not handles:
            payload = BytesStruct()
            self._raise_if_error(
                self._lib.cj_model_serialize_feature_stream(
                    POINTER(c_void_p)(), 0, options, pointer(payload)
                )
            )
            return self._take_bytes(payload)

        array_type = c_void_p * len(handles)
        array = array_type(*[c_void_p(handle) for handle in handles])
        payload = BytesStruct()
        self._raise_if_error(
            self._lib.cj_model_serialize_feature_stream(
                array, len(handles), options, pointer(payload)
            )
        )
        return self._take_bytes(payload)

    def serialize_cityjsonseq_with_transform(
        self,
        base_root_handle: int,
        feature_handles: list[int],
        transform: TransformStruct,
        options: CityJSONSeqWriteOptionsStruct,
    ) -> bytes:
        array_type = c_void_p * len(feature_handles)
        array = array_type(*[c_void_p(handle) for handle in feature_handles])
        payload = BytesStruct()
        self._raise_if_error(
            self._lib.cj_model_serialize_cityjsonseq_with_transform(
                c_void_p(base_root_handle),
                array,
                len(feature_handles),
                transform,
                options,
                pointer(payload),
            )
        )
        return self._take_bytes(payload)

    def serialize_cityjsonseq_auto_transform(
        self,
        base_root_handle: int,
        feature_handles: list[int],
        options: CityJSONSeqAutoTransformOptionsStruct,
    ) -> bytes:
        array_type = c_void_p * len(feature_handles)
        array = array_type(*[c_void_p(handle) for handle in feature_handles])
        payload = BytesStruct()
        self._raise_if_error(
            self._lib.cj_model_serialize_cityjsonseq_auto_transform(
                c_void_p(base_root_handle),
                array,
                len(feature_handles),
                options,
                pointer(payload),
            )
        )
        return self._take_bytes(payload)
