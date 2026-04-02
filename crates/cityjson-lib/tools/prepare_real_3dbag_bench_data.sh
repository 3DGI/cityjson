#!/usr/bin/env bash

set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
shared_corpus_root="${CJLIB_BENCH_SHARED_CORPUS_ROOT:-${repo_dir}/../cityjson-benchmarks}"
output_root="${CJLIB_BENCH_DATA_ROOT:-${repo_dir}/target/bench-data}/3dbag/v20250903"
release_path="v20250903"
cjlib_cargo_manifest="${CJLIB_BENCH_EXPORT_CARGO_MANIFEST:-${repo_dir}/Cargo.toml}"

base_tile="10-758-50"
merge_tiles=(
  "10-756-48"
  "10-756-50"
  "10-758-48"
)

base_output="${output_root}/${base_tile}.city.json"
base_cityarrow_output="${output_root}/${base_tile}.cjarrow"
base_cityparquet_output="${output_root}/${base_tile}.cjparquet"
merged_output="${output_root}/cluster_4x.city.json"
merged_cityarrow_output="${output_root}/cluster_4x.cjarrow"
merged_cityparquet_output="${output_root}/cluster_4x.cjparquet"
manifest_output="${output_root}/manifest.json"
shared_release_root="${shared_corpus_root}/artifacts/acquired/3dbag/${release_path}"
shared_merged_output="${shared_release_root}/cluster_4x.city.json"
shared_base_cityarrow_output="${shared_release_root}/${base_tile}.cjarrow"
shared_base_cityparquet_output="${shared_release_root}/${base_tile}.cjparquet"
shared_merged_cityarrow_output="${shared_release_root}/cluster_4x.cjarrow"
shared_merged_cityparquet_output="${shared_release_root}/cluster_4x.cjparquet"

for tool in cargo curl gunzip jq uvx; do
  if ! command -v "${tool}" >/dev/null 2>&1; then
    echo "missing required tool: ${tool}" >&2
    exit 1
  fi
done

mkdir -p "${output_root}"

copy_or_download_tile() {
  local tile_id="$1"
  local y z url shared_path output_path

  y="$(echo "${tile_id}" | cut -d- -f2)"
  z="$(echo "${tile_id}" | cut -d- -f3)"
  url="https://data.3dbag.nl/${release_path}/tiles/10/${y}/${z}/${tile_id}.city.json.gz"
  shared_path="${shared_corpus_root}/artifacts/acquired/3dbag/${release_path}/${tile_id}.city.json"
  output_path="${output_root}/${tile_id}.city.json"

  if [[ -f "${output_path}" ]]; then
    return
  fi

  if [[ -f "${shared_path}" ]]; then
    cp "${shared_path}" "${output_path}"
    return
  fi

  curl -fsSL "${url}" -o "${output_path}.gz"
  gunzip -f "${output_path}.gz"
}

copy_artifact() {
  local source_path="$1"
  local output_path="$2"
  rm -rf "${output_path}"
  cp -R "${source_path}" "${output_path}"
}

has_artifact() {
  local path="$1"
  [[ -f "${path}" ]]
}

validate_artifact() {
  local kind="$1"
  local path="$2"

  case "${kind}" in
    cityarrow)
      cargo run --quiet --manifest-path "${cjlib_cargo_manifest}" --bin bench_validate_formats -- \
        --arrow-file "${path}" >/dev/null 2>&1
      ;;
    cityparquet)
      cargo run --quiet --manifest-path "${cjlib_cargo_manifest}" --bin bench_validate_formats -- \
        --parquet-file "${path}" >/dev/null 2>&1
      ;;
    *)
      echo "unknown artifact kind '${kind}'" >&2
      exit 1
      ;;
  esac
}

ensure_valid_local_artifact() {
  local kind="$1"
  local path="$2"

  if has_artifact "${path}" && ! validate_artifact "${kind}" "${path}"; then
    rm -f "${path}"
  fi
}

ensure_native_formats() {
  local input_json="$1"
  local cityarrow_output="$2"
  local cityparquet_output="$3"
  local shared_cityarrow="$4"
  local shared_cityparquet="$5"
  local export_args=()

  ensure_valid_local_artifact cityarrow "${cityarrow_output}"
  ensure_valid_local_artifact cityparquet "${cityparquet_output}"

  if ! has_artifact "${cityarrow_output}" && has_artifact "${shared_cityarrow}" && validate_artifact cityarrow "${shared_cityarrow}"; then
    copy_artifact "${shared_cityarrow}" "${cityarrow_output}"
  fi

  if ! has_artifact "${cityparquet_output}" && has_artifact "${shared_cityparquet}" && validate_artifact cityparquet "${shared_cityparquet}"; then
    copy_artifact "${shared_cityparquet}" "${cityparquet_output}"
  fi

  if has_artifact "${cityarrow_output}" && has_artifact "${cityparquet_output}"; then
    return
  fi

  if ! has_artifact "${cityarrow_output}"; then
    export_args+=(--arrow-file "${cityarrow_output}")
  fi

  if ! has_artifact "${cityparquet_output}"; then
    export_args+=(--parquet-file "${cityparquet_output}")
  fi

  cargo run --quiet --manifest-path "${cjlib_cargo_manifest}" --bin bench_export_formats -- \
    --input "${input_json}" \
    "${export_args[@]}"

  validate_artifact cityarrow "${cityarrow_output}"
  validate_artifact cityparquet "${cityparquet_output}"
}

copy_or_download_tile "${base_tile}"
for tile in "${merge_tiles[@]}"; do
  copy_or_download_tile "${tile}"
done

if [[ -f "${shared_merged_output}" ]]; then
  cp "${shared_merged_output}" "${merged_output}"
else
  uvx --from cjio cjio \
    "${base_output}" \
    merge "${output_root}/10-756-48.city.json" \
    merge "${output_root}/10-756-50.city.json" \
    merge "${output_root}/10-758-48.city.json" \
    save "${merged_output}"
fi

ensure_native_formats \
  "${base_output}" \
  "${base_cityarrow_output}" \
  "${base_cityparquet_output}" \
  "${shared_base_cityarrow_output}" \
  "${shared_base_cityparquet_output}"

ensure_native_formats \
  "${merged_output}" \
  "${merged_cityarrow_output}" \
  "${merged_cityparquet_output}" \
  "${shared_merged_cityarrow_output}" \
  "${shared_merged_cityparquet_output}"

jq -n -S \
  --arg release_path "${release_path}" \
  --arg base_case "io_3dbag_cityjson" \
  --arg base_description "Pinned real 3DBAG tile from the shared corpus release v20250903." \
  --arg base_path "${base_output}" \
  --arg base_cityarrow_path "${base_cityarrow_output}" \
  --arg base_cityparquet_path "${base_cityparquet_output}" \
  --arg merged_case "io_3dbag_cityjson_cluster_4x" \
  --arg merged_description "Merged four-tile real 3DBAG workload built from contiguous v20250903 tiles." \
  --arg merged_path "${merged_output}" \
  --arg merged_cityarrow_path "${merged_cityarrow_output}" \
  --arg merged_cityparquet_path "${merged_cityparquet_output}" \
  --argjson merged_tiles "$(printf '%s\n' "${base_tile}" "${merge_tiles[@]}" | jq -R . | jq -s .)" \
  --argjson base_size "$(stat -c '%s' "${base_output}")" \
  --argjson base_cityarrow_size "$(stat -c '%s' "${base_cityarrow_output}")" \
  --argjson base_cityparquet_size "$(stat -c '%s' "${base_cityparquet_output}")" \
  --argjson merged_size "$(stat -c '%s' "${merged_output}")" \
  --argjson merged_cityarrow_size "$(stat -c '%s' "${merged_cityarrow_output}")" \
  --argjson merged_cityparquet_size "$(stat -c '%s' "${merged_cityparquet_output}")" \
  '
  {
    release_path: $release_path,
    cases: [
      {
        id: $base_case,
        description: $base_description,
        artifacts: {
          cityjson: {
            path: $base_path,
            byte_size: $base_size
          },
          cityarrow: {
            path: $base_cityarrow_path,
            byte_size: $base_cityarrow_size
          },
          cityparquet: {
            path: $base_cityparquet_path,
            byte_size: $base_cityparquet_size
          }
        }
      },
      {
        id: $merged_case,
        description: $merged_description,
        artifacts: {
          cityjson: {
            path: $merged_path,
            byte_size: $merged_size
          },
          cityarrow: {
            path: $merged_cityarrow_path,
            byte_size: $merged_cityarrow_size
          },
          cityparquet: {
            path: $merged_cityparquet_path,
            byte_size: $merged_cityparquet_size
          }
        },
        source_tiles: $merged_tiles
      }
    ]
  }
  ' > "${manifest_output}"

echo "prepared ${base_output}"
echo "prepared ${base_cityarrow_output}"
echo "prepared ${base_cityparquet_output}"
echo "prepared ${merged_output}"
echo "prepared ${merged_cityarrow_output}"
echo "prepared ${merged_cityparquet_output}"
echo "wrote ${manifest_output}"
