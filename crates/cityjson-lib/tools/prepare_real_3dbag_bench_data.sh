#!/usr/bin/env bash

set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
shared_corpus_root="${CJLIB_BENCH_SHARED_CORPUS_ROOT:-${repo_dir}/../cityjson-benchmarks}"
output_root="${CJLIB_BENCH_DATA_ROOT:-${repo_dir}/target/bench-data}/3dbag/v20250903"
release_path="v20250903"

base_tile="10-758-50"
merge_tiles=(
  "10-756-48"
  "10-756-50"
  "10-758-48"
)

base_output="${output_root}/${base_tile}.city.json"
merged_output="${output_root}/cluster_4x.city.json"
manifest_output="${output_root}/manifest.json"

for tool in curl gunzip jq uvx; do
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

copy_or_download_tile "${base_tile}"
for tile in "${merge_tiles[@]}"; do
  copy_or_download_tile "${tile}"
done

uvx --from cjio cjio \
  "${base_output}" \
  merge "${output_root}/10-756-48.city.json" \
  merge "${output_root}/10-756-50.city.json" \
  merge "${output_root}/10-758-48.city.json" \
  save "${merged_output}"

jq -n -S \
  --arg release_path "${release_path}" \
  --arg base_case "io_3dbag_cityjson" \
  --arg base_description "Pinned real 3DBAG tile from the shared corpus release v20250903." \
  --arg base_path "${base_output}" \
  --arg merged_case "io_3dbag_cityjson_cluster_4x" \
  --arg merged_description "Merged four-tile real 3DBAG workload built from contiguous v20250903 tiles." \
  --arg merged_path "${merged_output}" \
  --argjson merged_tiles "$(printf '%s\n' "${base_tile}" "${merge_tiles[@]}" | jq -R . | jq -s .)" \
  --argjson base_size "$(stat -c '%s' "${base_output}")" \
  --argjson merged_size "$(stat -c '%s' "${merged_output}")" \
  '
  {
    release_path: $release_path,
    cases: [
      {
        id: $base_case,
        description: $base_description,
        path: $base_path,
        byte_size: $base_size
      },
      {
        id: $merged_case,
        description: $merged_description,
        path: $merged_path,
        byte_size: $merged_size,
        source_tiles: $merged_tiles
      }
    ]
  }
  ' > "${manifest_output}"

echo "prepared ${base_output}"
echo "prepared ${merged_output}"
echo "wrote ${manifest_output}"
