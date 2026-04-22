#!/usr/bin/env bash

set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
csv_out="${repo_dir}/benches/results/history.csv"
bench_version="${CITYJSON_LIB_BENCH_VERSION:-v2}"
backend="default"
seed="real-3dbag-v20250903"
timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
commit="$(git -C "${repo_dir}" rev-parse --short HEAD 2>/dev/null || echo unknown)"
rustc_version="$(rustc --version)"
criterion_dir="${repo_dir}/target/bench/criterion"
profile_root="${repo_dir}/target/bench-profile"

profile_cases_raw="${PERF_PROFILE_CASES:-io_3dbag_cityjson io_3dbag_cityjson_cluster_4x}"
profile_workloads_raw="${PERF_PROFILE_WORKLOADS:-serde_json-read cityjson_lib-read cityjson-lib-json-read cityarrow-read cityparquet-read}"
profile_iterations="${PERF_PROFILE_ITERATIONS:-1}"
run_massif="${PERF_RUN_MASSIF:-0}"
massif_case="${PERF_MASSIF_CASE:-io_3dbag_cityjson_cluster_4x}"
massif_workload="${PERF_MASSIF_WORKLOAD:-cityjson_lib-read}"

read -r -a profile_cases <<<"${profile_cases_raw}"
read -r -a profile_workloads <<<"${profile_workloads_raw}"

usage() {
  cat >&2 <<'EOF'
Usage:
  ./tools/perf.sh <description> [full|fast]
  ./tools/perf.sh arrow [criterion-args...]
  ./tools/perf.sh profile <time|dhat|cachegrind|massif> <workload> <case> [iterations]
  ./tools/perf.sh check
  ./tools/perf.sh analyze [args...]
  ./tools/perf.sh plot [args...]
EOF
  exit 1
}

require_shared_benchmark_index() {
  if [[ -z "${CITYJSON_LIB_BENCH_SHARED_CORPUS_ROOT:-}" ]]; then
    echo "Set CITYJSON_LIB_BENCH_SHARED_CORPUS_ROOT to your cityjson-corpus checkout." >&2
    exit 1
  fi

  local index="${CITYJSON_LIB_BENCH_SHARED_CORPUS_ROOT}/artifacts/benchmark-index.json"
  if [[ ! -f "${index}" ]]; then
    echo "Benchmark index not found at ${index}" >&2
    exit 1
  fi
}

workload_bench_id() {
  local case_id="$1"
  local workload="$2"
  case "${workload}" in
    serde_json-read) echo "deserialize/${case_id}/serde_json::Value/read" ;;
    cityjson_lib-read) echo "deserialize/${case_id}/cityjson_lib/read" ;;
    cityjson-lib-json-read) echo "deserialize/${case_id}/cityjson_lib::json/read" ;;
    cityarrow-read) echo "deserialize/${case_id}/cityarrow/read" ;;
    cityparquet-read) echo "deserialize/${case_id}/cityparquet/read" ;;
    serde_json-write) echo "serialize/${case_id}/serde_json::Value/write" ;;
    cityjson_lib-write) echo "serialize/${case_id}/cityjson_lib/write" ;;
    cityjson-lib-json-write) echo "serialize/${case_id}/cityjson_lib::json/write" ;;
    cityarrow-write) echo "serialize/${case_id}/cityarrow/write" ;;
    cityparquet-write) echo "serialize/${case_id}/cityparquet/write" ;;
    *)
      echo "unknown workload '${workload}'" >&2
      exit 1
      ;;
  esac
}

run_full_campaign() {
  local -a args=("$@")
  if [[ ${#args[@]} -eq 0 ]]; then
    usage
  fi

  local mode="full"
  local last_index=$(( ${#args[@]} - 1 ))
  case "${args[$last_index]}" in
    full|fast)
      mode="${args[$last_index]}"
      unset 'args[$last_index]'
      ;;
  esac

  local description="${args[*]}"
  if [[ -z "${description}" ]]; then
    usage
  fi

  export CARGO_TARGET_DIR="${repo_dir}/target/bench"
  rm -rf "${criterion_dir}"

  local bench_cmd=(cargo bench --bench throughput --manifest-path "${repo_dir}/Cargo.toml")
  if [[ "${mode}" == "fast" ]]; then
    bench_cmd+=(-- --quick)
  fi

  echo "=== Throughput benchmarks: mode=${mode} ==="
  "${bench_cmd[@]}"

  python3 "${repo_dir}/tools/parse_criterion.py" \
    --criterion-dir "${criterion_dir}" \
    --timestamp "${timestamp}" \
    --commit "${commit}" \
    --description "${description}" \
    --mode "${mode}" \
    --backend "${backend}" \
    --seed "${seed}" \
    --bench-version "${bench_version}" \
    --rustc "${rustc_version}" \
    --out "${csv_out}"

  for case_id in "${profile_cases[@]}"; do
    for workload in "${profile_workloads[@]}"; do
      local bench_id
      bench_id="$(workload_bench_id "${case_id}" "${workload}")"

      echo "=== dhat: case=${case_id} workload=${workload} ==="
      "${repo_dir}/tools/profile_bench.sh" dhat "${workload}" "${case_id}" "${profile_iterations}"
      python3 "${repo_dir}/tools/parse_dhat.py" \
        --dhat-json "${profile_root}/dhat/${case_id}/${workload}/dhat-heap.json" \
        --timestamp "${timestamp}" \
        --commit "${commit}" \
        --description "${description}" \
        --mode "${mode}" \
        --backend "${backend}" \
        --bench "${bench_id}" \
        --seed "${seed}" \
        --bench-version "${bench_version}" \
        --rustc "${rustc_version}" \
        --out "${csv_out}"

      echo "=== cachegrind: case=${case_id} workload=${workload} ==="
      "${repo_dir}/tools/profile_bench.sh" cachegrind "${workload}" "${case_id}" "${profile_iterations}"
      python3 "${repo_dir}/tools/parse_cachegrind.py" \
        --cachegrind-out "${profile_root}/cachegrind/${case_id}/${workload}/cachegrind.out" \
        --timestamp "${timestamp}" \
        --commit "${commit}" \
        --description "${description}" \
        --mode "${mode}" \
        --backend "${backend}" \
        --bench "${bench_id}" \
        --seed "${seed}" \
        --bench-version "${bench_version}" \
        --rustc "${rustc_version}" \
        --out "${csv_out}"
    done
  done

  if [[ "${run_massif}" == "1" ]]; then
    echo "=== massif: case=${massif_case} workload=${massif_workload} ==="
    "${repo_dir}/tools/profile_bench.sh" massif "${massif_workload}" "${massif_case}" "${profile_iterations}"
  fi

  echo "=== plots: description=${description} timestamp=${timestamp} ==="
  uv run --script "${repo_dir}/tools/perf_plot.py" \
    --csv "${csv_out}" \
    --description "${description}" \
    --mode "${mode}" \
    --timestamp "${timestamp}"

  unset CARGO_TARGET_DIR
  echo "wrote ${csv_out}"
}

run_arrow_diagnostic() {
  require_shared_benchmark_index
  cargo bench --bench diagnostic --manifest-path "${repo_dir}/Cargo.toml" "$@"
}

run_profile() {
  require_shared_benchmark_index
  "${repo_dir}/tools/profile_bench.sh" "$@"
}

run_check() {
  cargo bench --all-targets --all-features --no-run
}

run_analyze() {
  python3 "${repo_dir}/tools/perf_analyze.py" "$@"
}

run_plot() {
  uv run --script "${repo_dir}/tools/perf_plot.py" "$@"
}

case "${1:-}" in
  arrow)
    shift
    run_arrow_diagnostic "$@"
    ;;
  profile)
    shift
    run_profile "$@"
    ;;
  check)
    shift
    [[ $# -eq 0 ]] || usage
    run_check
    ;;
  analyze)
    shift
    run_analyze "$@"
    ;;
  plot)
    shift
    run_plot "$@"
    ;;
  help|-h|--help)
    usage
    ;;
  "")
    usage
    ;;
  *)
    run_full_campaign "$@"
    ;;
esac
