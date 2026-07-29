# DEC-098: Migrate boundaries_path onto PrimaryRoot in this slice

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

`boundaries_path` (`src/state.rs:716`) migrates onto `PrimaryRoot` (DEC-095) **in
this slice**, as its own phase so it stays separable if it turns awkward:

```rust
// before
fn boundaries_path(cwd: &Path, slice_id: u32) -> anyhow::Result<PathBuf>
// after
fn boundaries_path(primary: &PrimaryRoot, slice_id: u32) -> PathBuf
```

It becomes **infallible**: the fallible half moves out to `PrimaryRoot::resolve`,
which is total (no repo ⇒ the given root is its own primary).

Surface: 3 production sites in `state.rs` (`:759`, `:790`, `:858`), reached via
`read_source_deltas` / `record_source_delta`, which have ~7 external production
callers (`slice.rs:839,1186,2855,3028`; `dispatch.rs:1877,3454,3471`).

## Why in-slice rather than deferred

- Deferring ships SL-237 with **two primary-resolution mechanisms coexisting in
  one module** — the asymmetry the slice exists to remove, relocated rather than
  removed.
- Objective 1 ("make runtime-state scope explicit at the path constructor") is
  only half delivered if the *other* repo-scoped constructor keeps resolving
  implicitly.
- Concrete win: `src/dispatch.rs:3454,3471` already resolve `primary_worktree`
  themselves and pass `&primary`, which `boundaries_path` then resolves **a
  second time** — a redundant git subprocess per call today. The migration
  deletes it.
- `set_phase_status` currently costs a separate boundaries-side resolution;
  afterwards it shares the one `PrimaryRoot` its caller already holds.

## Contract change to verify during the phase

A non-repo `cwd` currently makes `read_source_deltas` **error**; afterwards it
returns an **empty registry**.

- `slice.rs:839` and `slice.rs:1186` already swallow it (`if let Ok(rows) = …`),
  so they are unaffected — an empty registry is what they already fall through to.
- `slice.rs:2855` and `slice.rs:3028` **propagate** and must be checked: confirm
  that "no repo ⇒ empty registry" is correct for each, rather than silently
  converting a real misconfiguration into a benign-looking empty result.

Decided by the user in `/design`, 2026-07-29.
