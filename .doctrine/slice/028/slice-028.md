# Enact ADR-003 reconcile seam and lifecycle states

> **Cut locked (design session, 2026-06-09).** Scoped down from "build the whole
> reconcile capstone" to a **lifecycle-FSM vertical** plus a **full holistic
> revision of ADR-003**. The reconcile *machinery* (`/reconcile` skill, reconcile
> artefact entity + `slice reconcile` CLI, audit/close deep tuning, closure-gate
> enforcement, coverage derivation/registry/blocks) is **deferred to follow-on
> slices** — they attach to the canon this slice locks. Treat SL-028 as *review
> + refinement of ADR-003*, not straight implementation.

## Context

ADR-003 is **accepted** but largely **unenacted**: it sets the canonical loop
`slice → design → plan → phases → per phase [review] → audit → reconcile →
close` and the normative/observed doctrine behind it, then defers the machinery
honestly (§11). Most of that deferred set already has a home — tech specs
(SL-021), `/review` + review-ledger (IMP-001/ADR-007), `/dispatch` + worktree/
worker (IMP-003/004/ADR-006/008), contracts (deeper-deferred). The piece with
**no home** is the loop's own capstone: the **observe → reconcile → close** seam
(§3–§8) and the lifecycle states §9 leans on. `reconcile` appears across slice
prose as a *manual discipline step*; nothing builds it.

Two things sharpened in design:

- **The slice lifecycle is inert.** `slice-NNN.toml` `status` is hand-edited;
  there is **no transition verb** (`slices-spec.md` § Lifecycle, CLAUDE.md gap).
  The vocabulary `{proposed, ready, started, audit, done, abandoned}` predates
  ADR-003's reconcile step — it has no `reconcile` state, and no `review`.
- **Doctrine deliberately diverges from spec-driver, and ADR-003 half-states
  it.** spec-driver *derives* requirement truth from coverage by precedence
  (`sync`: `requirement.status = f(coverage)`). ADR-003 §4/§5 **forbid exactly
  that** — drift is a *prompt to reconcile*, authority is never rewritten by
  precedence/timestamp/overlay. So doctrine's differentiator is
  **reconcile-as-explicit-authorship vs derive-by-precedence**; ADR-003 commits
  to it but never names the mechanism it rejects. The ADR revision must.

## Scope & Objectives

### Built in this slice

1. **Slice lifecycle FSM.** New authored vocabulary + transition machinery:
   `proposed → design → plan → ready → started → review → audit → reconcile →
   done` (+ `abandoned`). Gates modelled as transitions, except `ready` — the
   lone gate-as-state, the "no code without an approved plan" human handoff.
   `design-ready` is dropped (reaching `plan` *is* design-accepted).
   Predicate-driven back-edges (see Risks). Terminal set stays `{done}`.

2. **Transition verb.** A `slice` verb that advances the FSM (and `abandon`),
   gate-aware (consults conduct `autonomy`, advisory), reusing
   `slice::is_terminal_status` and the edit-preserving authored-TOML status
   transition (`mem.pattern.entity.edit-preserving-status-transition`). Closes
   the CLAUDE.md "no slice lifecycle transition" gap.

3. **Conduct axis (vocabulary + advisory config).** `actor`
   (`agent|self|peer|team`) × `autonomy` (`auto|draft|gate`), assignable per
   state/gate. Home: a **new `doctrine.toml [conduct]`** table (structured
   sibling of `governance.md`). **Advisory v1** — parsed + surfaced, not enforced
   (ADR-003 §8 discipline-now-gate-later). Peer review is expressed as conduct
   *role assignment* (two canonical patterns), needing **no new states**.

4. **Requirement-lifecycle + coverage enums (stubbed).** The two-enum model lands
   as advisory Rust types beside the spec seam (`src/requirement.rs` /
   `src/spec.rs`): requirement-lifecycle (**authored, normative**) vs coverage
   (**observed, evidence**). Self-clearing `dead_code` ahead of consumers
   (`mem.pattern.lint.dead-code-self-clearing-leaf`). **No** derivation, registry,
   or coverage blocks — those are follow-on.

### Canon (the refinement half)

5. **Revise ADR-003** to the **full holistic model**: the slice FSM; the conduct
   axis (and its fold into ADR-006 D8 solo/team); the two-enum requirement/
   coverage engine; and the **explicit-reconcile-vs-derive** principle named
   against spec-driver as the rejected foil. Name what is deferred so the canon
   is not blind to the machinery (the central risk this cut was checked against).

6. **Revise `slices-spec.md` § Lifecycle** to the new FSM vocabulary and the
   (still-deferred-enforcement) transition/closure posture.

## Non-Goals

- **The reconcile machinery itself** — `/reconcile` skill, reconcile artefact
  entity, `slice reconcile` CLI, audit/close deep tuning, closure-gate
  *enforcement*. Named in canon, built in follow-on slices.
- **Coverage derivation / registry / coverage blocks** — the live
  requirement-state engine. Enums are stubbed; the engine is deferred (and is
  doctrine-divergent: explicit, not derived).
- **Conduct enforcement + full knob set** — v1 is advisory parse + surface only.
- **Already-homed ADR-003 machinery** — `/review` (IMP-001/ADR-007), `/dispatch`
  (IMP-003/004/ADR-006/008), tech-spec backfill (SL-021), contracts.
- **ADR-006 branch/reservation work** — already homed; SL-028 only ensures the
  FSM (esp. staleness back-edges) is *coherent with* it, deferring enforcement.

## Affected surface

*(Design refines.)*

- `src/slice.rs` — FSM vocabulary, transition verb, gate/conduct awareness
  (extends `SLICE_STATUSES`, `is_terminal_status`, divergence rollup).
- `src/main.rs` — CLI wiring for the transition verb.
- New conduct config module + `doctrine.toml [conduct]` parse/surface; tie-in to
  `src/boot.rs` and/or `slice show`.
- `src/requirement.rs`, `src/spec.rs` — requirement-lifecycle + coverage enum
  types (advisory).
- `.doctrine/adr/003/adr-003.{toml,md}` — the holistic revision (mechanics —
  amend vs supersede — is a triage item).
- `doc/slices-spec.md` § Lifecycle — vocabulary + posture revision.
- `install/` — `doctrine.toml` template/seed if shipped; any manifest/gitignore
  wiring for the new config (`mem.pattern.install.authored-entity-wiring` if it
  becomes authored).
- `.doctrine/state/boot.md` Core process + routing table — already name the loop;
  verify reconcile is distinct (ADR-003 Verification bullet 1).

## Risks, assumptions, open questions

- **ADR-003 revision mechanics.** Amend-in-place vs supersede with a new ADR
  (ADR-003 is `accepted`). Governance call — `/consult` if convention is unclear.
- **Back-edge semantics.** Stay in-state for corrections that don't invalidate an
  accepted gate; fall back to `started` (re-exec) or `design` (redesign)
  otherwise; `reconcile → design` escalation when reconcile discovers the spec/
  governance *model* itself is inadequate (not mere instance drift).
- **Staleness is surfaced, not enforced.** `ready`/`design` rot when sibling
  slices land past them = ADR-006 branch-point staleness; doctrine flags
  (`⚠`-style), never auto-demotes (§5 surfacing-not-overriding, mirrors memory
  staleness). Detection mechanism may be deferred — flag only.
- **Conduct is advisory v1.** Enums + parse + surface; enforcement deferred. The
  (a)-vs-(b) peer-pattern *default* is a config decision, not a state decision.
- **Enums land unused.** Requirement/coverage types are advisory ahead of their
  engine; self-clearing `dead_code` suppression per the leaf pattern.
- **`audit` already over-reaches the §7 seam** (it currently *writes* `design.md`
  / governance fixes). Tuning that out is **follow-on**, but the ADR revision
  must state the seam so the follow-on has a target.
- **Assumption:** the SL-015 composition seam (member edge + `REQ-NNN` peer,
  edit-preserving append) is the fixed substrate the enums attach to.

## Verification / closure intent

- The slice FSM vocabulary is authored and the transition verb advances it
  (incl. `abandon`), gate-aware, edit-preserving, terminal set `{done}`.
- The conduct axis parses from `doctrine.toml [conduct]` and is surfaced;
  advisory only (no enforcement). Peer patterns expressible as role assignment.
- Requirement-lifecycle + coverage enums exist as types beside the spec seam,
  advisory, with self-clearing suppression.
- ADR-003 revised to the full holistic model (FSM + conduct + two-enum engine +
  explicit-reconcile-vs-derive + deferred-machinery map); `slices-spec.md`
  § Lifecycle reconciled to the new FSM.
- `.doctrine/state/boot.md` Core process + routing table name the loop with
  reconcile distinct from audit and close.
- `just check` green; conventions honoured (storage rule, pure/imperative split,
  outbound-only relations, lint-as-you-go).

## Summary

## Follow-Ups

- `/reconcile` skill + reconcile artefact entity + `slice reconcile` CLI.
- Audit/close deep tuning to the §7 seam + §8 closure gate **enforcement**.
- Coverage derivation + requirement-state registry + coverage blocks (the live
  explicit-reconcile engine).
- Conduct **enforcement** + full knob set + per-run orchestration wiring.
- Staleness detection mechanism for `ready`/`design` (ADR-006 branch-point).
