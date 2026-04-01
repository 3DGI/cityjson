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
from pathlib import Path
import os
import sys


SUCCESS = 0
INVALID_ARGUMENT = 1


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
        return "cjindex.dll"
    if sys.platform == "darwin":
        return "libcjindex.dylib"
    return "libcjindex.so"


def _candidate_paths() -> list[Path]:
    lib_name = _library_name()
    package_dir = Path(__file__).resolve().parent

    candidates = []
    env = os.environ.get("CJINDEX_LIBRARY_PATH")
    if env:
        candidates.append(Path(env))

    candidates.append(package_dir / lib_name)
    if len(package_dir.parents) > 3:
        root = package_dir.parents[3]
        candidates.extend(
            [
                root / "target" / "debug" / lib_name,
                root / "target" / "release" / lib_name,
                root / "target" / "debug" / "deps" / lib_name,
                root / "target" / "release" / "deps" / lib_name,
            ]
        )
    unique_candidates: list[Path] = []
    for candidate in candidates:
        if candidate not in unique_candidates:
            unique_candidates.append(candidate)
    return unique_candidates


def load_library() -> CDLL | None:
    for candidate in _candidate_paths():
        if candidate.exists():
            return CDLL(str(candidate))
    return None


LIB = load_library()


def _configure(lib: CDLL) -> None:
    lib.cjx_clear_error.restype = c_int
    lib.cjx_last_error_kind.restype = c_int
    lib.cjx_last_error_message_len.restype = c_size_t
    lib.cjx_last_error_message_copy.argtypes = [c_char_p, c_size_t, POINTER(c_size_t)]
    lib.cjx_last_error_message_copy.restype = c_int
    lib.cjx_bytes_free.argtypes = [_Bytes]
    lib.cjx_bytes_free.restype = c_int
    lib.cjx_index_open.argtypes = [c_char_p, c_size_t, c_char_p, c_size_t, POINTER(c_void_p)]
    lib.cjx_index_open.restype = c_int
    lib.cjx_index_free.argtypes = [c_void_p]
    lib.cjx_index_free.restype = c_int
    lib.cjx_index_status.argtypes = [c_void_p, POINTER(_IndexStatus)]
    lib.cjx_index_status.restype = c_int
    lib.cjx_index_reindex.argtypes = [c_void_p]
    lib.cjx_index_reindex.restype = c_int
    lib.cjx_index_feature_ref_count.argtypes = [c_void_p, POINTER(c_size_t)]
    lib.cjx_index_feature_ref_count.restype = c_int
    lib.cjx_index_feature_ref_page.argtypes = [
        c_void_p,
        c_size_t,
        c_size_t,
        POINTER(POINTER(_FeatureRef)),
        POINTER(c_size_t),
    ]
    lib.cjx_index_feature_ref_page.restype = c_int
    lib.cjx_feature_ref_page_free.argtypes = [POINTER(_FeatureRef), c_size_t]
    lib.cjx_feature_ref_page_free.restype = c_int
    lib.cjx_index_get_bytes.argtypes = [c_void_p, c_char_p, c_size_t, POINTER(_Bytes)]
    lib.cjx_index_get_bytes.restype = c_int
    lib.cjx_index_get_model_bytes.argtypes = [c_void_p, c_char_p, c_size_t, POINTER(_Bytes)]
    lib.cjx_index_get_model_bytes.restype = c_int
    lib.cjx_index_read_feature_bytes.argtypes = [c_void_p, POINTER(_FeatureRef), POINTER(_Bytes)]
    lib.cjx_index_read_feature_bytes.restype = c_int
    lib.cjx_index_read_feature_model_bytes.argtypes = [
        c_void_p,
        POINTER(_FeatureRef),
        POINTER(_Bytes),
    ]
    lib.cjx_index_read_feature_model_bytes.restype = c_int


if LIB is not None:
    _configure(LIB)


def _last_error_message() -> str:
    if LIB is None:
        return "cjindex native library is not available"

    length = LIB.cjx_last_error_message_len()
    if length == 0:
        return ""

    buffer = create_string_buffer(length + 1)
    out_len = c_size_t()
    status = LIB.cjx_last_error_message_copy(buffer, len(buffer), byref(out_len))
    if status != SUCCESS:
        return "cjindex native call failed"
    return buffer.value.decode("utf-8", errors="replace")


def check_status(status: int) -> None:
    if status == SUCCESS:
        return
    message = _last_error_message()
    if message:
        raise RuntimeError(message)
    raise RuntimeError(f"cjindex native call failed with status {status}")


def clear_error() -> None:
    if LIB is None:
        return
    LIB.cjx_clear_error()


def open_index(dataset_dir: str, index_path: str | None) -> c_void_p:
    if LIB is None:
        raise RuntimeError("cjindex native library is not available")

    handle = c_void_p()
    dataset_bytes = dataset_dir.encode("utf-8")
    if index_path is None:
        index_bytes = None
        index_len = 0
    else:
        index_bytes = index_path.encode("utf-8")
        index_len = len(index_bytes)

    status = LIB.cjx_index_open(
        c_char_p(dataset_bytes),
        len(dataset_bytes),
        c_char_p(index_bytes) if index_bytes is not None else c_char_p(),
        index_len,
        byref(handle),
    )
    check_status(status)
    return handle


def close_index(handle: c_void_p) -> None:
    if LIB is None or not handle:
        return
    LIB.cjx_index_free(handle)


def index_status(handle: c_void_p) -> _IndexStatus:
    if LIB is None:
        return _IndexStatus()
    status = _IndexStatus()
    check_status(LIB.cjx_index_status(handle, byref(status)))
    return status


def reindex(handle: c_void_p) -> None:
    if LIB is None:
        return
    check_status(LIB.cjx_index_reindex(handle))


def feature_ref_count(handle: c_void_p) -> int:
    if LIB is None:
        return 0
    count = c_size_t()
    check_status(LIB.cjx_index_feature_ref_count(handle, byref(count)))
    return int(count.value)


def _bytes_to_py(value: _Bytes) -> bytes:
    if value.data is None or value.len == 0:
        return b""
    return string_at(value.data, value.len)


def feature_ref_page(handle: c_void_p, offset: int, limit: int) -> list[object]:
    if LIB is None:
        return []

    refs = POINTER(_FeatureRef)()
    count = c_size_t()
    status = LIB.cjx_index_feature_ref_page(handle, offset, limit, byref(refs), byref(count))
    check_status(status)

    try:
        if count.value == 0 or not refs:
            return []
        from . import FeatureRef

        result: list[FeatureRef] = []
        for i in range(count.value):
            ref = refs[i]
            result.append(
                FeatureRef(
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
        LIB.cjx_feature_ref_page_free(refs, count.value)


def get_bytes(handle: c_void_p, feature_id: str) -> bytes | None:
    if LIB is None:
        return None

    payload = feature_id.encode("utf-8")
    out = _Bytes()
    status = LIB.cjx_index_get_bytes(handle, c_char_p(payload), len(payload), byref(out))
    if status == INVALID_ARGUMENT:
        clear_error()
        return None
    check_status(status)
    try:
        return _bytes_to_py(out)
    finally:
        LIB.cjx_bytes_free(out)


def get_model_bytes(handle: c_void_p, feature_id: str) -> bytes | None:
    if LIB is None:
        return None

    payload = feature_id.encode("utf-8")
    out = _Bytes()
    status = LIB.cjx_index_get_model_bytes(handle, c_char_p(payload), len(payload), byref(out))
    if status == INVALID_ARGUMENT:
        clear_error()
        return None
    check_status(status)
    try:
        return _bytes_to_py(out)
    finally:
        LIB.cjx_bytes_free(out)


def read_feature_bytes(handle: c_void_p, source_path: str, offset: int, length: int) -> bytes:
    if LIB is None:
        return b"{}"

    source_bytes = source_path.encode("utf-8")
    source_buffer = create_string_buffer(source_bytes)
    native = _FeatureRef()
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
    status = LIB.cjx_index_read_feature_bytes(handle, byref(native), byref(out))
    check_status(status)
    try:
        return _bytes_to_py(out)
    finally:
        LIB.cjx_bytes_free(out)


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
    if LIB is None:
        return b"{}"

    feature_id_bytes = feature_id.encode("utf-8")
    feature_id_buffer = create_string_buffer(feature_id_bytes)
    source_bytes = source_path.encode("utf-8")
    source_buffer = create_string_buffer(source_bytes)
    member_ranges_bytes = member_ranges_json.encode("utf-8")
    member_ranges_buffer = create_string_buffer(member_ranges_bytes)
    native = _FeatureRef()
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
    status = LIB.cjx_index_read_feature_model_bytes(handle, byref(native), byref(out))
    check_status(status)
    try:
        return _bytes_to_py(out)
    finally:
        LIB.cjx_bytes_free(out)
