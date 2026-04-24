from __future__ import annotations

import json
import os
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any, Self

from . import _native

if TYPE_CHECKING:
    from cityjson_lib import CityModel


@dataclass(frozen=True, slots=True)
class FeatureRef:
    feature_id: str
    source_path: str
    offset: int = 0
    length: int = 0
    vertices_offset: int = 0
    vertices_length: int = 0
    member_ranges_json: str = ""
    source_id: int = 0
    row_id: int = 0

    @classmethod
    def from_native(cls, native: _native._FeatureRef) -> Self:
        return cls(
            feature_id=_native._bytes_to_py(native.feature_id).decode("utf-8"),
            source_path=_native._bytes_to_py(native.source_path).decode("utf-8"),
            offset=int(native.offset),
            length=int(native.length),
            vertices_offset=int(native.vertices_offset),
            vertices_length=int(native.vertices_length),
            member_ranges_json=_native._bytes_to_py(native.member_ranges_json).decode("utf-8"),
            source_id=int(native.source_id),
            row_id=int(native.row_id),
        )


@dataclass(frozen=True, slots=True)
class IndexStatus:
    exists: bool = True
    needs_reindex: bool = False
    indexed_feature_count: int = 0
    indexed_source_count: int = 0

    @classmethod
    def from_native(cls, native: _native._IndexStatus) -> Self:
        return cls(
            exists=bool(native.exists),
            needs_reindex=bool(native.needs_reindex),
            indexed_feature_count=int(native.indexed_feature_count),
            indexed_source_count=int(native.indexed_source_count),
        )


def _require_citymodel_type() -> type["CityModel"]:
    try:
        from cityjson_lib import CityModel
    except ImportError as exc:
        raise RuntimeError(
            "cityjson-index model APIs require the cityjson-lib Python package to be importable"
        ) from exc
    return CityModel


def _parse_citymodel_bytes(payload: bytes) -> "CityModel":
    try:
        from cityjson_lib import RootKind, probe_bytes
    except ImportError as exc:
        raise RuntimeError(
            "cityjson-index model APIs require the cityjson-lib Python package to be importable"
        ) from exc

    citymodel_type = _require_citymodel_type()
    probe = probe_bytes(payload)
    if probe.root_kind is RootKind.CITY_JSON_FEATURE:
        return citymodel_type.parse_feature_bytes(payload)
    return citymodel_type.parse_document_bytes(payload)


class OpenedIndex:
    def __init__(self, dataset_dir: str, index_path_override: str | None = None) -> None:
        self._dataset_dir = dataset_dir
        self._index_path_override = index_path_override
        self._handle = None

    def _require_handle(self):
        if self._handle is None:
            raise RuntimeError("OpenedIndex has already been closed or was not opened")
        return self._handle

    @classmethod
    def open(
        cls,
        dataset_dir: str | os.PathLike[str],
        index_path: str | os.PathLike[str] | None = None,
    ) -> Self:
        instance = cls(str(dataset_dir), None if index_path is None else str(index_path))
        instance._handle = _native.open_index(instance._dataset_dir, instance._index_path_override)
        return instance

    def close(self) -> None:
        if self._handle is None:
            return
        _native.close_index(self._handle)
        self._handle = None

    def __enter__(self) -> Self:
        return self

    def __exit__(self, exc_type, exc, tb) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:
            pass

    def status(self) -> IndexStatus:
        return IndexStatus.from_native(_native.index_status(self._require_handle()))

    def reindex(self) -> None:
        _native.reindex(self._require_handle())

    def feature_ref_count(self) -> int:
        return _native.feature_ref_count(self._require_handle())

    def feature_ref_page(self, offset: int, limit: int) -> list[FeatureRef]:
        return _native.feature_ref_page(self._require_handle(), offset, limit)

    def get_bytes(self, feature_id: str) -> bytes | None:
        return _native.get_bytes(self._require_handle(), feature_id)

    def get_model_bytes(self, feature_id: str) -> bytes | None:
        return _native.get_model_bytes(self._require_handle(), feature_id)

    def get(self, feature_id: str) -> "CityModel | None":
        payload = self.get_model_bytes(feature_id)
        if payload is None:
            return None
        return _parse_citymodel_bytes(payload)

    def get_json(self, feature_id: str) -> Any | None:
        payload = self.get_model_bytes(feature_id)
        if payload is None:
            return None
        return json.loads(payload)

    def read_feature_bytes(self, ref: FeatureRef) -> bytes:
        return _native.read_feature_bytes(self._require_handle(), ref.source_path, ref.offset, ref.length)

    def read_feature_model_bytes(self, ref: FeatureRef) -> bytes:
        return _native.read_feature_model_bytes(
            self._require_handle(),
            ref.feature_id,
            ref.source_path,
            ref.offset,
            ref.length,
            ref.vertices_offset,
            ref.vertices_length,
            ref.member_ranges_json,
            ref.source_id,
        )

    def read_feature(self, ref: FeatureRef) -> "CityModel":
        return _parse_citymodel_bytes(self.read_feature_model_bytes(ref))

    def read_feature_json(self, ref: FeatureRef) -> Any:
        return json.loads(self.read_feature_model_bytes(ref))


__all__ = ["FeatureRef", "IndexStatus", "OpenedIndex"]
