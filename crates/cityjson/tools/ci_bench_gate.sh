#!/usr/bin/env bash
# Compare benchmark metrics between base branch and PR head.
# Metrics: criterion speed, dhat allocations, cachegrind cache miss rates.
# Exits non-zero if any metric regressed beyond threshold, unless ALLOW_REGRESSION=1.
set -euo pipefail

BASE_REF="${BASE_REF:?BASE_REF must be set (e.g. origin/master)}"
ALLOW_REGRESSION="${ALLOW_REGRESSION:-0}"

# Thresholds (fraction, e.g. 0.05 = 5%)
SPEED_THRESHOLD="${SPEED_THRESHOLD:-0.05}"
ALLOC_THRESHOLD="${ALLOC_THRESHOLD:-0.02}"
CACHE_THRESHOLD="${CACHE_THRESHOLD:-0.10}"

BENCH_MODE="${BENCH_MODE:-fast}"
BENCH_SEED="${BENCH_SEED:-12345}"
TARGET_DIR="target/bench-ci"

run_benchmarks() {
    local label="$1"
    local out_dir="$TARGET_DIR/$label"
    mkdir -p "$out_dir"

    export CARGO_TARGET_DIR="$TARGET_DIR"
    export BENCH_MODE="$BENCH_MODE"
    export BENCH_SEED="$BENCH_SEED"

    # --- Criterion (speed) ---
    echo "[$label] Running criterion benchmarks..."
    local bench_args=()
    if [ "$BENCH_MODE" = "fast" ]; then
        bench_args+=(-- --quick)
    fi
    cargo bench --bench builder --bench processor "${bench_args[@]}" 2>&1 | tail -5

    # Extract mean times from criterion
    python3 -c "
import json, pathlib, sys
criterion_dir = pathlib.Path('$TARGET_DIR/criterion')
results = {}
for est in criterion_dir.rglob('new/estimates.json'):
    bench_id = est.parent.parent.name
    data = json.loads(est.read_text())
    mean_ns = data.get('mean', {}).get('point_estimate')
    if mean_ns is not None:
        results[bench_id] = float(mean_ns)
out = pathlib.Path('$out_dir/criterion.json')
out.write_text(json.dumps(results, indent=2))
print(f'  Wrote {len(results)} criterion results to {out}')
" || echo "  Warning: criterion extraction failed"

    # --- dhat (allocations) ---
    echo "[$label] Running dhat memory benchmark..."
    local dhat_file
    dhat_file="$(pwd)/$out_dir/dhat-heap.json"
    DHAT_OUTPUT="$dhat_file" cargo bench --bench memory 2>&1 | tail -3
    # dhat may write to cwd if DHAT_OUTPUT is ignored
    if [ ! -f "$dhat_file" ] && [ -f "dhat-heap.json" ]; then
        mv dhat-heap.json "$dhat_file"
    fi

    python3 -c "
import json, pathlib, sys
dhat_path = pathlib.Path('$dhat_file')
if not dhat_path.exists():
    # Search for it
    import glob
    candidates = sorted(glob.glob('$TARGET_DIR/**/dhat-heap.json', recursive=True), key=lambda p: pathlib.Path(p).stat().st_mtime, reverse=True)
    if candidates:
        dhat_path = pathlib.Path(candidates[0])
if not dhat_path.exists():
    print('  Warning: dhat output not found', file=sys.stderr)
    sys.exit(0)
data = json.loads(dhat_path.read_text())
results = {}
for key in ['total_bytes', 'total_blocks', 'max_bytes', 'max_blocks']:
    v = data.get(key)
    if v is not None:
        results[key] = float(v)
# Also check pps-based totals
if not results and 'pps' in data:
    tb = sum(e.get('tb', 0) for e in data['pps'] if isinstance(e, dict))
    tbk = sum(e.get('tbk', 0) for e in data['pps'] if isinstance(e, dict))
    gb = sum(e.get('gb', 0) for e in data['pps'] if isinstance(e, dict))
    gbk = sum(e.get('gbk', 0) for e in data['pps'] if isinstance(e, dict))
    results = {'total_bytes': tb, 'total_blocks': tbk, 'max_bytes': gb, 'max_blocks': gbk}
out = pathlib.Path('$out_dir/dhat.json')
out.write_text(json.dumps(results, indent=2))
print(f'  Wrote {len(results)} dhat results to {out}')
" || echo "  Warning: dhat extraction failed"

    # --- Cachegrind (cache efficiency) ---
    echo "[$label] Running cachegrind..."
    local cachegrind_out="$out_dir/cachegrind.out"
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --tool=cachegrind --cache-sim=yes --branch-sim=yes --cachegrind-out-file=$cachegrind_out" \
        cargo bench --bench processor -- --quick --exact compute_full_feature_stats 2>&1 | tail -3

    python3 -c "
import pathlib, sys
cg_path = pathlib.Path('$cachegrind_out')
if not cg_path.exists():
    print('  Warning: cachegrind output not found', file=sys.stderr)
    sys.exit(0)
import json
events = None
summary = None
for line in cg_path.read_text().splitlines():
    line = line.strip()
    if line.startswith('events:'):
        events = line.split()[1:]
    elif line.startswith('summary:'):
        summary = line.split()[1:]
if not events or not summary:
    print('  Warning: could not parse cachegrind output', file=sys.stderr)
    sys.exit(0)
values = {}
for name, val in zip(events, summary):
    try:
        values[name] = float(val.replace(',', ''))
    except ValueError:
        pass
results = {}
pairs = [('d1_miss_rate', 'D1mr', 'Dr'), ('ll_miss_rate', 'DLmr', 'Dr'), ('branch_miss_rate', 'Bcm', 'Bc')]
for metric, numer, denom in pairs:
    n = values.get(numer)
    d = values.get(denom)
    if n is not None and d is not None and d > 0:
        results[metric] = n / d
results['Ir'] = values.get('Ir', 0)
out = pathlib.Path('$out_dir/cachegrind.json')
out.write_text(json.dumps(results, indent=2))
print(f'  Wrote {len(results)} cachegrind results to {out}')
" || echo "  Warning: cachegrind extraction failed"

    unset CARGO_TARGET_DIR
}


echo "=== Running benchmarks on PR head ==="
run_benchmarks "pr"

echo ""
echo "=== Checking out base ($BASE_REF) ==="
PR_SHA=$(git rev-parse HEAD)
git stash --include-untracked -q 2>/dev/null || true
git checkout --detach "$BASE_REF" -q

echo "=== Running benchmarks on base ==="
run_benchmarks "base"

echo ""
echo "=== Restoring PR head ==="
git checkout --detach "$PR_SHA" -q
git stash pop -q 2>/dev/null || true

echo ""
echo "=== Comparing results ==="

REGRESSION_FOUND=0

python3 - "$TARGET_DIR" "$SPEED_THRESHOLD" "$ALLOC_THRESHOLD" "$CACHE_THRESHOLD" <<'PYEOF'
import json, pathlib, sys

target_dir = pathlib.Path(sys.argv[1])
speed_threshold = float(sys.argv[2])
alloc_threshold = float(sys.argv[3])
cache_threshold = float(sys.argv[4])

regressions = []

def compare(label, base_val, pr_val, threshold, higher_is_worse=True):
    """Return (regressed: bool, delta_pct: float)."""
    if base_val is None or base_val == 0:
        return False, 0.0
    if pr_val is None:
        return False, 0.0
    delta = (pr_val - base_val) / abs(base_val)
    regressed = delta > threshold if higher_is_worse else delta < -threshold
    return regressed, delta * 100

def load(path):
    if path.exists():
        return json.loads(path.read_text())
    return {}

base_dir = target_dir / "base"
pr_dir = target_dir / "pr"

# --- Criterion (speed: higher ns = worse) ---
base_crit = load(base_dir / "criterion.json")
pr_crit = load(pr_dir / "criterion.json")
if base_crit and pr_crit:
    print("Speed (criterion, mean ns):")
    for bench_id in sorted(set(base_crit) | set(pr_crit)):
        b = base_crit.get(bench_id)
        p = pr_crit.get(bench_id)
        if b is None or p is None:
            continue
        regressed, delta = compare(bench_id, b, p, speed_threshold, higher_is_worse=True)
        marker = "REGRESSION" if regressed else "ok"
        print(f"  {bench_id}: {b:.0f} -> {p:.0f} ({delta:+.1f}%) [{marker}]")
        if regressed:
            regressions.append(f"speed/{bench_id}: {delta:+.1f}%")

# --- dhat (allocations: higher = worse) ---
base_dhat = load(base_dir / "dhat.json")
pr_dhat = load(pr_dir / "dhat.json")
if base_dhat and pr_dhat:
    print("\nAllocations (dhat):")
    for metric in ["total_blocks", "max_blocks", "total_bytes", "max_bytes"]:
        b = base_dhat.get(metric)
        p = pr_dhat.get(metric)
        if b is None or p is None:
            continue
        regressed, delta = compare(metric, b, p, alloc_threshold, higher_is_worse=True)
        marker = "REGRESSION" if regressed else "ok"
        print(f"  {metric}: {b:.0f} -> {p:.0f} ({delta:+.1f}%) [{marker}]")
        if regressed:
            regressions.append(f"alloc/{metric}: {delta:+.1f}%")

# --- Cachegrind (miss rates: higher = worse; Ir: higher = worse) ---
base_cg = load(base_dir / "cachegrind.json")
pr_cg = load(pr_dir / "cachegrind.json")
if base_cg and pr_cg:
    print("\nCache efficiency (cachegrind):")
    for metric in ["d1_miss_rate", "ll_miss_rate", "branch_miss_rate", "Ir"]:
        b = base_cg.get(metric)
        p = pr_cg.get(metric)
        if b is None or p is None:
            continue
        regressed, delta = compare(metric, b, p, cache_threshold, higher_is_worse=True)
        if metric in ("d1_miss_rate", "ll_miss_rate", "branch_miss_rate"):
            print(f"  {metric}: {b:.6f} -> {p:.6f} ({delta:+.1f}%) [{('REGRESSION' if regressed else 'ok')}]")
        else:
            print(f"  {metric}: {b:.0f} -> {p:.0f} ({delta:+.1f}%) [{('REGRESSION' if regressed else 'ok')}]")
        if regressed:
            regressions.append(f"cache/{metric}: {delta:+.1f}%")

print()
if regressions:
    print("Regressions detected:")
    for r in regressions:
        print(f"  - {r}")
    sys.exit(1)
else:
    print("No regressions detected.")
    sys.exit(0)
PYEOF

if [ $? -ne 0 ]; then
    REGRESSION_FOUND=1
fi

if [ "$REGRESSION_FOUND" = "1" ]; then
    if [ "$ALLOW_REGRESSION" = "1" ]; then
        echo "::warning::Performance regression detected but override label is present."
        exit 0
    fi
    echo "::error::Performance regression detected. Add the 'perf-regression-approved' label to override."
    exit 1
fi
