#!/usr/bin/env bash
# Build and lint everything: the Rust workspace, the Lua plugins, and the web
# app. Used locally and by .github/workflows/ci.yml — keep this the single
# source of truth for what "passing" means so CI and local runs never drift.
#
# Prereqs: Rust toolchain, wasm-pack, Node + pnpm. The Lua step degrades to a
# syntax-only check when luacheck is missing; nothing else is optional.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> cargo build"
cargo build --workspace --all-targets

echo "==> cargo test"
cargo test --workspace

echo "==> cargo clippy"
cargo clippy --workspace --all-targets -- -D warnings

echo "==> cargo fmt --check"
cargo fmt --check

echo "==> lua lint (crates/cli/plugins/*.lua)"
if command -v luacheck >/dev/null 2>&1; then
  luacheck crates/cli/plugins/*.lua
else
  echo "luacheck not found locally; falling back to syntax-only check (luac -p)."
  echo "CI installs luacheck for full linting (unused vars, shadowing, etc)."
  luac_bin=$(command -v luac5.4 || command -v luac)
  for f in crates/cli/plugins/*.lua; do
    "$luac_bin" -p "$f" && echo "  ok: $f"
  done
fi

# The `jsontolang-wasm` workspace package lives in crates/wasm/pkg/, which is
# generated and gitignored — `pnpm install` cannot resolve it until wasm-pack
# has run, so this must come before any pnpm step.
echo "==> wasm build (crates/wasm/pkg)"
(cd crates/wasm && wasm-pack build --target web)

echo "==> pnpm install"
pnpm install --frozen-lockfile

echo "==> web typecheck"
pnpm --dir apps/web typecheck

echo "==> web lint"
pnpm --dir apps/web lint

echo "==> web build"
pnpm --dir apps/web build

echo "==> all checks passed"
