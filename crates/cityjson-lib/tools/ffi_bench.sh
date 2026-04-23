#!/usr/bin/env bash

set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
header_path="${repo_dir}/ffi/core/include/cityjson_lib/cityjson_lib.h"
target_dir="${repo_dir}/target/ffi-bench"
cpp_build_dir="${target_dir}/cpp-build"

mode="${1:-quick}"
case "${mode}" in
  quick|full)
    ;;
  --quick)
    mode="quick"
    ;;
  --full)
    mode="full"
    ;;
  *)
    echo "Usage: ./tools/ffi_bench.sh [quick|full]" >&2
    exit 1
    ;;
esac

require_tools() {
  for required in cargo cmake python3; do
    if ! command -v "${required}" >/dev/null 2>&1; then
      echo "missing required tool: ${required}" >&2
      exit 1
    fi
  done
}

require_header() {
  if [[ ! -f "${header_path}" ]]; then
    echo "Missing generated C ABI header at ${header_path}. Run 'just ffi build header' first." >&2
    exit 1
  fi
}

shared_library_name() {
  case "$(uname -s)" in
    Darwin) echo "libcityjson_lib_ffi_core.dylib" ;;
    MINGW*|MSYS*|CYGWIN*) echo "cityjson_lib_ffi_core.dll" ;;
    *) echo "libcityjson_lib_ffi_core.so" ;;
  esac
}

ensure_release_library() {
  CARGO_TARGET_DIR="${target_dir}" \
    cargo build --release --manifest-path "${repo_dir}/Cargo.toml" -p cityjson-lib-ffi-core
}

build_cpp_bench() {
  local release_lib="${target_dir}/release/$(shared_library_name)"
  cmake -S "${repo_dir}/ffi/cpp" -B "${cpp_build_dir}" \
    -DBUILD_TESTING=ON \
    -DCMAKE_BUILD_TYPE=Release \
    -DCITYJSON_LIB_FFI_CORE_SHARED_LIB="${release_lib}"
  cmake --build "${cpp_build_dir}" --target cityjson_lib_cpp_ffi_visibility
}

run_rust_bench() {
  local rust_output="${target_dir}/results/rust.json"
  mkdir -p "$(dirname "${rust_output}")"
  local bench_args=()
  if [[ "${mode}" == "quick" ]]; then
    bench_args+=(--quick)
  fi

  CARGO_TARGET_DIR="${target_dir}" \
    cargo run --release --manifest-path "${repo_dir}/ffi/core/Cargo.toml" --bin ffi_visibility -- \
    "${bench_args[@]}" | tee "${rust_output}"
}

run_python_bench() {
  local release_lib="${target_dir}/release/$(shared_library_name)"
  local python_output="${target_dir}/results/python.json"
  mkdir -p "$(dirname "${python_output}")"
  local bench_args=()
  if [[ "${mode}" == "quick" ]]; then
    bench_args+=(--quick)
  fi

  CITYJSON_LIB_FFI_CORE_LIB="${release_lib}" \
    PYTHONPATH="${repo_dir}/ffi/python/src" \
    python3 "${repo_dir}/ffi/python/benchmarks/ffi_visibility.py" \
    "${bench_args[@]}" | tee "${python_output}"
}

run_cpp_bench() {
  local release_lib="${target_dir}/release/$(shared_library_name)"
  local cpp_output="${target_dir}/results/cpp.json"
  mkdir -p "$(dirname "${cpp_output}")"
  local bench_args=()
  if [[ "${mode}" == "quick" ]]; then
    bench_args+=(--quick)
  fi

  CITYJSON_LIB_FFI_CORE_SHARED_LIB="${release_lib}" \
    "${cpp_build_dir}/cityjson_lib_cpp_ffi_visibility" \
    "${bench_args[@]}" | tee "${cpp_output}"
}

main() {
  require_tools
  require_header
  ensure_release_library
  build_cpp_bench
  run_rust_bench
  run_python_bench
  run_cpp_bench
}

main
