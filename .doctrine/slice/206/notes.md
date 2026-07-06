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

### Stable state / resume
- Coord 206 `68743c8b` (PHASE-04, untouched); coord 209 `fa951846` (rig, no phase
  driven); `main`=`fa951846`; edge working tree clean; probe isolation worktrees pruned.
- Corrected/hardened reference at `.dispatch/drive-slice-probe.js` (gitignored):
  the 3 contract fixes + slice guard, drive logic verbatim. Fix template.
- Next (corrected): (1) spike coord-tree placement of a workflow-spawned agent
  against SL-209; (2) if viable, patch §5.4 placement + FINDING 1/2, re-drive SL-209,
  then exercise EX-2 (inject a regression past the worker gate) + EX-3 (forbidden-write
  probe); (3) if not viable, reopen the slice premise. Teardown SL-209 + `DOCTRINE_BIN`
  repoint at PHASE-05 close.
- NOTE (infra): host disk `/dev/nvme0n1p2` hit 100% mid-session (accumulated dispatch
  `target/` caches across worktrees); reclaimed `.dispatch/SL-206/target` +
  `.dispatch/SL-165/target` (regenerable). Watch headroom on re-drive (each worker
  build is ~GB).
