#!/usr/bin/env bash
# Download and prepare Groningen 182-tile corpus for ADR-003 benchmarking
#
# Usage: ./tools/download-groningen-corpus.sh
#
# Environment variables:
#   CITYJSON_GRONINGEN_CORPUS - Directory containing extracted .city.json files
#   CITYJSON_GRONINGEN_CSV   - Path to selection CSV (default: derived from corpus path)
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Default paths
DEFAULT_CITYJSON_DIR="${REPO_ROOT}/target/benchmarks/groningen-182/cityjson"
DEFAULT_CSV_PATH="${REPO_ROOT}/crates/cityjson-index/tools/selection-groningen-182.csv"

# Override from environment
CITYJSON_DIR="${CITYJSON_GRONINGEN_CORPUS:-${DEFAULT_CITYJSON_DIR}}"
CORPUS_ROOT="$(dirname "${CITYJSON_DIR}")"
CSV_PATH="${CITYJSON_GRONINGEN_CSV:-${DEFAULT_CSV_PATH}}"

RAW_DIR="${CORPUS_ROOT}/raw"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

die() {
    log_error "$1"
    exit 1
}

# Check prerequisites
command -v curl >/dev/null 2>&1 || die "curl is required but not installed"
command -v unzip >/dev/null 2>&1 || die "unzip is required but not installed"
command -v cjindex >/dev/null 2>&1 || die "cjindex is required but not installed. Build with: cargo build --release -p cityjson-index"

# Validate CSV
if [ ! -f "${CSV_PATH}" ]; then
    die "CSV file not found at ${CSV_PATH}. Set CITYJSON_GRONINGEN_CSV or ensure the CSV is at ${DEFAULT_CSV_PATH}"
fi

log_info "Using CSV: ${CSV_PATH}"
log_info "CityJSON corpus: ${CITYJSON_DIR}"

# Create directories
mkdir -p "${RAW_DIR}"
mkdir -p "${CITYJSON_DIR}"

# Parse CSV and download files
# CSV format: bladnr,download_link,download_size_bytes,einddatum,id,jaargang_luchtfoto,startdatum
# We skip the header line and process each data line

TOTAL_COUNT=0
SUCCESS_COUNT=0
FAIL_COUNT=0

log_info "Reading CSV and downloading tiles..."

# Use awk to skip header (NR>1) and process each line
while IFS=, read -r bladnr download_link _rest; do
    # Skip header
    if [[ "${bladnr}" == "bladnr" ]]; then
        continue
    fi
    
    TOTAL_COUNT=$((TOTAL_COUNT + 1))
    
    # Extract filename from URL
    filename=$(basename "${download_link}")
    raw_path="${RAW_DIR}/${filename}"
    
    # Check if already downloaded
    if [ -f "${raw_path}" ]; then
        log_info "Already downloaded: ${filename}"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
        continue
    fi
    
    log_info "Downloading ${filename}..."
    
    # Download with retry
    max_retries=3
    retry_count=0
    download_success=false
    
    while [ $retry_count -lt $max_retries ]; do
        if curl -L --silent --show-error -o "${raw_path}.tmp" "${download_link}"; then
            mv "${raw_path}.tmp" "${raw_path}"
            download_success=true
            break
        fi
        retry_count=$((retry_count + 1))
        if [ $retry_count -lt $max_retries ]; then
            log_warn "Retry ${retry_count}/${max_retries} for ${filename}"
            sleep 2
        fi
    done
    
    if [ "${download_success}" = true ]; then
        log_info "Downloaded: ${filename}"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        log_error "Failed to download: ${filename}"
        rm -f "${raw_path}.tmp"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
done < "${CSV_PATH}"

log_info "Download complete: ${SUCCESS_COUNT}/${TOTAL_COUNT} succeeded, ${FAIL_COUNT} failed"

if [ $FAIL_COUNT -gt 0 ]; then
    die "Some downloads failed. Please retry or check your network connection."
fi

# Extract ZIP files to cityjson directory
log_info "Extracting CityJSON files..."

EXTRACTED_COUNT=0
SKIPPED_COUNT=0

for zip_file in "${RAW_DIR}"/*.zip; do
    [ -f "${zip_file}" ] || continue
    
    filename=$(basename "${zip_file}")
    # Extract to CITYJSON_DIR
    if unzip -q -o "${zip_file}" -d "${CITYJSON_DIR}"; then
        EXTRACTED_COUNT=$((EXTRACTED_COUNT + 1))
        log_info "Extracted: ${filename}"
    else
        log_warn "Failed to extract: ${filename}"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
done

# Clean up temporary files
rm -f "${RAW_DIR}"/*.tmp

log_info "Extraction complete: ${EXTRACTED_COUNT} files extracted"

if [ $FAIL_COUNT -gt 0 ]; then
    die "Some extractions failed."
fi

# Count extracted files
ACTUAL_FILES=$(find "${CITYJSON_DIR}" -name "*.city.json" | wc -l)
log_info "Found ${ACTUAL_FILES} CityJSON files in ${CITYJSON_DIR}"

if [ $ACTUAL_FILES -eq 0 ]; then
    die "No CityJSON files found after extraction. Check ZIP contents."
fi

# Index and validate the corpus with cjindex
log_info "Indexing and validating corpus with cjindex..."

if ! cjindex index "${CITYJSON_DIR}" >/dev/null 2>&1; then
    die "Failed to index corpus at ${CITYJSON_DIR}"
fi

if ! cjindex validate "${CITYJSON_DIR}" >/dev/null 2>&1; then
    die "Corpus validation failed at ${CITYJSON_DIR}"
fi

log_info "Corpus indexed and validated successfully"

log_info "Groningen corpus setup complete!"
log_info "Corpus location: ${CITYJSON_DIR}"
log_info "To use: export CITYJSON_GRONINGEN_CORPUS=${CITYJSON_DIR}"
