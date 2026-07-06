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
