# Orchestration worker-safety: accidental ref-corruption guard + OQ-D reframe

Reframes (does **not** "close") ADR-012 **OQ-D** (RV-023 F-2), the redress carried
as **IMP-065**. Pairs a cheap anti-accident guard with a **Revision** that retracts
the over-claimed "positive marker is the real close" framing. Successor concern to
SL-064. Standing residual tracked as **RSK-014**.

## Context — and the correction

ADR-012 deferred OQ-D as "the real close" of the dispatch impersonation gap
(ADR-011 D6/M2): an **unstamped worker** (stamp-hook failure / matcher drift —
ISS-011) is `is_linked && marker_absent`, **indistinguishable by absence** from the
markerless coordination tree SL-064 introduced (`coordinate.rs::run_coordinate`),
so its **Orchestrator verb class** (`fork`/`import`/`gc`/`coordinate`/sync) is
permitted fail-open. IMP-065 proposed a *positive* orchestrator marker to
disambiguate by presence.

**Design finding (this slice): a positive marker cannot close that gap.** Worker
identity is a presence-only file (`.doctrine/state/dispatch/worker`) + an optional
`DOCTRINE_WORKER` env. **Both are cooperative flags, not enforced boundaries** —
an agent sets/unsets its own env at will and writes/deletes/`cd`s around an
unprotected file at will. The *only* arm with real isolation is
`scripts/pi-spawn-confined.sh` (bwrap `--ro-bind / /`); claude `Agent` and vanilla
pi subagents have **none**. So an uncontained worker defeats any marker trivially
(`cd` into the coord tree, or forge the file); a bwrap worker can't misbehave
anyway. **In every arm the marker buys ~zero against a *capable* worker** — and
ADR-006 D2a already concedes this ("the fence … is the funnel + the jail, **not** a
fail-closed CLI floor"). RSK-014 records the full state of play.

**What is left, and is worth a cheap fix:** the **accidental, cooperative,
unstamped worker that stays in its own tree** and runs an Orchestrator verb there.
The R-5 import belt catches `.doctrine/` *file* writes at import but **not ref
mutation** (`gc`/`sync`/`import`/`candidate` rewrite shared refs, absent from the
`B..S` file delta) — so this one honest-failure path is genuinely uncaught today.
Anthropic models' strong alignment against *malicious compliance* means an
anti-accident guard goes further than the cooperative-flag critique implies: the
honest worker is the common case.

## Scope & Objectives

1. **Cheap accidental-ref-corruption guard.** Refuse an Orchestrator-classed verb
   (`fork`/`import`/`gc`/`coordinate`/`dispatch sync`/`candidate`/`arm-spawn`) when
   the cwd tree is a linked worktree that is **not** the registered coordination
   worktree. Framed and tested as **honest-failure defence-in-depth, NOT an
   impersonation boundary**. Preferred mechanism: derive coordination identity from
   **existing dispatch state** (the registered coord worktree / `dispatch/<slice>`
   branch the orchestrator already wrote) rather than minting a new marker file —
   DRY, no second cooperative artefact to forge. (Mechanism is the one open design
   decision; see OQ-1.)
2. **Revision against ADR-012** (and the ADR-006 D2a/D2b notes): retract "the
   positive marker is the real close of OQ-D"; reclassify the residual as
   **enforcement-bound and consciously accepted** — the genuine close is
   *confinement* (bwrap, linux-only; or `claude -p`, cost-untenable), tracked in
   RSK-014. Honest scope/bounds of orchestration worker-safety is the **primary
   deliverable**, the guard the secondary.

## Non-Goals

- **No new cooperative marker presented as security.** A positive marker file does
  not raise the enforcement altitude; do not build one and call it an impersonation
  fix. (If the guard ends up needing a stamped artefact, it is labelled
  anti-accident DiD, not a boundary.)
- **Does not close the impersonation gap.** A capable/misbehaving worker is out of
  scope — that needs confinement (RSK-014 / ADR-008 / D2b), unsolved on the claude
  arm. This slice does not pretend otherwise.
- **Does NOT gate ordinary authored writes.** D6a ("mode, not location, decides")
  lets a solo `/execute` in a worktree write directly, marker-absent. The guard is
  scoped to the **Orchestrator verb class** only — re-breaking general Write is the
  reverted-G2 hazard (SL-056 PHASE-05, owner-locked VH).
- Not a dispatch **topology** change (ADR-012, shipped).

## Affected Surface (coarse — `/design` refines)

- `src/commands/guard.rs` — `worker_guard` / `write_class`: the Orchestrator-class
  refusal gains the not-the-coord-tree check.
- `src/worktree/coordinate.rs` — `run_coordinate`: source of the registered-coord
  identity the guard checks (and/or the stamp point, if a marker is chosen).
- `src/worktree/marker.rs` — only if the mechanism needs a new predicate; minimise.
- `src/dispatch.rs` — Orchestrator dispatch verbs (`sync`/`candidate`/`arm-spawn`)
  share the guard.
- **REV** against `ADR-012` (+ ADR-006 D2a/D2b notes) — governance deliverable.

## Risks / Assumptions / Open Questions

- **OQ-1 (the one real design decision).** Mechanism for "is this the coordination
  tree?": (a) derive from existing dispatch state / `dispatch/<slice>` branch
  registration (preferred — no new artefact); (b) a new presence-only orchestrator
  marker stamped at `coordinate` (rejected unless (a) proves unworkable — adds a
  second forgeable file). Decide in design.
- **OQ-2 (verify, researcher thread 1).** Is the `coordinate`-created coord tree the
  **sole** legitimate caller of an Orchestrator verb from inside a *linked*
  worktree? If yes, the guard is unconditional on the class; if any other legit
  flow runs one from a linked tree, narrow it. (Determines correctness, not value.)
- **OQ-3.** Does SL-064's OQ-D plan-gate (i) "trusted orchestrator path" restriction
  exist in code? If so, this guard replaces/retires it; if not, this is the first
  implementation of that obligation.
- **A1.** `needs: ISS-011` closed (SL-124/SL-125) — actionable.
- **A2 (governance).** ADR-006 D2a/D2b owner-locked (VH). The REV is the sanctioned
  amendment path; the guard's value-claim must match the REV's honest framing.
- **A3 (premise of the value).** Anti-accident guard is worth it because honest
  failure (ISS-011 stamp drift) is real and the model is well-aligned against
  malice. If ISS-011 stamp reliability post-SL-124/125 made unstamped workers rare,
  the guard is near-pure DiD — still cheap, still honest, lower urgency.

## Verification / Closure Intent

- **Accidental case caught:** an unstamped worker (linked, not the coord tree)
  running `gc`/`sync`/`import` from its own cwd is **refused** (today: fail-open).
- **Legit case preserved:** the registered coordination tree runs Orchestrator
  verbs; a solo `/execute` in a worktree **still writes** general authored entities
  (D6a intact); general Write class unchanged.
- **Honest framing landed:** the REV retracts "real close"; SL-181 and IMP-065 no
  longer claim impersonation closure; RSK-014 carries the residual.
- **Tests labelled** anti-accident DiD, not impersonation proofs — no test asserts
  the guard stops a capable worker (it doesn't).
- Behaviour-preservation: existing `e2e_worktree_coordinate` + worker-guard suites
  stay green except where they encoded the absence dependence.

## Follow-Ups

- **RSK-014** — claude-subagent confinement state-of-play; the real close lives there.
- **IMP-065** — to be closed as "reframed, not closed-by-marker" when this lands.
