# Implementation Plan SL-237: Primary-resolve runtime phase state

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases, in `design.md` § 8a's order. PHASE-01 is the slice: it introduces
the two root capability types and migrates every phase-state function to them,
which is what actually single-homes repo-scoped runtime state in the primary
worktree. The remaining four phases are consequences — a patch retired
(PHASE-02), a resolver narrowed (PHASE-03), a contract change discharged
(PHASE-04), and the projection surfaces closed (PHASE-05).

The design was locked after five external review rounds (RV-322, 18 findings) and
one internal pass. The plan does not re-open it. Where a criterion looks like an
argument rather than a criterion, it is carrying a finding's reasoning forward so
the implementer does not have to re-derive it — and, more often, so they do not
undo it. Several criteria are explicitly *anti*-criteria: do not tidy
`single_commit_boundary`, do not sweep sibling worktrees, do not generically
migrate `integrity.rs`. Each names the finding that made it necessary.

PHASE-01 does not start until **REV-043 is `approved`**. That gate is authored
(`SL-237 needs REV-043`, RV-322 F-7) and is EN-1 on PHASE-01 — opening a REV is
not a mitigation if implementation can start alongside it.

## Sequencing & Rationale

**Why PHASE-01 is not split.** It is the largest phase by a wide margin: two new
types, twelve signature changes, two fan-out sites, and roughly fifty test
construction sites. The obvious split — land the types with their unit tests,
then ripple the call sites — was considered and rejected on a mechanical ground.
Clippy runs without `--all-targets`, so it compiles the library without
`cfg(test)`; a types-only phase whose only consumers are test modules trips
`dead_code` and fails `doctrine check gate`. The project's remedy for that is
`#[cfg_attr(not(test), expect(dead_code, …))]`, which would mean adding six
transient lint attributes across a brand-new public-facing API and removing them
one phase later. That is churn on the exact surface reviewers most need to read
cleanly. § 8a's lumping is therefore kept, and the size is managed by criteria
instead: EX-6 enumerates every § 5.2 row this phase owns, so a skipped function
is visible in one read rather than inferred from a green suite.

**Why the ordering is forced, not stylistic.** Each phase's entry criterion names
a real dependency:

- PHASE-02 cannot precede PHASE-01. `mirror_completion_into_primary` is the
  patch standing in for single-homing; deleting it before the sheets actually
  home in the primary reintroduces ISS-212 verbatim.
- PHASE-03 cannot precede PHASE-01. The truth resolver's coord branch is dead
  *because* sheets are single-homed. Narrowing first would delete a live path.
- PHASE-04 cannot precede PHASE-01. `boundaries_path` can only stop resolving git
  internally once its callers already hold a resolved root to hand in.
- PHASE-05 consumes the root types, so it follows PHASE-01 too.

Only PHASE-02..05 are mutually independent, and DEC-098 deliberately made
PHASE-04 separable. If phases are dispatched in parallel, 02/03/04/05 are the
candidates; 01 is a barrier.

**Where the plan makes a call the design left to it.** `design.md` § 5.2 assigns
each function a root type; § 8a assigns phases, but does not name every function.
Two assignments are the plan's:

1. **`read_source_deltas` is PHASE-04, not PHASE-01.** § 5.2 groups it with the
   pure readers and calls it "verified git-free". That is true only *after*
   DEC-098: today it inherits `boundaries_path`'s `primary_worktree` fork. The
   claim describes the post-PHASE-04 function, so the migration belongs there,
   next to the contract change that makes it true.
2. **`capture_phase_boundary` and `record_source_delta` are PHASE-01**, even
   though they touch the registry rather than phase sheets. Their change is the
   F-3 two-root split — a *signature* concern — not DEC-098's contract concern,
   and § 8a puts the two-root split in phase 1.

3. **The `init_phases` / `refresh_symlink` root sets are split across PHASE-01
   and PHASE-05.** § 5.2 gives `init_phases` one root and `refresh_symlink` two,
   and they are caller and *sole* callee (`state.rs:219`) — so the two rows
   cannot both hold at once. PHASE-01 honours the `init_phases` row as written,
   which incidentally delivers DEC-097's *mint* half: with a `&PrimaryRoot` in,
   the symlink lands in the primary from that phase onward. PHASE-05 then adds
   the second param, because *retirement* is the first thing that needs the
   invoked tree. Splitting it this way means no § 5.2 row is contradicted at the
   phase that owns it. Worth naming that the second root here is not a `git_cwd`:
   retiring a link is a filesystem op on the invoked tree, not a git query, so it
   borrows F-3's shape without being an F-3 case.

Consequence: PHASE-01 leaves a transitional redundancy. Those two functions are
handed a resolved primary which `boundaries_path` then re-resolves internally.
It is correct but wasteful — `primary_worktree` forks `git worktree list` per
call, uncached — and PHASE-04's EX-4 exists specifically so it is removed rather
than becoming permanent. It is recorded here so a reviewer of PHASE-01 reads it
as scheduled, not as a defect.

**`registry_completeness` takes `&PrimaryRoot` and that is not an R9 instance.**
R9 protects read verbs that have *no* git dependency today from gaining a fatal
one. `registry_completeness` is not one: it already calls `read_source_deltas`,
which already calls `boundaries_path`, which already fails on a git fault. Its
call sites therefore gain nothing new from a fallible resolve. `slice list` and
the map projection *are* R9 instances, which is why they get `ReadRoot` and why
V-13 pins the degradation instead of leaving it to inference.

**What the verification is shaped against.** The VT mandate proves substrings and
line-anchored patterns; it never runs a test. So each phase's exit criteria
enumerate the exact test function names, the VT patterns anchor on those names at
line start, and every phase carries `VA-NC` (each named test observed red before
green, failing output recorded), `VA-ES` (escape sweep), and `VA-G`
(`doctrine check gate`). `VA-G` is where V-1a — existing suites green, the
behaviour-preservation gate — is discharged; it has no VT row because a mandate
over "the suites" has no single `test_file`.

Two negative controls carry more weight than the rest and are called out in their
rows. `resolve_from_a_linked_worktree_yields_the_primary` must have been red
against an `is_dir()` spelling of the discriminator, because an `is_dir()` bug
passes every primary-rooted test unscathed (R10). And
`reseat_keeps_the_review_kind_on_the_invoked_root` must have been red against a
*generic* migration, not merely against an unmigrated guard — otherwise it is
evidence of nothing having happened rather than of a correctly scoped change.

## Notes

**The live implementation risk is the ripple, not the design.** RV-322 F-5/F-13
proved that a call-site census cannot find every affected site:
`integrity.rs:322-323` hand-builds its state path and never calls `phases_dir`,
so it would have silently stopped guarding. There is no guarantee another such
site exists, and none that this slice will not *add* one. V-12 is therefore
PHASE-05's `VA-1` with the corrected axis written into it — search for
`.doctrine/state` path *construction* — and it must cover sites the slice itself
introduced, not only the pre-existing corpus. Expect this to surface as a red
test rather than a design rewrite.

**Parked, and not to be pulled in.** QUE-199 (REQ-304's "by construction" versus
cooperative confinement) and IMP-354 (a platform-agnostic worker-confinement
wrapper) were argued inside REV-043 and removed at round 5. Neither gates this
slice. ADR-008 D-B3's staleness is named in REV-043 as a follow-up and is not
corrected here.

**Selectors.** `src/integrity.rs` was appended as a design-target selector during
planning; the design named it (F-5/F-13, PHASE-05) but the design-time selector
list did not carry it. `src/root.rs` is deliberately *not* a selector: PHASE-01
rides `root::find_from` without changing it.

**On editing this plan.** Phase ids and `EN-`/`EX-`/`VT-`/`VA-` ids are immutable.
If a phase turns out to be wrong, append — never renumber. Array order is
execution order.
