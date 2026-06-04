default: lint test install

check: fmt lint test build

list-memories:
  @fd . doc/memories

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
