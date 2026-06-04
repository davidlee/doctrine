default: lint test install

lint:
  cargo clippy

build:
  cargo build

test:
  cargo test

install:
  cargo install --path .
