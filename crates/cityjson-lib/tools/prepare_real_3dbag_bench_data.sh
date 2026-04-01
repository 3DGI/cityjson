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
base_arrow_output="${output_root}/${base_tile}.arrow-ipc"
base_parquet_output="${output_root}/${base_tile}.parquet"
merged_output="${output_root}/cluster_4x.city.json"
merged_arrow_output="${output_root}/cluster_4x.arrow-ipc"
merged_parquet_output="${output_root}/cluster_4x.parquet"
manifest_output="${output_root}/manifest.json"
shared_release_root="${shared_corpus_root}/artifacts/acquired/3dbag/${release_path}"
shared_merged_output="${shared_release_root}/cluster_4x.city.json"
shared_base_arrow_output="${shared_release_root}/${base_tile}.arrow-ipc"
shared_base_parquet_output="${shared_release_root}/${base_tile}.parquet"
shared_merged_arrow_output="${shared_release_root}/cluster_4x.arrow-ipc"
shared_merged_parquet_output="${shared_release_root}/cluster_4x.parquet"

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

copy_package_dir() {
  local source_dir="$1"
  local output_dir="$2"
  rm -rf "${output_dir}"
  cp -R "${source_dir}" "${output_dir}"
}

has_package_dir() {
  local path="$1"
  [[ -f "${path}/manifest.json" ]]
}

ensure_native_formats() {
  local input_json="$1"
  local arrow_output="$2"
  local parquet_output="$3"
  local shared_arrow="$4"
  local shared_parquet="$5"
  local export_args=()

  if ! has_package_dir "${arrow_output}" && has_package_dir "${shared_arrow}"; then
    copy_package_dir "${shared_arrow}" "${arrow_output}"
  fi

  if ! has_package_dir "${parquet_output}" && has_package_dir "${shared_parquet}"; then
    copy_package_dir "${shared_parquet}" "${parquet_output}"
  fi

  if has_package_dir "${arrow_output}" && has_package_dir "${parquet_output}"; then
    return
  fi

  if ! has_package_dir "${arrow_output}"; then
    export_args+=(--arrow-dir "${arrow_output}")
  fi

  if ! has_package_dir "${parquet_output}"; then
    export_args+=(--parquet-dir "${parquet_output}")
  fi

  cargo run --quiet --manifest-path "${cjlib_cargo_manifest}" --bin bench_export_formats -- \
    --input "${input_json}" \
    "${export_args[@]}"
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
  "${base_arrow_output}" \
  "${base_parquet_output}" \
  "${shared_base_arrow_output}" \
  "${shared_base_parquet_output}"

ensure_native_formats \
  "${merged_output}" \
  "${merged_arrow_output}" \
  "${merged_parquet_output}" \
  "${shared_merged_arrow_output}" \
  "${shared_merged_parquet_output}"

jq -n -S \
  --arg release_path "${release_path}" \
  --arg base_case "io_3dbag_cityjson" \
  --arg base_description "Pinned real 3DBAG tile from the shared corpus release v20250903." \
  --arg base_path "${base_output}" \
  --arg base_arrow_path "${base_arrow_output}" \
  --arg base_parquet_path "${base_parquet_output}" \
  --arg merged_case "io_3dbag_cityjson_cluster_4x" \
  --arg merged_description "Merged four-tile real 3DBAG workload built from contiguous v20250903 tiles." \
  --arg merged_path "${merged_output}" \
  --arg merged_arrow_path "${merged_arrow_output}" \
  --arg merged_parquet_path "${merged_parquet_output}" \
  --argjson merged_tiles "$(printf '%s\n' "${base_tile}" "${merge_tiles[@]}" | jq -R . | jq -s .)" \
  --argjson base_size "$(stat -c '%s' "${base_output}")" \
  --argjson base_arrow_size "$(du -sb "${base_arrow_output}" | awk '{print $1}')" \
  --argjson base_parquet_size "$(du -sb "${base_parquet_output}" | awk '{print $1}')" \
  --argjson merged_size "$(stat -c '%s' "${merged_output}")" \
  --argjson merged_arrow_size "$(du -sb "${merged_arrow_output}" | awk '{print $1}')" \
  --argjson merged_parquet_size "$(du -sb "${merged_parquet_output}" | awk '{print $1}')" \
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
          arrow_ipc: {
            path: $base_arrow_path,
            byte_size: $base_arrow_size
          },
          parquet: {
            path: $base_parquet_path,
            byte_size: $base_parquet_size
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
          arrow_ipc: {
            path: $merged_arrow_path,
            byte_size: $merged_arrow_size
          },
          parquet: {
            path: $merged_parquet_path,
            byte_size: $merged_parquet_size
          }
        },
        source_tiles: $merged_tiles
      }
    ]
  }
  ' > "${manifest_output}"

echo "prepared ${base_output}"
echo "prepared ${base_arrow_output}"
echo "prepared ${base_parquet_output}"
echo "prepared ${merged_output}"
echo "prepared ${merged_arrow_output}"
echo "prepared ${merged_parquet_output}"
echo "wrote ${manifest_output}"
