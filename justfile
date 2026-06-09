mod doctrine '.doctrine/doctrine.just'

default: lint test install

check: fmt lint test build

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

test:
  cargo test

install:
  cargo install --path .
