# Positive coordination-tree marker

Closes ADR-012 **OQ-D** (RV-023 F-2), the deferred redress carried as **IMP-065**.
Amends ADR-006 **D2a**. Successor concern to SL-064.

## Context

Dispatch coordination-tree write-permission today rests on marker **absence**.
The worker-mode verdict is `worker_mode = (is_linked_worktree && marker_present)
OR env DOCTRINE_WORKER`; a Write/Orchestrator-classed verb is refused **only**
when the worker marker is present (`marker.rs::Cause`, `guard.rs::worker_guard`).
SL-064 moved the orchestrator into a *linked* coordination worktree created
**markerless** (`coordinate.rs::run_coordinate`, D9 amendment) — so it is
permitted precisely because no worker marker is present.

The gap (ADR-006 D2b note; ADR-012 §Consequences/Negative): an **unstamped
worker** (SubagentStart stamp-hook failure / matcher drift — see ISS-011) is
`is_linked && marker_absent`, i.e. **indistinguishable by absence** from the
legitimate coordination tree. SL-064 widened the blast radius from "looks like a
solo `/execute`" to "looks like the tree that owns the funnel and the whole
**Orchestrator verb class** (`fork`/`import`/`gc`/`coordinate`/sync)". The D2b
fence (R-5 import belt + IMP-052 post-spawn check + env-worker-on-main catch +
bwrap-no-push) is **defence-in-depth, not a proof** it catches `gc`/`sync`
impersonation (RV-025 B3).

**The redress:** stamp a **positive** coordination-tree marker (orchestrator
identity) at markerless-creation time, so the identity guard distinguishes
legit-coordination-tree from unstamped-worker by **presence of the right
marker**, not absence of the worker marker. The positive marker must
**supersede** the absence dependence, not layer on top of it.

## Scope & Objectives

1. **Mint a positive orchestrator marker** at coordination-tree creation
   (`run_coordinate`, and the `create.rs` markerless variant) — a withheld-tier
   identity stamp distinct from the worker marker (`marker --stamp-subagent`).
2. **Key the Orchestrator verb class on marker presence.** In a linked worktree,
   an Orchestrator-classed verb (`fork`/`import`/`gc`/`coordinate`/sync) is
   permitted iff the orchestrator marker is **present**; absent ⇒ refuse. This
   inverts the absence dependence to a positive signal — an unstamped worker
   (neither marker) is now refused, not fail-open.
3. **Amend ADR-006 D2a** prose to record the positive coordination identity and
   retire the marker-absence transitional assumption (owner-locked VH — governance
   act, route through `/design` → `/inquisition`, likely a Revision).
4. **Impersonation tests** (OQ-D plan-gate (ii), mandatory): prove the marker
   catches `gc`/`sync`/`coordinate` impersonation the D2b fence could not.

## Non-Goals

- **Do NOT gate ordinary authored writes on the positive marker.** D6a ("mode,
  not location, decides") lets a **solo `/execute` in a worktree** write directly,
  marker-absent. Requiring the positive marker for *all* authored writes re-breaks
  that path — the exact failure that got the G2 fail-closed clause reverted
  (SL-056 PHASE-05, owner-locked VH). The positive requirement is scoped to the
  **Orchestrator verb class**, not the general Write class. (See Open Question 1 —
  must be confirmed in design, not assumed.)
- Not a dispatch **topology** change (that is ADR-012, shipped).
- Does **not** close the D2b raw-tree harness-confinement gap (bare `git commit`
  / hand-edit by an unconfined worker) — that stays deferred to sandbox/harness
  work (ADR-008).

## Affected Surface (coarse — `/design` refines)

- `src/worktree/marker.rs` — `Cause`/`StatusLine`/`resolve_mode`; the marker file
  model gains the orchestrator-identity stamp.
- `src/worktree/coordinate.rs` — `run_coordinate` (the markerless creation /
  positive-stamp point).
- `src/worktree/create.rs` — markerless creation variant (D9).
- `src/commands/guard.rs` — `worker_guard` / `write_class` (Orchestrator class
  refusal keyed on positive-marker presence).
- `src/worktree/subagent.rs` — worker-marker minting (the contrast verb).
- `.doctrine/adr/006/adr-006.md` — D2a amendment (governance).

## Risks / Assumptions / Open Questions

- **OQ-1 (scope-shaping, decide in design).** Does the positive requirement apply
  to the Orchestrator verb class **only**, or more broadly? Broader re-breaks
  solo-`/execute`-in-worktree (the reverted G2 hazard). Working assumption:
  Orchestrator-class only.
- **OQ-2.** OQ-D plan-gate (i) required SL-064 to restrict Orchestrator-verb
  invocation to "the trusted orchestrator path until a positive marker lands." No
  explicit trusted-path gate was found in `src/` during preflight — confirm
  whether it shipped (and under what name) and how SL-181 retires/replaces it.
- **OQ-3.** Marker minting/storage: the orchestrator stamp must not collide with
  the worker withheld-tier marker model, and must be a distinct mint verb/identity
  from `marker --stamp-subagent`. `Cause` enum likely gains a variant.
- **A1.** `needs: ISS-011` is **closed/done** (fulfilled by SL-124/SL-125) — this
  slice is actionable (ADR-017 gate clear).
- **A2 (governance).** ADR-006 D2a is **owner-locked (VH, SL-056 PHASE-05)**.
  Amending it is a governance act — `/design` → `/inquisition`, possibly a
  Revision; the plan is not higher authority than D2a/ADR-012.

## Verification / Closure Intent

- Unstamped worker (`is_linked && both markers absent`) is **refused** at the
  Orchestrator verb seam (the fail-open that exists today is closed).
- Legit coordination tree (orchestrator marker present) **writes**; solo
  `/execute` in a worktree (both markers absent) **still writes** its own doctrine
  state (D6a preserved).
- Impersonation tests for `gc`/`sync`/`coordinate` (plan-gate (ii)).
- ADR-006 D2a prose records the positive identity; absence dependence superseded,
  not layered.
- Behaviour-preservation: existing `e2e_worktree_coordinate` + worker-guard suites
  stay green except where they encoded the absence dependence (those update with
  the D2a amendment).

## Follow-Ups

- (none yet)
