#!/usr/bin/env bash

set -euo pipefail

if [[ $# -lt 3 || $# -gt 4 ]]; then
  echo "Usage: ./tools/profile_bench.sh <time|dhat|cachegrind|massif> <workload> <case> [iterations]" >&2
  exit 1
fi

tool="$1"
workload="$2"
case_id="$3"
iterations="${4:-1}"

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
output_dir="${repo_dir}/target/bench-profile/${tool}/${case_id}/${workload}"

for required in cargo jq python3; do
  if ! command -v "${required}" >/dev/null 2>&1; then
    echo "missing required tool: ${required}" >&2
    exit 1
  fi
done

mkdir -p "${output_dir}"

bench_executable="$(
  cargo bench --bench profile --no-run --message-format=json --manifest-path "${repo_dir}/Cargo.toml" \
    | jq -r 'select(.target.name == "profile" and .executable != null) | .executable' \
    | tail -n 1
)"

if [[ -z "${bench_executable}" || ! -x "${bench_executable}" ]]; then
  echo "failed to locate built profile benchmark executable" >&2
  exit 1
fi

args=(--workload "${workload}" --case "${case_id}" --iterations "${iterations}")

case "${tool}" in
  time)
    "${bench_executable}" "${args[@]}" | tee "${output_dir}/summary.json"
    ;;
  dhat)
    DHAT_OUTPUT="${output_dir}/dhat-heap.json" \
      "${bench_executable}" "${args[@]}" --profile dhat | tee "${output_dir}/summary.json"
    if [[ ! -f "${output_dir}/dhat-heap.json" && -f "${repo_dir}/dhat-heap.json" ]]; then
      mv "${repo_dir}/dhat-heap.json" "${output_dir}/dhat-heap.json"
    fi
    python3 "${repo_dir}/tools/parse_dhat.py" "${output_dir}/dhat-heap.json" \
      > "${output_dir}/dhat-summary.json"
    ;;
  cachegrind)
    command -v valgrind >/dev/null 2>&1 || {
      echo "missing required tool: valgrind" >&2
      exit 1
    }
    valgrind \
      --tool=cachegrind \
      --cache-sim=yes \
      --branch-sim=yes \
      --cachegrind-out-file="${output_dir}/cachegrind.out" \
      "${bench_executable}" "${args[@]}" | tee "${output_dir}/summary.json"
    python3 "${repo_dir}/tools/parse_cachegrind.py" "${output_dir}/cachegrind.out" \
      > "${output_dir}/cachegrind-summary.json"
    ;;
  massif)
    command -v valgrind >/dev/null 2>&1 || {
      echo "missing required tool: valgrind" >&2
      exit 1
    }
    valgrind \
      --tool=massif \
      --massif-out-file="${output_dir}/massif.out" \
      "${bench_executable}" "${args[@]}" | tee "${output_dir}/summary.json"
    ms_print "${output_dir}/massif.out" > "${output_dir}/massif.txt"
    ;;
  *)
    echo "unknown profiling tool: ${tool}" >&2
    exit 1
    ;;
esac

echo "wrote ${output_dir}"
