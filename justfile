default: lint test install

list-memories:
  @fd . doc/memories

lint:
  cargo clippy

build:
  cargo build

test:
  cargo test

install:
  cargo install --path .
