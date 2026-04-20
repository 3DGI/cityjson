from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import sys

from setuptools import setup
from setuptools.command.build_py import build_py as _build_py
from setuptools.command.sdist import sdist as _sdist


def _shared_library_name() -> str:
    if sys.platform.startswith("win"):
        return "cityjson_lib_ffi_core.dll"
    if sys.platform == "darwin":
        return "libcityjson_lib_ffi_core.dylib"
    return "libcityjson_lib_ffi_core.so"


def _repo_root() -> Path:
    env_root = os.environ.get("CITYJSON_LIB_REPO_ROOT")
    if env_root:
        candidate = Path(env_root).expanduser().resolve()
        if (candidate / "Cargo.toml").exists():
            return candidate

    for parent in Path(__file__).resolve().parents:
        if (parent / "Cargo.toml").exists():
            return parent

    raise FileNotFoundError(
        "could not locate the cityjson-lib repository root; set CITYJSON_LIB_REPO_ROOT"
    )


def _copy_path(source: Path, destination: Path) -> None:
    if source.is_dir():
        shutil.copytree(source, destination, dirs_exist_ok=True)
        return

    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)


def _write_trimmed_root_manifest(source: Path, destination: Path) -> None:
    lines = source.read_text(encoding="utf-8").splitlines()
    trimmed: list[str] = []
    skipping_workspace = False

    for line in lines:
        if line.strip() == "[workspace]":
            skipping_workspace = True
            continue

        if skipping_workspace and line.lstrip().startswith("["):
            skipping_workspace = False

        if not skipping_workspace:
            trimmed.append(line)

    manifest = "\n".join(trimmed) + "\n"
    manifest = manifest.replace('path = "../cityjson-rs"', 'path = "./cityjson-rs"')
    manifest = manifest.replace('path = "../cityjson-json"', 'path = "./cityjson-json"')
    manifest = manifest.replace('path = "../cityjson-arrow"', 'path = "./cityjson-arrow"')
    manifest = manifest.replace('path = "../cityjson-parquet"', 'path = "./cityjson-parquet"')
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(manifest, encoding="utf-8")


def _copy_release_tree_file(source_root: Path, release_root: Path, relative: str) -> None:
    _copy_path(source_root / relative, release_root / relative)


class build_py(_build_py):
    def run(self) -> None:
        built_library = self._build_native_library()
        super().run()
        target_dir = Path(self.build_lib) / "cityjson_lib"
        target_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(built_library, target_dir / built_library.name)

    def _build_native_library(self) -> Path:
        repo_root = _repo_root()
        subprocess.run(
            [
                "cargo",
                "build",
                "--release",
                "--lib",
                "--manifest-path",
                "ffi/core/Cargo.toml",
                "--target-dir",
                "target",
            ],
            check=True,
            cwd=repo_root,
        )
        built_library = repo_root / "target" / "release" / _shared_library_name()
        if not built_library.exists():
            raise FileNotFoundError(
                f"expected cityjson_lib shared library at {built_library}, but cargo build did not produce it"
            )
        return built_library


class sdist(_sdist):
    def make_release_tree(self, base_dir: str, files) -> None:  # type: ignore[override]
        super().make_release_tree(base_dir, files)

        release_root = Path(base_dir)
        source_root = _repo_root()
        _write_trimmed_root_manifest(source_root / "Cargo.toml", release_root / "Cargo.toml")
        for relative in (
            "src",
            "docs/public-api.md",
            "LICENSE",
            "LICENSE-APACHE",
            "tests/data",
        ):
            _copy_release_tree_file(source_root, release_root, relative)

        for relative in (
            "ffi/core/Cargo.toml",
            "ffi/core/README.md",
            "ffi/core/LICENSE",
            "ffi/core/LICENSE-APACHE",
            "ffi/core/cbindgen.toml",
            "ffi/core/include",
            "ffi/core/src",
            "ffi/core/tests",
        ):
            _copy_release_tree_file(source_root, release_root, relative)

        sibling_root = source_root.parent
        for crate_name in ("cityjson-rs", "cityjson-json", "cityjson-arrow", "cityjson-parquet"):
            crate_root = sibling_root / crate_name
            for relative in ("Cargo.toml", "README.md", "src"):
                _copy_path(crate_root / relative, release_root / crate_name / relative)


setup(
    cmdclass={"build_py": build_py, "sdist": sdist},
    package_data={"cityjson_lib": ["*.so", "*.dylib", "*.dll"]},
)
