mod? doctrine '.doctrine/doctrine.just'

default: lint test install

# Fast inner-loop gate — root package only.
check: fmt lint lint-js test build

# Full gate for end-of-phase / CI — includes the cordage workspace crate.
gate: fmt lint lint-js test-all build

list-memories:
  @cargo run -q -- memory list

# Refresh the spec index in README.md from .doctrine/spec/.
readme-index:
  @scripts/refresh-readme-index.sh

fmt:
  cargo fmt

lint:
  cargo clippy

lint-js:
  npx eslint web/map/

# Build the map frontend (typecheck + lint + test + vite build).
web-build:
  cd web/map && bun run build

# Rust map server on port 8080 (matches vite proxy).
map-serve:
  cargo run -- map serve --port 8080

# Vite dev server (proxies /api → localhost:8080). Run `just map-serve` alongside.
web-dev:
  cd web/map && bun run dev

# Fast frontend check (typecheck + lint + test only, no dist).
web-check:
  cd web/map && bun run typecheck && bun run lint && bun run test

build:
  cargo build

# Catches source-filter / asset-embed gaps `cargo build` can't (it reads the real
# web/map/dist on disk; the nix sandbox builds the frontend hermetically). Slow
# first run, crane-cached after. Host-real; a genuine failure exits non-zero, but
# skipped with a notice where nix is absent (bubblewrap jails).
# Validate the hermetic nix flake build.
nix-build:
  #!/usr/bin/env bash
  set -euo pipefail
  if command -v nix >/dev/null 2>&1; then
    nix build .#doctrine --no-link --print-out-paths
  else
    echo "nix-build: nix not on PATH (jail) — skipped" >&2
  fi

# Root package only — fast.
test:
  cargo test

# Whole workspace incl. cordage — slow; used by the end-of-phase gate.
test-all:
  cargo test --workspace

install: web-build
  cargo install --path .

# Run before a version bump / tag — this is where flake breakage (a new embed
# root absent from the crane source graft, a toolchain skew) actually bites.
# Pre-release gate: full workspace gate + hermetic nix flake build.
release-check: gate nix-build

publish: release-check
  cargo publish
