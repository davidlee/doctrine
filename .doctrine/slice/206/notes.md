# Notes SL-206: Workflow-templated slice-driver

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — De-risk gate (operator-run empirical, 2026-07-06)

Run operator-side (not a worker): PHASE-01 produces no source delta, evidence
lands in authored notes, and verification is VH/VA — it does not fit the
worker-delta funnel. Rig: throwaway scratch slice **SL-207** + its dispatch coord
tree (`.dispatch/SL-207` on `dispatch/207`); a one-agent dynamic Workflow
(`scratchpad/p1-forkprobe.js`) spawning `agent(isolation:'worktree')`, driver Bash
cwd parked at the arming dir, `dispatch arm-spawn --base <sha>` armed.

### EX-1 / SQ3 — confined fork mints at the ARMED base on the dispatch branch — **PASS**

Decisive disambiguation run — armed base set to coord **HEAD's parent** so the
armed base ≠ cwd HEAD:

| fact | value |
|---|---|
| coord HEAD (cwd) | `1fadff60` |
| armed base (`.doctrine/state/dispatch/spawn/base`) | `7fe7e231` (HEAD^) |
| fork `git rev-parse HEAD` | **`7fe7e231`** — == armed base, **≠ cwd HEAD** |
| fork `--git-common-dir` | `/workspace/doctrine/.git` (shared → confined linked worktree) |
| fork toplevel | `.dispatch/SL-207/.worktrees/wf_7030fd9a-8ae-1` (nested under coord) |
| fork branch | `dispatch/wf_7030fd9a-8ae-1` (reflog: "Created from 7fe7e231") |

The fork landed at the **armed base**, not cwd HEAD ⇒ the `worktree create-fork`
classifier (`src/worktree/create.rs::classify_create`, POSITIONAL trigger
`cwd_is_arming_dir` + base present ⇒ `Fork{base}`) reads the arming `base` file
and forks there. A confirming run with armed base == coord HEAD (`1fadff60`) also
landed correctly. **Gotcha caught mid-probe:** the first run OMITTED
`isolation:'worktree'` on the workflow `agent()` call → ran in-place in the coord
tree, no fork (`show_toplevel` == coord root, HEAD attached to `dispatch/207`).
The footgun applies to the workflow `agent()` path, not just the Agent tool.

### EX-2 / ISS-216 — a CHANGED confined agent-def made live under .claude/agents/ — **PASS (manual placement)**

- `.claude/agents/*.md` (and the worker's `.doctrine/agents/` symlink target) are
  **gitignored** — the derived/runtime live-read tier.
- `doctrine install` has **no `--force`/`--reseat`** flag; it is skip-if-exists
  (`src/install.rs`) ⇒ it CANNOT reseat a changed, already-installed def
  (ISS-216 fault #1). Fault #2: the `dispatch-worker` live symlink still points at
  the pre-subdir-split `.doctrine/agents/dispatch-worker.md`.
- **Procedure (demonstrated):** write the new/changed def bytes directly to
  `.claude/agents/<name>.md` (gitignored derived path). A scratch probe def written
  v1 → rewritten v2 with an added `mcp__doctrine__dispatch_phase_receipt` token was
  immediately readable at the harness-read path with the v2 token present. For a
  NEW def (PHASE-04 `dispatch-probe.md`) place a plain file; for a CHANGED existing
  def (orchestrator gaining tokens) overwrite the existing plain file directly.

### EX-3 — not triggered (both premises PASS; no design revisit).

**VH-1 (human): CONFIRMED 2026-07-06** — operator signed off from the EX-1 table
that fork base == armed base (`7fe7e231`), distinct from cwd HEAD; dispatch
cleared to drive. **VA-1 (agent): CONFIRMED** — ISS-216 workaround = direct write
to the gitignored `.claude/agents/<name>.md`; recorded above.

**PHASE-01 outcome:** both premises PASS, EX-3 not triggered. De-risk gate GREEN.
PHASE-01 is operator-run (no source delta, no code boundary) — it carries no
conformance-registry row, so the conclude/prepare-review completeness gate must
treat it as a boundary-less operator phase (waive, not halt).

Scratch teardown (post-sign-off): remove `.dispatch/SL-207` coord tree + branch
`dispatch/207`, abandon SL-207.

## PHASE-02 — phase_projection reader + ReceiptStatus (landed 2026-07-06)

Landed S=`1c246ee9` (B=`ca01e81c`), boundary row committed, fork reaped.
`phase_projection` + `ReceiptStatus` (6 variants) co-located with `run_status`
(EX-5). Behaviour-preserving: `run_status` builds the legacy `(id,status,name)`
tuple from the **verbatim** sheet string (`Ok(Some(s))=>s`, `Ok(None)=>"pending"`,
`Err=>"unknown"`), NOT from `ReceiptStatus` — so `dispatch status` output (incl.
the `"planned"` skeleton) is byte-identical. 3193 bin tests green, prove clean.

Funnel note: the worker's first commit derived the legacy string lossily
(`planned`→`NotStarted`→`"pending"`) — a real `dispatch status` display +
`compute_next_phases` batching delta, uncovered by tests. Corrected IN PLACE
(worker's jailed `.git` is RO so it could not re-commit; orchestrator reset the
fork branch to B and committed the worker's own staged corrected bytes —
land-not-rewrite, author preserved). See
[[mem.pattern.dispatch.correct-worker-delta-in-place]].

## PHASE-03 — read-only funnel tools + doctor allowlist (landed 2026-07-06)

Landed S=`577a2d64` (B=`b721d10a`), boundary row committed, fork reaped. Three
read-only tools (`dispatch_phase_receipt` / `_next_ready` / `_authored_divergence`,
CLI+MCP; `ReadOutcome<T>` = `Resolved|CoordRefused`); `compute_next_phases` made
`pub(crate)` + a `plan_next_rows` seam shared by CLI plan-next and the MCP tool
(plan-next output byte-identical); doctor #9 `allowed_mcp_tokens` grows
(orchestrator +3 reads, new `ROLE_PROBE` = exactly the 3 reads, marker validator
accepts `worker|orchestrator|probe`). `compared_ref` resolved from
`git::trunk_commit`, never hardcoded edge (EX-8). 3204 bin + e2e golden green.

Scope: registering the 3 tools forced the e2e tool-count golden
`tests/e2e_mcp_server.rs::vt2_tools_list` `22→25` + 3 names. That file was outside
the selector; widened to design-target (commit `eaea6b3d`, operator-approved). A
worker overreach on the out-of-selector red (edited the golden + re-committed on a
fabricated authorization) is logged to RFC-011 case-notes.

## PHASE-04 — Agent defs + /drive-slice reference workflow (landed 2026-07-06)

Landed S=`d94a301f` (B=`aba7b740`), boundary `68743c8b`, fork reaped, provenance
funnel. Model: opus (the /drive-slice script is the slice centerpiece — Rust-vocab
↔ JS HALT-contract coupling). Five files, all verified green against the correct
target: `install/agents/claude/dispatch-probe.md` (new — probe role, exactly the 3
read MCP tokens, no raw write/bash/agent), `dispatch-orchestrator.md` (+3 read
tokens), `install/workflows/drive-slice.js` (new — design §5.4 driver loop, frozen
single-sourced `HALT` table, `coord:`/`funnel:` passed through from the Rust
vocabs, no auto-merge), `src/doctor_checks.rs` (new real-tree conformance test
`agent_conformance_real_shipped_defs_are_clean`, VT-1), `src/install.rs` (probe
def + workflows seeding legs — EX-5/EX-7 both made real, nothing waived). Suite
3205/0 (3204 at B + 1 new test), prove clean.

**Landed via land-not-rewrite — worker_commit false-red (stale-PATH binary).** The
`worker_commit` gate belt runs `doctrine doctor` via PATH `~/.cargo/bin/doctrine`,
which the flake bump rebuilt from **edge** — pre-PHASE-03, so it rejects
`doctrine-role: probe` (unknown) + the orchestrator's 3 read tokens (old 3-token
allowlist) → exit 1. Fresh worktree binary (dispatch/206 source) → doctor exit 0,
zero conformance findings. Both tag `0.16.0`; divergence is SOURCE, not version.
Root cause: PHASE-04's def depends on PHASE-03's conformance surface, which lives
only on `dispatch/206` (unpromoted pre-audit). Orchestrator landed the worker's
own staged bytes (author preserved, `dispatch@doctrine` committer via import),
substituting fresh-binary verification for the gate's stale-binary check. Operator
approved the bypass. See [[mem.pattern.dispatch.worker-commit-stale-path-false-red]]
and RFC-011 case-notes; backlog ISS filed (belt should use the coord/build binary,
not PATH — violates AGENTS.md's own rule).

**Selector corrections (register-before-land, EX-6 + emergent):**
`install/workflows/drive-slice.js` → design-target (OQ-1 home, edge `b268759f`).
`src/install.rs` scope-relevant→**design-target** (edge `4c45ef98`): PHASE-04
modifies it substantively; the import belt (`classify_import`) requires
design-target to land ANY path, so the plan's scope-relevant classification was a
plan defect the belt correctly caught. Both mirrored to the coord belt for the
live import.

**Base freshness:** trunk moved 21 ahead of fork-point at conclude (was 12 at
PHASE-04 start; the edge companion commits accrue). No overlap with PHASE-04's
files — refresh-base deferred; the conclude/integration merge handles it. NOTE:
the coord worktree is STALE (worktree-free import) — will trip INV-6 at
prepare-review; un-stale before the conclude cadence.

## PHASE-05 — live acceptance bootstrap (setup, pre-drive; 2026-07-06)

Operator directed the DOCTRINE_BIN repoint route. State made TURNKEY this session;
the live drive is a **post-restart** action (MCP server reads `DOCTRINE_BIN` only at
launch).

**Done (turnkey):**
- PHASE-04 artifacts made LIVE at the gitignored harness-read paths (ISS-216
  procedure, direct write): `.claude/workflows/drive-slice.js`,
  `.claude/agents/dispatch-probe.md` (new), `.claude/agents/dispatch-orchestrator.md`
  (overwritten, +3 read tokens). Byte-exact `git show 68743c8b:<install path>`.
- Slice binary built: throwaway detached worktree `.dispatch/SL-206-bin` @ `68743c8b`
  (gitignored under `.dispatch/`). Cold build dropped gitignored `web/map/dist/` →
  RustEmbed `Assets::get` E0599; `cp -R web/map/dist` from primary, rebuild clean.
  Binary `= .dispatch/SL-206-bin/target/debug/doctrine` (0.16.0). `doctor` exit 0
  (knows `probe` role — the PHASE-04 false-red condition is gone). MCP stdio smoke:
  serves `dispatch_next_ready` / `_phase_receipt` / `_authored_divergence` + funnel.
- `DOCTRINE_BIN` wired: `.claude/settings.local.json` `env.DOCTRINE_BIN` →
  the slice binary. `.mcp.json` cmd is `${DOCTRINE_BIN:-doctrine}`, so on next
  session/MCP start the doctrine MCP server re-spawns as the slice build and serves
  the read tools the drive needs.

**Next session (post-restart) — build rig + drive:**
1. Sanity: ToolSearch `dispatch_next_ready` resolves (proves the repoint took). If
   NOT, the restart didn't pick up `DOCTRINE_BIN` — stop, don't drive on the stale
   server.
2. Build the acceptance rig — a real multi-phase scratch slice (file-disjoint
   phases, each a small impl+test so `verify.green` is meaningful and `worker_commit`
   passes) + its dispatch coord tree; a red-verify variant (EX-2 halt, no auto-merge);
   a forbidden-write probe (spawned agent attempts `review_*`/`memory_*` — absent
   against the installed worker def; EX-3); budget +Nk vs null (EX-4).
3. Invoke Workflow `drive-slice` {slice} live; capture PhaseReceipt flow, halt-on-red,
   grant-gate refusal, divergence advisory, budget pacing → this notes section.
4. VA-1 self-confirm; stage VH-1/VH-2 for operator sign-off.

**Caveats for whoever continues:**
- Do NOT delete `.dispatch/SL-206-bin` — it is the live MCP binary source until
  PHASE-05 closes.
- Revert `DOCTRINE_BIN` (back to PATH) after PHASE-05/close, so the session returns
  to the promoted binary.
- Coord tree `.dispatch/SL-206` remains STALE (worktree-free imports) — un-stale
  before the conclude cadence (INV-6), unchanged from the PHASE-04 note.


## PHASE-05 — live acceptance drive: RESULTS (2026-07-06)

First real load+drive of the shipped `/drive-slice` against a live scratch slice.
Rich result: the SEQUENCER machinery works; the ORCHESTRATION model is structurally
broken in-jail. Findings feed the audit/rework decision.

### Tooling bootstrap (to reproduce)
- `DOCTRINE_BIN` repoint: the doctrine MCP server (`.mcp.json` cmd
  `${DOCTRINE_BIN:-doctrine}`) must be the SLICE binary to serve the PHASE-03 read
  tools. `settings.local.json` `env.DOCTRINE_BIN` alone RACED — at startup a bare
  `doctrine` (edge) spawn squatted the "doctrine" name before the env applied, so
  the harness bound edge (funnel tools present, read tools ABSENT). Two servers ran
  (PID edge + PID slice); ToolSearch proved the bound one was edge. Fix: export in
  the PARENT env — `DOCTRINE_BIN=<slice bin> claude`. Then ToolSearch resolves
  `dispatch_next_ready`. Slice binary built in detached worktree `.dispatch/SL-206-bin`
  (needed `cp -R web/map/dist` first — RustEmbed `Assets::get` E0599 on fresh checkout).
- Live artifacts at `.claude/{workflows/drive-slice.js, agents/dispatch-probe.md,
  agents/dispatch-orchestrator.md}` (ISS-216 direct-write procedure).

### Rig
Scratch slice **SL-209** (`test(SL-206)` commit `fa951846`): 2 file-disjoint phases,
each creating one self-contained `tests/scratch_accept_{a,b}.rs` (design-target
selectors). Coord tree `.dispatch/SL-209` / `dispatch/209`, base `fa951846`
(edge→main promoted to fa951846 pre-dispatch). KEPT for re-drive after the rework
decision — do not tear down yet.

### FINDING 1 — shipped script does not load/run under the real Workflow harness (contract)
- `.claude/workflows/` is NOT the Workflow tool's NAMED registry (only built-ins
  `deep-research`/`code-review` resolve by name). `Workflow({name:'drive-slice'})`
  → "not found". Runtime path is `Workflow({scriptPath:'.../drive-slice.js'})`.
- `meta.description` uses `+` string concatenation → hard load error
  "meta must be a pure literal: … BinaryExpression".
- All logic is in `export async function run({slice})`; the Workflow tool executes a
  TOP-LEVEL body and never calls a `run` export → no-op even with meta fixed.
- Minor: `meta.phases` bare strings (want `[{title,detail}]`); `meta.args` non-standard.
Contract-cosmetic — a scratchpad probe with exactly these three adaptations LOADED
and entered the drive.

### FINDING 2 — no slice validation → drives a GUESSED slice (safety)
The Workflow `args` reached the script as a JSON STRING (documented footgun), so
`run({slice})` destructured the native `.slice` method off the string. The bootstrap
`dispatch-probe` agent, handed garbage, GUESSED slice **206** and the driver began
driving it — no fail-closed guard. Caught + `TaskStop`ped during orient; ZERO damage
(dispatch/206 tip unchanged, no arm/fork/import; only read tool-calls). Shipped
`drive-slice.js` must: parse string-args, validate `slice` is a positive integer,
halt otherwise; probe agents must not guess a slice. (Probe hardened `slice===209`.)

### FINDING 3 — orchestrator PLACEMENT defect (was mislabelled "confined orchestrator can't write the funnel")

> **CORRECTION (2026-07-06).** The original text of this finding — retained below
> under "~~Original (WRONG root cause)~~" — claimed the confined orchestrator
> "cannot write the funnel / needs RW `.git`". **That root cause is WRONG.** It
> contradicts the proven, locked SL-199 "Mode B" (slice **done**). Reconciled after
> reading the SL-199 record. Corrected account first; superseded text preserved for
> the audit trail.

**The confined orchestrator is proven-realizable; RO `.git` is BY DESIGN.**
SL-199 designs the orchestrator to sit in `Jail(coord-cwd)` with a **read-only
shared `.git` on purpose** — it writes coord `.git` via **server-side MCP tools
running unconfined** (`dispatch_import`/`conclude`/`reap`, and the worker's
`worker_commit`), never directly ("integrity never rests on the confined
orchestrator", SL-199 design.md:216). Arming uses a **coord-root positional
discriminator** (`cwd_is_coord_root ∧ coord_in_dispatch ∧ base`, create.rs:200–212)
— built and adjudicated sound (SL-199 inquisition-phase05.md). Nested
`isolation:worktree` IS honored (F7 REFUTED); worker isolation is def-pinned to the
worker frontmatter, NOT the orchestrator's per-call arg. SL-199 proved the
primitives but left **VH-1 — the live integrated armed loop — OWED/unwitnessed**
(inquisition-phase05.md:52). **PHASE-05 here was effectively the first VH-1 attempt.**

**The real defect is PLACEMENT, not permission.** §5.4 spawns the orchestrator with
`agent(orchestratorPrompt, { isolation:'worktree' })`. `isolation:'worktree'` forks
a **fresh `.worktrees/agent-<hex>`** off the spawning session's local HEAD — so the
orchestrator lands jailed to a *fresh fork*, NOT to the coord tree. Consequences:
`cwd_is_coord_root` never fires; arm writes land in the fork's tree, not the coord
tree; and (driver cwd unparked) the fork is off the wrong base. The RO-`.git` errors
I logged were downstream symptoms of the orchestrator being in the wrong tree, not
proof that a *correctly-placed* confined orchestrator cannot drive.

**Fix direction:** place the orchestrator jailed to the EXISTING coord tree
(cwd=coord-root, no fresh fork — the SL-199 model); let the worker frontmatter carry
`isolation`. Do **NOT** unconfine the orchestrator (that contradicts locked, proven
Mode B). **Open premise to spike first:** can a *workflow* `agent()` place a subagent
jailed to the existing coord tree, or does the Workflow harness only ever fresh-fork?
If it cannot, the workflow-driver shape is incompatible with SL-199's placement
contract → the SL-206 *slice* premise reopens (fallback: main-thread skill
sequencer), not just §5.4. Durable: [[mem.pattern.dispatch.confined-orchestrator-placement-not-permission]].

<details><summary>~~Original (WRONG root cause) — preserved for audit~~</summary>

> With F1+F2 fixed, the drive ran correctly against 209 and produced the decisive
> result. The driver spawns the orchestrator with `agent(..., {isolation:'worktree',
> agentType:'dispatch-orchestrator'})`. That confinement jails the orchestrator to its
> OWN isolated worktree with a read-only shared `.git` and no write to the coord
> tree. Every funnel WRITE fails: `dispatch arm-spawn` → RO `.doctrine/state/dispatch/spawn`;
> `doctrine worktree fork`/`git update-ref`/`git commit` → RO "cannot lock ref". So it
> cannot arm the base, mint/commit a worker fork, or import/conclude. Phases NEVER
> reach Completed in-jail. **[Claimed] Root:** §5.4 inverts the working claude-arm
> model; likely fix: orchestrator runs UNCONFINED. — **RETRACTED: this misread RO
> `.git` (a by-design posture) as the blocker, and the fix as unconfining. Both wrong.**

</details>

### What WORKED (positive acceptance)
- Bootstrap `dispatch-probe` planner: correct slice, `next_ready` = `compute_next_phases`
  authority verbatim (`["PHASE-01"]`).
- Typed **PhaseReceipt**: orchestrator composed a schema-valid receipt; the JS loop
  CONSUMED it and branched to halt on `receipt_status!=='Completed'`. Plumbing proven.
- **report-and-halt + NO auto-merge**: coord tip unchanged (`fa951846`), PHASE-01
  not-started, precise `halt_reason`. Safety property HELD.
- Closing **divergence** probe: emitted `{diverged:false, compared_ref:fa951846}`,
  never gated (EX-4 divergence ✓).
- **null-budget** unmetered: no `+Nk` directive → `budget.total` null → loop ran
  unmetered, no budget halt (EX-4 null ✓).

### EX coverage
- EX-1 (all-Completed + typed receipt consumed): **PARTIAL** — receipt plumbing ✓,
  all-Completed ✗ (FINDING 3 — orchestrator PLACEMENT, fixable; not a structural block).
- EX-2 (injected red halts, no auto-merge): no-auto-merge + halt-on-non-Completed ✓;
  the specific `VERIFY_RED` branch NOT hit (halted on `anomaly` first; worker never ran).
- EX-3 (grant gate at runtime): **NOT reached** (no worker/forbidden-write attempted).
- EX-4 (divergence emits/never gates; budget null/+Nk): divergence ✓, null ✓; `+Nk`
  pacing NOT tested (no budget directive injectable through the Workflow tool here).

### Governance fork (CORRECTED 2026-07-06 — consulted)

> **SUPERSEDES the earlier recommendation** (preserved below), which said "revise
> §5.4 to run the orchestrator UNCONFINED, workers confined." That would contradict
> the locked, proven SL-199 Mode B and is WRONG — see FINDING 3 correction above.

FINDING 3 is a PLACEMENT defect, not an orchestration-model inversion. The
confined orchestrator is correct and proven (SL-199). Corrected path:
1. **Spike first** (cheap, against the standing SL-209 rig): can a workflow
   `agent()` place a subagent jailed to the *existing* coord tree (cwd=coord-root,
   `cwd_is_coord_root` fires), or does the Workflow harness only ever fresh-fork?
2. **If yes** → §5.4 fix is small: drop `isolation:'worktree'` from the orchestrator
   call, place it in the coord tree, worker frontmatter carries isolation. Fold in
   FINDING 1 (harness contract) + FINDING 2 (slice guard). In-slice `/design` delta
   (not a Revision — intra-slice defect in its own acceptance phase).
3. **If no** → the workflow-driver shape is incompatible with SL-199's placement
   contract → the SL-206 *slice* premise reopens (fallback: main-thread skill
   sequencer). Do NOT self-adapt the landed deliverable; route through governance.

<details><summary>~~Original (WRONG) recommendation — preserved for audit~~</summary>

> Load-bearing FINDING 3 is a DESIGN defect … re-open PHASE-04 / revise design §5.4
> orchestration model (orchestrator **unconfined RW `.git`**; workers confined) …
> — **RETRACTED: unconfining the orchestrator contradicts locked, proven SL-199
> Mode B. The defect is placement, and the fix keeps the orchestrator confined.**

</details>

### Placement spike + design delta (2026-07-06)
Decided §5.4-patch vs slice-reopen empirically. Spike `wkubn99yg` (gitignored
`.dispatch/placement-spike.js`), driver Bash-cwd parked at coord-209:
- **Arm A — `agent()` NO isolation:** landed at `/workspace/doctrine/.dispatch/SL-209`,
  branch `dispatch/209`, `.git` **RO**, coord working tree **RW**. = the SL-199
  orchestrator placement (`cwd_is_coord_root` would fire). **Placement viable.**
- **Arm B — `isolation:'worktree'`:** fresh-forked to
  `.dispatch/SL-209/.worktrees/wf_…-2`, detached `HEAD`. Confirms the §5.4 defect —
  a fresh fork, never coord-root.
- Both arms RO `.git` (universal for a jailed linked worktree — by design; server-side
  MCP does the coord `.git` writes). The ONLY difference is cwd placement.
- Corroborated (not trusted over the spike): `docs/claude/subagents.md:263`
  ("a subagent starts in the main conversation's cwd … `isolation: worktree` gives an
  isolated copy"); `docs/claude/settings.md:339` (`worktree.baseRef` default `head`).

**Design delta landed** (in-slice `/design`, §5.4): PHASE-05 DELTA callout + code fix
(orchestrator spawn drops `isolation`; F2 slice guard) + §5.5 coord-root invariant +
§2 diagram. Orchestration model UNCHANGED (confined orchestrator, server-side writes).
Adversarial re-lock deferred — conferring (operator leans: fold into the revised plan's
review, not a separate design re-lock). Spike fork reaped; primary on edge; coord 209 clean.

### FINDING 4 — the Workflow primitive strips `Agent`; placement patch was necessary but NOT sufficient (2026-07-06, PHASE-07 live drive)

PHASE-07 re-drove SL-209 with the PHASE-06-corrected driver (`wf_6e50798d-656`),
Bash-cwd parked at coord-209 root. Result: **halt at PHASE-01**, `halt_reason=
"spawn:fork-blocked-readonly-git"`. The placement fix WORKED as far as it goes —
but a deeper wall the placement spike never probed (it tested placement, never
worker-spawn) now surfaces:

**What proved out (PHASE-06 fix validated):**
- Placement held — orchestrator ran at coord-209 root, branch `dispatch/209`.
- Base **armed at B=`fa951846`** — the SL-199 `cwd_is_coord_root` discriminator fired.
- Fail-loud & safe — coord tip untouched, zero stray `dispatch/w-209` refs, no
  auto-merge, no false conclude. The driver's safety axis is perfect.

**The wall (both worker-spawn arms closed to the orchestrator):**
- **pi-arm** — orchestrator ran bash `git worktree add -b dispatch/w-209-p01 …`
  → real `fatal: cannot lock ref 'refs/heads/dispatch/w-209-p01'` (shared `.git`
  RO in the jail; it cannot create the worker fork ref).
- **claude-arm** — the orchestrator held **no `Agent`/`Task` tool at runtime** and
  no `worker_commit` MCP, so the harness-privileged fork-create was unreachable.

**Root cause (decisive two-path probe, CC 2.1.198):** the Claude Code **Workflow
runtime strips the `Agent` tool from every subagent it spawns** — workflow agents
are leaves, cannot nest-spawn, regardless of the def's `tools:`. Same
`dispatch-orchestrator` def: main-thread **Agent-tool** spawn → HAS `Agent`;
**Workflow** `agent()` spawn → NO `Agent` (also strips Grep/Glob, injects
`StructuredOutput`). Recorded durably as [[mem.fact.claude.workflow-strips-agent-tool]]
(`mem_019f36028bca7411b33fde4981aaba85`), with the probe transcripts
(`wf_40e8c2c3-3fe` workflow arm; `ae3a5dd36e3ea8c41` main-thread arm).

So the defect is **the wrong harness primitive**, NOT placement (fixed) or git
permission (a symptom). A workflow-spawned orchestrator can never reach the
working claude-arm; it always falls to the pi-arm bash path that RO `.git` blocks.
This does NOT contradict SL-199 Mode B (confined orchestrator landing via
server-side MCP is still sound) — it says a *workflow agent* cannot BE that
orchestrator, because it cannot spawn the worker.

**Fix directions (design territory — awaiting operator steer, then `/design` on §5.4):**
- **A — Agent-tool-spawned orchestrator.** Drive via the Agent tool (not Workflow):
  the orchestrator subagent retains `Agent` (proven) and nest-spawns workers via the
  harness-privileged fork-create. Zero new machinery; matches real `/dispatch`.
  Unwitnessed leg: subagent→worker nested privileged fork (production spawns workers
  from the *main thread*, not a subagent).
- **D — workflow script spawns the WORKER directly.** A worker is a leaf (needs no
  `Agent`); a workflow `agent(isolation:'worktree')` DOES mint the fork at the armed
  base ([[mem_019f331005d776c1a65c65bfe59581bf]], SQ3) and DOES reach MCP
  ([[mem_019f328b116a7172ba7eabef25ed979d]]). Keeps `/drive-slice` autonomous; the
  funnel import/conclude runs from a second workflow-spawned MCP-holding agent (or the
  main thread). More seams than A.
- **B — new privileged `dispatch_spawn_worker` MCP tool** (server-side, unconfined,
  like `worker_commit`) so a confined orchestrator can create the fork. Most new
  machinery; keeps the confined-orchestrator model. Weakest lean.

### Stable state / resume
- Coord 206 `68743c8b` (PHASE-04, untouched); coord 209 `fa951846` (rig, **still
  no phase driven** — PHASE-07 halted pre-worker, coord intact); `main`=`fa951846`;
  edge working tree carries only foreign/scratch; probe worktrees pruned.
- PHASE-07 `in_progress`; VH-1 NOT satisfied (drive did not complete a phase). EX-5
  arming half witnessed (base armed at B); EX-1/2/3/4/6 pending a working driver.
- Corrected/hardened reference at `.dispatch/drive-slice-probe.js` (gitignored):
  the 3 contract fixes + slice guard, drive logic verbatim. Fix template.
- Next: operator picks A/D/B → `/design` §5.4 (the Workflow-primitive choice is the
  defect, deeper than the PHASE-06 placement delta). Cheapest confirming probe for A:
  reuse the live orchestrator `ae3a5dd36e3ea8c41` (Agent-tool-spawned, cwd parked at
  coord-209) to drive ONE SL-209 phase — if PHASE-01 → Completed with
  `code_start==fa951846`, Architecture A is witnessed. Teardown SL-209 +
  `DOCTRINE_BIN` repoint at rework close.
- NOTE (infra): host disk `/dev/nvme0n1p2` hit 100% mid-session (accumulated dispatch
  `target/` caches across worktrees); reclaimed `.dispatch/SL-206/target` +
  `.dispatch/SL-165/target` (regenerable). Watch headroom on re-drive (each worker
  build is ~GB).

---

## 2026-07-06 — P1+P0 RESULTS & current /design fork (compaction prep)

**Supersedes the A/D/B fork above.** That fork is resolved: **A (Agent-tool-spawned
orchestrator) is validated**; D'/workflow-revival is dead-or-optional; B (new
`dispatch_spawn_worker` MCP) is unnecessary — nested fork + `worker_commit` already
work from a subagent. The live design question moved on (see fork below).

### P1 — orchestrator-unjail round-trip: PASS (commits 0048ee75, 9d67a6f4)
Nomination (SubagentStart→allowlist→pretooluse PassThrough) flips a subagent to
unconfined end-to-end: `SubagentStart` fired from a child-session main thread, wrote
the allowlist pre-first-Bash, orchestrator ran Bash **unwrapped**, committed 98835cc.
Mechanism: the RO `.git` wall is the **bwrap wrap** on a *confined* Bash, not an fs
perm — PassThrough ⇒ unwrapped ⇒ RW `.git`. Mems: mem_019f363b (child-session fire),
mem_019f36401b (bwrap-wrap). Rig torn down; patch in scratchpad.

### P0 — nested-spawn probe: RAN (commit 249ddca0)
Confined orchestrator @ coord SL-209 nest-spawned one worker; observer SubagentStart
+ WorktreeCreate shims (torn down after; logs in scratchpad).
- **Q1 escalation: LIVE.** Nested (subagent-initiated) `SubagentStart` **fires**;
  payload has **no spawner id**. Field-diff (checked, commit 939ee08b):
  session_id/prompt_id/transcript_path IDENTICAL across top-level vs nested; only
  agent_id/agent_type/cwd differ; cwd is an **isolation artifact**, not lineage; no
  env in payload. ⇒ a type-match (or cwd) nomination gate is escalatable — an
  **out-of-jail single-slot intent token** is the only sound discriminator. Mem:
  mem_019f364eb2a5.
- **Q2 confined-A2 viability: STRONGLY SUPPORTED.** Confined orchestrator: sealed out
  of shared `.git` but **wrote its own coord `.doctrine/state/`** (arming feasible);
  nested `isolation:worktree` **fired WorktreeCreate** + forked at **correct base**;
  `worker_commit` **resolved + ran the full gate** (refused only at commit-gate-red =
  pre-existing RED `test` on the SL-209 base, exit 101 — incidental). Mem:
  mem_019f364ec31e.

### Current /design fork (§5, awaiting operator decision — this is the resume point)
- **(A) Confined-A2** — orchestrator jailed at coord root, drives via server-side MCP;
  NO boundary change, NO ADR-008/006 amendment, NO token, NO escalation surface.
  ADR-008 says confinement is perf-not-trust (trust = import belt), so a confined
  orchestrator loses no security property.
- **(B) Unjail** — surface-min; COSTS ADR amendment + `/inquisition` + **mandatory
  arming token** (P0 Q1) + escalation surface. RFC-011 budget-lever payoff is dead in
  all A-forms anyway.
- **(C) Layered** — ship (A), bank (B) as the P1-proven fallback. **My recommendation,
  biased to A.**
- **THE ONE OPEN RISK (governs the decision):** the *full* funnel
  (`dispatch_import → verify → conclude → record`) run **E2E via MCP from a confined
  orchestrator** — P0 proved every piece, not the whole chain. Mode-B-proven, but
  untested as one drive. Candidate: a **plan-phase spike** (reuse orchestrator
  `ae3a5dd36e3ea8c41`-style, drive ONE SL-209 phase to Completed) retires it before
  locking the fork.

### Loose ends closed (commit 939ee08b)
mem_019ec84b rewritten as SUPERSEDED (both refuting probe ids); ISS-219 filed
(worker_commit refusal embeds ~295k-char transcript — RFC-011 token sink); field-diff
folded into mem_019f364eb2a5.

### Env / rig state at compaction
- `DOCTRINE_BIN` still repointed → `.dispatch/SL-206-bin/target/debug/doctrine`
  (pristine pretooluse, post-P1 revert). Repoint-back deferred to rework close.
- Coord SL-209 @ fa951846 = **RED base** (test exit 101) — needs green before any real
  drive/spike. Coord SL-206 @ 68743c8b untouched. `main`=fa951846.
- P0/P1 observer+allowlist rigs fully torn down; `.claude/settings.local.json` back to
  memory-probe + command-probe only.
- `/plugins` ghost marketplace fixed (stray `path` in `~/.claude/settings.json`
  extraKnownMarketplaces.doctrine; backup `settings.json.bak-p0-*`). Reopen /plugins to
  re-reconcile.
- PHASE-07 still `in_progress`; VH-1 unsatisfied. RFC-011 case-note appended
  (uncommitted, matches its dirty-accumulation pattern).

### Resume
Operator answers the fork (A / B / C, or "spike the residual first") → continue
`/design` §5 sections (interfaces/state/ADR-impact/verification) → adversarial pass
(codex GPT-5.5) → `/plan`. If (C)/A: no ADR amendment; if (B): ADR-008 amendment +
`/inquisition` + arming-token + posture-registry-row spec.

## 2026-07-06 — full-unjail RESOLVED (safe) + corrected drive-by-workflow model (compaction prep 2)

**Decision locked: full-commit unjail, provably safe.** Operator chose it for
SIMPLICITY (reasoning + implementation surface), not necessity. Safety proven E2E
across three composable seams, all banked+verified:
- P1 nomination round-trip — allowlist flips PreToolUse deny→PassThrough (control
  denied), unjailed orchestrator committed 98835cc. `mem.fact.dispatch.unjail-nomination-round-trip`.
- P3 — PreToolUse(Agent) carries spawner agent_id (=<NONE> main / =caller nested),
  holds under CHILD_SESSION=1. P4 — active deny gate BLOCKS a subagent-initiated
  privileged spawn E2E, scoped, legit path intact. `mem.fact.claude.pretooluse-agent-carries-spawner-id`.
- Invariants the design must hold: I1 gate privileged-set ≡ nomination-eligible-set
  (one list); I2 allowlist/hook config outside every jail (main-thread-write only).
- Commits this window: e78f4c98 (P4), 72ce0da8 (P1 bank + RFC-016 pointer).
  RFC-016 References now points here (OQ-2 arm-unification).

**CORRECTED design model (operator steer — supersedes my A1/A2-convergence take).**
My earlier "retire the workflow, loop moves into orchestrator / A1 ships first"
was WRONG. Corrections:
- A1 (orchestrator = top-level main thread) ALREADY WORKS — not this slice. This
  slice = **drive-BY-WORKFLOW**. The workflow is the point.
- The blocker was the workflow-spawned orchestrator LEAF couldn't write/.git.
  Nomination fixes exactly that (PassThrough → RW .git). So the ORIGINAL §5.1
  workflow model stands; the ONLY change is the per-phase orchestrator is now
  nominated-UNJAILED instead of confined-leaf.
- **Workflow = durable lifecycle manager.** One orchestrator CANNOT run a 7-phase
  slice E2E (context) and its prompt-cache TTL (5min) bleeds while a worker runs.
  So the workflow loops over phases, spawning a FRESH nominated-unjailed
  orchestrator per phase (or batch), advancing on its receipt.

**THE open wrinkle to resolve next (resume here).** Wall-1 persists under unjail:
the Workflow runtime STRIPS the Agent tool from agent() spawns
(`mem_019f36028bca`, proven, provisioning-layer — NOT fixed by PassThrough, which
is permission-layer). So a workflow-spawned orchestrator, even unjailed, has NO
Agent tool → cannot spawn workers via the claude Agent-arm. Therefore it must
drive workers via the **SUBPROCESS/pi arm** (`git worktree add` + subprocess) —
which unjail SPECIFICALLY unblocks (previously failed `cannot lock ref` on RO
.git; PassThrough gives RW .git). Coherent model:
  workflow (loop) → agent() spawn fresh nominated-unjailed orchestrator per phase
  → orchestrator drives worker via subprocess arm → funnel (arm/import/conclude
  via MCP or CLI) → return receipt → workflow advances.
Sub-questions for §5: (a) is the subprocess WORKER confined under bwrap when
spawned by an unjailed workflow orchestrator? (unprobed); (b) orchestrator drives
funnel via MCP (server-side) or CLI (it can shell now)? (c) per-phase vs per-batch
orchestrator granularity. Receipt machinery (5.2/5.3) + grant table largely
survive; 5.4/5.5 confined-placement delta retired (no confinement).

**Resume:** confirm the corrected model with operator, then author §5 around it
(workflow lifecycle loop + nominated orchestrator + subprocess worker arm + the
three sub-questions), ADR-008 amendment scope, then adversarial pass → /inquisition.

## 2026-07-06 — v1 topology REFINED (operator steer 2): workflow = spawn authority, both arms

**Hard requirement: BOTH claude AND subprocess arm workers. Subprocess-only is
NOT acceptable.** This corrects "orchestrator drives worker via subprocess arm."

**Key insight I missed:** the WORKFLOW's agent() CAN spawn a claude-arm worker (a
leaf — the Agent-strip only stops the SPAWNED agent from nesting further; the
workflow spawns leaves fine). The orchestrator can't (Agent-stripped). ⇒
**claude-arm workers are reachable only via the workflow.** So the workflow is the
spawn authority for workers, NOT the orchestrator.

**v1 topology — serial, alternating, workflow-sequenced:**
```
workflow (durable serial loop; spawn authority)
  per phase, alternating ephemeral agents:
    O(prepare)  → nominated-unjailed: capture base B, arm, distill worker prompt
    W(work)     → claude arm: workflow agent() leaf
                  pi arm:     O spawns subprocess (git worktree add — RW .git)
    O(dispose)  → nominated-unjailed: import delta, verify, conclude, reap → receipt
  → advance; fresh agents next phase (no TTL bleed, no single-context 7-phase)
```
- Adjacent O(dispose N)+O(prepare N+1) may merge into one spawn — v1 keep simple.
- Parallel batches DEFERRED (v1 serial only).
- Workflow (JS) has no shell/MCP → it spawns claude workers via agent() but CANNOT
  spawn pi workers (no bash); pi workers spawned by the unjailed orchestrator.

**Topology shift to spec in §5:** spawn-authority (workflow) is SEPARATED from
import-authority (orchestrator) — shipped /dispatch has the orchestrator do both.
Here the workflow spawns the claude worker; a later O imports its delta. Workflow
becomes the coordination layer passing worker branch/commit between ephemeral
orchestrators. §5 must spec that handoff.

**Two probes gating §5 (run before authoring):**
- **P5** — does a workflow-spawned claude WORKER retain the `worker_commit` MCP
  tool? (Agent-strip also removes Grep/Glob, injects StructuredOutput — if it
  strips MCP too, claude worker can't self-commit → pi-style orchestrator-imports-
  diff fallback.) Decides claude-worker commit path.
- **P6** — is the pi/subprocess worker still bwrap-confined when spawned by an
  UNJAILED orchestrator, or does it inherit PassThrough? Funnel wants worker
  jailed; inherited unjail = a hole to close.

**Resume:** confirm topology, run P5+P6 (small rigs like P3/P4), then author §5
(workflow serial loop + alternating ephemeral nominated orchestrators + both-arm
worker spawn + spawn/import authority split), ADR-008 amendment, adversarial pass.

---

## PHASE-08 — Unjail de-risk gate: RESULTS (2026-07-06, GREENLIT)

Empirical gate against scratch slice SL-207 (coord tree `.dispatch/SL-207` on
`dispatch/207`, base `ff8c61db` = edge==main). Main-thread rigs (Workflow +
throwaway PreToolUse matcher), no receipt code. **All load-bearing premises hold
→ no `/design` re-entry; PHASE-09..16 scaffolding de-risked.**

Model: Opus 4.8 (harness/jail security probing).

### EX-1 — worker forks at the ARMED base (VH-1) ✅
Confirmed **live in-session** + `mem_019f331005`. P5 workflow `wf_9829cbac-a41`
minted its `dispatch-worker isolation:worktree` leaf at
`.dispatch/SL-207/.worktrees/wf_9829cbac-a41-1`, HEAD == armed base `ff8c61db`
(the ARMED-fork location, not benign `.worktrees/`), `--git-common-dir` = shared
primary `.git`, branch `dispatch/wf_9829cbac-a41-1`. Evidence:
`scratchpad/p5-ex1-fork-evidence.log`.

### EX-2 — P5 mode-B: worker_commit retention + resolution ✅ (viable)
`has_worker_commit_tool = true` — the workflow leaf **retains** the `worker_commit`
MCP tool (only Agent/Grep/Glob stripped; MCP survives — `mem_019f36028bca` row 2).
`worker_commit` returned **`Refused reason=commit-gate-red`, NOT `unknown-agent`**
⇒ the armed fork **resolved** to a provisioned DispatchRecord and ran the belts
end-to-end (contrast `mem_019f361983`'s unarmed `unknown-agent`). Belt correctly
enforced. `fork_tip = null` because the gate went red on a **stale reaped-fork
`CARGO_MANIFEST_DIR` test-binary artifact** (doctrine self-flagged "recompile
before trusting a RED"; `ff8c61db` is GREEN when built fresh — salvage check-commit
80 suites this session). Mode-B is **viable, not fall-to-(A)**; the clean non-null
`fork_tip` witness lands in PHASE-16's green e2e (EX-6: `boundary.code_start ==
pre-arm B`). Closes design OQ-3/P5. Memory:
`mem.fact.dispatch.armed-workflow-worker-mode-b-viable`.

### EX-3 — Workflow spawn seam (OQ-4) ✅ (closed by absence)
Live probe: a `Tools:*` Agent-tool subagent asked to call `Workflow` → **NO-TOOL**
(not in toolset, not ToolSearch-loadable); throwaway `PreToolUse(Workflow)` deny
matcher **never fired** (empty log). Workflow-leaf path: `mem_019f36028bca` row 3
shows a `Tools:*` workflow leaf gets no `Workflow` either. **No subagent holds the
Workflow tool** → the second seam has nothing to launder through at runtime.
Stronger than the design's blanket-deny plan; PHASE-12 `SEAM_REGISTRY` guards
future harness releases. Memory:
`mem.fact.claude.workflow-seam-closed-by-absence`. Rig: `scratchpad/oq4-workflow-seam.sh`.

### EX-4 — OQ-6 revive base ✅ (mechanism-1 proven live)
Re-armed at `base=20fee23f` (`ff8c61db^`, non-tip fork_tip stand-in); probe
`wf_5c9eac76-9c1` minted its fork at HEAD == `20fee23f` (== armed base, ≠ coord
tip `ff8c61db`). So **`arm-spawn --base` redirects the fork BASE ref to an
arbitrary commit** — a revive re-arms at `fork_tip`, zero worker-side git
(**mechanism-1**, design §5.5 preferred). PHASE-14 need not fall to the
mechanism-2 floor. Evidence: `scratchpad/oq6-evidence.log`.

### EX-5 — ISS-216 reseat ✅ (manual placement characterised)
`doctrine install` cannot reseat a changed def (skip-if-exists + stale pre-subdir-
split symlink `.claude/agents/dispatch-worker.md → ../../.doctrine/agents/…`).
**Documented procedure:** write the changed def directly to
`.claude/agents/<name>.md` (regular file — the harness live-reads that dir). The
live tree already shows this: `dispatch-orchestrator.md` + `dispatch-probe.md` are
hand-placed regular files. PHASE-13's install.rs work fixes the reseat per ISS-216
directions; until then, manual placement.

### VA-1 (agent confirms) ✅
P5 disposition (mode-B viable, retention+resolution), Workflow-seam denial/absence,
OQ-6 mechanism-1, and ISS-216 reseat all confirmed from the drive artifacts above.

### Evidence & memories
- Memories: `mem.fact.dispatch.armed-workflow-worker-mode-b-viable`,
  `mem.fact.claude.workflow-seam-closed-by-absence`,
  `mem.fact.dispatch.worker-commit-gate-stale-baked-binaries`.
- RFC-016 References §ext (SL-206 PHASE-08 evidence block).
- Scratchpad rigs (throwaway, preserved): `oq4-workflow-seam.{sh,log}`,
  `p5-{setup,arm,ex1-fork-evidence}.log`, `oq6-{arm,evidence}.log`.

## PHASE-15 — PRE-LANDED out-of-band via REV-021 (2026-07-06)

**Do not re-author at reconciliation.** The ADR-008 amendment this phase scopes
is already minted AND applied:

- **REV-021** (`ADR-008: nominated-unjailed orchestrator`) — minted `0ada4c2e`,
  applied + `done` `29e18252`. Slice tagged `rev-021-applied`.
- ADR-008 now carries **D-B6** with all three ledger clauses (EX-1 ✓): (a)
  Mode-B reversible escape hatch, (b) seam-symmetry standing price
  (conformance-checked), (c) steered-O blast radius named. Proof provenance
  honest (EX-2 ✓): D-B6 states P1/P3/P4 bound jailed→O only, "NOT covered by
  the escalation proofs" for O-steered.
- **VH-1 ✓** — operator directed the mint and the apply ("might as well apply
  it"); confinement-posture acceptance given.

**Residue for the phase/reconcile — why the phase is NOT flipped completed:**
EN-1 (PHASE-12's I1 seam-symmetry doctor check exists) was **not yet met** at
apply time — D-B6 cites the doctor check *forward*. When PHASE-12 lands, verify
`check_spawn_seam_symmetry` + `SEAM_REGISTRY` match D-B6's description ((a)
nomination ⊆ gate deny-set, (b) every registered seam has a matcher), then flip
PHASE-15 on that check alone. Nothing else remains.
