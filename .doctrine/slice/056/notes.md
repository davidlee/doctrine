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

## PHASE-05 — first code phase: dispatched, then BLOCKED on a governance conflict

Ran via `/dispatch`. Drive isolated onto a **coordination worktree** because `main`
was concurrently dirty + HEAD-moving (SL-057 design in flight): `git worktree add
.worktrees/sl056-coord -b sl056-coord <B>`, B=`b324547`. The whole dispatch drive
runs there; `main`'s foreign WIP is untouched. (NB: a coordination *worktree* is
itself `is_linked` — relevant to the floor decision below; this setup is a niche
workaround, not normal usage where coordination == primary tree.)

**Worker delta landed clean, then the conflict surfaced.** Worker fork
`sl056-p05-21784` → single non-merge commit, funnel-imported as **`ec81b5e`** on
`sl056-coord` (X-1/X-2/R-5 belt all clean; combined-tree verify green except one
pre-existing red — `governance_corpus_supersession`, un-migrated `adr-011.toml`,
SL-057 scope, red at B independent of the delta). The worker built `worker_mode =
(is_linked && marker_present) OR env` — marker-PRESENT→refuse, marker-ABSENT→allow —
faithful to plan PHASE-05 VT-1c + design §3.

**The conflict (load-bearing):** that's **Option C**, but it contradicts the locked
**ADR-006 D2a** fail-closed amendment (marker-absent linked worktree → REFUSE). And
D2a's fail-closed itself contradicts **D6a** ("mode, not location, decides"; solo
`/execute` direct-writes in its worktree). The G2 review (codex+Opus) closed the
claude stamp-failure hole via fail-closed but **never recorded the D6a conflict / the
solo-in-worktree cost.** Edit recency confirms fail-closed is the *later* edit
(742d839, 21:27) vs plan (c5b0404, 18:53) — but "later wins" is now itself in
question because the belt may make fail-closed unnecessary. See
[[mem.pattern.dispatch.verify-governance-freshness-before-distilling-worker]].

**Decision staged:** `worker-mode-floor-decision.md` (committed on main) lays out
**C** (drop fail-closed, the import R-5 belt is the fence, keep D6a + design §3 + plan
+ delta `ec81b5e`) vs **A→B** (keep fail-closed, additive writer-marker later;
high-churn: re-amend D6a, rewrite §3/plan, drop `marker --clear`, re-dispatch). Owner
steer: **C**. The belt-containment claim (an unstamped worker's doctrine writes are
caught at import / never imported / minting caught by validate) is C's load-bearing
premise — charge 1 of the adversarial agenda.

**PHASE-05 status = `blocked`** pending the codex adversarial pass on the decision,
then lock (VH), then resume. Worker fork `sl056-p05-21784` + delta `ec81b5e` parked.

### PHASE-05 — RESOLVED: locked **Option C + IMP-052 rider** (VH, 2026-06-13)

Codex (GPT-5.5) reviewed `worker-mode-floor-decision.md` §6 adversarially → verdict
**A→B**. Verified its findings against source:

- **Charge 1 (load-bearing):** codex's *mechanism* was wrong — `worker_guard` resolves
  the root via `root::find(None,…)` (cwd-walk), **not** the command `--path`
  (`src/main.rs`, delta `ec81b5e`). But the *conclusion* held: under C a marker-absent
  linked fork passes the guard, and the write verb's own `--path` can then target the
  coordination root, escaping the `B..S` import belt. A would close that for free. **But
  it only bites a worker that targets `-p <coord-root>`** — malice / derailment, which
  the note scopes out; the in-scope cwd-write accident the belt *does* contain.
- **Charge 4/6 (verified):** the fail-closed floor was baked into **ADR-011 D6** + the
  PHASE-03 lock B2/B3 disposition, not just D2a — so C is a multi-clause reversal.

**Lock rationale (owner risk calculus, not sunk cost):** `P(SubagentStart hook failure)
≈ 0` (hook blocks; a miss needs a crash) **×** *jail-bounded* harm (bubblewrap, no push
⇒ worst case = lost unpushed progress) ⇒ the security delta between A's floor and C's
funnel is **negligible**. The jail is the real outer fence. The one real residual under
C is **silence** (a failing hook quietly normalising), and the floor was the *wrong
layer* for it — closed instead, behaviour-independent at spawn time, by the
**orchestrator post-spawn marker check that aborts an unstamped fork (IMP-052)**:
enforce where the harness *can* abort, not at the CLI write seam. Full reasoning in the
decision note §7.

**Applied (authored, on `main`):** ADR-006 D2a re-amended (fail-closed → positive
signal, rationale recorded); ADR-011 D6 + M2 aligned (funnel+jail+IMP-052 posture);
design §3 belt-as-fence pointer added; plan PHASE-05 unchanged; **IMP-052** minted;
decision note stamped LOCKED + §7. Worker delta **`ec81b5e` kept** (it implements C).
**Next:** re-verify `ec81b5e` against the amended ADR, flip PHASE-05 `blocked →
completed`, continue the drive to PHASE-06.

## PHASE-06 — `fork` verb + per-wt env contract (done · /dispatch · `sl056-coord` e3d3ca2)

Driven via `/dispatch` (one worker, batch of one). Base **B = `ec81b5e`**; worker
fork `sl056-p06` returned **S = `ee504a2`** (`S^ == B`, single non-merge). Funnel
clean: precond clean+HEAD==B → S^==B → R-5 belt (src+tests only, no `.doctrine/`)
→ `git apply` net diff → verify → branch-point stationary → one commit `e3d3ca2`.

**Built (source delta):**
- `doctrine worktree fork --base/--branch/--dir [--worker]` (`run_fork`): step-1
  `git worktree add -b <branch> <dir> <B>` with pre-add refusals (dir-exists,
  branch-exists, B-not-a-commit → no fork); step-2 reuses `run_provision`; step-3
  `write_marker` under `--worker` (before any spawn window; solo omits); step-4 env
  contract on **stdout**, human status on **stderr**.
- **Compensating-cleanup rollback** (not a git transaction) factored as reusable
  `rollback_fork(repo, branch, dir)` — PHASE-10 `create-fork` reuse. Post-add
  failure reaps worktree+branch+dir; leftover → distinct non-zero
  `fork-rollback-debris:` token naming dir+branch; clean rollback re-raises the
  original cause.
- Pure `target_dir_for_branch(branch) -> wt/<branch>` (unit-tested). The
  `CARGO_TARGET_DIR` consumer is **project-declared** (`project_env_contract`, jail
  base from inherited `CARGO_TARGET_DIR` else `<fork>/target`) — kept separate from
  the generalisable mechanism; `run_fork` emits whatever pairs the consumer returns
  and never names `CARGO_TARGET_DIR` itself (ADR-008 D-B5 honoured).
- New **`Orchestrator(&'static str)` WriteClass** variant (first member; `import`/
  `land`/`gc` join later). `write_class` Fork arm → `Orchestrator("fork")`;
  `worker_guard` refuses Orchestrator via the SAME branches as Write. Other arms
  behaviour-preserving. Removed the `cfg(not(test)) expect(dead_code)` on
  `write_marker` (now has `run_fork` as non-test consumer); `git::git_opt` made
  `pub(crate)` for the B-is-a-commit probe.
- Floor posture **C** ridden as-is (no fail-closed floor added).
- Goldens `tests/e2e_worktree_fork.rs` (4): happy solo+worker (VT-1), pre-add
  refusals leave no fork, rollback-on-provision-failure (VT-2), Orchestrator
  refusal drives run() from marked-fork AND env-set (VT-4). VT-3 (parallel-build
  per-wt target) is codex/pi env-level — not in the worker delta.

**Verify:** `env -u DOCTRINE_WORKER cargo test -p doctrine` — all suites green
**except** the pre-existing `governance_corpus_supersession_…` (e2e_relation_
migration_storage.rs): foreign in-flight SL-048 relation-migration condition on
`.doctrine/adr/011/adr-011.toml` (not in the PHASE-06 delta). **Correction to the
PHASE-05 handover claim that it "PASSES at B":** it reds at B too — proven by
stashing the import and re-running at clean `ec81b5e` (identical failure). It is
NOT a PHASE-06 regression. clippy zero. (`just` broken in fork — used `cargo`.)

**Next:** PHASE-07 (`import` funnel verb) — EN: PHASE-06 green ✓.

## PHASE-07 — `import` verb + governance/config belt (done · /dispatch · `sl056-coord` 436da7d)

Driven via `/dispatch` (one worker, batch of one). Base **B' = `e3d3ca2`**; worker
fork `sl056-p07-import` returned **S = `5cc9c5b`** (`S^ == B'`, single non-merge).
Funnel clean: precond → S^==B' → R-5 belt (src+tests only) → `git apply` → verify
→ branch-point stationary → one commit `436da7d`.

**Built (source delta):**
- `doctrine worktree import --base <B> --fork <branch>` — mechanizes the funnel's
  deterministic steps as one fail-closed verb (v1 stationary-head only).
- Pure `classify_import(head_at_base, tree_clean, single_commit, &delta_paths) ->
  Result<Apply, Refusal>`: precond order HEAD→tree→single-commit→belt; belt
  prefix-matches `.doctrine/` then `.claude/` (no special-casing). Exhaustive
  refusal set `{head-moved, tree-unclean, multi-commit, doctrine-touch,
  claude-touch}`, each a distinct token.
- Shell `run_import` reuses `resolve_commit`/`matches` (HEAD==B), tracked-only
  `status --untracked-files=no`, `S^==B`, name-only `B..fork` diff → `git apply
  --3way --index` **non-committing** (orchestrator commits separately; no runtime
  receipt — landed-ness stays git-derived for gc §8.1). New impure
  `git::git_apply_index` (patch on stdin) — the only added git seam.
- `import` joins the **Orchestrator** class (`write_class` arm `Import =>
  Orchestrator("import")`; `worker_guard` unchanged — already refuses Orchestrator).
- Goldens `tests/e2e_worktree_import.rs` (8): VT-1 happy drives run() asserting
  delta STAGED+uncommitted & HEAD==B; VT-2 all 5 refusal tokens; VT-3 untracked
  scratch ignored; VT-4 Orchestrator refusal (marked-fork AND env-set).

**Verify:** full `-p doctrine` suite green EXCEPT the same pre-existing foreign
`governance_corpus_supersession_…` red (SL-048 WIP, reds at base too — not a
PHASE-07 regression). clippy zero.

**Next:** PHASE-08 (`land` verb — solo's non-squash coordination merge) — EN:
PHASE-07 green ✓. (This line previously misnamed PHASE-08 as `gc`; per plan.toml
PHASE-08 = `land`, PHASE-09 = `gc`.)

## PHASE-08 — `land` verb (solo non-squash coordination merge) (done · /dispatch · `sl056-coord` accfc0e)

Driven via `/dispatch` (one worker, batch of one). Base **B = `436da7d`**; worker
fork `sl056-p08-land` returned **S = `5dd654f`** (`S^ == B`, single non-merge).
Funnel clean: precond → S^==B → R-5 belt (src + new test only) → `git apply
--3way --index` → verify → branch-point stationary → one commit **`accfc0e`**;
fork reaped.

**Built (source delta — src/worktree.rs, src/main.rs, tests/e2e_worktree_land.rs):**
- `doctrine worktree land --fork <branch>` — solo `/execute`'s analog of `import`
  (design §6). Lands a solo MULTI-commit isolated-worktree TDD branch with ancestry
  PRESERVED via `git merge --no-ff` (the verb has **no `--squash` path** — squash is
  gc-uncertifiable, §8.1; ancestry preserved ⇒ gc ancestry leg reaps).
- Pure `classify_land(tree_status_clean, _head, ForkState{exists, has_live_worktree,
  bears_marker}) -> Result<Merge, LandRefusal>` returns the **4 precond** refusals
  only; precedence `tree-unclean → no-such-fork → worktree-gone → dispatch-fork`
  (worktree-gone gates dispatch-fork so the marker check can't pass *vacuously* on a
  worktree-less branch). The **3 merge-time** refusals are shell-determined.
- Exhaustive **7-token** `LandRefusal` (new enum — import's `Refusal` NOT widened):
  `{tree-unclean, no-such-fork, dispatch-fork, worktree-gone, merge-conflict,
  wedged-merge, inconsistent-merge-state}`, each a distinct token.
- Shell `run_land`: gather → classify → `git merge --no-ff --no-edit` (via
  `git_opt`). On merge failure: probe `MERGE_HEAD` — absent ⇒ `inconsistent-merge-state`;
  present ⇒ capture unmerged paths, **`git merge --abort` FIRST** (restore clean
  tree), success ⇒ `merge-conflict`, abort failure ⇒ `wedged-merge` (names MERGE_HEAD
  + unmerged paths + tree-not-clean + manual remedy). New shell gather
  `gather_fork_worktree` parses `git worktree list --porcelain` (`branch
  refs/heads/<fork>` → path) backing has_live_worktree + `marker_present`.
- **Shared, no parallel impl:** import's tracked-tree-clean gather extracted to
  `gather_tree_clean`; `run_import` rewired onto it. import 8/8 e2e stay green
  (behaviour-preservation gate held).
- **Orchestrator**-classed (`write_class` arm `Land => Orchestrator("land")`;
  `worker_guard` unchanged — auto-refuses). New `WorktreeCommand::Land` + dispatch arm.
- Goldens `tests/e2e_worktree_land.rs` (9): VT-1 happy (2-parent `--no-ff` merge,
  fork tip ancestor of HEAD, no `--squash` flag); VT-2 4 precond tokens; VT-3
  merge-conflict (abort-first → clean tree proof) + inconsistent-merge-state (unrelated
  histories); VT-4 Orchestrator refusal (marked-fork names `land` AND env-set
  dual-cause). Plus 3 lib unit tests (classify precedence, head-invariance, 7-token
  distinct/exhaustive table).

**Gotchas / decisions (durable):**
- `head` is **intentionally unused** by the 7-token pure logic — kept in the
  `classify_land` signature (as `_head`) to document the contextual "HEAD is the
  coordination branch" precond, which carries **no token** (design §6). A unit test
  pins verdict-invariance under any HEAD. No architectural escalation needed.
- `wedged-merge` is **not deterministically black-box reproducible** (fires only when
  `git merge --abort` itself fails — corrupted/locked git internals); its token is
  pinned by the exhaustive-table unit test, e2e gap documented. `inconsistent-merge-state`
  IS black-box reachable via an unrelated-history fork (git refuses to start the merge,
  no MERGE_HEAD). `merge-conflict` requires coord HEAD moved off B (a coord-side commit
  on the same file) — valid since `land` has NO head-at-base precond (unlike `import`).

**Verify:** full `-p doctrine` suite green EXCEPT one pre-existing **foreign** SL-048
relation-migration red. NOTE the foreign red **shifted** this session: the old
`governance_corpus_supersession_…` now PASSES; `scaffolded_entities_are_post_cut_shape_
all_six_paths` (`e2e_relation_migration_storage.rs`, slice-template `[relationships]`
cut) now reds — **proven independent** (stashed the land delta, it reds identically at
clean B; land touches no templates/install). clippy zero. land 9/9, import 8/8,
worker_guard green.

**Next:** PHASE-09 (`gc` verb — idempotent state machine, two-leg `git cherry` landed
oracle: ancestry leg for a landed `land`, all-`-` patch-id leg for a landed `import`)
— EN: PHASE-07 + PHASE-08 green ✓ (both landing routes now exist). Base **B = accfc0e**.

## PHASE-09 — `gc` verb (idempotent reaper + two-leg landed oracle) (done · /dispatch · `sl056-coord` 53c53fe)

Driven via `/dispatch` (one worker, batch of one). Base **B = `accfc0e`**; worker
fork `sl056-p09-gc` returned **S = `f655dac`** (`S^ == B`, single non-merge).
Funnel clean: precond → S^==B → R-5 belt (src/git.rs + src/main.rs + src/worktree.rs
+ tests/e2e_worktree_gc.rs) → `git apply --3way --index` → verify → branch-point
stationary → one commit **`53c53fe`**; fork reaped.

**Built (source delta — src/git.rs, src/worktree.rs, src/main.rs, tests/e2e_worktree_gc.rs):**
- `doctrine worktree gc --fork <branch> [--superseded-head <SHA>] [--force]
  [--dry-run]` — reaps a spent worktree fork in one idempotent act (design
  §8/§8.1/§8.2). Forced order: `git worktree remove` → `git branch -D` → reap
  `wt/<branch>` target dir → stderr-warn on stale `CARGO_MANIFEST_DIR`-baked binaries.
- Pure `classify_gc(GcState{branch_exists, worktree_present, target_present,
  landed_verdict: Option<bool>}, force, superseded_match, dry_run) -> GcVerdict`
  (`Reap(GcPlan{remove_worktree, delete_branch, reap_target})` | `Refuse(GcRefusal)`).
  Idempotent: a step is planned only when its target is present (skip completed);
  branch-gone ⇒ already-certified (deletion IS the certificate, `landed_verdict`
  None), reap only T from the branch NAME; each destructive step appends to a
  `leftovers` vec and exits non-zero `gc-incomplete` if its target survives; stale
  admin entry folded via `git worktree prune`.
- Two-leg landed oracle (shell `gather_landed`): ancestry (`git merge-base
  --is-ancestor <fork-tip> <coord-HEAD>`) ∪ patch-id (`git cherry <coord-HEAD>
  <fork>` all `-`). New git.rs reads: `git_cherry` (via `git_text`) + `git_status_ok`
  (exit-status boolean — `--is-ancestor` prints nothing, so `git_opt`'s `Some("")`
  is ambiguous; a dedicated boolean-exit helper reads cleanly).
- `--superseded-head <SHA>` reaps iff SHA == branch's current head (TOCTOU
  movement-guard, gathered in shell); `--force` bypasses the oracle; `--dry-run`
  prints the per-fork verdict, destroys nothing.
- T-reap base: `gc_target_dir` mirrors `project_env_contract` verbatim
  (`CARGO_TARGET_DIR` env base else `<fork>/target`, joined with pure
  `target_dir_for_branch` = `wt/<branch>`). Reuses `gather_fork_worktree` (PHASE-08)
  for `worktree_present`.
- **Orchestrator**-classed (`write_class` arm `Gc => Orchestrator("gc")`).
- Goldens `tests/e2e_worktree_gc.rs` (12) + 6 lib unit tests. VT-1 reap + both
  oracle legs + non-ancestor `+` refuses; VT-2 squash named-refusal + superseded-head
  honesty + dry-run; VT-3 idempotent rerun (fail-after-each-step, branch-gone+T,
  W-before-B); **VT-4 the EXHAUSTIVE 4-verb (fork/import/land/gc) Orchestrator-refusal
  set** from a marked fork AND env-set process (`marker --clear` kept out).

**RATIFIED DESIGN CORRECTION (the crux):** design §8.1 reads as if a squash-merge
gets its OWN detectable refusal. **Structurally impossible** — a multi-commit
`git merge --squash` makes `git cherry` list every fork commit `+` and the tip
non-ancestor, **identical to a never-landed fork** (verified empirically by me +
the worker). No durable git signal says "squashed". Collapsed to ONE
`GcRefusal::NotLanded` whose message names BOTH remedies (`--force` /
`--superseded-head` for the spent-and-abandoned case AND re-land via `worktree land
--no-ff` for the squash case) — faithful to §8.1's actual text ("trips neither leg
and gc refuses with a **named message**" — a message, not a distinct token). This
is the load-bearing reason solo MUST land non-squash (§6). Recorded as
`mem.pattern.dispatch.gc-squash-indistinguishable-from-unlanded`
(mem_019ec166d8bf7903a353688035ce38b4).

**Verify:** full `-p doctrine` suite green EXCEPT the one pre-existing **foreign**
SL-048 relation-migration red in `e2e_relation_migration_storage.rs` (the failing
test NAME flapped again this session — now `governance_corpus_supersession_…` —
as SL-048 actively lands; proven foreign by stashing the gc delta → reds identically
at clean B). clippy zero. gc 12/12, import 8/8, land 9/9, worker_guard 6/6 green.
Clippy notes: `superseded_head` is `Option<&str>` (not `Option<String>`) to satisfy
`needless_pass_by_value`; dispatch arm passes `.as_deref()`. The 4-flag `Gc` clap
variant did NOT trip the bool/arg ceiling (a derive struct variant, not a fn).

**Next:** PHASE-10 (claude `create-fork` WorktreeCreate hook handler +
`run_stamp_subagent` + `install/agents/claude/dispatch-worker.md`) — EN: PHASE-02
green ✓ (the O3 spike disposition). Base **B = 53c53fe**. NOTE: the four core verbs
(fork/import/land/gc) are now COMPLETE; remaining phases (10–13) are the claude
spawn path + `claude install` + bwrap profile + skill rewrites.

## PHASE-10 — `marker --stamp-subagent` (SubagentStart provision+mark) (done · /dispatch · `sl056-coord` a74a513)

**RE-SCOPED** off the dead WorktreeCreate `create-fork` path (PHASE-02 O3-RED) onto
the **SubagentStart-stamp** mechanism (re-scope authored on main `9f265eb`: design
§4b/§5/§9/§11/§12 + decision-tree + plan.toml PHASE-10). Driven via `/dispatch` (one
worker, batch of one). Base **B = `53c53fe`**; worker fork `sl056-p10-stamp` returned
**S = `0274417`** (`S^ == B`, single non-merge). Funnel clean: precond → S^==B → R-5
belt (install/agents/claude/dispatch-worker.md + src/main.rs + src/worktree.rs +
tests/e2e_worktree_stamp.rs — NO `.doctrine/`) → `git apply --3way --index` → verify
→ branch-point stationary → one commit **`a74a513`**; fork reaped.

**Built (source delta — src/worktree.rs, src/main.rs, install/agents/claude/dispatch-worker.md, tests/e2e_worktree_stamp.rs):**
- `doctrine worktree marker --stamp-subagent` — the claude harness spawn-path's
  provision+mark step. Reads `{cwd, agent_type}` JSON on stdin (SubagentStart
  payload), and on a valid dispatch-worker worktree runs `run_provision` (the SOLE
  copier, source = orchestrator tree) THEN `write_marker`. `create-fork` /
  WorktreeCreate is DROPPED (its payload carries no `agent_type`/path — see
  `mem.pattern.dispatch.claude-worktreecreate-payload-minimal-no-type-no-path`).
- Pure `classify_stamp(agent_type, cwd_present, cwd_is_under_repo_linked_worktree)
  -> Result<Stamp, StampRefusal>` — TWO-valued (Stamp::Ok vs Refuse), NO PlainCreate
  branch (the matcher scopes the hook to dispatch workers, so a benign subagent never
  reaches the verb). Precond order cwd-presence → dir-validity → agent-type; three
  distinct fail-closed tokens `missing-cwd` / `bad-dir` / `missing-agent-type`.
- `DISPATCH_WORKER_AGENT_TYPE = "dispatch-worker"` const — SINGLE source of truth for
  the discriminator; the agent-def `name:` must equal it (T6 drift test pins them).
- `cwd_shares_repo` = git-common-dir EQUALITY, not a path `starts_with` prefix (a
  linked worktree is a SIBLING dir, not a child) — same notion `verify_sibling_worktree`
  uses. Impure git read gathered in the shell; `classify_stamp` stays pure.
- `WriteClass::Hookmint(&'static str)` variant (exhaustive, no wildcard) — refused
  under worker-mode via the SAME branch as Orchestrator/Write, **NO verb-identity
  carve-out**. The legit first stamp lands on a marker-absent worktree ⇒ `worker_mode
  == false` ⇒ allowed automatically. The `Marker { stamp_subagent: true, .. }`
  discriminate arm MUST sit BEFORE the `Marker { .. } => MarkerClear` catch-all.
- **M3 failure posture:** on provision/mark failure print a LOUD stderr diagnostic +
  exit non-zero, but do NOT `git worktree remove` (Claude owns the worktree; the
  worker is already cleared to run). No compensating rollback — the half-stamped fork
  is left for the orchestrator's post-spawn check. SubagentStart is read-only
  (exit≠0 does NOT abort the subagent — `mem.pattern.dispatch.subagentstart-blocking-but-not-failclosable`).
- `install/agents/claude/dispatch-worker.md` (NEW; source in `install/`, not
  `.doctrine/`) — agent def, `name: dispatch-worker` pinned to the const.
- Goldens `tests/e2e_worktree_stamp.rs` (8, VT-1..5 black-box, stdin-fed JSON) + 4
  pure `classify_stamp` arm tests + the T6 drift test.

**Worker decisions verified at import (all SOUND):** (1) `run_provision(Some(orchestrator_tree),
cwd)` — the literal prompt's `Some(cwd), cwd` would self-bail on `verify_sibling_worktree`;
worker mirrored `run_fork --worker`. (2) "under repo" = git-common-dir equality, not
prefix. (3) VT-1 asserts `run_provision`'s own `provisioned <path>` STDOUT line (the
sole copier, reused — cannot suppress); the stamp verb itself writes only STDERR.
(4) clippy-denied `unreachable!` replaced with a fail-closed `bad-dir` refusal.

**Verify:** full `-p doctrine` suite green EXCEPT the one pre-existing **foreign**
SL-048 relation-migration red in `e2e_relation_migration_storage.rs`
(`governance_corpus_supersession_…`, reds identically at clean B — delta touches
neither that test nor `.doctrine/adr`). New stamp suite 8/8, clippy zero.

**Next:** PHASE-11 (`claude install`: rename `skills install` → `claude install` +
hidden deprecated alias; agents-symlink leg; the **SubagentStart** HookSpec matcher
`dispatch-worker` merged via the `src/boot.rs` HookSpec core; charge-5 worker-mode
refusal). ⚠ plan.toml PHASE-11 still says "WorktreeCreate hook" in places — PHASE-11
prep must re-scope its hook leg to **SubagentStart**, mirroring PHASE-10's re-scope.
⚠ design §5 Hook-mint reconciliation flagged for the PHASE-10 VH lock — owner confirms
at close that the exemption rides Option C's positive signal (no verb-identity carve-out).

## PHASE-11 — `claude install` rename + agents leg + SubagentStart hook wiring (done · /dispatch · `sl056-coord` e87c522)

Re-scoped onto SubagentStart first (plan re-scope `7a56d96`: plan.toml PHASE-11
objective/EN-1/EX-1/VT-3 + plan.md PHASE-10/11 prose; the dead WorktreeCreate
`create-fork` references retired). Driven via `/dispatch` (one worker, batch of one).
Base **B = `a74a513`**; worker fork `sl056-p11-claude-install` returned **S =
`33f815f`** (`S^ == B`, single non-merge). Funnel clean: precond → S^==B → R-5 belt
(README.md + src/boot.rs + src/install.rs + src/main.rs + src/skills.rs +
tests/e2e_claude_install.rs + tests/e2e_worker_guard.rs — NO `.doctrine/`) →
`git apply --3way --index` → verify → branch-point stationary → one commit
**`e87c522`**; fork reaped.

**Built (source delta):**
- **THE CRUX — generalized the `src/boot.rs` HookSpec merge core over event+matcher
  (no parallel impl).** The core was generic over `HookSpec{command, is_ours}` but
  **hardcoded** the event (`SessionStart`) and matcher (`SESSION_MATCHER="startup|clear"`)
  at `desired_entry`/`session_start_array_mut`. Widened `HookSpec` with
  `event: &'static str` + `matcher: &'static str`; `desired_entry(spec)` emits
  `spec.matcher`; `session_start_array_mut` → generalized `hook_array_mut(value, event)`
  (navigates `hooks.<event>`); `fallback_for` reads the spec. boot/sync pass
  `("SessionStart", SESSION_MATCHER)` — **behaviour-preserving** (their tests @1869/@1905
  green unchanged). ONE implementation, three callers.
- `HookSpec::stamp_subagent(exec)` — command `<exec> worktree marker --stamp-subagent`,
  event `SubagentStart`, matcher = `crate::worktree::DISPATCH_WORKER_AGENT_TYPE` (the
  const, not a re-spelled literal — drift-pinned), `is_ours = is_doctrine_stamp_command`
  (suffix-strip ` worktree marker --stamp-subagent`, disjoint from boot/sync predicates).
- **`Command::Claude{ClaudeCommand::Install}`** (main.rs) — same args as
  `SkillsCommand::Install`; `Command::Skills` hidden (`#[command(hide=true)]`) as the
  deprecated alias → BOTH dispatch the one `skills::run_install` handler (SR-3). The
  separate top-level `Command::Install` (project-files installer) untouched. Args
  bundled in `ClaudeInstallArgs` (clap arg-ceiling, [[mem.pattern.lint.cli-handler-args-struct]]).
- **Agents leg** (skills.rs): `install_agents` mirrors the skills canonical+link model —
  materialize `.doctrine/agents/dispatch-worker.md` from the `install/` embed, symlink
  `.claude/agents/dispatch-worker.md` via the existing `relative_target`/`classify_link`/
  `write_link` (no parallel symlink). `.doctrine/agents/*` added to the derived-tree
  gitignore self-enforce (mirrors SL-010 F4 for skills). Idempotent reinstall.
- Agents + hook legs **gated on Claude being a resolved target**; `--global` **skips
  the hook** (the command is an absolute exec path that belongs out of git, consistent
  with boot/sync settings staying project-local). [worker judgment calls, reviewed
  SOUND at import.]
- `write_class`: `Claude{Install}` AND the hidden `skills install` alias both →
  `Write("claude install")` (charge-5, refused under worker-mode; exhaustive, no wildcard).
- Goldens moved `skills install`→`claude install` (e2e_worker_guard `WRITE_VERBS`,
  main.rs write_class_tests). README swept. New `tests/e2e_claude_install.rs` (VT-1
  alias→same-handler, VT-2 agent-def resolves, VT-3 hook merge preserves unrelated
  SessionStart hooks + idempotent). VT-5 (`claude_install_and_skills_alias_refuse_in_worker_mode`)
  drives `run()` from marked-fork AND env-set.

**Verify:** full `-p doctrine` green EXCEPT the one foreign SL-048
`e2e_relation_migration_storage` red (red at B; delta touches neither it nor
`.doctrine/adr`). Bin unit 1108 passed (+4 boot.rs merge tests over PHASE-10's 1104);
e2e_claude_install 2/2; e2e_worker_guard 7/7; clippy zero.

**Durable gotcha recorded:** the boot.rs HookSpec merge core was SessionStart/matcher-
hardcoded — generalized over event+matcher in PHASE-11 (memory
`mem.pattern.distribution.hookspec-merge-core-generalized-event-matcher`). The
`skill-refresh-command` memory updated (`skills install` → `claude install`).

**Deferred (backlog):** `.doctrine/spec/tech/010` still documents the `skills
install` surface name — the live tech-spec rename is orchestrator/authored work
outside the worker belt; captured to backlog rather than swept silently mid-dispatch.

**Next:** PHASE-12 (`dispatch-worker` bwrap profile — LAND the OS-floor D2b confinement
OR formal BACK-OUT to D-B1 + the D2a CLI guard, recorded symmetrically). Then PHASE-13
(the router/skill-rewrite prose leg). Base for PHASE-12 = `e87c522`.

## PHASE-12 — O7 nested-bwrap spike (codex/pi-only) — empirically GREEN → **LAND**

Inline spike (codex/pi-only, no worker dispatch — claude's `Agent` is not a
subprocess to wrap). Feasibility gate = **unprivileged userns creation inside the
jail**; probed empirically per the standing note, not inferred.

**Probe (EX-1, the feasibility gate) — GREEN:**
- `bwrap --unshare-user --ro-bind / / true` run **in-jail** → **exit 0**. The outer
  bubblewrap does **not** seccomp-block `clone(CLONE_NEWUSER)`; nested unprivileged
  userns creation is permitted. (`bubblewrap 0.11.2`, pre-staged in `jailPkgs`.)
- Userns is genuine, not a harness false-green: `id` inside the nested bwrap reports
  `uid=1000(david) gid=100(users)` — a real user-namespace mapping.

**D6 conformance (VT-1) — empirically demonstrated with the canonical profile:**
canonical arg-vector
`bwrap --ro-bind / / --bind "$D" "$D" --bind "$TARGET" "$TARGET" --ro-bind "$MARKER" "$MARKER" --dev /dev --proc /proc --unshare-user --chdir "$D" env DOCTRINE_WORKER=1 $fork_env <spawn>`.
The marker ro-bind **overlays** the rw worktree bind (bwrap applies binds in order,
so `--ro-bind $MARKER` AFTER `--bind $D` wins for that path):
- out-of-tree write (`touch /tmp/…`) → **DENIED** `Read-only file system` (`--ro-bind / /`);
  host `/tmp` file never created → no escape.
- in-tree source write → **OK** (worktree is rw).
- **`.claude/settings.local.json` write → OK** — it is inside the rw worktree and is
  NOT overlaid, so the harness permission-grant persistence keeps working under
  confinement (§9 constraint honoured: ro-bind the marker ONLY).
- marker `rm` → **DENIED** `Device or resource busy`; marker overwrite → **DENIED**
  `Read-only file system`. A confined worker cannot clear its own marker to escape
  worker-mode — D6's load-bearing invariant holds at the OS floor.

**Verdict → LAND (EX-2):** the `dispatch-worker` bwrap profile discharges ADR-006
**D2b** (worker-on-main / self-clear) at the OS floor for the codex/pi subprocess
arm. This is the **codex/pi-only** floor — claude's marker-only altitude (§10/G3)
is unchanged; D2b stays the deferred residual there (no subprocess to wrap).

**Landing form (no flake wrapper):** the profile is the **confined spawn template**,
not a packaged binary — design §11 line 152/637 places `bwrap … --chdir "$D"` inline
as the confined equivalent of the `env -C "$D"` spawn, and line 639 makes the
`flake.nix` change *conditional*. Since `bubblewrap` is already in `jailPkgs` and the
inline arg-vector is validated, **no flake.nix change is taken.** The profile is
recorded here (durable) and **embedded into `/dispatch-subprocess` prose in PHASE-13**
(its sole consumer) — avoiding a parallel home. [Phase-boundary judgment: PHASE-12
owns the validated profile + verdict; PHASE-13 owns its prose embedding. Flagged for
audit — consistent with the design's existing inline-template framing, not novel.]

**Symmetry note:** the LAND mirrors the SubagentStart fail-open back-out's recording
discipline — both spike outcomes recorded explicitly with their altitude confessed.
Here the spike greened, so VT-1 is **demonstrated** (not N/A).

**Verify:** probe + conformance run live in-jail (`bubblewrap 0.11.2`); no source/test
change in PHASE-12 (the spike is empirical + the profile is PHASE-13 prose). The VT-1
conformance becomes a durable assertion in the `/dispatch-subprocess` prose's spawn
template + a memory.

**Next:** PHASE-13 — embed the validated bwrap profile into `/dispatch-subprocess`,
the skill-prose rewrite (call-the-verbs), and the `/dispatch` harness router split.
