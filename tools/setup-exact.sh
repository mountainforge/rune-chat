#!/usr/bin/env bash
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
sdk="$root/.deps/exact"
rev=d5a51c9bb37efeecdd81fa0539adc2fe611d91b6
command -v git >/dev/null
command -v bun >/dev/null
command -v rustup >/dev/null
mkdir -p "$root/.deps"
if [[ ! -d "$sdk/.git" ]]; then
  git clone https://github.com/expo/exact.git "$sdk"
  git -C "$sdk" checkout --detach "$rev"
fi
if [[ "$(git -C "$sdk" rev-parse HEAD)" != "$rev" ]]; then
  echo "Exact SDK revision differs from the tested pin; use a separate checkout or restore it yourself." >&2
  exit 1
fi
export EXACT2="$sdk"
(cd "$sdk" && bun scripts/exact.mjs setup)
(cd "$root/apps/exact-client" && bun install --frozen-lockfile && bun exact.mjs update)
