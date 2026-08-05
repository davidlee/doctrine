# `requires` is inert on an arg that is a required ArgGroup member

`#[arg(long, requires = "commit")]` does **not** produce a clap error when
`--commit` is absent, if `commit` is declared `required = true` inside an
`ArgGroup` that a sibling flag can satisfy.

## Why

Observed on `slice record-delta` (`src/slice.rs`), whose two modes share an
`ArgGroup`:

```rust
#[arg(long, group = RECORD_DELTA_MODE, required = true)]
commit: Option<String>,
#[arg(long, group = RECORD_DELTA_MODE, required = true, requires = "end")]
start: Option<String>,
#[arg(long, requires = "commit")]   // <- inert
force: bool,
```

`doctrine slice record-delta N PHASE-01 --start A --end B --force` exits 0.
`--start` satisfies the group, and clap reads `commit`'s requirement as already
met — so `force`'s `requires` never fires.

This is NOT a bool-flag limitation. The same crate's `slice conformance` has
`strict: bool` with `requires = "against"` against a plain non-group
`Option<String>`, and that one errors correctly. The group membership is the
difference.

## How to apply

When the target of a `requires` is a required member of an `ArgGroup`, do not
trust clap to enforce it. Validate at the top of the handler and say why:

```rust
if force && commit.is_none() {
    anyhow::bail!("--force is only meaningful with --commit — …");
}
```

Pin it with a test that runs the binary; a unit test on the parser will not
distinguish the two. The codebase already prefers handler-level `anyhow::bail!`
for cross-flag rules (`memory.rs` status/`--by`, `memory edit`'s at-least-one-flag).

## Related

[[mem.pattern.doctrine.conformance-needs-a-correct-boundary-row]] — the boundary
practice `--force` guards.

[[mem_019fa8f1ade17f31822539fa80d778f4]] — the sibling clap trap: a global arg
and a same-id subcommand arg coexist silently. Same shape of failure — clap
accepts a declaration that reads as enforcement and quietly enforces nothing.
