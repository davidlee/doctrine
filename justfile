mod doctrine '.doctrine/doctrine.just'

default: lint test install

# Fast inner-loop gate — root package only.
check: fmt lint test build

# Full gate for end-of-phase / CI — includes the cordage workspace crate.
gate: fmt lint test-all build

list-memories:
  @cargo run -q -- memory list

# Refresh the spec index in README.md from .doctrine/spec/.
readme-index:
  @scripts/refresh-readme-index.sh

fmt:
  cargo fmt

lint:
  cargo clippy

build:
  cargo build

# Root package only — fast.
test:
  cargo test

# Whole workspace incl. cordage — slow; used by the end-of-phase gate.
test-all:
  cargo test --workspace

install:
  cargo install --path .

publish:
  cargo publish
