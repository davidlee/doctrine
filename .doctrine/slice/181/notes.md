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

## Update (2026-07-01) — probe-h1 reframe integrated

RSK-014 probe-h1 **proved** claude-arm confinement is achievable
(`PreToolUse(Bash)→nested bwrap` + `Edit|Write` pathcheck; full escape battery
contained, necessity + fail-open closed). Mechanism **unchanged**; three framing
edits integrated into `design.md`:
- **§5 REV** — residual is now "closable, not-yet-landed" (not "unclosable on
  claude"). The genuine close = a forthcoming slice graduating `probe-h1/` into the
  doctrine skill hooks. SL-181 = the anti-accident interim.
- **§4** — claude branch-check is load-bearing *but time-bound* (demotes to
  belt-and-suspenders once confinement lands; stays load-bearing on macOS claude
  until `sandbox-exec`, IMP-045).
- **§3 / OQ-A** — claude worker fork is **detached-HEAD** (probe finding 5), not
  `worktree-agent-<id>`. Detached → `None` → refuse: verdict unchanged, OQ-A
  *better-supported* (fork machinery owns the branch, not the harness).

**Plan A (User, 2026-07-01): land SL-181 with this framing, then beeline the
confinement slice (the real OQ-D close).**

## INQUISITION VERDICT (2026-07-01, RV-199) — SLICE BLOCKED, back to /design

The inquisition (RV-199, facet design, codex/GPT-5.5 cross-exam) found the
mechanism **structurally unsound**. One unresolved **blocker** gates the slice.

- **F-1 (blocker, design-wrong, OPEN — gates /plan).** `is_coordination_worktree`
  = `is_linked && branch.starts_with("dispatch/")` is **NOT coord-unique.** The
  claude worker fork rides `dispatch/<name>` unconditionally (`create.rs:238-243`
  `act_on_create::Fork`); the coord tree rides `dispatch/<NNN>` (`coordinate.rs`).
  Both match the prefix ⇒ predicate TRUE for a worker fork ⇒ guard ALLOWS the
  Orchestrator verb ⇒ **void for the unstamped worker it targets.** The probe-h1
  detached-HEAD evidence was the BENIGN `Passthrough` path, never the armed Fork.
  Confirmed independently by codex.
- **F-2 (major, design-wrong).** §1/§3/§4 conflate two marker lifecycles: armed
  `create-fork → fork_core(worker=true)` stamps atomically + rolls back (no
  unstamped state reachable there); the reachable unstamped state is the legacy
  failable `SubagentStart` stamp (`subagent.rs:135-148,224-239`). §4 "load-bearing
  on claude" unsupported until the operative marking path + its branch shape are
  pinned with a test.
- **F-3 (major, fix-now).** My commit 244d7bc4 generalised the Passthrough
  detached-HEAD onto the Fork path — §3/OQ-A/§5 now positively assert a falsehood.
  Undo in the rework; re-derive §5's confinement leg (independent, may survive).
- **Acquittals:** `--path` seam SOUND (caller-cwd, `guard.rs:362-373`); OQ-C CLEAN
  (refresh-base leaves `MERGE_HEAD`, not detached, `dispatch.rs:680-763`).

### Next step (NOT /plan — blocked)

`/design` rework of the coordination signal: anchor on the **registered
coordination-worktree dispatch state** (the slice's own Scope obj-1 already asked
for this; the branch-prefix shortcut abandoned it), provably unique vs the
`dispatch/<name>` worker shape. Then pin F-2's marker lifecycle, repair F-3, and
`review verify RV-199 F-1` only once the reworked signal holds. Durable trap
recorded: `mem.fact.dispatch.dispatch-branch-prefix-not-coord-unique`.

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

1. **OQ-A — the load-bearing assumption (now with fresh evidence).** The guard
   only works on the claude arm (its sole-fence arm) if an **unstamped claude worker
   runs on a non-`dispatch/N` branch** *during execution* — the `dispatch/N` link is
   collapse-time, after the worker exits. If a claude dispatch-worker ever executes
   with `HEAD` on `dispatch/N`, the branch-check **false-allows** it and the guard
   is **void on the one arm that needs it**. **New evidence (RSK-014 probe-h1
   finding 5):** doctrine's `WorktreeCreate → worktree create-fork` makes a
   **detached-HEAD** tree (`cwd == .worktrees/agent-<id>`); detached → `None` →
   refuse. So the assumption is *better-supported* — but the probe used a bare
   `isolation: worktree` spawn, **not** the real dispatch-agent path. **#1 break
   target for codex:** does `dispatch arm-spawn --base B` → `WorktreeCreate` also
   yield detached/non-`dispatch/N` at execution time, or could `--base B` ride a
   branch? If the real spawn path puts the worker on `dispatch/N`, the guard is void
   on the load-bearing arm and the mechanism needs rework (marker or other signal).
   Cross-check mems `…claude-subagentstart-worker-identity`,
   `…claude-arm-isolation-fallback` against probe finding 5.
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

- **RSK-014** — claude-subagent confinement (the genuine close lives here). **No
  longer "unsolved": probe-h1 proved it achievable; close = a forthcoming slice that
  graduates `probe-h1/` → doctrine skill hooks. This is the post-SL-181 beeline.**
- **IMP-045** — macOS `sandbox-exec` seam (the confinement slice's cross-platform leg).
- **CHR-032** — STD-001 `dispatch/` prefix scatter (deliberately out of scope).
- **IMP-065** — to be closed "reframed, not closed-by-marker" when this lands.
- Researcher evidence (assumption ledger + verb table + OQ-1 branch analysis) is in
  the design conversation; key conclusions are folded into `design.md`.

## Durable facts worth a memory (candidates — record after inquisition confirms)

- "Dispatch worker-mode marker (file or env) is a cooperative flag, not an enforced
  boundary, except under bwrap (`--ro-bind / /`)." — likely already implied by
  ADR-006 D2a + RSK-014; check before recording a duplicate.
- "Coord tree is identifiable by HEAD on `dispatch/<NNN>`; an unstamped worker never
  is (codex `agent-*`, claude **detached-HEAD** per probe-h1 finding 5) — *during
  execution*." — record only once OQ-A is confirmed by the inquisition against the
  real dispatch-agent spawn path (it's the fragile bit).
