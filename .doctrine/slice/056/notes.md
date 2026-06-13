# SL-056 — implementation notes

Durable cross-phase facts harvested as phases land. Runtime progress lives in the
gitignored phase sheets (`phases/phase-NN.{toml,md}`); this file holds what must
survive a handover.

## PHASE-01 — G1+G3 governance gate (done)

**G1 — ADR-008 accepted.** Revised → `accepted` (was `proposed`). Three folds:
- §5.1 false-green/false-red evidence cluster folded into Context (validates D-B1
  empirically): shared-target false-green (touch+rerun), worktree-removal false-red
  (stale `CARGO_MANIFEST_DIR`), debug-vs-release timing.
- D-B2 sharpened to a **standing structural fact** — flake ro-binds
  `~/.cargo/bin/doctrine` ⇒ in-jail `cargo install` is structurally impossible ⇒
  **no install race because no install** (cannot regress).
- D-B3 re-scoped around the **nested-userns feasibility question (OQ-2)**, not cost:
  the gate is whether unprivileged userns creation works in-jail (`bwrap
  --unshare-user --ro-bind / / true`), since the outer bwrap may seccomp-block
  `clone(CLONE_NEWUSER)`. codex/pi-only; ro-binds the marker only, never
  `settings.local.json`. Also reframed: build isolation is a perf/false-green
  concern, **not a trust signal** — trust rides the marker + import belt.

**G3 — the spawn-interface ADR is `ADR-011`** (slug `spawn-interface-agnostic-contract`).
**Cite ADR-011 by this id in PHASE-03 (G2) and PHASE-04 (G4).** It states the
harness-agnostic **contract** (D1 marker identity, D2 orchestrator owns
fork-or-mark+provision+per-wt-env-emission) + the **per-harness altitude table**
(D3) + codex/pi enhancement layering (D4) + the charge-2 base-pinning residual (D5)
+ the **φ two-valued claude fail-closed cell** (D6, marked `proposed`) + the **σ
blast-radius confession** (D7). It **references ADR-006, does not amend it** (that
is G2/PHASE-03, post-spike).

**Status discipline:** ADR-011 stays `proposed` — its φ cell (D6) is O3-spike-
contingent. PHASE-03 (G2) firms the φ cell to the PHASE-02 spike outcome and is the
phase that promotes ADR-011 toward `accepted`.

**VH-1 gate:** PHASE-01 exit requires the **design owner to accept** the ADR-008
transition and the ADR-011 draft before PHASE-02 (the O3 spike) begins. **Accepted**
(ADR-008 accept + the two ADR-011 review fixes: D4 base-pinning overclaim dropped,
D2 env emission-vs-delivery distinction; commits 75570b6, 7da9907).

## PHASE-02 — O3 spike result (THE gate) — empirically RED for the WorktreeCreate fail-closed path

Probed the **live** Claude Code harness (temporary WorktreeCreate + SubagentStart
hooks, `isolation: worktree` Agent spawns; all artifacts cleaned up). The deployed
behaviour **contradicts the published hook docs** — verified empirically, as the
standing note mandates.

**WorktreeCreate (the design's fail-closed primary path) — RED:**
- Fires for an `isolation: worktree` Agent spawn; **mid-session settings-hook edits
  take effect without restart** (agent-def *registration* does NOT — a mid-session
  `.claude/agents/*.md` is absent from the Agent registry until a fresh session).
- **Replaces creation, fail-closed:** a no-stdout hook → `WorktreeCreate hook
  failed: no output` → creation aborted. The fail-closed *capability* is real.
- **Actual payload:** `{session_id, transcript_path, cwd, hook_event_name, name}`
  where `name` = the agent *instance* id (`agent-<hex>`). **NO `subagent_type`/
  `agent_type`, NO `worktree_path`, NO `base_path`/`ref`.** (The docs claimed all of
  these — wrong for this version.)
- Consequences: (1) **cannot gate on subagent_type** at WorktreeCreate; (2) **no
  target dir in the payload** → a `create-fork` cannot know where to create; (3)
  **no base/parent field → charge-2 session-HEAD residual STANDS** (no hook-time
  `base-moved` pre-refusal possible). The default-created worktree HEAD was *not*
  the orchestrator's HEAD — base is opaque + not orchestrator-controlled, sharpening
  charge-2. **So `run_create_fork`-via-WorktreeCreate as specced in PHASE-10/§4b is
  NOT buildable on this version.**

**SubagentStart (the design's O3-red fallback) — GREEN:**
- Payload `{session_id, transcript_path, cwd, agent_id, agent_type, hook_event_name}`.
  **`agent_type` present** (named, e.g. `general-purpose` — matcher-scopable to
  `dispatch-worker`); **`cwd` = the worker's worktree path**.
- → A SubagentStart hook **can provision + stamp the marker into `cwd`** (the
  worktree), scoped by `matcher: dispatch-worker`. Fires *after* creation ⇒ the
  **fail-open created-but-unstamped window** (the D6 O3-red altitude, verbatim).
- **σ note (better than the WorktreeCreate branch):** the SubagentStart `matcher`
  scopes the stamp to `dispatch-worker` cleanly ⇒ **no blast radius on benign
  isolated subagents** (they never hit the stamp hook). The WorktreeCreate
  matcher/else-branch question is **moot** — there is no usable WorktreeCreate
  gating at all.

**Untested (low marginal value):** the literal `dispatch-worker` agent_type through
SubagentStart (registry needs a fresh session) — but `general-purpose` already
proved the *mechanism*; a custom name uses the same `agent_type` field + matcher.
WorktreeCreate's RED is **schema-level** (no type/path fields exist), so name-
independent — a custom def changes nothing there.

**Verdict (feeds PHASE-03 G2 + ADR-011 φ firming + PHASE-10):**
- **φ cell → O3-RED row.** claude fail-closed via WorktreeCreate is **not
  achievable** on this version; the achievable altitude is **SubagentStart-stamp
  (fail-open window) → prompt-enforced**, accident-fenced + prompt-enforced, not
  malice-proof (ADR-011 D6 O3-red).
- **PHASE-10 pivot:** drop `run_create_fork`-via-WorktreeCreate as the primary;
  **`run_stamp_subagent` (SubagentStart, matcher-scoped) becomes the primary claude
  mechanism** + Claude default worktree creation + provision-at-SubagentStart. The
  WorktreeCreate create-fork is deferred until the harness payload carries
  type+path (or an IDE-004 channel lands).
- **charge-2 base-pinning residual STANDS** and is sharper than assumed (opaque base).
- This is **within the locked design** — D6 is explicitly two-valued and named the
  O3-red row; the spike merely *selects* it (and finds it harder-red than the
  optimistic arm). PHASE-03 (VH-1) encodes it into ADR-006 + firms ADR-011 φ.

## PHASE-03 prep — pivot decision + open blocking probe (design-owner steer)

**Decision (design owner):** pivot **#1 — SubagentStart-stamp is the PRIMARY claude
mechanism**; defer/drop the WorktreeCreate `run_create_fork` verb. **AND fold the
ADR revisions (ADR-006 G2 amendments + ADR-011 φ firming) into the reviewed design
surface for coherence, and LOCK them after the probe + scoped review — before
continuing the drive.**

**Planned sequence A→D (do not start PHASE-04+ until D locks):**
- **A. Empirical blocking probe (RE-RUN — incomplete).** Crux: does a SubagentStart
  command hook **block** the worker until it exits (stamp lands before the worker's
  first write), or run **concurrently** (race → wider fail-open window)? First run:
  hook fired fine (agent_type + cwd=worktree confirmed, 3s sleep observed, marker
  stamped) but the **general-purpose worker REFUSED** a bare "run this, nothing else"
  prompt — so no WORKER_FIRST timestamp. **Re-run with a legitimately-framed worker**
  (frame as a real harness diagnostic with context; bare command-only prompts trip
  the agent's skepticism — itself a note for real worker-prompt design: pre-distill
  *task context*, not bare orders). Probe: SubagentStart hook records HOOK_START/sleep
  3/stamp+HOOK_DONE timestamps; worker's first action timestamps itself + checks
  marker presence; compare WORKER_FIRST vs HOOK_DONE. Settings backup→merge→spawn→
  read→**restore** discipline (as used; always clean up `.claude/`).
- **B. Draft** the ADR-006 G2 amendments (D5/D9 creation-ladder: claude → SubagentStart-
  stamp primary + Claude default creation; D2a marker-primary + Orchestrator class) +
  firm **ADR-011 φ to the O3-red row** + note σ blast-radius becomes **moot** (matcher
  scopes SubagentStart cleanly — a simplification, confirm not a hidden loss).
- **C. Scoped adversarial review** (codex GPT-5.5 default; optional Opus pass for
  variety) over the pivot delta + the B drafts: SubagentStart blocking/race, what
  WorktreeCreate-primary silently covered, the dropped `run_create_fork` gap, provision-
  at-SubagentStart timing.
- **D. Fold findings → present for LOCK (VH-1 design-owner).** Then resume the drive
  (PHASE-04 G4 SPEC-012, then code phases 05+).

**Empirically reconfirmed this session:** SubagentStart carries `agent_type`
(general-purpose) + `cwd`=worktree; hook can write the marker into cwd; hook
duration honoured (3s). Blocking semantics: **OPEN**.

### PHASE-03 prep — A result (blocking probe RESOLVED — CLOSED)

Probe re-run with a legitimately-framed `general-purpose` worker at `isolation:
worktree`; SubagentStart matcher-`*` hook timestamping HOOK_START / sleep / stamp
marker into `cwd` / HOOK_DONE to a shared absolute log; worker's first command
timestamps WORKER_FIRST + marker-presence into the same log. Settings backed up,
hook merged mid-session (took effect, no restart), restored + worktrees/artifacts
removed after. **Two findings:**

1. **BLOCKING — CONFIRMED.** The SubagentStart **command** hook gates worker
   execution until the hook process exits. Two runs, sleep scaled to disambiguate
   from worker startup latency:
   - sleep 3s → HOOK_DONE at +3.0s, WORKER_FIRST at +7.0s from HOOK_START.
   - sleep 10s → HOOK_DONE at +10.0s, WORKER_FIRST at +13.7s from HOOK_START.
   WORKER_FIRST tracks the sleep **1:1** and is strictly **after** HOOK_DONE in
   both → the worker cannot run until the hook returns. The worker–done lag is a
   ~constant ~3.7s intrinsic post-unblock startup, not a race. **Marker `present`
   at the worker's first action both runs.** ⇒ The D6 "created-but-unstamped"
   fail-open window is **NOT a worker-execution race** — the stamp is mechanically
   guaranteed present before any worker command, *when the hook succeeds*.

2. **NOT FAIL-CLOSABLE — confirmed by official docs + empirics (triple-checked,
   load-bearing for the ADR).** Settled against the authoritative source after two
   exit-code corrections from the design owner (read the docs first next time):
   - **Official docs** (`code.claude.com/docs/en/hooks.md`): **SubagentStart is a
     read-only event — "no blocking or decision control."** Exit 2 only shows
     stderr to the user; the subagent runs anyway. The exit-2-blocks table covers
     only `PreToolUse`/`PermissionRequest`/`UserPromptSubmit`/`UserPromptExpansion`/
     `Stop`/`SubagentStop`/`PreCompact`/`WorktreeCreate`; "other events" (incl.
     SubagentStart, SessionStart, Setup) are non-blocking. **There is NO documented
     hook event that fail-closed-aborts a subagent before it works.** Claude Code
     hooks are **sync by default** (`async: true` opts out).
   - **Empirics agree** (deployed version): an `exit 2` no-stamp hook fired around a
     benign **two-step** worker (`date +%s` then `expr 6 \* 7`) and it returned
     `STEP1=… STEP2=42 DONE` — both steps **and** the final summary completed. The
     prior single-tool run returned `391` likewise. So `exit 2` neither blocks nor
     defer-terminates the subagent. (The owner-cited "waits for current tool call
     completion before termination" is **not** in the docs and does not describe
     SubagentStart.)
   - Contrast **WorktreeCreate**, which **is** fail-closed (any non-zero exit fails
     *creation*). SubagentStart has no equivalent.
   - **Matcher confirmed doc-supported** on `agent_type` (`general-purpose`,
     `Explore`, `Plan`, custom names) — so scoping the stamp to `dispatch-worker` is
     spec-blessed, not just empirically observed (feeds the σ-moot finding in B).
   ⇒ SubagentStart-stamp **cannot be made fail-closed.** The
   "guaranteed-present-before-worker" property (finding 1) holds **only when the
   hook succeeds**; on stamp failure the worker proceeds **unstamped and
   un-gateable by any hook**. The fail-open residual is a **hook-failure case**, not
   a timing race — so the fence against an unstamped worker is the **import belt +
   `DOCTRINE_WORKER` worker-mode guard + the pre-distilled prompt**, never a hook
   exit status. (Docs list the WorktreeCreate payload as `{…, worktree_id,
   base_path}`; the *deployed* payload showed `name`=instance-id and no base_path —
   docs and deployed still diverge, but **both lack `agent_type`** ⇒ create-fork
   cannot gate-on-type regardless.)

**Net for B/ADR:** the achievable claude altitude is **stronger on the race axis**
than ADR-011 D6 O3-red optimistically claimed (no worker-write race at all), but
carries a **distinct hook-failure fail-open** that WorktreeCreate did not (its
fail-closed capability was real). The φ row should read: *SubagentStart-stamp,
blocking ⇒ stamp-before-worker on hook success; fail-OPEN on hook failure (no
exit-status gate); accident-fenced + prompt-enforced, not malice-proof.* charge-2
opaque-base residual unchanged.

**Worker-prompt note (reconfirmed, sharper):** the bare/suspicious framing trips
`general-purpose` skepticism — the 3rd worker **refused** the failure-variant
command (flagged it as writing a misrepresenting `WORKER_RAN` marker + bypassing
`/route`). Its *turn ran* (giving the fail-open datum), but it declined the op.
Real dispatch-worker prompts must carry genuine task context + rationale, never
bare "run this" orders — itself input to PHASE-05+ worker-prompt design.

## PHASE-03 (G2) — LOCKED ✅ (B→C→D complete)

The A→D sequence is done; the governance is locked. Surface: `g2-draft.md` (pivot
delta + draft amendments + the full C-review findings + D-disposition).

**B — drafts** authored in `g2-draft.md` (ADR-006 G2 amendment, ADR-011 φ/σ/D5
firming, PHASE-10 re-scope, open items).

**C — adversarial review** (codex GPT-5.5 primary + independent Opus verify/extend).
Net **3 blockers, 4 majors, 3 minors** — all in `g2-draft.md §6`. Headlines: B1 lost
pre-dispatch baseline-verify; B2 stamp-failure privilege fail-open; B3 legit-hook
exemption breaks (SubagentStart `cwd` IS the worker worktree); M1 base residual
sharper (clean-applying-semantically-wrong import possible); σ-moot verified SOUND.

**D — owner rulings (VH-1):**
- **B1 → accept weaker class.** Claude loses ADR-006 D9 pre-dispatch baseline-verify;
  an unbuildable fork is caught late at `import → verify` (cost: wasted run ×batch
  width). Folded into ADR-006 D9 amendment.
- **B2+B3 → adopt the fix.** `is_linked_worktree && !marker_present` ⇒ **fail-closed**
  (refuse Orchestrator/Hook-mint/write); `marker --stamp-subagent` exempt **by verb
  identity**. Also closes the deliberate self-clear. Folded into ADR-006 D2a + ADR-011
  D6.
- **M1 → accept as confessed residual.** Sharpened opaque-base residual named in
  ADR-011 D5; import-time content-base assertion deferred to IMP-043.

**Locked artifacts:**
- **ADR-006** amended (`accepted`): D2a (marker-primary signal + Orchestrator class +
  the marker-absent fail-closed rule + stamp-verb identity exemption); D9 amendment
  (claude rung = SubagentStart-stamp; create-fork deferred; baseline-verify weaker
  class).
- **ADR-011** firmed + **promoted `proposed → accepted`**: D3 table (claude marker
  writer / base / fail-closed cells), D4 (create-fork stale ref fixed), D5 (opaque
  base + M1 residual + multi-commit not head-moved), D6 (φ RESOLVED to O3-red:
  blocking + read-only + privilege-fenced), D7 (σ WITHDRAWN), Consequences +
  Verification rows.
- **design.md §4b** — SUPERSEDED banner added (pivot pointer); detailed §4b/§5/§11/§12
  rewrite deferred to PHASE-10 prep.

**Carried into PHASE-10 (code re-scope — see `g2-draft.md §4` + §6 M4):**
1. `run_create_fork` **deferred/dropped from v1**; `run_stamp_subagent` is the primary
   claude verb (thinner: no `git worktree add`; provision+stamp into `cwd`).
2. `classify_create` three-valued **collapses** (no `PlainCreate` else-branch — σ moot).
3. Worker-mode: implement the **marker-absent-in-linked-worktree fail-closed** rule;
   `run_stamp_subagent` exempt by verb identity (B2/B3).
4. `src/boot.rs`: WorktreeCreate `HookSpec` → **matcher-scoped `SubagentStart`
   HookSpec**.
5. design §4b/§5/§11/§12 are **internally stale** vs the deferral (M4) — rewrite at
   PHASE-10 prep, not carried as the build target.
6. **M3:** provision now runs inside the read-only stamp hook — a mid-copy provision
   failure leaves a half-provisioned worktree + a running worker (un-rollback-able,
   un-abortable). Design the stamp verb's failure posture accordingly.

**Open IMP (tracked in backlog):** fresh-session probe of the literal `dispatch-worker`
SubagentStart **matcher** path (M2 — currently doc-supported, not end-to-end proven;
agent-def registry needs a fresh session).

**Next:** resume the dispatch drive — PHASE-04 (G4 SPEC-012 rewrite, inline), then code
phases 05+ via workers.

## PHASE-04 — G4 SPEC-012 rewrite (done · commit 8dbc029)

Prose-only; no code, no gate. SPEC-012 rewritten downstream of locked ADR-006 (G2)
+ ADR-011.

- **Overview/Concerns reframed** — funnel = enforced `Orchestrator` verb family +
  the worker-mode guard, not "a discipline carried by skill text". Forbidden framing
  swept (VA-1 grep clean): no "discipline, not enforced code" / "fails open at the
  env boundary".
- **D3 rewritten** env-fails-open → **disk-marker-primary, fail-closed guard**
  (`worker_mode = (is_linked && marker_present) OR env DOCTRINE_WORKER`; marker-absent
  linked worktree refused). **+D5** funnel verb family; **+D6** per-harness altitude
  (codex/pi explicit-base + pre-dispatch verify vs claude O3-red SubagentStart-stamp —
  not fail-closable, no pre-dispatch verify, opaque base/IMP-043); **+D7** honest scope
  (narrow `.claude/` belt = force-add only / solo non-squash `land` / quiescent v1
  import, one-landing-per-base).
- **New "Per-harness altitude" subsection** states the codex/pi vs claude reach
  faithfully to ADR-011 D3/D5/D6 (the locked O3-red truth, NOT the deferred create-fork
  story).
- **Requirements:** revised **REQ-192 (FR-004)** to the marker-primary guard (slug +
  symlink renamed `worker-mode-guard-disk-marker-primary-fail-closed`); added
  **FR-006..010 = REQ-248..252** (fork / import / land / gc / per-wt-env-contract). All
  `pending` (forward-intent — verbs land PHASE-05+). `spec validate SPEC-012` clean.
- Watch-out honoured: did NOT describe `create-fork`/WorktreeCreate as the live claude
  mechanism (named deferred); altitude reflects the weaker baseline class + the
  marker-absent fail-closed rule.

**Next:** PHASE-05 — first code phase (agnostic trust core: `worker_mode`
marker-primary + Orchestrator class + `worktree status`/`marker --clear`). Re-read the
**dispatch skill** funnel/escalation contract before the first worker phase; PHASE-05+
implement the B2/B3 marker-absent fail-closed rule (notes PHASE-10 carry-forward +
g2-draft §4/§6).
