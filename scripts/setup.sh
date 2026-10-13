#!/usr/bin/env bash
# Fetch missing game data and build the native desktop game. Re-runnable.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

scripts/fetch-data.sh
scripts/native-build.sh

echo "All set. Launch the desktop game with: make start"
