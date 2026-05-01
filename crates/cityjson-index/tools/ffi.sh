#!/usr/bin/env bash

set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_dir=$(CDPATH= cd -- "${script_dir}/.." && pwd)
workspace_root=$(CDPATH= cd -- "${repo_dir}/../.." && pwd)
core_manifest="${repo_dir}/ffi/core/Cargo.toml"
python_dir="${repo_dir}/ffi/python"
python_dist_dir="${python_dir}/dist"
lib_python_dist_dir="${workspace_root}/crates/cityjson-lib/ffi/python/dist"

usage() {
    cat >&2 <<'EOF'
Usage:
  ./tools/ffi.sh check
  ./tools/ffi.sh fmt
  ./tools/ffi.sh doc
  ./tools/ffi.sh clean
  ./tools/ffi.sh test
  ./tools/ffi.sh test python
  ./tools/ffi.sh ci
  ./tools/ffi.sh build core
  ./tools/ffi.sh build python
EOF
    exit 1
}

build_core() {
    cargo build --manifest-path "${core_manifest}" \
        --release \
        --lib \
        --target-dir "${repo_dir}/target"
}

build_python() {
    cd "${repo_dir}"
    uv build ffi/python/ --wheel --out-dir ffi/python/dist "$@"
}

check() {
    cargo check --manifest-path "${repo_dir}/Cargo.toml" \
        --workspace \
        --all-targets \
        --all-features
}

fmt() {
    cargo fmt --manifest-path "${repo_dir}/Cargo.toml" \
        -p cityjson-index \
        -p cityjson-index-ffi-core
}

doc() {
    cargo doc --manifest-path "${repo_dir}/Cargo.toml" \
        --workspace \
        --no-deps \
        --all-features
}

test_core() {
    build_core
    cargo test --manifest-path "${core_manifest}" \
        --all-features \
        --target-dir "${repo_dir}/target"
}

test_python() {
    build_python

    tmp_root=$(mktemp -d)
    trap 'rm -rf "$tmp_root"' EXIT INT TERM HUP

    out_dir="${tmp_root}/dist"
    cache_dir="${tmp_root}/uv-cache"
    mkdir -p "${out_dir}" "${cache_dir}"
    cp "${lib_python_dist_dir}"/*.whl "${out_dir}"

    cd "${python_dir}"
    UV_CACHE_DIR="${cache_dir}" UV_FIND_LINKS="${out_dir}" uv run tox run
}

clean() {
    cd "${repo_dir}"
    cargo clean --target-dir target
    rm -rf "${python_dist_dir}" "${python_dir}/build"
}

ci() {
    check
    fmt
    test_core
    test_python
    doc
}

case "${1:-}" in
    build)
        shift
        case "${1:-}" in
            core)
                shift
                [[ $# -eq 0 ]] || usage
                build_core
                ;;
            python)
                shift
                build_python "$@"
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
    clean)
        shift
        [[ $# -eq 0 ]] || usage
        clean
        ;;
    test)
        shift
        case "${1:-}" in
            "")
                [[ $# -eq 0 ]] || usage
                test_core
                ;;
            python)
                shift
                [[ $# -eq 0 ]] || usage
                test_python
                ;;
            *)
                usage
                ;;
        esac
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
