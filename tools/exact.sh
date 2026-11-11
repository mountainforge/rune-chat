#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
export EXACT2="${EXACT2:-$root/.deps/exact}"
cd "$root/apps/exact-client"
exec bun exact.mjs "$@"
