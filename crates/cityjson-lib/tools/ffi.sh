#!/usr/bin/env bash

set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
header_path="${repo_dir}/ffi/core/include/cityjson_lib/cityjson_lib.h"
cpp_build_dir="${repo_dir}/target/ffi-cpp-build"

cd "${repo_dir}"

usage() {
  cat >&2 <<'EOF'
Usage:
  ./tools/ffi.sh check
  ./tools/ffi.sh fmt
  ./tools/ffi.sh doc
  ./tools/ffi.sh clean
  ./tools/ffi.sh test
  ./tools/ffi.sh ci
  ./tools/ffi.sh build header
  ./tools/ffi.sh build cpp [cmake-build-args...]
  ./tools/ffi.sh build python [uv-build-args...]
  ./tools/ffi.sh build wasm [cargo-build-args...]
EOF
  exit 1
}

require_header() {
  if [[ ! -f "${header_path}" ]]; then
    echo "Missing generated C ABI header at ${header_path}. Run 'just ffi build header' first." >&2
    exit 1
  fi
}

build_header() {
  mkdir -p "$(dirname "${header_path}")"
  cbindgen "${repo_dir}/ffi/core" \
    --config "${repo_dir}/ffi/core/cbindgen.toml" \
    --output "${header_path}"
}

build_cpp() {
  require_header
  cargo build --manifest-path "${repo_dir}/Cargo.toml" \
    -p cityjson-lib-ffi-core \
    -p cityjson-lib-wasm \
    --all-targets \
    --all-features
  cmake -S "${repo_dir}/ffi/cpp" -B "${cpp_build_dir}"
  cmake --build "${cpp_build_dir}" "$@"
}

build_python() {
  uv build ffi/python/ --wheel --out-dir ffi/python/dist "$@"
}

build_wasm() {
  cargo build --manifest-path "${repo_dir}/Cargo.toml" \
    -p cityjson-lib-ffi-core \
    -p cityjson-lib-wasm \
    --all-targets \
    --all-features \
    "$@"
}

check() {
  cargo check --manifest-path "${repo_dir}/Cargo.toml" \
    -p cityjson-lib-ffi-core \
    -p cityjson-lib-wasm \
    --all-targets \
    --all-features
  python3 -m compileall -q ffi/python/src
}

fmt() {
  cargo fmt --manifest-path "${repo_dir}/Cargo.toml" \
    -p cityjson-lib-ffi-core \
    -p cityjson-lib-wasm
}

doc() {
  cargo doc --manifest-path "${repo_dir}/Cargo.toml" \
    -p cityjson-lib-ffi-core \
    -p cityjson-lib-wasm \
    --no-deps \
    --all-features
}

test() {
  build_header
  build_cpp
  cargo test --manifest-path "${repo_dir}/Cargo.toml" \
    -p cityjson-lib-ffi-core \
    -p cityjson-lib-wasm \
    --all-features
  PYTHONPATH=ffi/python/src python3 -m unittest ffi/python/tests/test_api.py
  ctest --test-dir "${cpp_build_dir}" --output-on-failure
}

clean() {
  rm -rf "${repo_dir}/ffi/python/dist" \
    "${repo_dir}/ffi/python/build" \
    "${cpp_build_dir}"
}

ci() {
  check
  build_header
  build_cpp
  build_python
  build_wasm
  test
  doc
}

case "${1:-}" in
  build)
    shift
    case "${1:-}" in
      header)
        shift
        [[ $# -eq 0 ]] || usage
        build_header
        ;;
      cpp)
        shift
        build_cpp "$@"
        ;;
      python)
        shift
        build_python "$@"
        ;;
      wasm)
        shift
        build_wasm "$@"
        ;;
      *)
        usage
        ;;
    esac
    ;;
  check)
    shift
    [[ $# -eq 0 ]] || usage
    check
    ;;
  fmt)
    shift
    [[ $# -eq 0 ]] || usage
    fmt
    ;;
  doc)
    shift
    [[ $# -eq 0 ]] || usage
    doc
    ;;
  test)
    shift
    [[ $# -eq 0 ]] || usage
    test
    ;;
  clean)
    shift
    [[ $# -eq 0 ]] || usage
    clean
    ;;
  ci)
    shift
    [[ $# -eq 0 ]] || usage
    ci
    ;;
  help|-h|--help|"")
    usage
    ;;
  *)
    usage
    ;;
esac
