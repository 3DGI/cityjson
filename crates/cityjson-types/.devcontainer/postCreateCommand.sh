#!/bin/bash
set -euo pipefail

# Enable debug output
set -x

echo "=== Starting postCreateCommand ==="

# Init Node
. ${NVM_DIR}/nvm.sh
nvm install --lts
pnpm setup

# Install just
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/.local/bin
just --version

# Install Claude
curl -fsSL https://claude.ai/install.sh | bash
claude --version

# Install Codex
pnpm install -g @openai/codex
codex --version

## Create vcpkg directories
#mkdir -p "${VCPKG_ROOT}" "${VCPKG_BINARY_CACHE}"

## Install vcpkg dependencies
#proj_dir="$(pwd)"
#cd "${VCPKG_ROOT}"
#git config --global --add safe.directory /usr/local/vcpkg
#git pull --ff-only
#cd "${proj_dir}"
#vcpkg install

echo "=== postCreateCommand completed ==="
