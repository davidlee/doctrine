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
   * The RV ledger is resolved (`done · await=none`). **Zero-finding reviews
    may report `active` rather than `done` (known issue IMP-098); treat them as
    terminal — the absence of unresolved blockers is what gates the transition,
    not the derived status string.**
   * The reconcile outcome is recorded (REV rationale and/or RV
     `## Reconciliation Outcome`).
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
   slice was driven by `/dispatch`, project the audited units now:

   ```bash
   # 1. Admit a close_target candidate (if not already done during /reconcile):
   doctrine dispatch candidate create --slice <N> --label close-001 \
     --role close_target --payload code --base refs/heads/main \
     --source refs/heads/review/<N>
   doctrine dispatch candidate admit --slice <N> --role close_target \
     --candidate refs/heads/candidate/<N>/close-001 --review RV-NNN

   # 2. Project onto trunk (--trunk is REQUIRED; omitting it is a dry run):
   doctrine dispatch sync --slice <N> --integrate --trunk refs/heads/main
   ```

   When a candidate workflow is active this targets the immutable **admitted
   `close_target` OID** (and `--edge` the admitted `review_surface` OID) under a
   fast-forward-only CAS row — **never a raw `phase/*`/`review/*` tip or the
   mutable candidate ref**, and never a close-time merge. A moved trunk refuses
   (admit a superseding close-target candidate on the new base). This is the
   **only** place `--integrate` runs — never at `/dispatch` conclude, only here,
   post-audit.

   **Verify.** After `--integrate --trunk`, confirm the slice's code delta is on
   the target branch:
   ```bash
   git diff --stat refs/heads/main~1..refs/heads/main -- src/
   ```
   The output must include the files the slice changed. If the delta is absent,
   integration did not project code — do **not** proceed to step 4.

   > **TODO:** Once project config (`doctrine.toml [dispatch] deliver_to`) lands,
   > the trunk ref and verification will be derived from config, not hard-coded
   > here. The mandatory `--trunk` requirement and the verification step are a
   > stopgap against silent dry-run integration (see SL-102 close).
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
