# Design SL-199: Confined subagent orchestrator drive-loop (Mode B capstone)

<!-- Reference forms: entity ids padded (SL-199, SL-198, ADR-012, ADR-011);
     doc-local refs bare — §A/§B/§C/§D, D-B1, OS1, R1. Status: REVISED post external
     review (codex GPT-5.5, 2026-07-04) — F1/F2/F3 verified against source, integrated.
     RE-LOCKED 2026-07-05 after a 2nd hostile inquisition (codex GPT-5.5) + prose↔source
     reconciliation (A6–A12); machinery source-verified sound, prose fixed, no redesign.
     Locked: §A (discriminator + one-shot hook-consumed arm), §B (funnel — every
     committed-output tool commits server-side; scope is a HARD pre-commit gate;
     conclude-phase = the BOUNDARY commit is atomic, the `completed` flip is disposable
     runtime that self-heals on retry — A6), §C (agent-def + lint), §D (drive-loop),
     §E (verification), §6 (probe).
     PHASE-05 DELTA (2026-07-05, findings F9–F13, reconciled + RE-LOCKED): F7 ("nested
     isolation:worktree not honored / §D infeasible") REFUTED — two recipe/def
     defects, not a harness ceiling. A1 arm base defaults to coord HEAD (§A/§D
     step1, IMP-268 defers the branch-guard); A2 worker isolation rides the
     dispatch-worker DEF FRONTMATTER, not a per-call arg (§C/§D step2); A4 base-
     control claim CORRECTED (§D step2, F9) — armed-path base is entirely the
     create-fork hook; parent-HEAD inheritance only shows on the useless unarmed
     Passthrough path; §6 re-validated primitive-correct.
     `/fork` write-policy out of scope → IMP-269.
     POST-INQUISITION RECONCILIATION (2026-07-05, codex GPT-5.5 hostile pass —
     prose↔source honesty, no redesign): A6 conclude-atomicity restated to the
     SHIPPED two-tier self-healing shape (the `completed` flip is a gitignored
     runtime sheet, NEVER committed; only the boundary commits, working-tree-free;
     completed-without-boundary is the EXPECTED retry-healed fault, not an
     "unreachable" state) — §B table / D-B1 / D-B3 / §D step5 / §E. A7 A1's "never a
     bad land" corrected (§A/§D step1): the gates check consistency-with-the-ARM,
     not with truth (`dispatch_import` recomputes merge-base, ignoring the armed
     base; `worker_commit` checks `C^==record.base`), so a wrong-but-consistent base
     CAN land silently on the misplaced/main-thread arm — the confined path stays
     safe only by cwd-construction; IMP-268 deferral re-weighed. A9 §6 tempered to
     primitive-confirmed / VH-1-owed. A10 jail-POLICY vs dispatch-RECORD naming
     disambiguated (§2). A8 built-vs-owed demarcated —
     BUILT (earlier SL-199 phases + SL-198, LIVE in the coord binary): §A confined
     Fork trigger + one-shot base consume (create.rs:200-230,321-325); §B funnel
     tools `dispatch_import`/`dispatch_conclude_phase`/`commit_on_behalf` +
     working-tree-free compose (mcp_server/dispatch.rs).
     OWED (PHASE-05 remainder): A1 `arm-spawn --base` default-to-HEAD (binary still
     REQUIRES it); A2 worker-def frontmatter `isolation` (def has no such field);
     VH-1 live integrated armed-loop witness.
     FORWARD-COMPAT GUARDS (cheap now, costly to retrofit): A11 frames §7 governance
     harness-NEUTRAL (the confined-orchestrator actor class ≠ "claude"; defined by the
     jail/transport boundary, per-harness altitude per ADR-011); A12 pins the funnel
     MCP tools TRANSPORT-AGNOSTIC (D-B5 + §E VT — no harness token in the tool body;
     the arm split lives in the spawn seam), so a future out-of-jail (e.g. http)
     transport — which could confine the pi arm too — reuses the tools + governance
     unchanged. Re-lock is the User's; these amendments reconcile prose to source and
     fence the arm boundary pre-lock. -->

## 1. Design Problem

Move dispatch orchestration **off the main thread** onto a fully-confined
`dispatch-orchestrator` subagent. It sits in `Jail(coord-cwd)`: raw `Edit`/`Bash`
reach the coordination tree (inside its cwd), but the shared `.git` is RO-walled,
so every boundary-crossing git write rides a **doctrine MCP tool** (witnessed to
bypass the `PreToolUse` wall — RSK-225 discharged). No exemption, no un-jailing:
the MCP tool surface *is* the trusted-write split. This is the serial-dependent
**capstone** of RFC-005 Mode B, building on SL-198's `worker_commit` keystone,
per-worktree jail record, and conformance lint. It unlocks off-main-thread and
parallel/hierarchical dispatch (the RFC-011 lever: main context stays a thin
cache-stable router; the ~40–49k/phase funnel ceremony burns a disposable
subagent context).

## 2. Current State (witnessed 2026-07-04, Claude Code 2.1.198)

- **The wall** (`src/worktree/pretooluse.rs`, `jail.rs::resolve_target`): every
  subagent carries `agent_id` ⇒ `Jail(cwd)` (cwd a linked worktree) or `Reject`
  (else). MCP tools bypass it (`decide()` mediates only `Bash|Edit|Write`).
- **create-fork** (`src/worktree/create.rs`, the `WorktreeCreate` hook): forks a
  worker at the armed base + provisions the per-worktree jail record
  (`jail/<name>.toml`) **iff the payload `cwd` == the arming spawn dir**
  (positional discriminator, `create.rs:362`); else Passthrough (detached, no
  record). The main thread satisfies this by parking its **persistent** Bash cwd
  in the arming dir.
- **MCP server** (`src/mcp_server/tools.rs`): exports review/memory/onboard only;
  `root` fixed to **primary**. Each tool = parse args → call the same `run_*` the
  CLI uses → serialize. No dispatch surface yet.
- **Funnel engine seams already callable** (clap-free, explicit params):
  `run_import` (import.rs:264), `run_gc` (gc.rs:221), `run_record_boundary`
  (dispatch.rs:718), `run_phase` (slice.rs). The dispatch mutations are **not**
  CLI-shell-coupled at the engine level — only unexposed via MCP.
- **SL-198 (needs, `ready`, not executed)** delivers: `worker_commit`, the
  enriched dispatch **record** (`.doctrine/state/dispatch/record/<name>.toml`,
  distinct from the jail-**policy** file at `.../dispatch/jail/<name>.toml`)
  `{name,dir,branch,base,coord}` + its gc deletion, the coord-tree enumerate/probe resolver, the conformance lint (already
  handling the `orchestrator` marker for SL-199 reuse), the import-a-commit switch.

## 3. Forces & Constraints

- **RFC-005 threat model (locked).** No un-jailing; the design must not rest
  integrity on trusting a worker *or* the orchestrator. Belts are server-side.
- **ADR-012** — coordination worktree is the sole write target; no per-worker
  projection onto shared branches; integration-sync is a verb, conflict ops
  report-and-halt, never auto-land.
- **ADR-011 D6** — the `WorktreeCreate` payload is thin (no agent_type/base/path);
  positional/marker discrimination only. Mode B revises the D6 risk calculus
  (governance §7).
- **ADR-006 sole-writer / ADR-008 + SL-182 confinement** — Mode B punches no hole
  in the wall; SL-181 (Mode A exemption) is out of scope.
- **DRY / behaviour-preservation** — reuse `run_import`/`run_gc`/
  `run_record_boundary`/`run_phase` and SL-198's resolver; existing funnel +
  main-thread suites stay green unchanged.

## 4. Guiding Principles

Smallest coherent capstone over existing seams. The confined orchestrator does
*local* work raw (coord-tree edits, arming writes, `Agent` spawn) and every
*boundary-crossing* write through a thin MCP tool that rides an existing engine
belt. One seam, two doors — no forked funnel logic.

## 5. Proposed Design

### 5.A — create-fork Fork discriminator (linchpin, LOCKED)

**Why net-new.** A **jailed subagent's Bash cwd resets to its assigned worktree
root every tool call** (doc-confirmed `subagents.md:263`; no spawn-time cwd
override exists — full negative result in
[[mem.fact.dispatch.confined-subagent-cwd-resets-breaks-positional-arming]]).
So a confined orchestrator can only ever present **coord-root** as a nested
worker's `WorktreeCreate` payload cwd — never the arming dir. The positional
discriminator therefore always takes Passthrough: no `dispatch/<name>` branch, no
jail record ⇒ SL-198's `worker_commit` cannot resolve the worker. Witnessed live.

**Target — an additive second Fork trigger; positional path unchanged.**
`classify_create` (pure, table-tested) gains two inputs:

```
classify_create(cwd_resolved,
                cwd_is_arming_dir,   // main-thread positional — UNCHANGED
                cwd_is_coord_root,   // NEW: payload.cwd == resolved root
                coord_in_dispatch,   // NEW: root on a dispatch/<NNN> branch
                base, name) -> CreateAction
```

Fork iff `cwd_is_arming_dir` **(main-thread)** OR
`cwd_is_coord_root ∧ coord_in_dispatch ∧ base present/plausible` **(confined)**;
else Passthrough. Both arms run the same `act_on_create::Fork` → `fork_core` +
`provision_jail_policy` (no second impl).

**Shell gather** (create.rs ~362): `cwd_is_coord_root = (root == payload.cwd)`;
`coord_in_dispatch = git -C root branch --show-current =~ ^dispatch/[0-9]+$`; read
`base` whenever either Fork condition is live. **The `base` file is the same
location for both arms** — `<root>/.doctrine/state/dispatch/spawn/base` — only the
*trigger* differs (positional cwd vs coord-root∧dispatch-branch); no second base
source, no divergent read path.

**Disarm discipline (confined) — one-shot, consumed in the hook (revised, ext-review
F4).** No cd-out self-clear, and *not* a disarm-after-return (a crash between spawn
and return would leave a stale `base` that force-forks the *next* benign spawn off an
old base — manufacturing a fake worker identity + stale jail record, per codex F4).
Instead the drive-loop writes `base`+`jail.toml` immediately before each spawn
(`dispatch arm-spawn --path .` from coord-root — cwd-safe; `--base` defaults to the
coord-root `HEAD` when omitted — PHASE-05 delta A1, so the recipe carries no
LLM-composed sha), and the **create-fork hook
consumes (deletes) `base` atomically the moment it Forks** — so the arm is strictly
one-shot and cannot survive to mis-fork a second spawn even across an orchestrator
crash. Serial: unambiguous. Parallel batch: one shared `base` per batch (existing
single-slot semantics, [[mem.fact.dispatch.single-slot-arming-rendezvous]]) — hook
consumption there is a plan detail (per-spawn token vs batch slot).

**The branch guard is hygiene, not a security boundary — forge-proofing not
required.** Satisfying `coord_in_dispatch` grants a spawn *nothing* a plain jailed
worker lacks — a forked worker is walled by `PreToolUse` and belt-gated at
`worker_commit` regardless of Fork/Passthrough. Failure modes are benign: a loose
guard at worst spuriously Forks a benign spawn (debris, not escape); a jailed
worker cannot spoof it (the spawn dir is outside its worktree = RO, shared `.git`
= RO, so it can neither arm nor create a `dispatch/<NNN>` branch — only the
trusted orchestrator arms, inside its own coord-root jail). Contrast the SL-181
coordinator marker, which *un-jails* and therefore *must* be forge-proof; this
discriminator lifts nothing, so a cheap branch-name check suffices. Real security
lives in the wall (confinement) + SL-198's server-side belts (blast radius),
neither of which depends on this guard being correct.

**Invariants / tests (pure TDD):** positional cases unchanged → Fork; confined
`coord_root ∧ dispatch-branch ∧ base` → Fork; benign `coord_root ∧ dispatch-branch
∧ ¬base` → Passthrough; `coord_root ∧ ¬dispatch-branch ∧ base` → Passthrough;
`coord_root ∧ base ∧ bad-sha` → `BadBase`. Both Fork arms provision the record (e2e).

**OS → plan.** OS1: is a coord tree *always* on `dispatch/<NNN>` (never detached
mid-op)? Verified live for SL-198/185; confirm `dispatch setup` never detaches —
if it can, the guard needs a state-file marker instead of the branch check. OS2:
keep-both vs retire-positional — recommend keep-both (lower risk, positional's
cd-out self-clear is a free safety; retiring is a larger behaviour change for no
confined-arm gain).

### 5.B — Dispatch MCP funnel surface (LOCKED)

Four **discrete** MCP tools, one per engine seam. Each: parse `{slice, …}` →
**resolve the coord tree server-side by slice-id** → call the existing `run_*`
with `path = <coord>` → serialize.

Three tools (was four; the phase-conclusion pair merges — D-B1). **Every tool whose
output must live in committed `dispatch/<slice>` history commits server-side** — the
confined orchestrator can't reach coord `.git`, and the CLI arm's "orchestrator
commits next" assumption (dispatch.rs:714, import.rs:82-97) does not hold for it.

| Tool | Wraps | Args | Commits? | Returns |
|---|---|---|---|---|
| `dispatch_import` | `run_import` (fork arm) | `{slice, name}` → base=coord tip, fork=`dispatch/<name>` | **yes** (code) | `{coord_tip}` |
| `dispatch_conclude_phase` | `run_phase`(completed) sheet-flip (runtime) **+** `run_record_boundary` commit | `{slice, phase, code_start, code_end, note?}` | **boundary only** (1 commit; flip is gitignored) | — |
| `dispatch_reap` | `run_gc` (patch-id landed-oracle belt) | `{slice, name}` | no (worktree/branch delete) | — |

(A start-of-phase `in_progress` flip, if committed, rides a thin `run_phase` call —
plan detail; the load-bearing sequence is import → conclude → reap.)

- **D-B0 — committed outputs commit server-side; scope is a HARD pre-commit gate**
  (revised, ext-review F1/F2/F5). `dispatch_import` applies **then** commits (the
  unconfined server writes coord `.git`, as `worker_commit` does for a worker),
  returning the new `coord_tip` (feeds `conclude`'s `code_end` + the next phase's
  `B`). **Scope/undeclared is the existing HARD belt, not advisory:** `classify_import`
  already refuses undeclared paths (`Refusal`, import.rs:130-134) — SL-199 keeps that
  refusal, server-side, **before** apply/commit. A scope violation ⇒ nothing lands ⇒
  **report-and-halt** to the main thread. This is *why* "one seam, two doors" holds:
  the belt is preserved intact, not weakened to advisory. It also removes the trust
  inversion codex F5 flagged — integrity never rests on the confined orchestrator
  blessing an already-committed over-reach it cannot `git reset` away.

- **D-B1 — discrete ops; the boundary commit is the atomic unit, the flip is
  disposable runtime** (revised ext-review F1/F3; RECONCILED to source 2026-07-05,
  delta A6). `import` and `reap` stay discrete — genuinely independent ops, distinct
  belts/timing, each rides its `run_*` seam. `dispatch_conclude_phase` composes
  `run_phase` + `run_record_boundary` but is **NOT** a single flip-plus-boundary
  commit: the shipped tool (mcp_server/dispatch.rs:499-509) keeps **two tiers** —
  (a) `set_phase_status` flips the **gitignored** phase sheet to `completed`
  (disposable runtime, idempotent on retry, **never in committed history**), and
  (b) **one** working-tree-free `commit_on_behalf` lands the `(B, coord_tip)`
  boundary row (all-or-nothing). The **atomic unit is the boundary commit**, not the
  flip. DRY intact (composes the two `run_*` seams, no forked logic). *Earlier delta
  language ("flip + boundary in one atomic commit", "unreachable") was source-stale
  and is retracted here.* *Alternative considered (rejected):* fold the flip INTO the
  commit — impossible, the flip is gitignored runtime by design (D-B4) and must not
  enter `dispatch/<NNN>` history.
- **D-B2 — coord resolved server-side by slice-id; no caller-supplied path**
  (mirrors SL-198 X1). Resolver = `git worktree list --porcelain` (primary) →
  worktree on `dispatch/<slice>`. **Sibling of SL-198's worker-by-agent
  resolver — shared enumerate step lands in SL-198; SL-199 adds coord-by-slice.**
- **D-B3 — the clobber doesn't occur in coord context; completed-without-boundary is
  the EXPECTED, self-healing fault** (revised ext-review F3; RECONCILED 2026-07-05,
  delta A6). The [[mem.pattern.doctrine.phase-complete-clobbers-boundary]] clobber is
  **suppressed** here: `set_phase_status`'s arm-guard skips solo-binding whenever a
  *live* `dispatch/<slice>` worktree exists (state.rs:542-551) — exactly the confined
  orchestrator's context. So flipping `completed` installs **no** degenerate row to
  overwrite. A crash between the (runtime) flip and the boundary commit leaves a
  **completed sheet with no committed boundary** — and per the shipped design that is
  the *only* fault outcome and it is **self-healing**: the sheet is disposable, so a
  retry re-runs `dispatch_conclude_phase` and re-composes the same boundary
  (idempotent). The durable completion signal is the **committed boundary** on
  `dispatch/<NNN>`, never the sheet flip. (This retires the earlier claim that
  atomicity made the state "unreachable" — the state IS reachable and is designed to
  heal, not to be prevented.)

- **D-B4 — the server commit is WORKING-TREE-FREE; provenance reuses the codebase
  convention** (added — 2nd codex pass, pre-lock plan review 2026-07-04). The load-
  bearing correction the 2nd review forced: because the confined orchestrator cannot
  reach coord `.git` (RO-walled) it also cannot `git reset` — so a server commit that
  staged into the **live** coord index (`git apply --index`, import.rs:346; or
  `run_record_boundary`'s live-file write) would leave a **poisoned, unrecoverable**
  dirty tree on any fault, and sweep pre-existing residue into the next commit. The
  fix rides plumbing the repo already has: `commit_on_behalf` composes the commit
  **working-tree-free** — a tree-level compose (`merge_tree --write-tree`, git.rs:846,
  `working-tree-free, object-db only`; `commit_tree`, git.rs:828) or a **scratch
  `GIT_INDEX_FILE`** (`filter_tree` kit, git.rs:677-732) applied `--cached`,
  `write-tree`, `commit_tree`, `update-ref` — so the live coord index+worktree stay
  **byte-unchanged** until the ref moves; a fault leaves nothing to reset. **Working-
  tree-free bar (2nd-pass new finding):** `git apply --index` writes the WORKING TREE
  — only `--cached` is index-only — so `dispatch_import` composes the committed worker
  branch onto the coord tip TREE-LEVEL (`merge_tree`/`commit_tree`), or if any apply
  path is used it is `--cached` against the scratch index, NEVER `--index`.
  `dispatch_conclude_phase` reuses `run_record_boundary`'s **pure** `BoundaryRow`
  compute (NOT its live-file write) and hands the new `boundaries.toml` blob to the same
  primitive. The **completed-flip** stays gitignored runtime (`.doctrine/state/`,
  disposable, re-established on retry); the **committed boundary** on `dispatch/<NNN>`
  is the durable completion signal. **Provenance** (R4, was open) is now DECIDED by
  reusing the existing `GIT_AUTHOR_*/GIT_COMMITTER_*` = `<id> <id@doctrine>` convention
  (git.rs:2157-2160, asserted git.rs:3789), as `worker_commit` does: import preserves
  the worker's author + dispatch committer; conclude sets author==committer==dispatch
  id; the message carries a funnel marker naming slice/phase. The `git cherry` patch-id
  landed-oracle is diff-based ⇒ **provenance-immune** — reap is unaffected. **Resolver
  seam**: `resolve_coord` ENUMERATES (`list_worktrees`, git.rs:1390), not the single-hit
  `worktree_for_ref`/`live_worktree_for_ref` probes — only enumeration can raise the
  defensive `ambiguous(>1)` arm.

- **D-B5 — the funnel tools are TRANSPORT-AGNOSTIC; the arm split lives in the spawn
  seam, not the tool body** (invariant, delta A12). Each tool is `parse →
  `run_*`/compose → serialize` with **no** harness assumption inside — no
  stdio / `WorktreeCreate` / `agent_id` dependence. Harness-specific mechanism lives
  ONLY in the spawn/fork seam (ADR-011) and §A's create-fork discriminator. So the
  identical tool set serves any transport that can sit the MCP server where the caller
  reaches it: **stdio-inside-the-jail today** (claude confined arm), an **out-of-jail
  http** server tomorrow (which could confine the pi arm too — the asymmetry is a
  transport-placement property, not a pi-arm-inherent limit). Coord is resolved
  server-side by slice-id (D-B2) and integrity rests on the wall + server-side belts,
  never the caller's identity — so a new transport needs **no** new authorization
  model. **Guard against rot:** a §B VT asserts no funnel-tool body references a harness
  token; coupling that leaks back in silently re-splits the arms.

**Trust posture.** Called by the confined orchestrator (not a worker), on the
coord tree it already governs. No new belts — the engine seams' belts come along.
The orchestrator's tool-surface is pinned by SL-198's conformance lint (the
`orchestrator` marker) — §C.

**OS → plan.** `run_phase` semantics under `path=coord` (cross-tree guard,
slice.rs:1046?); the exact commit provenance (author/message) for server-side
commits-on-behalf (ext-review F6); whether a separate `in_progress` start-flip tool
is needed or the flip rides `conclude` only.

### 5.C — `dispatch-orchestrator` agent-def + conformance lint (LOCKED)

**Agent-def** — `.claude/agents/dispatch-orchestrator.md`, modelled on
`dispatch-worker.md`:

```yaml
---
name: dispatch-orchestrator
description: Confined dispatch orchestrator — drives the funnel for one slice from
  inside its coordination worktree; nested-spawns workers, lands their deltas via
  doctrine MCP tools, reports conflict-judgement back to the main thread.
doctrine-role: orchestrator
tools: Read, Edit, Write, Bash, Grep, Glob, Agent,
  mcp__doctrine__dispatch_import, mcp__doctrine__dispatch_conclude_phase,
  mcp__doctrine__dispatch_reap
---
```
(Final funnel membership tracks §B/D-B1 — the three core tools shown, plus a start-
`in_progress` flip tool if plan finds one needed; the allowlist is whatever §B ships.)

**Worker isolation is def-pinned too (PHASE-05 delta A2).** By the same
"deterministic config rides the def surface, not an LLM per-call arg" principle
that motivates the MCP allowlist, the nested `dispatch-worker`'s `isolation:
worktree` lives on the **worker** def FRONTMATTER (`install/agents/claude/
dispatch-worker.md` + its materialized copy), NOT on the orchestrator's per-call
`Agent` spawn. F12 confirmed frontmatter isolation is honored for a nested spawn
(Claude Code 2.1.198); F13 directs that anything deterministic ride the
frontmatter/tool surface (even Opus intermittently omits a per-call isolation
arg). This closes one F7 co-cause — the orchestrator LLM omitting the per-call
isolation arg, which left the worker running un-isolated in the coord tree.

**Two layers pin two different things** (why native writers + a marker coexist):
- **The wall** (runtime) bounds *where* raw `Edit`/`Write`/`Bash` land — coord cwd
  only, shared `.git` RO. Native writers are safe *because* the wall confines
  their blast radius.
- **The lint** (static) bounds the *MCP* surface — the only privileged
  cross-boundary door. Asserts the def's `mcp__*` tokens ⊆ the `orchestrator`
  allowlist = exactly the §B funnel set. Any other `mcp__*` (a second server, a bare
  `mcp__doctrine` grant, `worker_commit`) ⇒ lint fail. Stops a doctored orchestrator
  smuggling a second write channel.

**§C's delta over SL-198's lint.** SL-198 lands the lint mechanism + parses the
marker/`tools:` list + the `worker → {worker_commit}` allowlist row. §C adds one
row: `orchestrator → {the §B funnel tools}` (`dispatch_import`,
`dispatch_conclude_phase`, `dispatch_reap`, + any start-flip tool). Per STD-001 the
allowlist references the **same named constants** §B defines for the tool names — no
second literal. §C is data (a table row), not new parsing.

**Reads go raw; only git-writes go MCP.** The tight allowlist is sufficient
*because* the orchestrator reads slice/phase/dispatch state via the raw `doctrine`
CLI over Bash (in-jail, cwd-safe — `doctrine slice show --path .`, `dispatch
status`, corpus search from the coord tree's `./target/debug/doctrine`). Reads
never cross the wall, so they need no MCP grant. Only the **git-boundary writes**
(§B) do. **Soft coupling:** the main thread spawns via `Agent
subagent_type: dispatch-orchestrator`, referencing this `name:` from a
skill/prompt (not a Rust constant, unlike the worker's) — lower-stakes drift,
pinned by prose not a test.

**Placement contract — self-enforcing, no hook.** Unlike the worker, the
orchestrator is **not** forked by `WorktreeCreate`: it's a plain subagent spawned
into a **pre-existing** coord tree (created by `dispatch setup`), the main thread
having parked its persistent cwd there before the `Agent` spawn ⇒ `agent_id`
present + cwd = a linked worktree ⇒ `Jail(coord)`. So no `SubagentStart` provision
hook, no jail record for the orchestrator itself, and therefore **no
`name:`↔Rust-constant drift pin** (the worker needs one because its
`SubagentStart` matcher keys on `DISPATCH_WORKER_AGENT_TYPE`; the orchestrator is
matched by the `doctrine-role` marker, a free string the lint owns). Mis-placement
(spawned from primary cwd) fails loud — first raw `Bash`/`Edit` ⇒ `Reject`.

**OS → plan.** OS-C1: is the allowlist a **ceiling only** (prevent excess
privilege) or also a **floor** (reject a def *missing* `Agent`/a funnel tool)?
Lean ceiling-only — SL-198's grain; an under-equipped def fails loud at runtime,
not a security concern. OS-C2: confirm SL-198's lint iterates **all** marked defs
(a client project's second `orchestrator`-marked def is covered) vs. name-matching
one file.

### 5.D — The drive-loop (LOCKED)

**Scope line — the orchestrator owns the per-phase fork→land loop *into the coord
tree*; cross-tree/trunk ops stay main-thread.** `prepare-review` and `integrate`
write trunk (outside the coord jail, RO) — the confined orchestrator *cannot*
touch them. It drives ready phases, lands each into the coord branch, then
**reports-and-halts** to the main thread, which runs `dispatch sync --integrate`.
This is why the funnel is a tight three-tool set, not more: the trunk-facing verbs
are never the orchestrator's.

**Serial per-phase cadence (happy path):**

1. **`arm-spawn --path .`** (raw Bash, cwd = coord-root — cwd-safe; writes
   `base=B`+`jail.toml` into `.doctrine/state/dispatch/spawn/`, inside its own
   jail). `B` = current coord tip. **`--base` defaults to the coord-root `HEAD`
   when omitted** (PHASE-05 delta A1, option b) — so `B` is correct-by-construction
   from the confined orchestrator's cwd-pinned coord-root, no LLM-composed sha in
   the recipe. **On the confined arm this is correct-by-construction** (cwd is always
   the coord-root, so the default reads the true coord tip). **Off the confined arm
   the default is NOT self-catching** (delta A7, corrected): a misplaced/main-thread
   `arm-spawn` run from the wrong root captures a wrong base `B'`, and every gate is
   consistent with `B'` — `worker_commit` checks `C^==record.base(=B')` (passes) and
   `dispatch_import` recomputes `merge-base(coord.tip, fork_tip)` (dispatch.rs:434),
   never comparing against the armed base — so a wrong-but-consistent base 3-way-
   composes onto the coord tip and can **land silently**. The earlier "fails loud at
   `C^==B`, never a bad land" claim is therefore **false for the misplaced arm** and
   is retracted. The early-catch dispatch-branch guard (**IMP-268**) is the actual
   safety net for that case; its deferral is a token-cost bet on the confined arm
   being the sole caller, NOT on a downstream gate catching a bad base.
2. **Spawn nested `dispatch-worker`** (`Agent subagent_type: dispatch-worker`).
   **Isolation rides the WORKER def frontmatter (`isolation: worktree`), not a
   per-call `Agent` arg** (PHASE-05 delta A2; §C, F12/F13) — deterministic,
   independent of the orchestrator LLM emitting an arg it may omit. Confined-
   subagent cwd = coord-root ⇒ §A's confined arm Forks: `dispatch/<name>` at `B`,
   jail record provisioned. **Base-control in the armed (Fork) path is entirely the
   create-fork hook** — it reads the arm file `B` and sets base+branch, *overriding*
   `baseRef=head` (the fail-closed provisioning seam, unlike the read-only
   `SubagentStart` stamp, mechanics:154-166). *Accuracy note (PHASE-05 delta A4,
   F9):* the mechanics-doc phrasing "vanilla forks off *session* HEAD"
   (dispatch-mechanics.md:39-42) is imprecise for a spawn from a **linked** tree —
   the *unarmed* (Passthrough) fork point is the **spawner's worktree HEAD** (coord
   tip), not a fixed default/session branch. But that Passthrough path yields a
   **detached** tree with **no `dispatch/<name>` branch and no jail record** →
   `worker_commit` cannot resolve it, so a coincidentally-correct base buys nothing.
   The arm is therefore the **sole source of *usable* base-at-`B`** (base + branch +
   record together); parent-HEAD inheritance does not reduce its necessity.
3. **Worker self-commits** via `worker_commit` (SL-198) — one gated commit `C`
   (`C^==B`) on its own branch.
4. **`dispatch_import`** → apply **+ commit** server-side (D-B0), returns
   `{coord_tip}`. Scope violation ⇒ hard refuse ⇒ report-and-halt (nothing lands).
5. **`dispatch_conclude_phase`** — flip the **gitignored** phase sheet to `completed`
   (disposable runtime) **+** land the true `(B, coord_tip)` boundary as **one**
   working-tree-free commit (D-B1/D-B3). The `completed` flip never enters committed
   history; the committed **boundary** is the durable completion signal. A crash
   between the flip and the boundary commit self-heals on retry (idempotent
   re-compose), not an atomicity guarantee that the pair land together.
6. **`dispatch_reap`** — belt-gated by the **patch-id landed-oracle** (`git
   cherry`, mechanics:108-117): `run_gc` refuses to delete a fork whose patch isn't
   yet in coord history. Crash-proof, sibling-move-proof; inherited free.
7. **Disarm is automatic** — the create-fork hook already consumed `base` at step 2
   (one-shot, §A F4); no explicit clear. Next phase re-arms with the new `coord_tip`
   as `B`.

**Conflict-judgement → report-and-halt.** Any op needing main-thread/human
judgement — a red worker verify, a **hard import scope refusal** (server-side, not
an orchestrator bless), a `refresh-base`/`candidate`/`integrate` race
(mechanics:134-152, all
report-and-halt by design) — the orchestrator returns a **structured summary** and
stops. Never auto-merges, never self-unblocks a dirty-trunk refusal.

**Serial vs parallel: deferred to plan.** The serial loop is the design baseline;
a parallel batch reuses the same per-phase steps under the existing single-slot
arming rendezvous (one shared `B` per batch,
[[mem.fact.dispatch.single-slot-arming-rendezvous]]) — worker count / file-disjoint
batching is a plan concern, not a design fork.

### 5.E — Verification alignment

What evidence each part must add/change (mode per criterion — VT test / VA agent /
VH human):

- **§A — VT (pure, table-driven).** Extend `classify_create`'s existing table:
  positional cases unchanged → Fork (regression, must stay green); confined
  `coord_root ∧ dispatch-branch ∧ base` → Fork; benign `coord_root ∧ dispatch-branch
  ∧ ¬base` → Passthrough; `coord_root ∧ ¬dispatch-branch ∧ base` → Passthrough;
  `coord_root ∧ base ∧ bad-sha` → `BadBase`. Plus e2e: both Fork arms provision the
  jail record; **one-shot arm (F4)** — the hook consumes `base`, so a second spawn
  with no re-arm takes Passthrough (no double-fork off a stale base).
- **§B — VT (tool integration).** Per tool: parse args → resolve coord by slice →
  `run_*` → serialize. `dispatch_import` asserts **commit happened** (coord tip
  advanced, one non-merge commit) **and** that an **undeclared-scope delta is HARD-
  refused before any commit** (F2 — nothing lands, report-and-halt). `dispatch_conclude_phase`
  asserts flip+boundary land in **one** commit and that a crash-simulated abort leaves
  *neither* (atomicity, F1/F3). Server-side commit **provenance** (author/message) is
  asserted against the contract fixed at plan (F6). Belt regressions (import scope,
  reap landed-oracle) ride the existing `run_*` suites unchanged (behaviour-
  preservation gate). **Transport-agnostic invariant (D-B5, A12):** a VT asserts no
  funnel-tool body references a harness token (stdio / `WorktreeCreate` / `agent_id`)
  — the arm split stays in the spawn seam, so a future out-of-jail transport reuses
  the tools unchanged.
- **§C — VT (lint).** Extend SL-198's lint suite: the sanctioned orchestrator def
  passes; a def granting an extra `mcp__*` / a bare `mcp__doctrine` / `worker_commit`
  fails, message naming the offender. A fixture def under `.claude/agents/`.
- **§D — VH + VA (VH-1, still OWED).** The end-to-end confined drive-loop is
  orchestration — **not yet witnessed**: VH-1 owes a live run where a real phase
  forks→lands→concludes→reaps under the REAL confined `dispatch-orchestrator` + REAL
  `dispatch-worker` + `worker_commit` → import → conclude → reap. The §6 probe proved
  only the isolation **primitive** (throwaway def, general-purpose orchestrator,
  unarmed), NOT this integrated armed loop. The mechanical pieces (steps 4–6) are
  covered by §B VTs; VH-1 is the integration proof and remains outstanding.
  **Recovery VT (F3, reconciled A6):** simulate a crash between the runtime flip and
  the boundary commit — assert the phase sheet is `completed` while the boundary is
  `Missing`, then assert a retry of `dispatch_conclude_phase` re-composes the boundary
  idempotently (self-heal). (Not "unreachable" — the state is reachable by design and
  the VT proves it heals.)

## 6. Feasibility probe (empirical basis, 2026-07-04)

A confined `general-purpose` subagent (cwd = a linked coord tree → `Jail`) was
driven live:
- **Wall holds:** write inside coord tree OK; escape to `/workspace/doctrine`
  denied read-only. "No un-jailing" stands.
- **Nested `isolation:worktree` spawn works** from the confined subagent; forked
  at the armed base.
- **Positional arming broke:** the subagent's Bash cwd reset from the arming dir
  to coord-root between calls → `WorktreeCreate` payload cwd = coord-root →
  create-fork Passthrough → **detached, no branch, no jail record** (witnessed).
- **No override:** doc-confirmed inherent subagent behaviour; no frontmatter cwd
  field, no `Agent` cwd param, no hook-return lever
  ([[mem.fact.dispatch.confined-subagent-cwd-resets-breaks-positional-arming]]).

**PHASE-05 re-validation (2026-07-05, F9–F13).** The "nested `isolation:worktree`
spawn works from the confined subagent" bullet was **primitive-correct** and is
re-confirmed live (F9/F10/F12). An interim finding (F7) claimed that nested
isolation was *not* honored on Claude Code 2.1.198 and that §5.D was therefore
infeasible — that was **two recipe/def defects, not a harness ceiling**: (1) the
orchestrator LLM omitting the per-call `isolation` arg → worker ran un-isolated in
the coord tree (F9/F10 — fixed by def-frontmatter isolation, delta A2), and (2) the
arm recipe missing the required `--base` → create-fork Passthrough, no branch/record
(F11 — fixed by the default-to-coord-`HEAD` `arm-spawn`, delta A1). Both fixes are
**designed, not yet built** (A1: the binary still REQUIRES `--base`; A2: the worker
def carries no `isolation` field) and the isolation **primitive** is confirmed live
(F9/F10/F12) — but the **integrated armed drive-loop is NOT yet witnessed** (VH-1,
§5.E, still owed). So the capstone is assessed **realizable** on the claude arm on
the strength of the primitive + the already-built §A/§B machinery, **not** "confirmed
feasible" end-to-end; VH-1 is the outstanding proof (delta A9). The **cwd-reset** bullet
above is untouched — still true, still the basis for §A's coord-root Fork trigger.

This is the empirical basis for §A. MCP-write bypass (the §B premise) was
witnessed prior (RSK-225, SL-198).

## 7. Governance (to ratify at reconcile)

Per RFC-005 §7: an **ADR-012 REV** (topology: the confined-orchestrator actor
class + the dispatch MCP funnel surface), an **ADR-011 D6 amendment** (the
positional-arming discriminator gains the confined-arm branch; the D6 risk
calculus under MCP-mediated writes), and a **note on SL-182/ADR-008** that Mode B
adds a *sanctioned MCP write surface* to a confined orchestrator without lifting
the wall (made safe by the conformance lint). ADRs are owner-locked VH.

**Frame the actor class harness-NEUTRALLY (delta A11).** Ratify the confined
orchestrator as *"an orchestrator confined to its coordination worktree whose
git-boundary writes ride a sanctioned MCP surface — on any harness whose MCP
transport sits where the confined caller can reach it"*, NOT as "the claude
confined orchestrator". The claude / stdio-in-jail arm is the **first instance**,
not the definition; an out-of-jail http transport (which could confine the pi arm
too — the block is transport placement, not the arm) is a future instance that must
NOT need a fresh governance round. Transport placement is a per-harness altitude
detail (ADR-011), never an actor-class boundary.

**Doc + spec deltas (deliverables, not just ratification):** the shipped
`install/dispatch-mechanics.md` needs a **Mode B section** (the confined-
orchestrator arm: MCP funnel, `dispatch_import` folds the commit, reads-raw/writes-
MCP split) — it currently documents only the main-thread + pi arms. And check
tech-spec-021 (dispatch orchestrator process) for a **REQ delta** covering the
confined-orchestrator actor class. Both land in-slice; sized at reconcile/plan.

## 8. Risks (initial)

- **R1 — OS1 (coord-tree branch assumption). RESOLVED at plan.** `coordinate()`
  always `git worktree add -b dispatch/<NNN>` (Create) / `add <dir> <branch>`
  (Resume), never `--detach` (coordinate.rs:212-227) — the coord tree is always on
  `dispatch/<NNN>`, so the `coord_in_dispatch` branch guard is sound; no state-file
  marker needed. Residual: re-confirm only if `dispatch setup`'s worktree-add changes.
- **R2 — MCP soft-dependency (load-bearing on this arm).** A confined orchestrator
  has no raw-git fallback; MCP-server health is a dispatch-stopper. Mitigation:
  main-thread dispatch remains the MCP-down fallback; document.
- **R3 — belt drift** between the funnel MCP tools and CLI verbs. Mitigation: call
  the same `run_*`; no forked copies (behaviour-preservation gate).
- **R4 — commit-on-behalf provenance (ext-review F1/F6). RESOLVED at plan (2nd pass).**
  Provenance is DECIDED (D-B4): reuse the codebase `<id> <id@doctrine>` GIT_AUTHOR_*/
  GIT_COMMITTER_* convention — import preserves worker author + dispatch committer;
  conclude author==committer==dispatch id; funnel-marker message names slice/phase.
  Asserted (§5.E, PHASE-02 VT-3). The patch-id landed-oracle is provenance-immune.
- **R6 — poisoned-index / server-commit residue (2nd codex pass). CLOSED by design
  (D-B4).** A server commit staging into the live coord index would strand an
  unrecoverable dirty tree on fault (the confined orchestrator cannot `git reset`).
  Closed: `commit_on_behalf` composes working-tree-free via a scratch `GIT_INDEX_FILE`
  + `commit_tree`, leaving the live tree byte-unchanged until the ref moves. Residual:
  none for correctness; verify the scratch-index isolation (PHASE-02 VT-4).
- **R5 — arm crash window (ext-review F4).** Closed by design: the create-fork hook
  consumes `base` one-shot, so an orchestrator crash mid-spawn cannot leave a stale
  arm that mis-forks the next benign spawn. Residual: verify the hook's consume is
  atomic w.r.t. the fork (plan).

**External review provenance.** codex (GPT-5.5) hostile pass, 2026-07-04 — F1
(`record_boundary` uncommitted), F2/F5 (undeclared is a HARD belt, not advisory;
no post-commit bless), F3 (`set_phase_status` suppresses solo-bind in coord context;
real risk is a missing row), F4 (one-shot arm), F6 (verification) — all verified
against source and integrated above. Trunk-facing report-and-halt boundary survived.
**2nd pass (pre-lock plan review, 2026-07-04):** two blockers — live-index commit
residue the confined actor cannot reset, and conclude atomicity defined only against
committed history — both verified against source and collapsed to one fix (working-
tree-free `commit_on_behalf`, D-B4); plus the resolver naming the wrong (single-hit)
reuse seam and provenance under-decided — all integrated (D-B4, R4/R6, PHASE-02/03).
