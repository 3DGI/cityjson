#!/usr/bin/env bash
# Emit the set of crates affected by a push, given a diff range.
#
# Inputs (env):
#   GITHUB_EVENT_NAME     "push" | "pull_request" | "workflow_dispatch"
#   GITHUB_EVENT_BEFORE   parent SHA for push events; empty or all-zeros on
#                         first push / force push (triggers full-suite fallback)
#   GITHUB_SHA            current SHA
#
# Outputs (appended to $GITHUB_OUTPUT when set, else printed to stdout):
#   matrix       JSON array of crate names to test, e.g. ["cityjson-fake"]
#   any          "true" if anything to run, "false" for docs-only changes
#   run_python   "true" if cityjson-lib or cityjson-index is in the set
#
# Non-push events (PR, manual dispatch) always emit the full crate list.
# A docs-only change on main emits matrix=[], any=false.
#
# Local usage:
#   GITHUB_EVENT_NAME=push \
#   GITHUB_EVENT_BEFORE=<sha> GITHUB_SHA=HEAD \
#   bash .github/scripts/affected-crates.sh

set -euo pipefail

ALL_CRATES=(cityjson cityjson-json cityjson-arrow cityjson-parquet cityjson-lib cityjson-fake cityjson-index)
PYTHON_CRATES=(cityjson-lib cityjson-index)

# Downstream closure for each crate (including the crate itself).
# Keep in sync with the dep graph in README.md.
declare -A CLOSURE
CLOSURE[cityjson]="cityjson cityjson-json cityjson-arrow cityjson-parquet cityjson-lib cityjson-fake cityjson-index"
CLOSURE[cityjson-json]="cityjson-json cityjson-lib cityjson-fake cityjson-index"
CLOSURE[cityjson-arrow]="cityjson-arrow cityjson-parquet cityjson-lib cityjson-fake cityjson-index"
CLOSURE[cityjson-parquet]="cityjson-parquet cityjson-lib cityjson-fake cityjson-index"
CLOSURE[cityjson-lib]="cityjson-lib cityjson-fake cityjson-index"
CLOSURE[cityjson-fake]="cityjson-fake"
CLOSURE[cityjson-index]="cityjson-index"

emit() {
    local matrix_json any run_python
    matrix_json="$1"
    any="$2"
    run_python="$3"
    if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
        {
            echo "matrix=${matrix_json}"
            echo "any=${any}"
            echo "run_python=${run_python}"
        } >>"$GITHUB_OUTPUT"
    fi
    echo "matrix=${matrix_json}"
    echo "any=${any}"
    echo "run_python=${run_python}"
}

emit_full() {
    local matrix_json
    matrix_json=$(printf '%s\n' "${ALL_CRATES[@]}" | jq -R . | jq -s -c .)
    emit "$matrix_json" "true" "true"
}

emit_empty() {
    emit "[]" "false" "false"
}

emit_set() {
    # Args: space-separated crate names, possibly with duplicates.
    local -a crates
    read -r -a crates <<<"$(printf '%s\n' "$@" | sort -u | tr '\n' ' ')"
    if [[ ${#crates[@]} -eq 0 ]]; then
        emit_empty
        return
    fi
    local matrix_json
    matrix_json=$(printf '%s\n' "${crates[@]}" | jq -R . | jq -s -c .)
    local run_python="false"
    for c in "${crates[@]}"; do
        for p in "${PYTHON_CRATES[@]}"; do
            if [[ "$c" == "$p" ]]; then
                run_python="true"
                break 2
            fi
        done
    done
    emit "$matrix_json" "true" "$run_python"
}

event_name="${GITHUB_EVENT_NAME:-push}"

if [[ "$event_name" != "push" ]]; then
    emit_full
    exit 0
fi

before="${GITHUB_EVENT_BEFORE:-}"
sha="${GITHUB_SHA:-HEAD}"

# First push / force push / missing parent → safe fallback.
if [[ -z "$before" || "$before" =~ ^0+$ ]]; then
    emit_full
    exit 0
fi

if ! git cat-file -e "${before}^{commit}" 2>/dev/null; then
    emit_full
    exit 0
fi

mapfile -t changed < <(git diff --name-only "$before" "$sha")

if [[ ${#changed[@]} -eq 0 ]]; then
    emit_empty
    exit 0
fi

declare -A selected
workspace_change="false"

classify() {
    local path="$1"
    case "$path" in
        # Workspace-level triggers → full suite.
        Cargo.toml|Cargo.lock|rust-toolchain.toml|justfile|release.toml)
            workspace_change="true" ;;
        .github/workflows/ci.yml|.github/scripts/*)
            workspace_change="true" ;;

        # Docs-only paths → no jobs.
        README.md|CONTRIBUTING.md|CHANGELOG.md|LICENSE|LICENSE-*)
            : ;;
        docs/*)
            : ;;
        crates/*/README.md|crates/*/CHANGELOG.md|crates/*/LICENSE|crates/*/LICENSE-*)
            : ;;

        # Per-crate paths → crate closure.
        crates/cityjson/*)         for c in ${CLOSURE[cityjson]};         do selected[$c]=1; done ;;
        crates/cityjson-json/*)    for c in ${CLOSURE[cityjson-json]};    do selected[$c]=1; done ;;
        crates/cityjson-arrow/*)   for c in ${CLOSURE[cityjson-arrow]};   do selected[$c]=1; done ;;
        crates/cityjson-parquet/*) for c in ${CLOSURE[cityjson-parquet]}; do selected[$c]=1; done ;;
        crates/cityjson-lib/*)     for c in ${CLOSURE[cityjson-lib]};     do selected[$c]=1; done ;;
        crates/cityjson-fake/*)    for c in ${CLOSURE[cityjson-fake]};    do selected[$c]=1; done ;;
        crates/cityjson-index/*)   for c in ${CLOSURE[cityjson-index]};   do selected[$c]=1; done ;;

        # Unknown path → conservative full suite.
        *)
            workspace_change="true" ;;
    esac
}

for path in "${changed[@]}"; do
    classify "$path"
    [[ "$workspace_change" == "true" ]] && break
done

if [[ "$workspace_change" == "true" ]]; then
    emit_full
    exit 0
fi

emit_set "${!selected[@]}"
