from __future__ import annotations

import os
from dataclasses import dataclass
from typing import Self

from . import _native


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


class OpenedIndex:
    def __init__(self, dataset_dir: str, index_path_override: str | None = None) -> None:
        self._dataset_dir = dataset_dir
        self._index_path_override = index_path_override
        self._handle = None

    @classmethod
    def open(
        cls,
        dataset_dir: str | os.PathLike[str],
        index_path: str | os.PathLike[str] | None = None,
    ) -> Self:
        instance = cls(str(dataset_dir), None if index_path is None else str(index_path))
        if _native.LIB is not None:
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
        if self._handle is None:
            return IndexStatus()
        if _native.LIB is None:
            return IndexStatus()
        return IndexStatus.from_native(_native.index_status(self._handle))

    def reindex(self) -> None:
        if self._handle is None:
            return
        _native.reindex(self._handle)

    def feature_ref_count(self) -> int:
        if self._handle is None:
            return 0
        return _native.feature_ref_count(self._handle)

    def feature_ref_page(self, offset: int, limit: int) -> list[FeatureRef]:
        if self._handle is None:
            return []
        return _native.feature_ref_page(self._handle, offset, limit)

    def get_bytes(self, feature_id: str) -> bytes | None:
        if self._handle is None:
            return None
        return _native.get_bytes(self._handle, feature_id)

    def read_feature_bytes(self, ref: FeatureRef) -> bytes:
        if self._handle is None:
            return b"{}"
        return _native.read_feature_bytes(self._handle, ref.source_path, ref.offset, ref.length)


__all__ = ["FeatureRef", "IndexStatus", "OpenedIndex"]
