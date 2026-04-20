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
        return "cityjson_index.dll"
    if sys.platform == "darwin":
        return "libcityjson_index.dylib"
    return "libcityjson_index.so"


def _repo_root() -> Path:
    env_root = os.environ.get("CITYJSON_INDEX_REPO_ROOT")
    if env_root:
        candidate = Path(env_root).expanduser().resolve()
        if (candidate / "Cargo.toml").exists():
            return candidate

    for parent in Path(__file__).resolve().parents:
        if (parent / "Cargo.toml").exists():
            return parent

    raise FileNotFoundError(
        "could not locate the cityjson-index repository root; set CITYJSON_INDEX_REPO_ROOT"
    )


def _copy_path(source: Path, destination: Path) -> None:
    if source.is_dir():
        shutil.copytree(source, destination, dirs_exist_ok=True)
        return

    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)


def _write_trimmed_root_manifest(
    source: Path,
    destination: Path,
    *,
    drop_sections: set[str] | frozenset[str] = frozenset(),
    drop_lines: set[str] | frozenset[str] = frozenset(),
    replacements: dict[str, str] | None = None,
) -> None:
    lines = source.read_text(encoding="utf-8").splitlines()
    trimmed: list[str] = []
    skipping_section: str | None = None

    for line in lines:
        stripped = line.strip()
        if stripped in drop_lines:
            continue

        if stripped.startswith("[") and stripped in drop_sections:
            skipping_section = stripped
            continue

        if skipping_section is not None and stripped.startswith("["):
            skipping_section = None

        if skipping_section is None:
            trimmed.append(line)

    manifest = "\n".join(trimmed) + "\n"
    for old, new in (replacements or {}).items():
        manifest = manifest.replace(old, new)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(manifest, encoding="utf-8")


class build_py(_build_py):
    def run(self) -> None:
        built_library = self._build_native_library()
        super().run()
        target_dir = Path(self.build_lib) / "cityjson_index"
        target_dir.mkdir(parents=True, exist_ok=True)
        shutil.copy2(built_library, target_dir / built_library.name)

    def _build_native_library(self) -> Path:
        repo_root = _repo_root()
        subprocess.run(
            ["cargo", "build", "--release", "--lib"],
            check=True,
            cwd=repo_root,
        )
        built_library = repo_root / "target" / "release" / _shared_library_name()
        if not built_library.exists():
            raise FileNotFoundError(
                f"expected cityjson-index shared library at {built_library}, but cargo build did not produce it"
            )
        return built_library


class sdist(_sdist):
    def make_release_tree(self, base_dir: str, files) -> None:  # type: ignore[override]
        super().make_release_tree(base_dir, files)

        release_root = Path(base_dir)
        source_root = _repo_root()
        _write_trimmed_root_manifest(
            source_root / "Cargo.toml",
            release_root / "Cargo.toml",
            drop_lines={"autobins = false", 'default-run = "cjindex"'},
            drop_sections={"[[bin]]", "[[bench]]"},
            replacements={
                'path = "../cityjson-lib"': 'path = "./cityjson-lib"',
                'path = "../cityjson-lib/ffi/core"': 'path = "./cityjson-lib/ffi/core"',
            },
        )
        _copy_path(source_root / "src", release_root / "src")
        _copy_path(source_root / "python" / "README.md", release_root / "README.md")

        sibling_root = source_root.parent
        for crate_name in ("cityjson-lib", "cityjson-rs", "cityjson-json"):
            crate_root = sibling_root / crate_name
            for relative in ("Cargo.toml", "README.md", "src"):
                _copy_path(crate_root / relative, release_root / crate_name / relative)

        for relative in ("docs/public-api.md", "ffi/core/Cargo.toml", "ffi/core/src"):
            _copy_path(sibling_root / "cityjson-lib" / relative, release_root / "cityjson-lib" / relative)
        _write_trimmed_root_manifest(
            sibling_root / "cityjson-lib" / "Cargo.toml",
            release_root / "cityjson-lib" / "Cargo.toml",
            drop_sections={"[workspace]"},
        )


setup(
    cmdclass={"build_py": build_py, "sdist": sdist},
    package_data={"cityjson_index": ["*.so", "*.dylib", "*.dll"]},
)
