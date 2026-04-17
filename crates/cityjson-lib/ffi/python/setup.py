from __future__ import annotations

from pathlib import Path
import shutil
import subprocess
import sys

from setuptools import setup
from setuptools.command.build_py import build_py as _build_py


def _shared_library_name() -> str:
    if sys.platform.startswith("win"):
        return "cityjson_lib_ffi_core.dll"
    if sys.platform == "darwin":
        return "libcityjson_lib_ffi_core.dylib"
    return "libcityjson_lib_ffi_core.so"


class build_py(_build_py):
    def run(self) -> None:
        built_library = self._build_native_library()
        super().run()
        target_dir = Path(self.build_lib) / "cityjson_lib"
        target_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(built_library, target_dir / built_library.name)

    def _build_native_library(self) -> Path:
        repo_root = Path(__file__).resolve().parents[2]
        subprocess.run(
            [
                "cargo",
                "build",
                "--release",
                "--package",
                "cityjson-lib-ffi-core",
                "--features",
                "arrow",
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


setup(
    cmdclass={"build_py": build_py},
    package_data={"cityjson_lib": ["*.so", "*.dylib", "*.dll"]},
)
