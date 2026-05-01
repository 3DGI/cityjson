from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import sys

from setuptools import setup
from setuptools import Distribution
from setuptools.command.bdist_wheel import bdist_wheel as _bdist_wheel
from setuptools.command.build_py import build_py as _build_py


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
                "--features",
                "native-formats",
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


class bdist_wheel(_bdist_wheel):
    def finalize_options(self) -> None:
        super().finalize_options()
        self.root_is_pure = False

    def get_tag(self) -> tuple[str, str, str]:
        _, _, plat = super().get_tag()
        return ("py3", "none", plat)


class BinaryDistribution(Distribution):
    def has_ext_modules(self) -> bool:
        return True


setup(
    cmdclass={"build_py": build_py, "bdist_wheel": bdist_wheel},
    distclass=BinaryDistribution,
    package_data={"cityjson_lib": ["*.so", "*.dylib", "*.dll"]},
)
