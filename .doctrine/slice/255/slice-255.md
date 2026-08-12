# Provision dispatch workers as clones

## Context

Split out of `SL-254` by `DEC-213` on 2026-08-13, before either slice was
implemented. `SL-254` was originally scoped as one shippable change taking every
deletion the clone forces on the worker side — `DEC-203`'s Bounded Pole B. The
owner judged that line too wide at the close of the exploring stage, on two
grounds that are different in kind: the clone half is a separate concern from
the arm collapse, and what stays useful under clone-backed workers is materially
less clear-cut than what stays useful under a confined subprocess.

The split is along a fault line the two halves already had. `SL-254` is a
**substitution** with a known target — the pi arm's confined-subprocess spawn
runs in production, and the claude delta is rebinding `$HOME/.pi` to
`$HOME/.claude` and changing the exec target. This slice is a **topology
change** that moves the transport, the write floor's caller graph, and the
self-commit trade at once, and it rests on an assumption nobody has verified.

`RFC-025` already schedules this as capsule roadmap step 3 ("raw-clone
provisioning"). `IDE-024` (2026-06-30) reached the same shape independently,
including the self-commit consequence.

### The assumption this slice exists to verify

**`A1` — clone self-commit.** A clone's writable `.git` dissolves the pi arm's
standing trade: *worker cannot self-commit (read-only `.git` for linked
worktrees) → orchestrator imports the working-tree diff*. Inherited from
`SL-254` **still unverified**, and that is why it left. `SL-254` should not have
to clear an unverified topology assumption to land a verified substitution.

Verify `A1` before anything downstream of it is built. If it fails, this slice's
objectives 2 and 3 fail with it and the slice becomes a much smaller thing.

### What `SL-254` already settled and this slice inherits

Do not re-decide these. They are topology-independent and already hold:

- **`DEC-207`** — worker identity is the `DOCTRINE_WORKER` env leg; the disk
  marker is deleted. Verified independent of clone-versus-linked-worktree, so it
  lands in `SL-254` and needs no revisiting here.
- **`DEC-205`**, **`DEC-206`**, **`DEC-208`**, **`DEC-209`**, **`DEC-210`** —
  the in-session apparatus, the jail-primitive re-homing, the one-arm posture,
  the confinement prefix's generalisation, and the credential mount.
- **`DEC-203`** — worktrees and the dispatch workflow are separate capabilities.
  `ADR-012`'s coordination topology stays untouched; solo `/worktree` keeps the
  linked-worktree machinery.

### The decision that becomes live again here

**`DEC-204`** — retire `worker_commit`'s transport and re-home its two
non-topological belts. Its hazard did not arise in `SL-254`, because the
incumbent `classify_import` survived there as the scope belt's enforcing caller.
It arises **here**: replacing the import transport with fetch-from-clone moves
that caller, and the `.doctrine/` / `.claude/` authored-state write floor is
single-sourced (`import.rs:24`) precisely so a second caller cannot drift from
the first. Read `DEC-204` before designing the fetch/admit step — its six-belt
count and its four-topology-coupled / two-not split are the analysis this slice
needs, and they were measured rather than assumed.

## Scope & Objectives

**One shippable change: dispatch workers are provisioned as clones, self-commit
into them, and the orchestrator fetches.**

1. **Verify `A1` first.** A worker in a clone can commit to its own branch, and
   the orchestrator can reach that commit. Everything below is conditional on
   this.
2. **Provision workers as clones, not linked worktrees.** Dispatch stops calling
   the linked-worktree provisioning path; solo `/worktree` keeps it.
3. **Worker self-commit, orchestrator fetch.** `git fetch <clone> <branch>`
   replaces the working-tree-diff import belt. The branch-point guard re-homes
   onto fetched refs — `C^ == B` works the same on a fetched branch.
4. **Re-home the scope belt** to the fetch/admit step per `DEC-204`, taking
   `DOCTRINE_PREFIX`, `CLAUDE_PREFIX` and the `undeclared_paths` predicate from
   the same single source, so `VT-3`'s cannot-diverge property survives with the
   new caller substituted for the old.
5. **Land the governance.** Scope survey is this slice's own to run — see
   *Governance*, which is deliberately not inherited.

## Non-Goals

- **The arm collapse.** `SL-254` owns it: confined-subprocess spawn, env
  identity, the in-session apparatus, the skill merge. This slice assumes it has
  landed.
- **The capsule contract.** No capsule provisioning, admission, conformance, or
  microVM work. `SPEC-030` untouched, `ADR-020` not pre-empted.
- **Solo worktrees.** `/worktree` for non-dispatch isolation stays on linked
  worktrees. Two provisioning paths — clones for dispatch, linked worktrees for
  solo — is the intended terminus, not duplication (`DEC-203`).
- **`REQ-335` / `FR-007`**, the confined-orchestrator mediated-write tier. Stays
  pending.

## Governance

**Not inherited — survey it fresh.** `SL-254`'s governance survey was scoped to
the arm collapse and `DEC-211` narrowed its REV accordingly. The surfaces this
slice touches are a different set, and at least these need checking rather than
assuming:

- **`ADR-012`** — untouched by `SL-254` and load-bearing there. This slice moves
  the transport, so whether its coordination topology and class routing survive
  intact is a real question, not a formality.
- **`SPEC-021`** — its funnel cadence is a governed ordered contract
  (`spec-021.toml:15`): *precondition → delta-check → R-5 belt → import
  (non-committing) → verify → branch-point guard → one coordination commit →
  record*. Replacing import with fetch changes governed prose.
- **`SPEC-012`** — `import` is described as "the belted dispatch funnel"
  (`spec-012.toml:18`); it survived `SL-254` and does not survive this.
- **`ADR-006`** — the orchestrator-sole-writer posture and `INV-2`'s
  worker-cannot-skip-a-belt statement. `DEC-204` reads this as changing to
  orchestrator-enforces-at-admit, which is a governance-visible posture change.

## Risks, assumptions, open questions

- **`R1` — `A1` is unverified and everything rests on it.** Mitigation:
  objective 1 verifies it before anything is built on it.
- **`R2` — the write floor loses its enforcing caller mid-change.** The scope
  belt is single-sourced with two callers; `SL-254` deletes one (`worker_commit`)
  and this slice moves the other. `DEC-204` is the analysis; objective 4 is the
  mitigation. This is the sharpest correctness risk in the slice.
- **`R3` — clone disk and time cost is unmeasured** for this repo. `POL-002`
  forbids baking a local measurement into the platform, so whatever is measured
  informs the design without becoming a platform constant.
- **`OQ-1`** — Does the orchestrator keep a long-lived clone per worker, or
  clone per phase and discard? Cost, staleness and the reap path all turn on it.
- **`OQ-2`** — Does `crates/doctrine-control`'s existing clone-inside-`bwrap`
  implementation (`provision.rs:949`, `backend/bubblewrap.rs:1110`) get reused
  here? `DEC-209` declined to take a dependency on it for the *prefix*; the
  provisioning question is separate and was left open as `QUE-215`.

## Summary

## Follow-Ups
