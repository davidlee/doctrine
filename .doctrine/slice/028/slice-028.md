# Enact ADR-003 reconcile seam and lifecycle states

## Context

ADR-003 is **accepted** but largely **unenacted**: it sets the canonical loop
`slice → design → plan → phases → per phase [review] → audit → reconcile →
close` and the normative/observed doctrine behind it, then defers the machinery
honestly (§11, "the age, not the intent"). Most of that deferred set already has
a home:

- **Tech specs** (§9, §11) → SL-021.
- **`/review`** — per-phase review, review-ledger kind (§6, §11) → ADR-007 /
  IMP-001.
- **`/dispatch`** — parallel/worktree execution (§10, §11) → ADR-006/008,
  IMP-002/003/004.
- **Contracts** — the deterministic observed-truth corpus (§11) — explicitly the
  *deeper*-deferred piece (efficiency, not concept; language-limited). Out.

The one ADR-003 piece with **no slice and no backlog item** is the loop's own
capstone: the **observe → reconcile → close** seam (§3–§8). `reconcile` appears
across slice prose (design/audit/notes of SL-007, SL-009, SL-011, SL-019,
SL-020…) as a *manual discipline step*, but nothing builds it: there is no
`/reconcile` skill, no `slice reconcile` CLI, no reconcile artefact, and `/audit`
/ `/close` are still tuned to the *slice* lifecycle, not the spec-reconciliation
lifecycle. And the **per-requirement lifecycle/coverage states** that §9's
PROD-vs-TECH strictness depends on (`planned`/`in-progress`/`verified` + a
baseline) are unmodelled — the spec composition seam (SL-015,
`mem.system.spec.composition-seam`) carries no coverage state today.

This slice enacts that capstone: the reconcile seam plus the lifecycle/coverage
states it leans on. It is the move from "ADR-003 by discipline" toward "ADR-003
with machinery" for the reconcile half of the loop (the review/dispatch halves
remain with their own slices).

## Scope & Objectives

1. **`/reconcile` skill — the sole spec-reconciliation writer (§7).** Authors a
   new skill that consumes the reconciliation context `/audit` assembled and the
   spec changes `/audit` *identified*, then **writes** those spec edits against
   observed truth. It is the single point in the loop where specs regain
   authority (§3, §7). Distinct from `/audit` (identifies, never writes) and from
   `/close` (confirms coherence, never writes specs).

2. **The reconcile artefact — durable record of what reconcile changed and why.**
   The `spec-driver` *revision* analog (name provisional per §11). Captures the
   reconciled outcome so closure is auditable. **Its schema is deferred by
   ADR-003** — settling it (lightweight prose log vs full authored entity kind;
   where it attaches: slice↔spec) is the central design decision here, and may
   warrant its own ADR (altitude check below).

3. **`slice reconcile` CLI — the artefact's producer verb.** Parallels the known
   `slice audit` scaffold gap (CLAUDE.md "known CLI gaps"): today `audit.md` is
   hand-made; reconcile should not repeat that. Shape follows from the artefact
   decision (2).

4. **Audit → close skill tuning (the §7 seam, §8 gate).**
   - `/audit`: tighten so it **identifies** the spec changes and **assembles** the
     reconciliation context — and explicitly does **not** write spec edits.
   - `/close`: refuse a terminal status while owning specs remain drifted (§8).
     Discipline-by-skill now; a command gate is enforcement and ADR-003 §11 marks
     enforcement deferred — design decides build-now vs discipline-only.

5. **Per-requirement lifecycle / coverage states (§9).** Model
   `planned`/`in-progress`/`verified` plus a baseline (`spec-driver`'s
   `asserted`/`legacy_verified`) on the spec composition seam. §9 hinges on
   this: PROD specs may carry planned intent ahead of implementation *provided*
   state distinguishes `planned` from `verified`; TECH specs reconcile from
   observed. Where the state lives — the mobile per-spec membership edge row vs
   the durable `REQ-NNN` peer entity — is a design decision (coverage is
   plausibly per-membership; baseline plausibly per-requirement).

## Non-Goals

- **Already-homed ADR-003 machinery** — `/review` + review-ledger
  (IMP-001/ADR-007), `/dispatch` + worktree/worker (IMP-003/004/ADR-006/008),
  tech-spec backfill (SL-021). Linked as neighbours, not built here.
- **Contracts** — the deferred observed-truth corpus (§11). Out; reconcile leans
  on audit close-reading as ADR-003 intends.
- **Re-deciding ADR-003 canon** — ADR-003 is the governing constraint, not
  revised here. (A *new* ADR for the reconcile-artefact schema is in-scope to
  *propose* if the altitude check demands it.)
- **Reconciling specs that don't exist yet** — tech specs are pending (SL-021),
  so the live reconcile targets today are the PRD corpus + (later) tech specs.
  This slice builds the *mechanism*; it does not backfill targets. Ordering
  tension flagged below.

## Affected surface

*(Tentative — design refines; some items drop depending on the artefact
decision.)*

- `plugins/doctrine/skills/reconcile/SKILL.md` — **new** skill (currently unbuilt
  per §11).
- `plugins/doctrine/skills/audit/SKILL.md`, `.../close/SKILL.md` — tuning to the
  §7 seam / §8 gate.
- `.doctrine/state/boot.md` routing table + Core process — already name the loop;
  verify they reflect reconcile **distinct** from audit/close (ADR-003
  Verification bullet 1).
- `src/` — reconcile artefact entity if it becomes an authored kind (new tree +
  `entity.rs` materialiser reuse; install wiring per
  `mem.pattern.install.authored-entity-wiring`), `slice reconcile` command,
  lifecycle/coverage state fields on the spec seam (`src/spec.rs`,
  `src/requirement.rs`, `src/registry.rs`).
- `install/` manifest dir + `.gitignore` negation for any new authored kind
  (`mem.pattern.install.authored-entity-wiring`).
- Templates under `install/templates/` for the new skill / artefact.

## Risks, assumptions, open questions

- **Altitude — does the reconcile artefact need an ADR?** ADR-003 leaves the
  artefact's schema and the closure-gate's enforcement open as project-global
  shape decisions. If reconcile becomes a new authored entity kind, that is
  plausibly ADR-level (cf. ADR-006/007 each precede their IMP slices). **Resolve
  in `/design`; do not pre-commit to "just a skill".**
- **Slice likely sprawls — split candidate.** Five objectives spanning skill
  authoring, a new entity + CLI, two skill retunings, and a spec-seam state
  change is large (cf. IMP-001 "largest of the four — likely multi-phase"). It
  may want to split (artefact+CLI / skill seam tuning / lifecycle states) or
  shed the lifecycle states back to their own slice. Design settles the cut.
- **Coverage-state placement.** Edge row (mobile, per-spec membership — matches
  `label`/`order` already on the edge) vs `REQ-NNN` peer (durable identity).
  Coverage may be per-membership, baseline per-requirement — needs design.
- **Closure-gate enforcement.** §11 marks enforcement deferred. Building a hard
  command gate now may overreach ADR-003's stated posture; discipline-by-skill
  may be the correct v1. Design decides.
- **Ordering vs SL-021.** Reconcile's richest targets are tech specs, which don't
  exist yet. The mechanism is still worth building (PRD corpus is a live target,
  and the loop is incomplete without it), but live end-to-end exercise is thin
  until SL-021 lands. Assumption: build mechanism now, accept limited live
  targets.
- **Assumption:** the SL-015 composition seam (member edge + `REQ-NNN` peer,
  edit-preserving `toml_edit` append) is the fixed substrate the coverage state
  attaches to.

## Verification / closure intent

- A `/reconcile` skill exists, is the **sole** spec-reconciliation writer, reads
  the audit-assembled context, and writes spec edits against observed truth
  (ADR-003 Verification bullets 3–4, 7).
- `/audit` **identifies** spec changes + assembles reconciliation context and
  does **not** write spec edits; the seam is hard (§7).
- `/close` refuses a terminal status while owning specs remain drifted —
  discipline or command gate per the design decision (§8).
- The reconcile artefact records what changed and why; its producer verb emits it;
  `validate`/`show` (if an entity) reassemble it cleanly.
- Per-requirement lifecycle/coverage states are modelled such that §9's
  PROD-carries-planned-intent vs TECH-reconciled-from-observed posture is
  expressible.
- `.doctrine/state/boot.md` Core process + routing table name the loop with
  reconcile distinct from audit and close.
- `just check` green; conventions honoured (storage rule — no derived data in
  prose; pure/imperative split; outbound-only relations).

## Summary

## Follow-Ups
