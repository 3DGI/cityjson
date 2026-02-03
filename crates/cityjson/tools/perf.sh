#!/usr/bin/env bash
set -euo pipefail

DESCRIPTION="${1:-}"
BACKEND_DEFAULT="both"
MODE_DEFAULT="full"
SEED_DEFAULT=""
SIZE_DEFAULT=""

BACKEND="$BACKEND_DEFAULT"
MODE="$MODE_DEFAULT"
SEED_ARG="$SEED_DEFAULT"
SIZE_ARG="$SIZE_DEFAULT"
BACKEND_SPLIT="${BACKEND_SPLIT:-1}"

shift || true

for arg in "$@"; do
    if [[ "$arg" == *=* ]]; then
        key="${arg%%=*}"
        value="${arg#*=}"
        case "$key" in
            backend) BACKEND="$value" ;;
            mode) MODE="$value" ;;
            seed) SEED_ARG="$value" ;;
            size) SIZE_ARG="$value" ;;
            *) ;;
        esac
    elif [ -n "$arg" ]; then
        case "$arg" in
            default|nested|both) BACKEND="$arg" ;;
            *) ;;
        esac
    fi
done

if [ -z "$DESCRIPTION" ]; then
    echo "Usage: ./tools/perf.sh \"description\" [backend] [mode] [seed] [size]" >&2
    exit 1
fi

BENCH_VERSION="v1"
DEFAULT_SEED="12345"
SEED="$SEED_ARG"
if [ -z "$SEED" ]; then
    SEED="$DEFAULT_SEED"
fi

DEFAULT_SIZE_MEMORY="7000"
FAST_SIZE_MEMORY="1000"

TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
RUSTC_VERSION="$(rustc --version)"
CSV_OUT="bench_results/history.csv"

HEADER="timestamp,commit,description,mode,backend,bench,metric,value,unit,seed,bench_version,rustc"
if [ ! -f "$CSV_OUT" ]; then
    mkdir -p "$(dirname "$CSV_OUT")"
    echo "$HEADER" > "$CSV_OUT"
else
    CURRENT_HEADER="$(head -n 1 "$CSV_OUT")"
    if [ "$CURRENT_HEADER" != "$HEADER" ]; then
        echo "$HEADER" > "$CSV_OUT"
    fi
fi

run_backend() {
    local backend="$1"
    local target_dir="target/bench-$backend"
    local criterion_dir="$target_dir/criterion"
    local dhat_dir="$target_dir/dhat"
    local dhat_file="$dhat_dir/memory.json"

    mkdir -p "$dhat_dir"

    export CARGO_TARGET_DIR="$target_dir"
    export BENCH_MODE="$MODE"
    export BENCH_SEED="$SEED"
    export BENCH_BACKEND="$backend"
    if [ -n "$SIZE_ARG" ]; then
        export BENCH_SIZE="$SIZE_ARG"
    else
        unset BENCH_SIZE
    fi

    export DHAT_OUTPUT="$(pwd)/$dhat_file"

    bench_cmd=(cargo bench --bench backend_comparison --bench builder --bench processor --features "backend-$backend")
    if [ "$BACKEND_SPLIT" = "1" ] && [ "$backend" = "nested" ]; then
        bench_cmd=(cargo bench --no-default-features --bench backend_comparison --bench builder --bench processor --features "backend-$backend")
    fi
    if [ "$MODE" = "fast" ]; then
        bench_cmd+=(-- --quick)
    fi

    echo "=== Benchmarks: backend=$backend mode=$MODE seed=$SEED ==="
    "${bench_cmd[@]}"

    python3 tools/parse_criterion.py \
        --criterion-dir "$criterion_dir" \
        --timestamp "$TIMESTAMP" \
        --commit "$COMMIT" \
        --description "$DESCRIPTION" \
        --mode "$MODE" \
        --backend "$backend" \
        --seed "$SEED" \
        --bench-version "$BENCH_VERSION" \
        --rustc "$RUSTC_VERSION" \
        --out "$CSV_OUT"

    if [ -n "$SIZE_ARG" ]; then
        memory_size="$SIZE_ARG"
    elif [ "$MODE" = "fast" ]; then
        memory_size="$FAST_SIZE_MEMORY"
    else
        memory_size="$DEFAULT_SIZE_MEMORY"
    fi

    memory_cmd=(cargo bench --bench memory --features "backend-$backend")
    if [ "$BACKEND_SPLIT" = "1" ] && [ "$backend" = "nested" ]; then
        memory_cmd=(cargo bench --no-default-features --bench memory --features "backend-$backend")
    fi
    "${memory_cmd[@]}"

    dhat_input="$dhat_file"
    if [ ! -f "$dhat_input" ]; then
        dhat_found=$(find "$target_dir" -name dhat-heap.json -type f -printf '%T@ %p\n' 2>/dev/null | sort -nr | head -n 1 | cut -d' ' -f2-)
        if [ -n "$dhat_found" ]; then
            dhat_input="$dhat_found"
        fi
    fi
    if [ ! -f "$dhat_input" ] && [ -f "dhat-heap.json" ]; then
        mkdir -p "$dhat_dir"
        cp "dhat-heap.json" "$dhat_file"
        dhat_input="$dhat_file"
    fi

    python3 tools/parse_dhat.py \
        --dhat-json "$dhat_input" \
        --timestamp "$TIMESTAMP" \
        --commit "$COMMIT" \
        --description "$DESCRIPTION" \
        --mode "$MODE" \
        --backend "$backend" \
        --bench "memory/build_model/$memory_size" \
        --seed "$SEED" \
        --bench-version "$BENCH_VERSION" \
        --rustc "$RUSTC_VERSION" \
        --out "$CSV_OUT"

    echo "=== Valgrind profiling: backend=$backend bench=processor/compute_full_feature_stats ==="
    PROFILE_BACKEND="$backend" \
        PROFILE_BENCH="processor" \
        PROFILE_BENCH_ID="compute_full_feature_stats" \
        PROFILE_MODE="$MODE" \
        PROFILE_SEED="$SEED" \
        PROFILE_SIZE="$SIZE_ARG" \
        BACKEND_SPLIT="$BACKEND_SPLIT" \
        just profile-bench-all
}

case "$BACKEND" in
    both)
        run_backend "nested"
        run_backend "default"
        ;;
    default|nested)
        run_backend "$BACKEND"
        ;;
    *)
        echo "Unknown backend: $BACKEND (use default|nested|both)" >&2
        exit 1
        ;;
 esac

unset CARGO_TARGET_DIR
unset DHAT_OUTPUT
unset BENCH_MODE
unset BENCH_SEED
unset BENCH_SIZE
unset BENCH_BACKEND
