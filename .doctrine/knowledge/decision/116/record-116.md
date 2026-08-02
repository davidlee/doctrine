# DEC-116: Adapter crate is linted by a gate-only lint-all, mirroring test-all

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

The justfile gains `lint-all: cargo clippy --workspace`, and `gate` runs it.
`lint`, and therefore `quick` / `check` / `prove`, stays as it is. The Rust
adapter crate (SL-243 `O3`) is linted at the gate under the full workspace lint
set — no exemption.

## Why the adapter would otherwise go unlinted

A workspace member is linted by `cargo clippy` iff it is **built**, and it is
built iff something in the invoked package graph depends on it. `cordage` is
linted today for exactly that reason: it is a workspace member *and* a path
dependency of the root package, so `cargo clippy` at the root runs
`clippy-driver` over it with the full lint set.

`O3` puts the adapter outside the shipped binary, so the root package will not
depend on it. That dependency edge is deliberately absent, and its absence is
what takes the crate out of clippy's reach. Not a policy gap — a mechanical
consequence.

Verified by running `cargo clippy -v` and reading the invocation: `cordage` is
compiled by `clippy-driver` with `--deny=clippy::cargo`, `--deny=clippy::pedantic`
and `--deny=warnings`.

## Why the pairing rather than widening `lint`

The justfile already models exactly this tradeoff for tests: `test` is root-only
for the fast path, `test-all: cargo test --workspace` runs at the gate. Adding
`lint-all` beside it rides that convention instead of minting a new one, and
leaves the inner loop's cost unchanged.

It also closes an asymmetry that already exists: `gate` is workspace-wide for
tests but root-only for lint, so a workspace member could pass the gate with
failing lints today.

Exempting the crate was rejected on what the crate is *for*. It is the reference
implementation of the adapter contract — the thing another project copies — and
the scope carries publishing it as a follow-up. Shipping it unlinted while
offering it as exemplary is not a position that survives being asked about.

## What this obliges

`clippy::cargo` includes `cargo_common_metadata`, so the adapter's manifest owes
`description`, `license`, `repository`, `readme`, `keywords` and `categories`.
`crates/cordage/Cargo.toml` already carries all six and is the template.

The crate is unlinted code until `lint-all` first runs over it, so pedantic sees
it for the first time during implementation rather than after. The phase that
writes the adapter should expect that work rather than treat it as a tidy-up.

## Provenance

Settled at the `inq-7` fork of design run `dr-019fc13a` (SL-243). The research
round recorded that a member absent from the root's `[dependencies]` is never
linted; the mechanism behind it, and the fact that `cordage` *is* linted today,
were verified against `cargo clippy -v` while settling the fork.

## Related

- [[mem.pattern.lint.new-workspace-member-cargo-metadata]] — the new-crate lint
  checklist this decision accepts.
