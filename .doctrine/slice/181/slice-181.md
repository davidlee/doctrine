# OQ-D reframe: confinement is the close, retract the positive-marker overclaim

**REV-only** (reshaped 2026-07-03, A2). Lands a **Revision** against ADR-012 that
retracts the over-claimed "positive marker is the real close of OQ-D" and records
the genuine close as **shipped confinement** (SL-182/183/185). The anti-accident
guard this slice once carried is **retired** — no code ships. Successor concern to
SL-064; residual tracked as **RSK-014**; originates from **IMP-065**.

## Context — the reshape

ADR-012 deferred OQ-D (RV-023 F-2) as "the real close" of the dispatch
impersonation gap (ADR-011 D6/M2): an **unstamped worker** is `is_linked &&
marker_absent`, indistinguishable by absence from the markerless SL-064
coordination tree (`coordinate.rs::run_coordinate`), so its **Orchestrator verb
class** (`fork`/`import`/`gc`/`sync`/`candidate`) runs fail-open. IMP-065 proposed a
*positive* marker to disambiguate by presence.

Two findings collapsed that plan (design.md §0–§1):

1. **A positive marker cannot close it.** Worker identity is a presence-only file +
   optional `DOCTRINE_WORKER` env — **cooperative flags, not enforced boundaries**
   (RSK-014; ADR-006 D2a already concedes it). A capable worker forges or `cd`s
   around either trivially.
2. **The genuine close — confinement — has shipped.** SL-182 (Linux/bwrap claude
   arm, **ro shared `.git`**), SL-183 (macOS Seatbelt), SL-185 (subprocess parity).
   Where a backend runs, worker ref-mutation is blocked at the OS floor, stamped or
   not.

And the interim guard was found to have **no configuration where it both matters
and works** (design §1): confined → floor already blocks; no-backend → SL-182
fail-closed, worker runs nothing; standalone clone → contained + not `is_linked`;
unconfined → RV-199 **F-1** voids the mechanism (worker fork rides `dispatch/<name>`
⇒ false-allow). So the guard is retired, not reworked.

## Scope & Objective

**One deliverable: the Revision.** A **REV against ADR-012** (+ ADR-006 D2a/D2b
notes) that:
- retracts "the positive coordination marker (IMP-065) is the real close of OQ-D";
- records the genuine close as **shipped confinement** (SL-182/183/185), superseding
  the prior "closable, not-yet-landed" / "unclosable on claude" framing;
- reclassifies the RSK-014 residual as **enforcement-closed on every arm running a
  confinement backend**, with a bounded, self-limiting residual (no-backend
  fail-closes; standalone clone is contained);
- records that the anti-accident branch-check guard was designed, inquisited
  (RV-199 F-1), and **retired** — SL-181 ships no code.

## Non-Goals

- **No guard, no new code.** The anti-accident branch-check is retired (design §1).
  `slice conformance` holds SL-181 to a **zero touch-set**; any `src/` edit is an
  undeclared-edit finding.
- **No new cooperative marker.** A presence-only marker never raised the enforcement
  altitude; none is built or claimed.
- **Not a topology or confinement change.** Confinement already shipped
  (SL-182/183/185); this slice does not re-open it — it only makes ADR-012 honest
  about it.
- **Does not itself close IMP-065 / downgrade RSK-014.** Those are the REV's
  *effects*, recorded at reconcile/close (design §4), not authored during
  design/execute of the REV.

## Affected Surface

- **REV** against `ADR-012` (+ ADR-006 D2a/D2b notes) — the **sole** deliverable.
- **No `src/` files.** All prior design-target + scope-relevant selectors cleared.
- **Not touched:** `guard.rs`, `worktree/shared.rs`, `git.rs`, `dispatch.rs`,
  `marker.rs`, `subagent.rs` — the guard plan is dropped.

## Risks / Assumptions / Open Questions

**Resolved:**
- **A2 chosen (retire, not rework)** — user, 2026-07-03. F-1 voided the mechanism;
  confinement voided the need.
- **OQ-D close identified** — confinement, shipped (SL-182/183/185). No marker
  residual remains.
- **Guard's two-residual challenge dissolved** — both cases the guard might still
  serve (linux-no-bwrap; writable standalone clone) either fail-closed or fall
  outside the linked-worktree domain (design §1).

**Assumptions:**
- **A1 (governance).** ADR-006 D2a/D2b and ADR-012 owner-locked (VH); the REV is the
  sanctioned amendment path, routed via reconcile — not hand-edited.
- **A2 (enforcement floor).** SL-182's `PreToolUse` hook is unconditionally active on
  the claude dispatch path and fail-closed on a missing backend. (Confirmed from the
  shipped `pretooluse-wrap.sh`; re-confirm at execute if the REV leans on it hard.)

## Verification / Closure Intent

- **REV lands** against ADR-012 (+ D2a/D2b notes); its statements match design §2.
- **Zero code delta** — `slice conformance SL-181` reports an empty design-target
  set and no undeclared edits.
- **RV-199 resolved** — F-1/F-2/F-3 terminal (accepted; guard retired); no open
  blocker gates the slice.
- **Reconcile effects:** IMP-065 closed (reframed-superseded); RSK-014 downgraded
  ("enforcement-closed on confined arms; bounded residual").

## Follow-Ups (reconcile/close)

- **RSK-014** — downgrade to "solved on confined arms; bounded residual." The REV
  carries the reclassification.
- **IMP-065** — close as reframed/superseded-by-confinement when the REV lands.
