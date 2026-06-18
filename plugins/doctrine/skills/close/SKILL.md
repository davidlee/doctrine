---
name: close
description: Use to formally close a slice once its phases are complete, audited, and reconciled — confirm the rollup, verify spec-coherence, harvest durable findings, reconcile lifecycle status, and land a clean final commit. Routed to from /reconcile.
---

# Close

You are executing formal closure, not just marking work done.

Inputs:

- completed, audited, reconciled implementation phases
- the reconciliation review (RV) with every finding terminal and the
  `## Reconciliation Outcome` recorded (see `/reconcile`)
- the governing slice id

## Process

1. **Pre-check:**
   - Phase exit criteria (`EX-`) and verification (`VT-`) are met. Confirm the
     rollup: `doctrine slice list` should show the slice as `X/X complete` with
     no `!N` blocked, no `?N` anomalous, no `—` untracked.
   - `/reconcile` is done (via `/audit → /reconcile`): the RV ledger is resolved,
     every governance/spec finding is dispositioned, and the RV carries a
     `## Reconciliation Outcome` section. If the reconciliation brief was empty
     (no-op), the outcome confirms that explicitly.
   - Durable facts, patterns, or gotchas from the slice are harvested into
     `notes.md`, and reusable ones captured via `/record-memory`,
     before closure — or consciously rejected. Durable follow-up **work** the
     slice leaves behind (risks / issues / chores) is captured as backlog items
     with `backlog new` (the work / knowledge / decision boundary:
     `using-doctrine.md`), or consciously rejected.
   - `just check` is green.
2. **Spec-coherence gate — confirm reconciliation is complete before `done`:**
   Before the terminal transition, verify every item from the audit's
   reconciliation brief is resolved through one of four paths:
   * **REV done** — governance/spec items covered by a `done` REV
     (`revision status REV-N done`). The REV rationale (`revision-NNN.md`)
     carries the reconcile narrative.
   * **Withdrawn** — finding withdrawn in the RV with rationale.
   * **Tolerated** — finding tolerated in the RV with rationale.
   * **Escalated to design** — slice transitioned back to `design` via the
     ADR-009 §1 back-edge (`reconcile → design`).
   Additionally:
   * Every per-slice direct-edit item is applied to `design.md` /
     `slice-NNN.md` and recorded in the `## Reconciliation Outcome`.
   * The RV ledger is resolved (`done · await=none`).
   * The reconcile outcome is recorded (REV rationale and/or RV
     `## Reconciliation Outcome`).

   **Orphan check (advisory).** Read `plan.md` `## Requirements verification`
   and collect every `REQ-DNN` handle listed. For each, follow the read-path
   below to determine its outcome:
   - **Placed** — the reconciliation outcome records a `REQ-DNN → REQ-NNN`
     mapping → pass.
   - **Withdrawn** — the reconciliation outcome records a withdrawal with
     rationale → pass.
   - **Stuck** — the reconciliation outcome records "stuck — `/consult`" with no
     mapping → **refuse close.** Return to `/reconcile`.
   - **Absent** — no outcome recorded → **refuse close.** Return to `/reconcile`.

   **Read-path.** The reconciliation outcome lives in `review-NNN.md`
   `## Reconciliation Outcome` (written by reconcile step 6), which *points to*
   the REV narrative in `revision-NNN.md` `### Orphan placements` for the
   `REQ-DNN → REQ-NNN` mappings. Close reads `review-NNN.md` first; if the
   outcome references a REV for orphan placements, follow the pointer into
   `revision-NNN.md` to read the mappings. Do not read `revision-NNN.md`
   directly without the `review-NNN.md` pointer — the RV is the reconciled-truth
   surface.

   **Advisory, not enforced.** This check is agent discipline, not a binary
   gate: nothing in the CLI refuses close on an unplaced orphan (the existing
   close-gate binary enforces only unresolved RV blockers). The close skill's
   own walkthrough is the backstop. The long-term fix — orphan status riding the
   RV ledger, where the existing close-gate binary already enforces — is filed
   as a follow-up IMP; SL-098 ships the advisory version and names it honestly.

   No free-floating "rejected" disposition is permitted — every finding
   must land in one of the terminal states above.
   If any item is unresolved, **refuse close** and return to `/reconcile`.
   (The structural closure seam is the mechanical backstop; this gate is the
   substantive check — the verb enforces; the skill verifies.)
3. **Commit cleanly:** land `.doctrine/**` workflow artefacts in small, clean
   conventional commits scoped with the slice id, rather than letting them
   accumulate. Code and workflow edits go together or separately, whichever
   commits cleanly first.
3a. **Dispatched slice — integrate the admitted OID (post-audit only).** If the
   slice was driven by `/dispatch`, project the audited units now with `doctrine
   dispatch sync --slice <N> --integrate [--trunk <ref>] [--edge <ref>]`. When a
   candidate workflow is active this targets the immutable **admitted `close_target`
   OID** (and `--edge` the admitted `review_surface` OID) under a fast-forward-only
   CAS row — **never a raw `phase/*`/`review/*` tip or the mutable candidate ref**,
   and never a close-time merge. A moved trunk refuses (admit a superseding
   close-target candidate on the new base). This is the **only** place `--integrate`
   runs — never at `/dispatch` conclude, only here, post-audit.
4. **Transition lifecycle:** confirm the slice is in `reconcile` (flip it with
   `doctrine slice status <id> reconcile` if `/audit` didn't), then
   `doctrine slice status <id> done` (`<id>` is the bare number, e.g. `40`).
   The closure seam enforces the order and refuses while an RV targeting the
   slice carries an unresolved blocker. Re-run `doctrine slice list` and confirm
   the `⚠` divergence marker is gone — authored status now agrees with the
   rollup. (The terminal set is `{done, abandoned}`; closure here is the `done`
   path — abandoning a slice is a separate decision, not this skill.)
5. **Close the originating backlog item:** if a backlog item (ISS/IMP/CHR/RSK)
   spawned this slice, transition it too — `doctrine backlog edit <ID>
   --status resolved --resolution fixed` (or the resolution that fits). A closed
   slice with its origin still open is hygiene debt.
6. If a blocking finding remains unresolved, do **not** close — return to
   `/reconcile`. If closure depends on tolerated drift with material
   tradeoffs, `/consult` before normalising around it.

## Outcomes

- The slice is `done`, with the rollup in agreement.
- The originating backlog item (if any) is resolved.
- Durable guidance is captured in memory or consciously rejected.
- The tree is clean and the workflow artefacts are committed.
