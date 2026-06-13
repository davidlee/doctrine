---
name: close
description: Use to formally close a slice once its phases are complete and audited — confirm the rollup, harvest durable findings, reconcile lifecycle status, and land a clean final commit. Routed to from /audit.
---

# Close

You are executing formal closure, not just marking work done.

Inputs:

- completed, audited implementation phases
- the reconciliation review (RV) with every finding dispositioned (see `/audit`)
- the governing slice id

## Process

1. **Pre-check:**
   - Phase exit criteria (`EX-`) and verification (`VT-`) are met. Confirm the
     rollup: `doctrine slice list` should show the slice as `X/X complete` with
     no `!N` blocked, no `?N` anomalous, no `—` untracked.
   - `/audit` is done: every finding on the RV ledger has a disposition;
     "design was wrong" findings already reconciled into `design.md`.
   - Durable facts, patterns, or gotchas from the slice are harvested into
     `notes.md`, and reusable ones captured via `/record-memory`,
     before closure — or consciously rejected. Durable follow-up **work** the
     slice leaves behind (risks / issues / chores) is captured as backlog items
     with `backlog new` (the work / knowledge / decision boundary:
     `using-doctrine.md`), or consciously rejected.
   - `just check` is green.
2. **Commit cleanly:** land `.doctrine/**` workflow artefacts in small, clean
   conventional commits scoped with the slice id, rather than letting them
   accumulate. Code and workflow edits go together or separately, whichever
   commits cleanly first.
3. **Transition lifecycle:** confirm the slice is in `reconcile` (flip it with
   `doctrine slice status <id> reconcile` if `/audit` didn't), then
   `doctrine slice status <id> done` (`<id>` is the bare number, e.g. `40`).
   The closure seam enforces the order and refuses while an RV targeting the
   slice carries an unresolved blocker. Re-run `doctrine slice list` and confirm
   the `⚠` divergence marker is gone — authored status now agrees with the
   rollup. (The terminal set is `{done, abandoned}`; closure here is the `done`
   path — abandoning a slice is a separate decision, not this skill.)
4. **Close the originating backlog item:** if a backlog item (ISS/IMP/CHR/RSK)
   spawned this slice, transition it too — `doctrine backlog edit <ID>
   --status resolved --resolution fixed` (or the resolution that fits). A closed
   slice with its origin still open is hygiene debt.
5. If a blocking finding remains unresolved, do **not** close — return to
   `/audit` or `/execute`. If closure depends on tolerated drift with material
   tradeoffs, `/consult` before normalising around it.

## Outcomes

- The slice is `done`, with the rollup in agreement.
- The originating backlog item (if any) is resolved.
- Durable guidance is captured in memory or consciously rejected.
- The tree is clean and the workflow artefacts are committed.
