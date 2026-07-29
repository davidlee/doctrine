# DEC-095: PrimaryRoot newtype resolves repo-scoped runtime state

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Repo-scoped runtime state resolves through a **`PrimaryRoot` newtype minted once
in the impure shell** and threaded down, not through a git call inside the path
constructor.

- `PrimaryRoot::resolve(cwd)` performs the single `crate::git::primary_worktree`
  call. **Total only for the designed invariant** — no repo at all ⇒ the given
  root is its own primary (ASM, and the reason ~20 `tempfile::tempdir()` tests
  keep passing) — and **fallible for a genuine resolution fault** (amendment 2).
- `PrimaryRoot::resolve_for_read(cwd)` — the **total** read counterpart
  (amendment 4).
- `PrimaryRoot::assume(path)` — the caller asserts this path is already the
  primary. **Test seam only** (amendment 5, which supersedes amendment 3's
  production claim).
- `phases_dir(primary: &PrimaryRoot, slice_id)` (`src/state.rs:135`) stays a
  **pure, total path join** — it merely takes an already-resolved value.

## Amendment 1 (RV-322 F-3) — the parameter splits; the newtype survives

**The original form of this decision was wrong in the same way SL-237 exists to
fix.** `project_root` was one identifier serving **two meanings**:

1. *where the state file lives* → the primary, and
2. *which worktree's git am I asking* → the invoked tree.

`capture_phase_boundary` uses it for both — `live_worktree_for_ref(project_root,
…)` (`src/state.rs:617`) and `resolve_ref(project_root, "HEAD")` (`:629`),
reached from `set_phase_status`. Replacing that single parameter with
`&PrimaryRoot` would record the **primary's** HEAD as a solo linked worktree's
source-delta boundary — wrong revision identity, silently.

So the phase-mutation APIs carry **both roots, explicitly**:

```rust
set_phase_status(primary: &PrimaryRoot, git_cwd: &Path, slice_id: u32, …)
```

Pure path constructors (`phases_dir`, `boundaries_path`) take only
`&PrimaryRoot`. Only the functions that genuinely ask git take the second root.
Naming both is the point — the defect was one name for two jobs.

**Applied per function against its actual `git::` calls, not by tier (RV-322-B
F-D / F-E).** The membership is a verified property, not an intuition:
`record_source_delta` **does** ask git (`is_ancestor` at `src/state.rs:776`,
`parents` at `:783`) and so takes both roots, while `edit_phase_sheet` and
`reconcile_phase_status` make **zero** git calls and take the primary alone —
widening them would be this same defect mirrored. `registry_completeness` already
carries two roots that this slice collapses onto one meaning, so it reduces to a
single parameter rather than half-migrating.

## Amendment 2 (RV-322 F-6) — total for the invariant, fallible for a fault

`resolve` must not be unconditionally total. A warning is not a safeguard;
degrading to `cwd` on a genuine git fault would home state in the linked tree —
exactly this slice's bug class at the new seam. The discriminator (a `.git` in
`cwd.ancestors()`) is promoted from *emit a warning* to *return `Err`*. **A
caller that cannot resolve a primary does not get to write primary-owned state.**

## Amendment 3 (RV-322 F-9) — `assume` is a production seam, not a test hatch

The tuple field is private to `git.rs`, so no `state.rs`/`slice.rs` test module
can construct one directly, and routing ~50 unit tests through the impure
`resolve` would turn pure path-join tests into git-dependent ones — changing what
the existing suites prove. `PrimaryRoot::assume(path)` closes that, and has a
real production consumer: `src/dispatch.rs:3449-3471` already resolves
`primary_worktree` itself and passes `&primary`. DEC-098's stated win (deleting
that double resolution) depends on this constructor existing.

## Amendment 4 (RV-322-B F-B) — the fallible constructor is for writers only

Amendment 2 promoted a resolution fault from *warn* to *`Err`* on a write
argument: a caller that cannot resolve a primary must not write primary-owned
state. Correct — but the seam it was promoted at is also crossed by pure reads.
`phases_dir` is a pure join today, so `slice list` (`src/slice.rs:2161`) and the
map projection (`src/lazyspec.rs:535`) carry **no git dependency at all**; a
single fallible `resolve` would newly hard-fail them whenever git is unhappy.
Verified: a linked worktree whose admin dir was pruned keeps its `.git` file, but
`git worktree list` there exits `fatal: not a git repository` — and stale forks
are ordinary residue of this repo's dispatch workflow.

So the consequence is split by caller intent. `resolve` (fallible) for anything
that will write; `resolve_for_read` (total, falls back to `cwd`) for reads. The
fallback is not amendment 2's mistake returning: that mistake was letting a
**write** proceed after a signal. A read degrading to the answer it already gives
today writes nothing, so there is nothing for observability to have prevented.

## Amendment 5 (RV-322-B F-C) — `assume` is a test seam; amendment 3's production consumer is not one

Amendment 3 justified `assume` as "a real production seam, not a test hatch" on
`src/dispatch.rs:3449-3471`. Reading that site refutes it: `prepare_review` does
`let primary = git::primary_worktree(root)?`, which is precisely what
`resolve(root)` does, at identical cost. `resolve` fits there exactly, so
`assume` has **no** production consumer.

Amendment 3's *implementability* argument survives untouched — the private tuple
field still blocks a test module, and routing ~50 unit tests through the impure
`resolve` would still convert pure path-join tests into git-dependent ones. Only
its production claim falls. `assume` is therefore `#[cfg(test)]`: an honest test
hatch is better than an invariant-bypassing public constructor defended by a use
that does not exist. DEC-098's "deletes an existing double resolution" win is
unaffected — it comes from `boundaries_path` no longer re-resolving, not from
this constructor.

## Why this shape and not the obvious one

Two independent constraints reject putting `primary_worktree` inside
`phases_dir`, and both are satisfied by this one shape:

1. **Cost.** `primary_worktree` (`src/git.rs:564`) forks
   `git worktree list --porcelain` per call, uncached. `list_rows`
   (`src/slice.rs:2161`) maps `phase_rollup` over every slice meta — 221 slice
   dirs in this repo — so internal resolution turns `doctrine slice list` from
   zero git subprocesses into ~221.
2. **Layering (ADR-001).** A git subprocess in a leaf path constructor inverts
   leaf ← engine ← command. `boundaries_path` (`src/state.rs:716`) already does
   this; it is precedent for the violation, not for the design.

Two independent constraints selecting one design was the strongest signal in the
pre-design round.

## Alternatives rejected

- **Memoise `primary_worktree`, resolve inside `phases_dir`.** Cheapest diff and
  leaves tests untouched, but keeps the ADR-001 violation, makes `phases_dir`
  fallible or silently swallowing, and delivers none of objective 1. The cache
  must be path-keyed and lives in a test binary whose threads share one process.
- **Per-call-site resolution.** No new type, no test churn — but it re-scatters
  the decision this slice exists to centralise, and nothing stops the next caller
  passing a cwd. That is precisely today's bug; this would be its sixth patch.

## What it buys beyond the fix

Objective 1 — the scope rule becomes visible in the **type**, not inferable only
by reading a function body. The mis-scoping becomes unrepresentable rather than
fixed once. It also gives `boundaries_path` a seam to migrate onto, retro-fixing
the precedent instead of copying it.

## Accepted cost

~10 production call sites thread the value; **~50 test construction sites in
`src/state.rs` change shape**. The pre-design claim that "~20 tests stay green
unchanged" is the behaviour-preservation proof therefore weakens to *assertions
unchanged, construction adapted mechanically* — still a real proof, but named
here rather than left implicit. The churn is one `PrimaryRoot::assume(...)`
wrapper per site (amendments 3 and 5); the "one-line test helper" first proposed
here was unimplementable against the private tuple field.

Decided by the user in `/design`, 2026-07-29.
