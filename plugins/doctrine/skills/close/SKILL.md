---
name: close
description: Use to formally close a slice once its phases are complete and audited — confirm the rollup, harvest durable findings, reconcile lifecycle status, and land a clean final commit. Routed to from /audit.
---

# Close

You are executing formal closure, not just marking work done.

> **Tooling gaps.** Doctrine has no `complete slice` command and no lifecycle
> transition verb — `slice-nnn.toml` `status` is hand-edited. `slice list`
> *reveals* divergence between the authored status and the phase rollup (`⚠`),
> but reconciling it is this skill's manual job. The terminal-status set lives in
> `slice::is_terminal_status` (v1 `{"done"}`).

Inputs:

- completed, audited implementation phases
- `audit.md` with every finding dispositioned (see `/audit`)
- the governing slice id

## Process

1. **Pre-check:**
   - Phase exit criteria (`EX-`) and verification (`VT-`) are met. Confirm the
     rollup: `doctrine slice list` should show the slice as `X/X complete` with
     no `!N` blocked, no `?N` anomalous, no `—` untracked.
   - `/audit` is done: every finding has a disposition; "design was wrong"
     findings already reconciled into `design.md`.
   - Durable facts, patterns, or gotchas from the slice are harvested into
     `notes.md` / `audit.md`, and reusable ones captured via `/record-memory`,
     before closure — or consciously rejected. Durable follow-up **work** the
     slice leaves behind (risks / issues / chores) is captured as backlog items
     with `backlog new` (the work / knowledge / decision boundary:
     `using-doctrine.md`), or consciously rejected.
   - `just check` is green.
2. **Commit cleanly:** land `.doctrine/**` workflow artefacts in small, clean
   conventional commits scoped with the slice id, rather than letting them
   accumulate. Code and workflow edits go together or separately, whichever
   commits cleanly first.
3. **Transition lifecycle:** hand-edit `slice-nnn.toml` `status` to a terminal
   value (`done`). Re-run `doctrine slice list` and confirm the `⚠` divergence
   marker is gone — authored status now agrees with the rollup.
4. If a blocking finding remains unresolved, do **not** close — return to
   `/audit` or `/execute`. If closure depends on tolerated drift with material
   tradeoffs, `/consult` before normalising around it.

## Outcomes

- The slice is in a terminal lifecycle status, with the rollup in agreement.
- Durable guidance is captured in memory or consciously rejected.
- The tree is clean and the workflow artefacts are committed.
