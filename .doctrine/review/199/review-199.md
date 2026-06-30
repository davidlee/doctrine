# Review RV-199 — design of SL-181

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Inquisition (facet: design) on SL-181's locked-but-self-reviewed `design.md`.
Lines of interrogation, in priority order:

1. **The load-bearing assumption (OQ-A).** Does the REAL dispatch-agent spawn path
   (`dispatch arm-spawn --base B` → `WorktreeCreate` → `worktree create-fork`) yield
   a worker HEAD that the predicate `is_coordination_worktree` correctly EXCLUDES?
   Invariant held to: the "positive coordination signal" must be UNIQUE to the
   SL-064 coordination worktree, distinguishable from every worker fork.
2. The `--path` seam — does the guard always judge the caller's cwd root, never a
   `--path` target?
3. OQ-2 — is the coord tree the SOLE legitimate linked-worktree Orchestrator caller?
4. OQ-C — any merge-conflict/land-abort path leaving the coord HEAD detached?
5. REV reframe honesty (§5) vs RSK-014 probe-h1.

## Synthesis — VERDICT: HERESY CONFESSED. The slice is BLOCKED.

The accused presented a "positive coordination-tree signal" and called it the sole
structural fence on the load-bearing claude arm. Under cross-examination the signal
was revealed a fraud — and the design's freshly-committed "correction" (244d7bc4)
gilded the lie. One unresolved **blocker** (F-1), two **majors** (F-2, F-3). Both
the Inquisitor's own code-reading and an independent reviewer (codex / GPT-5.5)
return the same verdict: **F-1 holds, decisively.**

### The mortal heresy (F-1, blocker — gates the slice)

`is_coordination_worktree = is_linked && current_branch.starts_with("dispatch/")`
(design §2.3) is **not unique to the coordination worktree.** The claude dispatch
worker fork is minted on `dispatch/<name>` **unconditionally** — `act_on_create::
Fork`, `src/worktree/create.rs:238-243` (`let branch = format!("dispatch/{name}")`)
via `fork_core` (`fork.rs:130-142`). The coord tree rides `dispatch/<NNN>`
(`coordinate.rs:156,212-223`). **Both satisfy the prefix.** For a worker fork the
predicate returns TRUE ⇒ the guard's `!is_coordination_worktree` condition is FALSE
⇒ the Orchestrator verb is **ALLOWED**. The guard is void for exactly the unstamped
worker it claims to fence. The probe-h1 evidence the design leaned on observed only
the BENIGN `act_on_create::Passthrough` path (`create.rs:247-266`, detached) — never
the armed Fork the real dispatch worker takes.

### The reachability conflation (F-2, major)

The atomic `create-fork → fork_core(worker=true)` path cannot leave branch-without-
marker (rollback, `fork.rs:145-168`); the **only** reachable unstamped state is the
legacy, failable `SubagentStart` stamp (`subagent.rs:135-148,224-239`). The design's
§1/§3/§4 model conflates the two lifecycles. Until the rework pins which marking is
operative on claude — and what branch the SubagentStart-unstamped tree carries — §4's
"sole structural fence, load-bearing" stands unsupported.

### The Inquisitor's own taint (F-3, major)

Commit 244d7bc4 (this session) generalised the Passthrough detached-HEAD observation
onto the Fork path, making §3/OQ-A/§5 positively assert a falsehood. To be undone in
the rework. The §5 REV confinement leg (RSK-014 bwrap "closable not unclosable") is
independent of the branch signal and may survive — re-derived cleanly.

### Penance (ordered)

1. **Back to `/design`.** Re-derive the coordination signal from the REGISTERED
   coordination-worktree dispatch state — the slice's own Scope obj-1 already
   demanded this; the branch-prefix shortcut abandoned it. The new signal must be
   provably coord-unique against the `dispatch/<name>` worker-fork shape. **F-1 stays
   open until then — it gates the slice's advance to /plan.**
2. Pin the claude marker lifecycle (F-2) with a test; scope or withdraw §4's
   load-bearing claim accordingly.
3. Repair the 244d7bc4 §3/§4/OQ-A/§5 falsehoods (F-3) in the rework; re-derive the
   §5 confinement reframe cleanly.

### Acquittals (charges that did NOT stick — examined, found clean)

- **`--path` seam — SOUND.** `worker_guard` resolves the caller cwd via
  `root::find(None,…)` (`guard.rs:362-373`); the handler's `--path` is later and
  cannot rescue a refusal. (Codex minor: keep this explicit in the design.)
- **OQ-C — CLEAN.** `refresh-base` runs `git merge --no-ff` in the live coord
  worktree; a conflict leaves `MERGE_HEAD`, NOT a detached HEAD (`dispatch.rs:680-763`).
  No self-brick path found.

> **HERESIS URITOR; DOCTRINA MANET**
