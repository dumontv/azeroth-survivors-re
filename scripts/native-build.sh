#!/usr/bin/env bash
# Build the native desktop game; extra arguments are forwarded to cargo build.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
cargo build --locked --release -p warcraft-survivors "$@"
