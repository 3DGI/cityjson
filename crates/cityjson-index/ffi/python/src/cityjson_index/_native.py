from __future__ import annotations

from ctypes import (
    CDLL,
    POINTER,
    Structure,
    byref,
    c_bool,
    c_char_p,
    c_int,
    c_int64,
    c_size_t,
    c_uint64,
    c_void_p,
    cast,
    create_string_buffer,
    string_at,
)
from enum import IntEnum
from pathlib import Path
import os
import sys


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


SUCCESS = Status.SUCCESS.value
INVALID_ARGUMENT = Status.INVALID_ARGUMENT.value


class CjxError(RuntimeError):
    def __init__(self, status: Status, kind: ErrorKind, message: str) -> None:
        super().__init__(message)
        self.status = status
        self.kind = kind


class _Bytes(Structure):
    _fields_ = [("data", c_void_p), ("len", c_size_t)]


class _IndexStatus(Structure):
    _fields_ = [
        ("exists", c_bool),
        ("needs_reindex", c_bool),
        ("indexed_feature_count", c_size_t),
        ("indexed_source_count", c_size_t),
    ]


class _FeatureRef(Structure):
    _fields_ = [
        ("row_id", c_int64),
        ("feature_id", _Bytes),
        ("source_path", _Bytes),
        ("offset", c_uint64),
        ("length", c_uint64),
        ("vertices_offset", c_uint64),
        ("vertices_length", c_uint64),
        ("member_ranges_json", _Bytes),
        ("source_id", c_int64),
    ]


def _library_name() -> str:
    if sys.platform.startswith("win"):
        return "cityjson_index_ffi_core.dll"
    if sys.platform == "darwin":
        return "libcityjson_index_ffi_core.dylib"
    return "libcityjson_index_ffi_core.so"


def _find_repo_root(package_dir: Path) -> Path | None:
    for candidate in package_dir.parents:
        if (candidate / "Cargo.toml").exists() and (candidate / "ffi" / "python" / "pyproject.toml").exists():
            return candidate
    return None


def _candidate_paths() -> list[Path]:
    lib_name = _library_name()
    package_dir = Path(__file__).resolve().parent

    candidates: list[Path] = []
    env = os.environ.get("CITYJSON_INDEX_LIBRARY_PATH")
    if env:
        candidates.append(Path(env))

    candidates.append(package_dir / lib_name)

    repo_root = _find_repo_root(package_dir)
    if repo_root is not None:
        candidates.extend(
            [
                repo_root / "target" / "release" / lib_name,
                repo_root / "target" / "debug" / lib_name,
                repo_root / "target" / "release" / "deps" / lib_name,
                repo_root / "target" / "debug" / "deps" / lib_name,
            ]
        )

    unique_candidates: list[Path] = []
    for candidate in candidates:
        if candidate not in unique_candidates:
            unique_candidates.append(candidate)
    return unique_candidates


def _load_cdll() -> CDLL:
    for candidate in _candidate_paths():
        if candidate.exists():
            return CDLL(str(candidate))

    searched = ", ".join(str(candidate) for candidate in _candidate_paths())
    raise FileNotFoundError(f"could not locate cityjson-index shared library; searched: {searched}")


def _coerce_status(value: int) -> Status:
    return Status(value) if value in Status._value2member_map_ else Status.INTERNAL


def _coerce_error_kind(value: int) -> ErrorKind:
    return ErrorKind(value) if value in ErrorKind._value2member_map_ else ErrorKind.INTERNAL


class FfiLibrary:
    def __init__(self, library: CDLL) -> None:
        self._lib = library
        self._configure()

    @classmethod
    def load(cls) -> "FfiLibrary":
        return cls(_load_cdll())

    def _configure(self) -> None:
        self._lib.cjx_clear_error.argtypes = []
        self._lib.cjx_clear_error.restype = c_int
        self._lib.cjx_last_error_kind.argtypes = []
        self._lib.cjx_last_error_kind.restype = c_int
        self._lib.cjx_last_error_message_len.argtypes = []
        self._lib.cjx_last_error_message_len.restype = c_size_t
        self._lib.cjx_last_error_message_copy.argtypes = [c_char_p, c_size_t, POINTER(c_size_t)]
        self._lib.cjx_last_error_message_copy.restype = c_int
        self._lib.cjx_bytes_free.argtypes = [_Bytes]
        self._lib.cjx_bytes_free.restype = c_int
        self._lib.cjx_index_open.argtypes = [c_char_p, c_size_t, c_char_p, c_size_t, POINTER(c_void_p)]
        self._lib.cjx_index_open.restype = c_int
        self._lib.cjx_index_free.argtypes = [c_void_p]
        self._lib.cjx_index_free.restype = c_int
        self._lib.cjx_index_status.argtypes = [c_void_p, POINTER(_IndexStatus)]
        self._lib.cjx_index_status.restype = c_int
        self._lib.cjx_index_reindex.argtypes = [c_void_p]
        self._lib.cjx_index_reindex.restype = c_int
        self._lib.cjx_index_feature_ref_count.argtypes = [c_void_p, POINTER(c_size_t)]
        self._lib.cjx_index_feature_ref_count.restype = c_int
        self._lib.cjx_index_feature_ref_page.argtypes = [
            c_void_p,
            c_size_t,
            c_size_t,
            POINTER(POINTER(_FeatureRef)),
            POINTER(c_size_t),
        ]
        self._lib.cjx_index_feature_ref_page.restype = c_int
        self._lib.cjx_feature_ref_page_free.argtypes = [POINTER(_FeatureRef), c_size_t]
        self._lib.cjx_feature_ref_page_free.restype = c_int
        self._lib.cjx_index_get_bytes.argtypes = [c_void_p, c_char_p, c_size_t, POINTER(_Bytes)]
        self._lib.cjx_index_get_bytes.restype = c_int
        self._lib.cjx_index_get_model_bytes.argtypes = [c_void_p, c_char_p, c_size_t, POINTER(_Bytes)]
        self._lib.cjx_index_get_model_bytes.restype = c_int
        self._lib.cjx_index_read_feature_bytes.argtypes = [c_void_p, POINTER(_FeatureRef), POINTER(_Bytes)]
        self._lib.cjx_index_read_feature_bytes.restype = c_int
        self._lib.cjx_index_read_feature_model_bytes.argtypes = [
            c_void_p,
            POINTER(_FeatureRef),
            POINTER(_Bytes),
        ]
        self._lib.cjx_index_read_feature_model_bytes.restype = c_int

    def clear_error(self) -> None:
        self._check_status(self._lib.cjx_clear_error())

    def _last_error_message(self) -> str:
        length = self._lib.cjx_last_error_message_len()
        if length == 0:
            return ""

        buffer = create_string_buffer(length + 1)
        out_len = c_size_t()
        status = self._lib.cjx_last_error_message_copy(buffer, len(buffer), byref(out_len))
        if status != Status.SUCCESS:
            return ""
        return buffer.value.decode("utf-8", errors="replace")

    def _check_status(self, status: int) -> None:
        if status == Status.SUCCESS:
            return

        status_enum = _coerce_status(status)
        kind = _coerce_error_kind(self._lib.cjx_last_error_kind())
        message = self._last_error_message()
        if not message:
            message = f"cityjson-index native call failed with status {status_enum.value}"
        raise CjxError(status=status_enum, kind=kind, message=message)

    def open_index(self, dataset_dir: str, index_path: str | None) -> c_void_p:
        handle = c_void_p()
        dataset_bytes = dataset_dir.encode("utf-8")
        if index_path is None:
            index_bytes = None
            index_len = 0
        else:
            index_bytes = index_path.encode("utf-8")
            index_len = len(index_bytes)

        self._check_status(
            self._lib.cjx_index_open(
                c_char_p(dataset_bytes),
                len(dataset_bytes),
                c_char_p(index_bytes) if index_bytes is not None else c_char_p(),
                index_len,
                byref(handle),
            )
        )
        return handle

    def close_index(self, handle: c_void_p) -> None:
        if not handle:
            return
        self._check_status(self._lib.cjx_index_free(handle))

    def index_status(self, handle: c_void_p) -> _IndexStatus:
        status = _IndexStatus()
        self._check_status(self._lib.cjx_index_status(handle, byref(status)))
        return status

    def reindex(self, handle: c_void_p) -> None:
        self._check_status(self._lib.cjx_index_reindex(handle))

    def feature_ref_count(self, handle: c_void_p) -> int:
        count = c_size_t()
        self._check_status(self._lib.cjx_index_feature_ref_count(handle, byref(count)))
        return int(count.value)

    def feature_ref_page(self, handle: c_void_p, offset: int, limit: int) -> list[object]:
        refs = POINTER(_FeatureRef)()
        count = c_size_t()
        self._check_status(
            self._lib.cjx_index_feature_ref_page(handle, offset, limit, byref(refs), byref(count))
        )

        try:
            if count.value == 0 or not refs:
                return []

            from . import FeatureRef

            result: list[FeatureRef] = []
            for index in range(count.value):
                ref = refs[index]
                result.append(
                    FeatureRef(
                        row_id=int(ref.row_id),
                        feature_id=_bytes_to_py(ref.feature_id).decode("utf-8"),
                        source_path=_bytes_to_py(ref.source_path).decode("utf-8"),
                        offset=int(ref.offset),
                        length=int(ref.length),
                        vertices_offset=int(ref.vertices_offset),
                        vertices_length=int(ref.vertices_length),
                        member_ranges_json=_bytes_to_py(ref.member_ranges_json).decode("utf-8"),
                        source_id=int(ref.source_id),
                    )
                )
            return result
        finally:
            self._check_status(self._lib.cjx_feature_ref_page_free(refs, count.value))

    def _maybe_get_bytes(self, status: int, out: _Bytes) -> bytes | None:
        if status == Status.INVALID_ARGUMENT:
            self.clear_error()
            return None

        self._check_status(status)
        try:
            return _bytes_to_py(out)
        finally:
            self._check_status(self._lib.cjx_bytes_free(out))

    def get_bytes(self, handle: c_void_p, feature_id: str) -> bytes | None:
        payload = feature_id.encode("utf-8")
        out = _Bytes()
        status = self._lib.cjx_index_get_bytes(handle, c_char_p(payload), len(payload), byref(out))
        return self._maybe_get_bytes(status, out)

    def get_model_bytes(self, handle: c_void_p, feature_id: str) -> bytes | None:
        payload = feature_id.encode("utf-8")
        out = _Bytes()
        status = self._lib.cjx_index_get_model_bytes(
            handle, c_char_p(payload), len(payload), byref(out)
        )
        return self._maybe_get_bytes(status, out)

    def read_feature_bytes(self, handle: c_void_p, source_path: str, offset: int, length: int) -> bytes:
        source_bytes = source_path.encode("utf-8")
        source_buffer = create_string_buffer(source_bytes)
        native = _FeatureRef()
        native.row_id = 0
        native.feature_id.data = None
        native.feature_id.len = 0
        native.source_path.data = cast(source_buffer, c_void_p)
        native.source_path.len = len(source_bytes)
        native.offset = offset
        native.length = length
        native.vertices_offset = 0
        native.vertices_length = 0
        native.member_ranges_json.data = None
        native.member_ranges_json.len = 0
        native.source_id = 0

        out = _Bytes()
        self._check_status(self._lib.cjx_index_read_feature_bytes(handle, byref(native), byref(out)))
        try:
            return _bytes_to_py(out)
        finally:
            self._check_status(self._lib.cjx_bytes_free(out))

    def read_feature_model_bytes(
        self,
        handle: c_void_p,
        feature_id: str,
        source_path: str,
        offset: int,
        length: int,
        vertices_offset: int,
        vertices_length: int,
        member_ranges_json: str,
        source_id: int,
    ) -> bytes:
        feature_id_bytes = feature_id.encode("utf-8")
        feature_id_buffer = create_string_buffer(feature_id_bytes)
        source_bytes = source_path.encode("utf-8")
        source_buffer = create_string_buffer(source_bytes)
        member_ranges_bytes = member_ranges_json.encode("utf-8")
        member_ranges_buffer = create_string_buffer(member_ranges_bytes)

        native = _FeatureRef()
        native.row_id = 0
        native.feature_id.data = cast(feature_id_buffer, c_void_p)
        native.feature_id.len = len(feature_id_bytes)
        native.source_path.data = cast(source_buffer, c_void_p)
        native.source_path.len = len(source_bytes)
        native.offset = offset
        native.length = length
        native.vertices_offset = vertices_offset
        native.vertices_length = vertices_length
        native.member_ranges_json.data = cast(member_ranges_buffer, c_void_p)
        native.member_ranges_json.len = len(member_ranges_bytes)
        native.source_id = source_id

        out = _Bytes()
        self._check_status(
            self._lib.cjx_index_read_feature_model_bytes(handle, byref(native), byref(out))
        )
        try:
            return _bytes_to_py(out)
        finally:
            self._check_status(self._lib.cjx_bytes_free(out))


def _bytes_to_py(value: _Bytes) -> bytes:
    if value.data is None or value.len == 0:
        return b""
    return string_at(value.data, value.len)


_ffi = FfiLibrary.load()


def clear_error() -> None:
    _ffi.clear_error()


def open_index(dataset_dir: str, index_path: str | None) -> c_void_p:
    return _ffi.open_index(dataset_dir, index_path)


def close_index(handle: c_void_p) -> None:
    _ffi.close_index(handle)


def index_status(handle: c_void_p) -> _IndexStatus:
    return _ffi.index_status(handle)


def reindex(handle: c_void_p) -> None:
    _ffi.reindex(handle)


def feature_ref_count(handle: c_void_p) -> int:
    return _ffi.feature_ref_count(handle)


def feature_ref_page(handle: c_void_p, offset: int, limit: int) -> list[object]:
    return _ffi.feature_ref_page(handle, offset, limit)


def get_bytes(handle: c_void_p, feature_id: str) -> bytes | None:
    return _ffi.get_bytes(handle, feature_id)


def get_model_bytes(handle: c_void_p, feature_id: str) -> bytes | None:
    return _ffi.get_model_bytes(handle, feature_id)


def read_feature_bytes(handle: c_void_p, source_path: str, offset: int, length: int) -> bytes:
    return _ffi.read_feature_bytes(handle, source_path, offset, length)


def read_feature_model_bytes(
    handle: c_void_p,
    feature_id: str,
    source_path: str,
    offset: int,
    length: int,
    vertices_offset: int,
    vertices_length: int,
    member_ranges_json: str,
    source_id: int,
) -> bytes:
    return _ffi.read_feature_model_bytes(
        handle,
        feature_id,
        source_path,
        offset,
        length,
        vertices_offset,
        vertices_length,
        member_ranges_json,
        source_id,
    )
