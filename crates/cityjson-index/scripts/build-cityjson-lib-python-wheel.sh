#!/usr/bin/env sh
set -eu

out_dir=$1
lib_root=$(cd "${CITYJSON_LIB_REPO_ROOT:-../cityjson-lib}" && pwd)
backup_dir=$(mktemp -d)

restore() {
    cp "$backup_dir/cityjson-lib.Cargo.toml" "$lib_root/Cargo.toml"
    cp "$backup_dir/cityjson-lib-ffi-core.Cargo.toml" "$lib_root/ffi/core/Cargo.toml"
    cp "$backup_dir/cityjson-lib-python-ffi.py" "$lib_root/ffi/python/src/cityjson_lib/_ffi.py"
    cp "$backup_dir/cityjson-lib-python-setup.py" "$lib_root/ffi/python/setup.py"
    rm -rf "$backup_dir"
}

cp "$lib_root/Cargo.toml" "$backup_dir/cityjson-lib.Cargo.toml"
cp "$lib_root/ffi/core/Cargo.toml" "$backup_dir/cityjson-lib-ffi-core.Cargo.toml"
cp "$lib_root/ffi/python/src/cityjson_lib/_ffi.py" "$backup_dir/cityjson-lib-python-ffi.py"
cp "$lib_root/ffi/python/setup.py" "$backup_dir/cityjson-lib-python-setup.py"
trap restore EXIT INT TERM

if command -v python3 >/dev/null 2>&1; then
    py=python3
elif command -v python >/dev/null 2>&1; then
    py=python
else
    py="uv run python"
fi

$py - "$lib_root" <<'PY'
from __future__ import annotations

import sys
from pathlib import Path


root = Path(sys.argv[1])

manifest = root / "Cargo.toml"
lines = manifest.read_text(encoding="utf-8").splitlines()
trimmed: list[str] = []
skip_workspace = False

for line in lines:
    stripped = line.strip()
    if stripped == "[workspace]":
        skip_workspace = True
        continue
    if skip_workspace and stripped.startswith("["):
        skip_workspace = False
    if skip_workspace:
        continue
    if stripped == 'arrow = ["json", "dep:cityjson-arrow"]':
        trimmed.append("arrow = []")
        continue
    if stripped == 'parquet = ["json", "dep:cityjson-arrow", "dep:cityjson-parquet"]':
        trimmed.append("parquet = []")
        continue
    if stripped.startswith(("cityjson-arrow =", "cityjson-parquet =")):
        continue
    trimmed.append(line)

manifest.write_text("\n".join(trimmed) + "\n", encoding="utf-8")

ffi_manifest = root / "ffi" / "core" / "Cargo.toml"
text = ffi_manifest.read_text(encoding="utf-8")
text = text.replace('default = ["native-formats"]', "default = []")
text = text.replace('native-formats = ["cityjson_lib/arrow", "cityjson_lib/parquet"]', "native-formats = []")
ffi_manifest.write_text(text, encoding="utf-8")

setup_py = root / "ffi" / "python" / "setup.py"
text = setup_py.read_text(encoding="utf-8")
text = text.replace(
    """                "--features",
                "native-formats",
""",
    "",
)
setup_py.write_text(text, encoding="utf-8")

ffi_py = root / "ffi" / "python" / "src" / "cityjson_lib" / "_ffi.py"
text = ffi_py.read_text(encoding="utf-8")
text = text.replace(
    """        self._lib.cj_model_parse_arrow_bytes.argtypes = [
            POINTER(c_ubyte),
            c_size_t,
            POINTER(c_void_p),
        ]
        self._lib.cj_model_parse_arrow_bytes.restype = c_int
        self._lib.cj_model_parse_parquet_file.argtypes = [StringViewStruct, POINTER(c_void_p)]
        self._lib.cj_model_parse_parquet_file.restype = c_int
        self._lib.cj_model_parse_parquet_dataset_dir.argtypes = [
            StringViewStruct,
            POINTER(c_void_p),
        ]
        self._lib.cj_model_parse_parquet_dataset_dir.restype = c_int
""",
    """        if hasattr(self._lib, "cj_model_parse_arrow_bytes"):
            self._lib.cj_model_parse_arrow_bytes.argtypes = [
                POINTER(c_ubyte),
                c_size_t,
                POINTER(c_void_p),
            ]
            self._lib.cj_model_parse_arrow_bytes.restype = c_int
            self._lib.cj_model_parse_parquet_file.argtypes = [StringViewStruct, POINTER(c_void_p)]
            self._lib.cj_model_parse_parquet_file.restype = c_int
            self._lib.cj_model_parse_parquet_dataset_dir.argtypes = [
                StringViewStruct,
                POINTER(c_void_p),
            ]
            self._lib.cj_model_parse_parquet_dataset_dir.restype = c_int
""",
)
text = text.replace(
    """        self._lib.cj_model_serialize_arrow_bytes.argtypes = [c_void_p, POINTER(BytesStruct)]
        self._lib.cj_model_serialize_arrow_bytes.restype = c_int
        self._lib.cj_model_serialize_parquet_file.argtypes = [c_void_p, StringViewStruct]
        self._lib.cj_model_serialize_parquet_file.restype = c_int
        self._lib.cj_model_serialize_parquet_dataset_dir.argtypes = [c_void_p, StringViewStruct]
        self._lib.cj_model_serialize_parquet_dataset_dir.restype = c_int
""",
    """        if hasattr(self._lib, "cj_model_serialize_arrow_bytes"):
            self._lib.cj_model_serialize_arrow_bytes.argtypes = [c_void_p, POINTER(BytesStruct)]
            self._lib.cj_model_serialize_arrow_bytes.restype = c_int
            self._lib.cj_model_serialize_parquet_file.argtypes = [c_void_p, StringViewStruct]
            self._lib.cj_model_serialize_parquet_file.restype = c_int
            self._lib.cj_model_serialize_parquet_dataset_dir.argtypes = [c_void_p, StringViewStruct]
            self._lib.cj_model_serialize_parquet_dataset_dir.restype = c_int
""",
)
ffi_py.write_text(text, encoding="utf-8")
PY

cd "$lib_root/ffi/python"
uv build --wheel --out-dir "$out_dir"
