#!/usr/bin/env bash
set -euo pipefail

DESCRIPTION="${1:-}"
MODE_DEFAULT="full"
SEED_DEFAULT=""
SIZE_DEFAULT=""

MODE="$MODE_DEFAULT"
SEED_ARG="$SEED_DEFAULT"
SIZE_ARG="$SIZE_DEFAULT"

shift || true

for arg in "$@"; do
    if [[ "$arg" == *=* ]]; then
        key="${arg%%=*}"
        value="${arg#*=}"
        case "$key" in
            mode) MODE="$value" ;;
            seed) SEED_ARG="$value" ;;
            size) SIZE_ARG="$value" ;;
            backend) ;; # backward compatibility: ignored
            *) ;;
        esac
    elif [ -n "$arg" ]; then
        case "$arg" in
            full|fast) MODE="$arg" ;;
            *) ;;
        esac
    fi
done

if [ -z "$DESCRIPTION" ]; then
    echo "Usage: ./tools/perf.sh \"description\" [mode] [seed] [size]" >&2
    exit 1
fi

BACKEND="default"
BENCH_VERSION="v2"
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

target_dir="target/bench"
criterion_dir="$target_dir/criterion"
dhat_dir="$target_dir/dhat"
dhat_file="$dhat_dir/memory.json"

mkdir -p "$dhat_dir"

export CARGO_TARGET_DIR="$target_dir"
export BENCH_MODE="$MODE"
export BENCH_SEED="$SEED"
if [ -n "$SIZE_ARG" ]; then
    export BENCH_SIZE="$SIZE_ARG"
else
    unset BENCH_SIZE
fi

export DHAT_OUTPUT="$(pwd)/$dhat_file"

bench_cmd=(cargo bench --bench builder --bench processor)
if [ "$MODE" = "fast" ]; then
    bench_cmd+=(-- --quick)
fi

echo "=== Benchmarks: backend=$BACKEND mode=$MODE seed=$SEED ==="
"${bench_cmd[@]}"

python3 tools/parse_criterion.py \
    --criterion-dir "$criterion_dir" \
    --timestamp "$TIMESTAMP" \
    --commit "$COMMIT" \
    --description "$DESCRIPTION" \
    --mode "$MODE" \
    --backend "$BACKEND" \
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

memory_cmd=(cargo bench --bench memory)
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
    --backend "$BACKEND" \
    --bench "memory/build_model/$memory_size" \
    --seed "$SEED" \
    --bench-version "$BENCH_VERSION" \
    --rustc "$RUSTC_VERSION" \
    --out "$CSV_OUT"

stream_mode="e2e"
stream_size="$SIZE_ARG"
if [ -z "$stream_size" ]; then
    if [ "$MODE" = "fast" ]; then
        stream_size="1000"
    else
        stream_size="10000"
    fi
fi
stream_batch="${STREAM_BATCH:-1000}"
stream_out="$target_dir/streaming-metrics.json"

stream_cmd=(cargo bench --bench streaming)

STREAM_MODE="$stream_mode" \
STREAM_SIZE="$stream_size" \
STREAM_BATCH="$stream_batch" \
STREAM_OUT="$stream_out" \
"${stream_cmd[@]}"

if [ -f "$stream_out" ]; then
    python3 tools/parse_streaming.py \
        --stream-json "$stream_out" \
        --timestamp "$TIMESTAMP" \
        --commit "$COMMIT" \
        --description "$DESCRIPTION" \
        --mode "$MODE" \
        --backend "$BACKEND" \
        --bench "streaming/$stream_mode" \
        --seed "$SEED" \
        --bench-version "$BENCH_VERSION" \
        --rustc "$RUSTC_VERSION" \
        --out "$CSV_OUT"
else
    echo "streaming metrics not found at $stream_out" >&2
fi

echo "=== Valgrind profiling: backend=$BACKEND bench=processor/compute_full_feature_stats ==="
PROFILE_BENCH="processor" \
    PROFILE_BENCH_ID="compute_full_feature_stats" \
    PROFILE_MODE="$MODE" \
    PROFILE_SEED="$SEED" \
    PROFILE_SIZE="$SIZE_ARG" \
    just profile-bench-all

cachegrind_out=$(find profiling -name cachegrind.out -type f -printf '%T@ %p\n' 2>/dev/null | sort -nr | head -n 1 | cut -d' ' -f2-)
if [ -n "$cachegrind_out" ] && [ -f "$cachegrind_out" ]; then
    python3 tools/parse_cachegrind.py \
        --cachegrind-out "$cachegrind_out" \
        --timestamp "$TIMESTAMP" \
        --commit "$COMMIT" \
        --description "$DESCRIPTION" \
        --mode "$MODE" \
        --backend "$BACKEND" \
        --bench "processor/compute_full_feature_stats" \
        --seed "$SEED" \
        --bench-version "$BENCH_VERSION" \
        --rustc "$RUSTC_VERSION" \
        --out "$CSV_OUT"
else
    echo "cachegrind.out not found; skipping cache metrics" >&2
fi

unset CARGO_TARGET_DIR
unset DHAT_OUTPUT
unset BENCH_MODE
unset BENCH_SEED
unset BENCH_SIZE
