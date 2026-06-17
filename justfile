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

# Root package only — fast.
test:
  cargo test

# Whole workspace incl. cordage — slow; used by the end-of-phase gate.
test-all:
  cargo test --workspace

install: web-build
  cargo install --path .

publish:
  cargo publish
