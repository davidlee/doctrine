mod? doctrine '.doctrine/doctrine.just'

default: lint test install

installer:
  doctrine install -y
  npx skills add davidlee/doctrine --agent universal -y

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

ff:
  git fetch . edge:main

force-push-main:
  git push . edge:main

# Push local edge and main to origin — works from any branch
push-upstream:
  git push origin edge:refs/heads/edge main:refs/heads/main

# Run before a version bump / tag — this is where flake breakage (a new embed
# root absent from the crane source graft, a toolchain skew) actually bites.
# Pre-release gate: full workspace gate + hermetic nix flake build.
release-check: gate nix-build

# Pass an explicit X.Y.Z or a level: `just release 0.6.0` / `just release minor`.
# Refuses a dirty Cargo.toml/Cargo.lock so the commit is the bump alone. Tags
# locally; push and `just publish` stay manual.
# Cut a release: bump version, run the pre-release gate, commit + tag the chore.
release bump:
  #!/usr/bin/env bash
  set -euo pipefail
  git diff --quiet -- Cargo.toml Cargo.lock || { echo "release: Cargo.toml/Cargo.lock already modified — commit or revert first" >&2; exit 1; }
  case "{{bump}}" in
    major|minor|patch) cargo set-version -p doctrine --bump "{{bump}}" ;;
    *)                 cargo set-version -p doctrine "{{bump}}" ;;
  esac
  version="$(cargo pkgid -p doctrine | sed 's/.*[#@]//')"
  git rev-parse -q --verify "refs/tags/v${version}" >/dev/null && { echo "release: tag v${version} already exists" >&2; exit 1; }
  just release-check
  git add Cargo.toml Cargo.lock
  git commit -m "chore: v${version}"
  git tag "v${version}"
  echo "released v${version} (committed + tagged) — push with: git push && git push --tags"

publish: release-check
  cargo publish
