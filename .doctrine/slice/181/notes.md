# Notes SL-181: Worker-safety: accidental ref-corruption guard + OQ-D reframe

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Status (2026-06-30)

Lifecycle: **design** (design.md written + self-reviewed; no plan/phases yet).
Commits: `546810ab` (Fork-B reframe + RSK-014), `a334a962` (design locked).
All `.doctrine/` committed promptly; **no code written** (gate N/A — design stage).

**Next step: `/inquisition` on `design.md` with codex (GPT-5.5, default reviewer).**
Then integrate findings → `/plan`.

## HANDOVER — for the inquisition agent

Read in order: `doctrine slice show SL-181`, then `design.md` (the target),
`doctrine backlog show RSK-014` (the residual the reframe leans on),
`doctrine adr show ADR-012` (OQ-D), `doctrine adr show ADR-006` (D2a/D2b).

### What this slice became (and why it matters for the inquisition)

It started as IMP-065 "positive coordination marker = the real close of OQ-D."
The design conversation **killed that framing**: the worker-mode marker is a
**cooperative flag, not an enforced boundary** on every spawn arm except
bwrap-pi-subprocess (RSK-014). ADR-006 D2a already concedes this. So the slice was
reframed (Fork B) to a **cheap anti-accident guard + a Revision** that retracts the
overclaim. The guard catches the *accidental* unstamped worker (stamp-hook failure,
ISS-011 — still live, `SubagentStart` is un-failclosable); it does **not** and
**cannot** stop a capable worker (it `cd`s into the coord tree or forges
`git checkout -b dispatch/N`). The design says this honestly throughout — **do not
let the inquisition "discover" that the guard is cooperative as if it were a flaw;
it is the stated thesis.** The real target is whether the design is honest,
correct, and whether the anti-accident claim actually holds.

### The design in one breath

Orchestrator-verb class, in a linked worktree, is refused unless the tree's `HEAD`
rides `dispatch/<NNN>` (`is_coordination_worktree` in `worktree/shared.rs`; one
clause in `worker_guard`; `current_branch` promoted to `git.rs` for DRY). No new
marker. General `Write` class untouched (D6a — anti-G2). REV against ADR-012 +
ADR-006 D2a/D2b retracts "the real close."

### The sharp targets (where I want codex to push hardest)

1. **OQ-A — the load-bearing assumption.** The guard only works on the claude arm
   (its sole-fence arm) if an **unstamped claude worker runs on a non-`dispatch/N`
   branch** (`worktree-agent-<id>`) *during execution* — the `dispatch/N` link is
   collapse-time, after the worker exits. If a claude dispatch-worker ever executes
   with `HEAD` on `dispatch/N`, the branch-check **false-allows** it and the guard
   is **void on the one arm that needs it**. Evidence: mem
   `mem.pattern.dispatch.claude-subagentstart-worker-identity` (branch
   `worktree-agent-<agentId>`) vs mem `mem.pattern.dispatch.claude-arm-isolation-fallback`
   (commit lands on `dispatch/N`, worktree collapses onto parent). I rated this
   ~85% and **harness-version-fragile**. **This is the #1 thing to break.** If codex
   can show the claude worker runs on `dispatch/N`, the mechanism needs rework
   (back to a marker, or a different signal).
2. **`--path` seam** — I claim the guard judging the *caller's* cwd branch closes
   it (an unstamped worker can't borrow coord identity via `-p /coord`). Verify the
   guard truly always resolves cwd (`root::find(None,…)`, guard.rs) and never the
   `--path` target before refusing.
3. **Unconditional-on-class safety (OQ-2)** — rests on the researcher verdict that
   the coord tree is the *sole* legitimate linked-worktree Orchestrator caller.
   Probe the verb table (design refs): is there any flow — esp. `create-fork`
   (runs in the arming dir *inside* the coord tree) or a resume/`refresh-base`
   merge-conflict path — that runs an Orchestrator verb from a linked tree NOT on
   `dispatch/N`? That would be a legit false-refuse.
4. **Detached-HEAD (OQ-C)** — does any merge-conflict path (`land` abort,
   `refresh-base`) leave the coord `HEAD` detached mid-operation, self-bricking the
   next Orchestrator verb? Design claims the branch is always restored; verify.
5. **Reframe honesty** — does the REV actually retract the OQ-D "real close" claim,
   or does some scope/test still implicitly assert impersonation closure? The
   impersonation tests MUST be labelled anti-accident, asserting nothing about a
   capable worker.

### Reviewer setup

Default reviewer = **codex mcp (GPT-5.5)** per project CLAUDE.md. Prime the review
with `design.md` §8 (design decisions/OQs) and §9 (self-review F1–F7) so codex
attacks the *residuals*, not the already-integrated findings. An Opus sub-agent
second pass is useful for variety afterward.

### Deliverables this slice owes (so the inquisition can check completeness)

- The guard + predicate + DRY `current_branch` move (code).
- The **REV** (governance — the *primary* deliverable, not the code).
- Impersonation tests labelled anti-accident; the §3 truth-table goldens.
- Honest per-harness value note (load-bearing claude / redundant codex-pi).

### Related entities

- **RSK-014** — claude-subagent confinement unsolved (the genuine close lives here).
- **CHR-032** — STD-001 `dispatch/` prefix scatter (deliberately out of scope).
- **IMP-065** — to be closed "reframed, not closed-by-marker" when this lands.
- Researcher evidence (assumption ledger + verb table + OQ-1 branch analysis) is in
  the design conversation; key conclusions are folded into `design.md`.

## Durable facts worth a memory (candidates — record after inquisition confirms)

- "Dispatch worker-mode marker (file or env) is a cooperative flag, not an enforced
  boundary, except under bwrap (`--ro-bind / /`)." — likely already implied by
  ADR-006 D2a + RSK-014; check before recording a duplicate.
- "Coord tree is identifiable by HEAD on `dispatch/<NNN>`; an unstamped worker never
  is (codex `agent-*`, claude `worktree-agent-*`) — *during execution*." — record
  only once OQ-A is confirmed by the inquisition (it's the fragile bit).
