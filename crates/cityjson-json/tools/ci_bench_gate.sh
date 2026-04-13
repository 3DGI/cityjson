#!/usr/bin/env bash
# Compare criterion benchmark metrics between base branch and PR head.
# Exits non-zero if any metric regressed beyond threshold, unless ALLOW_REGRESSION=1.
set -euo pipefail

BASE_REF="${BASE_REF:?BASE_REF must be set (e.g. origin/master)}"
ALLOW_REGRESSION="${ALLOW_REGRESSION:-0}"

SPEED_THRESHOLD="${SPEED_THRESHOLD:-0.05}"

TARGET_DIR="target/bench-ci"

run_benchmarks() {
    local label="$1"
    local out_dir="$TARGET_DIR/$label"
    mkdir -p "$out_dir"

    export CARGO_TARGET_DIR="$TARGET_DIR"

    echo "[$label] Running criterion benchmarks..."
    cargo bench --bench read --bench write -- --quick 2>&1 | tail -5

    python3 -c "
import json, pathlib
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

python3 - "$TARGET_DIR" "$SPEED_THRESHOLD" <<'PYEOF'
import json, pathlib, sys

target_dir = pathlib.Path(sys.argv[1])
speed_threshold = float(sys.argv[2])

regressions = []

def compare(base_val, pr_val, threshold):
    if base_val is None or base_val == 0 or pr_val is None:
        return False, 0.0
    delta = (pr_val - base_val) / abs(base_val)
    return delta > threshold, delta * 100

def load(path):
    if path.exists():
        return json.loads(path.read_text())
    return {}

base_dir = target_dir / "base"
pr_dir = target_dir / "pr"

base_crit = load(base_dir / "criterion.json")
pr_crit = load(pr_dir / "criterion.json")
if base_crit and pr_crit:
    print("Speed (criterion, mean ns):")
    for bench_id in sorted(set(base_crit) | set(pr_crit)):
        b = base_crit.get(bench_id)
        p = pr_crit.get(bench_id)
        if b is None or p is None:
            continue
        regressed, delta = compare(b, p, speed_threshold)
        marker = "REGRESSION" if regressed else "ok"
        print(f"  {bench_id}: {b:.0f} -> {p:.0f} ({delta:+.1f}%) [{marker}]")
        if regressed:
            regressions.append(f"speed/{bench_id}: {delta:+.1f}%")

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
