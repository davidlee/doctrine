# Gate lints and tests one named package list

Supersedes DEC-116, whose mechanism (`cargo clippy --workspace`) was overtaken by RV-353 `F-4` after it was accepted.

## Decision

The justfile defines the gate's package set once, as a variable, and both `test-all` and a new `lint-all` expand it:

```just
gate-packages := "-p doctrine -p cordage -p <adapter>"

lint-all:
  cargo clippy {{gate-packages}}

test-all:
  cargo test {{gate-packages}}
```

`gate` runs `lint-all` in place of `lint`. `lint`, and so `quick` / `check` / `prove`, is unchanged. The SL-243 `O3` adapter crate joins the list, so it is linted **and tested** at the gate under the full workspace lint set, with no exemption.

`crates/doctrine-control` stays off the list. It keeps its own explicitly invoked recipe, `just capsule-check`, which already lints it.

## Context

DEC-116 (2026-08-02) chose `lint-all: cargo clippy --workspace`. SL-248 then added `crates/doctrine-control`, which is Linux-only (ISS-343) and whose rows are proved by live `bwrap` (ISS-342). RV-353 `F-4` removed it from every default selection, and `test-all` now names its packages. `--workspace` would pull the crate back into `gate` and turn it red off-jail.

DEC-116 also covered linting only. A crate linted at the gate but absent from `test-all` would never have its tests run by `gate`.

## Alternatives

- **Name the packages in `lint-all` separately from `test-all`.** Two lists, each of which a new member must be added to, that can drift apart.
- **`--workspace --exclude doctrine-control`.** New members join the gate automatically. That reverses RV-353's opt-in selection, and a future capability-dependent member would turn `gate` red off-jail.

## Consequences

- Adding a gated crate is one edit, and it cannot half-happen.
- A new workspace member is still not auto-gated. The justfile comment above `test-all` moves to the variable and keeps saying so.
- DEC-116's remaining obligation carries over: the adapter manifest owes its `cargo_common_metadata` fields, and pedantic sees the crate for the first time in the phase that writes it.