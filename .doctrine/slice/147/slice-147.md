# Audit path-conformance delta

## Context

RFC-004 ("Path-intent selector") diagnoses the review `domain_map` as a dead
authoring tax: hand-authored once, cold, by the reviewer, and **never read
back** except for one weak staleness path. Its prose tier (areas/invariants/
risks) has zero runtime readers (RFC-004 OQ-5, *settled — dead*); only the
path-set tier feeds `review status` → `stale_paths`.

The RFC's escape is to stop trying to cheapen *production* of an unread artifact
and instead surface a **mechanical consumer**: at audit, compute the drift
between what the design *declared* it would touch and what git shows it *actually*
touched. That diff is the "killer consumer" — high signal, no prose, retroactively
justifying the structure.

This slice implements RFC-004's **v0.1 — constrained-but-sufficient scope**: the
smallest thing that delivers the north star — *conformance drift between declared
design-targets and git actuals is computed, not hand-hunted* — for four roles
(slice author, design author, reviewer, auditor), and nothing else. It is the
prove-value prototype the RFC's "Approach to proving value" calls for; if the diff
yields signal an auditor would otherwise find by hand, the primitive is justified
and later consumers follow.

Load-bearing constraint: **POL-002** (platform independence). The slice-delta must
be computed from contracts doctrine owns — **recorded source-delta SHAs** captured
at land-time — never by grepping a host's `(SL-NNN)` commit convention (RFC-004
OQ-11/11a, *resolved*). This is POL-002's originating worked example.

## Scope & Objectives

**North star.** `doctrine` can emit, against one slice, the set algebra of
declared design-targets vs git actual-delta:
`declared ∩ actual` (conformant) · `declared \ actual` (undelivered) ·
`actual \ declared` (undeclared / surprise).

In scope (the four roles RFC-004 v0.1 names):

1. **Declared-target data shape.** One authored, committed list per slice. Each
   entry: `path` (path | glob — PathRef only), `intent` (`scope-relevant` |
   `design-target`), `note?` (optional one line, anchored — NOT the old prose
   tier). No per-`PHASE-NN` attribution; no create/modify/delete verb from the
   author (git supplies the verb); `layer` collapses into `intent`. CLI to seed
   and refine entries.
   - *slice author* seeds coarse `scope-relevant` globs.
   - *design author* refines to specific `design-target` paths — the load-bearing
     input the audit diff keys on.

2. **Source-delta SHA recording (the owned contract).** Capture each phase's
   single source-delta commit SHA as **runtime/derived state at land-time**
   (RFC-004 OQ-11a). Capture points: the dispatch funnel (`integrate`, the
   one-commit-per-phase beat, src/dispatch.rs) and the solo `/execute` landing
   path. `actual-delta = ⋃ name-status of the recorded SHAs` (each `sha^..sha`),
   so interleaved trunk merges (the edge/main dance) contribute nothing and no
   base ref is needed. Squash-durable: SHAs live on preserved code branches
   (ADR-012).

3. **Auditor consumer (new CLI verb).** Resolve `design-target` globs → set; take
   git `name-status` over the recorded SHAs → set; emit the algebra. There is **no
   `doctrine audit` CLI today** — this is a new read-only verb. Reuses the existing
   resolve+hash seam (`tracked_paths`/`baseline`/`git_text`, src/review.rs).

4. **Reviewer re-point.** Re-point the existing live reader (`review status` →
   `stale_paths`, src/review.rs:2682) at the unified declared list instead of the
   hand-authored `domain_map`, so the staleness reader keeps working for zero new
   authoring tax while the dead prose authoring is removed. *(Boundary ambiguity —
   see OQ-A below; design settles exactly how much of `domain_map` v0.1 retires.)*

## Non-Goals

Explicitly deferred by RFC-004 v0.1 (goal in back of mind, not in this slice):

- Target sum type / non-entity edge generalization (`EntityRef | InternalDocRef |
  PathRef | ExternalRef`, OQ-6) — **PathRef only** here.
- Per-`PHASE-NN` attribution — slice-level declaration only (per-phase is the
  obvious v0.2; slice-level diff already delivers the north star).
- Verb sub-tags (create/modify/delete) — author declares intent-to-touch; git
  supplies the verb.
- New verify mode (VG by git delta, OQ-9) — the diff is *evidence* feeding an
  existing VA/VH; conformance gate, not correctness gate.
- Prose invariants/risks tier (the dead `domain_map` prose).
- dispatch disjointness check (consumer #3); IMP-012 backlog-trigger wiring.
- MCP surface — **CLI-first**; a thin reader is plausible but out of v0.1.

Also not a goal: judging *whether* a change is correct. Path-conformance is
necessary, not sufficient — it says *where to look*, not *whether it passes*.

## Affected surface

- `src/review.rs` — resolve+hash seam (`tracked_paths` :2464, `baseline` :2475,
  `cache_staleness` :2514, `run_status` :2682); `Cache`/`CacheArea`/`CacheNote`
  domain_map schema (:2429–2449); `validate_domain_map` :2612. Reviewer re-point
  + reuse for the audit diff.
- `src/slice.rs` — `SliceDoc` (:1226) authored tier; home (or sibling file) for
  the declared-target list + its schema/CLI.
- `src/state.rs` — runtime phase state (`set_phase_status` :363, `phases_dir`
  :133); home for recorded source-delta SHAs.
- `src/dispatch.rs` — `integrate` (:519/:1550), the one-commit-per-phase beat;
  SHA capture point.
- `src/git.rs` — `git_text`/`git_bytes` (:521/:537); `name-status` invocation.
- Solo `/execute` landing path — second SHA capture point.
- `src/root.rs` — new primary-working-tree resolver (git-common-dir → primary
  worktree) so the registry is shared across worktrees (design R5/a).
- A new pure module for the set algebra; lift `worktree/allowlist::glob_matches`
  to a shared leaf (design D6/D7).
- Skills: `/audit` (new diff consumer), `/slice` + `/design` (authoring the
  declared list), `/execute` (record-delta). CLI surface for the new verb(s).

## Verification / closure intent

- Auditor verb, run against **one real already-closed slice**, emits the three
  algebra cells correctly (RFC-004's prove-value test: does the diff surface
  conformance signal a human would otherwise hand-hunt?).
- Slice-delta computed purely from recorded SHAs; no `(SL-NNN)` grep anywhere
  (POL-002 conformance — a reviewer challenge gate).
- Recorded SHAs survive trunk-merge interleaving and squash integration.
- `review status` staleness keeps working off the unified declared list; dead
  prose authoring removed without breaking the live reader.
- Pure/imperative split honoured (set algebra pure; git/disk in the shell).
- `just gate` green.

## Open questions — resolved in design.md

- **OQ-A** *Resolved.* **Burn `domain_map`** — remove the hand-authored input and
  the prose tier; re-point `review status` staleness to resolve the declared
  selector list (design D4). One path-set surface; the dead anchor goes.
- **OQ-B** *Resolved.* `[[selector]]` table in `slice-NNN.toml` on `SliceDoc`
  (design D2). Entry noun: **selector** (path|glob neutral).
- **OQ-C** *Resolved.* `doctrine slice selector add|note|list|rm` (batch-first,
  variadic) + `doctrine slice conformance <SL>` (the killer consumer) + `doctrine
  slice record-delta` (the writer). Staleness stays `review status`, re-pointed.
  Under the `slice` namespace — no `audit` namespace minted (design D3).
- **OQ-D** *Resolved (A + degrade).* Arm-neutral recorded source-delta registry
  reusing ledger's `BoundaryRow`, written by both the dispatch `integrate` beat
  and a solo `/execute` `record-delta` call; absent → honest "audit manually"
  degrade. Registry resolves against the **primary working tree** (new `root`
  helper) so all worktrees share one file (design D5, R5 option a).

Intent vocabulary settled minimal: `scope-relevant` vs `design-target`; git's
A/M/D carries the actual-side change verb (design D1). MCP reader deferred to a
mechanical fast-follow.

## Summary

Implement RFC-004 v0.1: a slice-level declared-target list (PathRef + intent),
recorded source-delta SHAs as the owned slice-delta contract, and a new auditor
verb that computes declared-vs-actual path conformance — proving the killer
consumer before the broader path-intent primitive is extracted.

## Follow-Ups

- v0.2: per-`PHASE-NN` attribution; phase EX criterion `L1 ∩ git L2`.
- Target sum type / non-entity edge generalization (OQ-6).
- Further consumers: dispatch disjointness, IMP-012 triggers, MCP reader,
  all-RV-format surfaces (inquisition / code-review, IMP-042).
