default: lint test install

check: fmt lint test build

list-memories:
  @cargo run -q -- memory list

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
