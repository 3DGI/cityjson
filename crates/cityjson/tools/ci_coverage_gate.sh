#!/usr/bin/env bash
# Compare tarpaulin coverage between base branch and PR head.
# Exits non-zero if coverage decreased, unless ALLOW_DECREASE=1.
set -euo pipefail

BASE_REF="${BASE_REF:?BASE_REF must be set (e.g. origin/master)}"
ALLOW_DECREASE="${ALLOW_DECREASE:-0}"

extract_coverage() {
    cargo tarpaulin --all-features --out Stdout 2>&1 \
        | grep -oP '^\d+\.\d+(?=% coverage)' \
        | tail -1
}

echo "=== Measuring coverage on PR head ==="
PR_COV=$(extract_coverage)
echo "PR coverage: ${PR_COV}%"

echo "=== Checking out base ($BASE_REF) for comparison ==="
PR_SHA=$(git rev-parse HEAD)
git stash --include-untracked -q 2>/dev/null || true
git checkout --detach "$BASE_REF" -q

echo "=== Measuring coverage on base ==="
BASE_COV=$(extract_coverage)
echo "Base coverage: ${BASE_COV}%"

echo "=== Restoring PR head ==="
git checkout --detach "$PR_SHA" -q
git stash pop -q 2>/dev/null || true

echo ""
echo "Base: ${BASE_COV}%  ->  PR: ${PR_COV}%"

DECREASED=$(python3 -c "
base = float('$BASE_COV')
pr = float('$PR_COV')
delta = pr - base
print(f'Delta: {delta:+.2f}%')
exit(0 if pr >= base else 1)
" 2>&1) || FAILED=1

echo "$DECREASED"

if [ "${FAILED:-0}" = "1" ]; then
    if [ "$ALLOW_DECREASE" = "1" ]; then
        echo "::warning::Coverage decreased but override label is present."
        exit 0
    fi
    echo "::error::Coverage decreased. Add the 'coverage-decrease-approved' label to override."
    exit 1
fi

echo "Coverage did not decrease."