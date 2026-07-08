mod? doctrine '.doctrine/doctrine.just'

# formt, lint, test, build
default: fmt lint test build

# initial setup for fresh repo
setup: web-build build

## DOCTRINE CHECKS:
##
## doctrine check <type>
##

quick: fmt lint validate

# Fast inner-loop gate — root package only.
check: fmt lint lint-js validate test build

# Full gate for end-of-phase / CI — includes the cordage workspace crate.
gate: fmt lint lint-js validate test-all build

# non-mutating prove-clean cadence — asserts fmt+lint clean, never fixes
prove: fmt-check lint

## Sanity Check
##

validate:
  doctrine prompt check
  doctrine doctor

validate-full: validate

# format rust code
fmt:
  cargo fmt

# assert rust formatting is clean without mutating (the non-mutating twin of `fmt`)
fmt-check:
  cargo fmt --check

# lint rust (aggressive)
lint:
  cargo clippy

# lint js (web/map)
lint-js:
  @if [ -d web/map/node_modules ]; then cd web/map && bun run lint; else echo "lint-js: node_modules not found, skipping (restore with: cd web/map && bun install)"; fi

## Tests
##

# Root package only — fast.
test:
  cargo test

# Whole workspace incl. cordage — slow; used by the end-of-phase gate.
test-all:
  cargo test --workspace

fake-darwin:
  cargo check --target aarch64-apple-darwin


## WEB
##

# Install JS deps (idempotent — fast no-op when already installed).
web-install:
  cd web/map && bun install

# Build the map frontend (typecheck + lint + test + vite build).
web-build: web-install
  cd web/map && bun run build

# Rust map server on port 8080 (matches vite proxy).
map-serve:
  cargo run -- map serve --port 8080

# Vite dev server (proxies /api → localhost:8080). Run `just map-serve` alongside.
web-dev: web-install
  cd web/map && bun run dev

# Fast frontend check (typecheck + lint + test only, no dist).
web-check: web-install
  cd web/map && bun run typecheck && bun run lint && bun run test

# Embed-integrity smoke gate: build a local binary with a freshly re-embedded
# web/map/dist, then run scripts/smoke.sh against the actual shipped bytes
# (--version, the install/ embed, the map embed). Same script the release
# workflow runs on each artifact before upload — single source, no CI duplicate
# (SL-174). `touch` forces rust-embed to re-embed the rebuilt dist.
smoke: web-build
  touch src/map_server/assets.rs
  cargo build
  scripts/smoke.sh ./target/debug/doctrine

## Build
##

# cargo build
build:
  cargo build

clean:
  touch crates/cordage/src/lib.rs

## INSTALL
##

# Catches source-filter / asset-embed gaps `cargo build` can't (it reads the real
# web/map/dist on disk; the ix sandbox builds the frontend hermetically). Slow
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
  direnv reload

# Blanks the pin to fakeHash, lets nix report the real `got:`, writes it back.
# Host-only (needs nix); the FOD hash is intrinsic to bun deps and must change
# whenever the lockfile does — this kills the manual `got:` chase.
# Regenerate the webModules FOD hash in flake.nix after web/map/bun.lock changes.
web-hash:
  #!/usr/bin/env bash
  set -uo pipefail
  command -v nix >/dev/null || { echo "web-hash: needs nix (host, not jail)" >&2; exit 1; }
  fake="sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
  sed -i "s|outputHash = \"sha256-[^\"]*\"|outputHash = \"$fake\"|" flake.nix
  got=$(nix build .#doctrine 2>&1 | grep -oP 'got:\s+\K sha256-\S+' | tr -d ' ' | head -1)
  if [ -n "$got" ]; then
    sed -i "s|outputHash = \"$fake\"|outputHash = \"$got\"|" flake.nix
    echo "web-hash: pinned $got"
  else
    echo "web-hash: no got: hash — hash already correct?" >&2
    exit 1
  fi

# install with vite stage
install: web-build
  cargo install --path .

# # list memories
# list-memories:
#   @cargo run -q -- memory list

# doctrine + skills reinstall; idempotent
reinstall:
  doctrine install -y
  npx skills add . --agent universal -y

## GIT
##

# integrate main into edge
ff:
  git merge main
  git fetch . edge:main

promote:
  git fetch . edge:main

# integrate edge into main
push-main:
  git push . edge:main

# lazy .doctrine commit to clear the decks
commit-doctrine:# readme-index
  git reset && git add .doctrine/ && git ci -m doctrine

# Push local edge and main to origin — works from any branch
push-upstream:
  git push origin edge:refs/heads/edge main:refs/heads/main



## RELEASE
##

# # Refresh the spec index in README.md from .doctrine/spec/.
readme-index:
  @scripts/refresh-readme-index.sh


# Run before a version bump / tag — this is where flake breakage (a new embed
# root absent from the crane source graft, a toolchain skew) actually bites.
# Pre-release gate: full workspace gate + hermetic nix flake build.
release-check: gate nix-build

# Pass an explicit X.Y.Z or a level: `just release 0.6.0` / `just release minor`.
# Refuses a dirty Cargo.toml/Cargo.lock so the commit is the bump alone. Tags
# locally; push and `just publish` stay manual.
# NB: pushing the `v*` tag triggers .github/workflows/release.yml, which builds
# and publishes the prebuilt macOS binaries to the GitHub Release (SL-174).
# Cut a release: bump version, run the pre-release gate, commit + tag the chore.
release bump: # readme-index
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
  echo "  (pushing the tag triggers release.yml → prebuilt macOS binaries on the GitHub Release)"

# crates.io release; requires token in ENV.
# --allow-dirty: web/map/dist is gitignored (by choice) but force-included in the
# package (Cargo.toml include), so cargo's VCS check always reads it as
# uncommitted. The flag skips that working-tree walk — which also silences the
# .direnv symlink-loop warnings. release-check (gate + nix-build) is the real
# correctness guard; the tree is otherwise clean.
publish: web-build release-check
  cargo publish --allow-dirty


