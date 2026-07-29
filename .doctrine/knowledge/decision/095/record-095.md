# DEC-095: PrimaryRoot newtype resolves repo-scoped runtime state

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Repo-scoped runtime state resolves through a **`PrimaryRoot` newtype minted once
in the impure shell** and threaded down, not through a git call inside the path
constructor.

- `PrimaryRoot::resolve(cwd)` performs the single `crate::git::primary_worktree`
  call. It is **total**: no repo ⇒ the given root is its own primary (ASM, and
  the reason ~20 `tempfile::tempdir()` tests keep passing).
- `phases_dir(primary: &PrimaryRoot, slice_id)` (`src/state.rs:135`) stays a
  **pure, total path join** — it merely takes an already-resolved value.

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
here rather than left implicit. A one-line test helper keeps the churn to a
single wrapper call per site.

Decided by the user in `/design`, 2026-07-29.
