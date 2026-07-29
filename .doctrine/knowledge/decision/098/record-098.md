# DEC-098: Migrate boundaries_path onto the root newtype in this slice

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

`boundaries_path` (`src/state.rs:716`) migrates onto the root newtype DEC-095
mints — `&ReadRoot`, the weaker of the two — **in this slice**, as its own phase
so it stays separable if it turns awkward:

```rust
// before
fn boundaries_path(cwd: &Path, slice_id: u32) -> anyhow::Result<PathBuf>
// after
fn boundaries_path(root: &ReadRoot, slice_id: u32) -> PathBuf
```

It becomes **infallible**: the fallible half moves out to the root constructors.

## Amendment 1 (RV-322 F-16) — the fallible half moved to *two* constructors, and the param is `ReadRoot`

This record originally said "the fallible half moves out to
`PrimaryRoot::resolve`, which is total (no repo ⇒ the given root is its own
primary)". Two later corrections supersede that sentence:

- `resolve` is **not** total — DEC-095 amendment 2 (RV-322 F-6) made a genuine
  resolution fault return `Err`; totality survives only for the no-repo
  invariant.
- There are now **two** constructors — DEC-095 amendments 4 and 5 with RV-322
  F-10 split them into `PrimaryRoot::resolve` (fallible, mints the *write*
  capability) and `ReadRoot::resolve_for_read` (total, falls back to `cwd`).

`boundaries_path` is a pure path constructor, so it takes the weaker
**`ReadRoot`**; a writer converts its `PrimaryRoot` on the way in. Nothing about
DEC-098's own decision changes — dropping the `Result` still holds, and the
double-resolution deletion at `dispatch.rs:3454,3471` still lands. Only the name
of the type it takes, and the claim about which constructor is total, were stale.

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
