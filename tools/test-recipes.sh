#!/usr/bin/env bash
# Exercise every `just` recipe in the workspace and report pass/fail.
# Skips recipes that cannot succeed in a non-interactive harness
# (long-running servers, reports that require prior bench output, etc.).
#
# Usage: tools/test-recipes.sh [--include-benches] [--include-skipped]

set -u

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -f "$ROOT/.envrc" ]]; then
    # shellcheck disable=SC1091
    source "$ROOT/.envrc"
fi

INCLUDE_BENCHES=0
INCLUDE_SKIPPED=0
for arg in "$@"; do
    case "$arg" in
        --include-benches)  INCLUDE_BENCHES=1 ;;
        --include-skipped)  INCLUDE_SKIPPED=1 ;;
        -h|--help)
            sed -n '2,9p' "$0"; exit 0 ;;
        *)
            echo "unknown flag: $arg" >&2; exit 2 ;;
    esac
done

# Recipes that cannot pass in an automated sweep.
#   *-serve      : long-running dev servers
#   bench-report-readme : requires prior bench run output
SKIP_ALWAYS=(
    docs-serve site-serve
    bench-report-readme
)

is_skipped() {
    local recipe="$1"
    for s in "${SKIP_ALWAYS[@]}"; do
        [[ "$recipe" == "$s" ]] && return 0
    done
    # `bench-check` is a fast compile-only check — always run it.
    if [[ $INCLUDE_BENCHES -eq 0 && "$recipe" == bench* && "$recipe" != "bench-check" ]]; then
        return 0
    fi
    return 1
}

PASS=()
FAIL=()
SKIP=()

run_justfile() {
    local dir="$1"
    echo ""
    echo "== Recipes in: $dir =="

    # Enumerate recipes, skipping those whose signature declares parameters
    # (variadic `*args`, `+args`, or required/default positional args) — they
    # cannot run with no arguments in an unattended sweep.
    mapfile -t recipes < <(
        just -f "$dir/justfile" --list --unsorted 2>/dev/null \
            | awk '
                /^[[:space:]]+[A-Za-z0-9_-]+/ {
                    line = $0
                    sub(/[[:space:]]+#.*$/, "", line)   # strip trailing comment
                    sub(/^[[:space:]]+/, "", line)
                    if (line ~ /[[:space:]]/) next      # has args — skip
                    print line
                }' \
            | grep -v '^_'
    )

    for recipe in "${recipes[@]}"; do
        if is_skipped "$recipe"; then
            if [[ $INCLUDE_SKIPPED -eq 0 ]]; then
                printf "  %-28s SKIP\n" "$recipe"
                SKIP+=("$dir $recipe")
                continue
            fi
        fi
        printf "  %-28s " "$recipe"
        if ( cd "$dir" && just "$recipe" ) >/tmp/recipe.out 2>&1; then
            echo "PASS"
            PASS+=("$dir $recipe")
        else
            echo "FAIL"
            FAIL+=("$dir $recipe")
        fi
    done
}

run_justfile "$ROOT"
while IFS= read -r -d '' jf; do
    run_justfile "$(dirname "$jf")"
done < <(find "$ROOT/crates" -mindepth 2 -maxdepth 2 -name justfile -print0 | sort -z)

echo ""
echo "=== Summary ==="
echo "  passed:  ${#PASS[@]}"
echo "  failed:  ${#FAIL[@]}"
echo "  skipped: ${#SKIP[@]}"

if ((${#FAIL[@]})); then
    echo ""
    echo "=== Failed ==="
    printf '  %s\n' "${FAIL[@]}"
    exit 1
fi
