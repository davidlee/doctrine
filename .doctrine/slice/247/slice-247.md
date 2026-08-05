# Usable non-worktree subagents

## Context

A Claude subagent spawned without worktree isolation is currently useless. The
`PreToolUse` confinement wall denies it `Bash` and `Edit`/`Write`
(`worktree-jail: cwd-not-a-worktree`); only `Read` and `Agent` pass. It can read
files and spawn nested subagents, and nothing else.

This carves the acute leg of IMP-401. The companion legs — retiring the Claude
plugin delivery channel (IMP-400) and the per-worktree `.claude/` question
(IMP-401 `OQ-2`/`OQ-3`) — are deliberately deferred to a second slice; see
*Non-Goals*.

### The mechanism

`resolve_target` (`src/worktree/jail.rs:377`) maps
`(agent_id, cwd, cwd_is_project_worktree, is_nominated)` → `Target` in four arms:

| condition | verdict | effect |
|---|---|---|
| no `agent_id` — interactive / orchestrator | `Orchestrator` | `PassThrough` |
| `agent_id` + **nominated** | `Orchestrator` | `PassThrough` |
| `agent_id` + cwd *is* a linked project worktree | `Jail(cwd)` | rw cwd only; shared `.git` RO |
| `agent_id`, not nominated, cwd not a worktree | `Reject` | `Deny` (`jail.rs:387`) |

That last arm is every ordinary subagent. It is `Reject` **by design**: the
fail-closed rule was written as *"pass through iff `agent_id` is ABSENT"*,
deliberately not *"jail when in a worktree, else pass through"* — the latter
fails open for `isolation: none`, which carries an `agent_id` with cwd = repo
root. Correct for dispatch workers. Fatal for ordinary use.

**IMP-401's card describes three arms and omits nomination.** The card is right
about the outcome and wrong about the solution space being empty. A
`PassThrough` escape hatch already exists and is already load-bearing: the
nomination allowlist (SL-206 PHASE-11, design §5.6) at the fixed, out-of-jail
path `.doctrine/state/orch-allowlist.txt`, read by
`is_nominated` (`src/worktree/subagent.rs:502`, called from
`src/worktree/pretooluse.rs:444`), checked *before* the topology leg, scoped to
exactly one `agent_id`, and fail-safe (absent or unreadable file ⇒ not
nominated). It is granted by an explicit write on the confined-orchestrator
path (Mode B), never by default — so no ordinary subagent is ever nominated.

This matters for scope: the change may well be *"grant the existing verdict to
a broader population"* rather than *"add a new verdict"*, which puts the
subagent lifecycle seam (`src/worktree/subagent.rs`, the `SubagentStart`
handler) inside the fence alongside `jail.rs`. The design picks; the scope must
not pre-empt it by fencing only `jail.rs`.

### What the wall actually covers

Verified live against Claude Code 2.1.198
(`mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only`, trust: high).
Do not re-derive:

- `decide()` matches on `tool_name` only: `Bash` → nested-bwrap wrap,
  `Edit|Write` → `realpath ⊆ cwd`, **everything else → `PassThrough`**.
- `Agent` and `Read` pass through even under `Reject`. Nested spawns reach
  ≥ depth-3.
- **Writable `mcp__*` tools bypass the wall entirely** on the Claude arm — the
  MCP server is a stdio child of the top-level harness, outside every subagent's
  bwrap, resolving paths against the primary repo root. Proven under *both*
  verdicts. This is RSK-225, and it means the wall's containment claim is
  already weaker than its posture suggests.
- `PreToolUse` hooks fail open; only exit 2 blocks.
- With the Bash wrapper disabled (control, 2026-07-01) a worktree subagent's
  Bash wrote to repo root, shared `.git`, `/tmp`, `$HOME`, and the host
  mount-ns. Re-arming flipped every vector to `Read-only file system`. The
  harness imposes **zero** native Bash containment. It *does* natively deny
  subagent `Edit`/`Write` to the shared checkout before any hook runs.

The asymmetry — `Edit`/`Write` has a native floor, `Bash` has none — is why
leg 1 (worktree subagents stay in their worktree) cannot be delivered by simply
dropping the wall.

## Scope & Objectives

**One shippable change: make a non-worktree subagent useful without letting a
worktree subagent out of its worktree.**

1. **Settle the policy for the fourth arm.** Decide what an ordinary,
   un-nominated, non-worktree subagent may do — full pass-through, or a floor
   that denies only writes outside the repo root / to `.git`. This is the
   decision the slice exists to make; it is a policy question, not a bug fix.
2. **Implement it** on whichever seam the design selects — a new verdict in
   `resolve_target`, a broadened nomination grant, or a policy input to the
   existing arms.
3. **Preserve the worktree wall.** `Jail(wt)` behaviour for
   `isolation: worktree` subagents is unchanged and stays proven by the existing
   suites (the behaviour-preservation gate — shared-machinery change, existing
   tests stay green unchanged).
4. **State the wall's actual guarantee.** Given the MCP bypass (RSK-225), decide
   whether the wall still claims adversarial containment or is restated as a
   guard-rail against accident, and say so where the posture is documented.

### Constraints

- **Interim and deletable.** RFC-025's capsule programme (ADR-020) retires most
  of dispatch, worker stamping, and the in-session confinement apparatus. This
  slice must not accrete anything the capsule cutover then has to unpick.
  Prefer the smallest patch that works; throwing it away should cost nothing.
- **ADR-011 is incumbent authority** until the capsule cutover; ADR-020 is the
  successor. This sits under the incumbent and must not pre-empt the successor.
- **Purity split** — `resolve_target` and `decide()` are pure leaves; topology
  and allowlist reads are the shell's
  (`mem.pattern.dispatch.jail-resolve-inputs-injected-env`). Any new input
  arrives injected, not read in the pure layer.
- **POL-002** — harness-specific knowledge stays out of the engine core.

## Non-Goals

- **IMP-400** — retiring the Claude plugin delivery channel, moving hook
  activation into `.claude/settings*.json`, and the install/doctor legs. Second
  slice. Touches SPEC-010 / PRD-003 and needs its own design run.
- **IMP-401 `OQ-2`/`OQ-3`** — whether a worktree-local `.claude/` binds for an
  in-session `isolation: worktree` subagent (Case B), and whether Case A
  (a session *started* in a worktree) is worth building for. These answer to
  IMP-400's `OQ-2` (tracked `settings.json` vs gitignored
  `settings.local.json`), so they travel with that slice.
- **Closing RSK-225** — the writable-`mcp__*` bypass. This slice must *account*
  for it when stating what the wall guarantees, and may narrow worker tool
  surfaces if that falls out cheaply, but fixing the bypass is separate work.
- **Dispatch itself.** No change to the dispatch funnel, worker spawn, import
  belts, or the `worker_commit` gate.
- **Restructuring `src/worktree/{jail,pretooluse}.rs`.** IMP-401 `OQ-5` asks how
  much survives capsules; the answer bounds appetite. Smallest patch wins.

## Affected surface

- `src/worktree/jail.rs` — `resolve_target`, `Target`, `decide_bash`,
  `decide_write`, `decide_agent`.
- `src/worktree/pretooluse.rs` — the `decide()` entry, nomination read,
  `project_anchor()` resolution.
- `src/worktree/subagent.rs` — `SubagentStart` role resolution and the
  nomination write, if the design routes through nomination.
- Agent definitions under `.agents/` / `plugins/doctrine/` — if worker tool
  surfaces get pinned.

## Risks, assumptions, open questions

- **`R1` — fail-open regression.** Any broadening of `PassThrough` risks
  restoring the hole the `Reject` arm was written to close. The
  `isolation: none` case (carries `agent_id`, cwd = repo root) is
  indistinguishable at `PreToolUse` from a mis-placed dispatch worker, because
  the `PreToolUse` payload carries `agent_id` + `cwd` but **not** `agent_type`.
  Role must be resolved at `SubagentStart` if it is to be resolved at all.
- **`A1` — dispatch is out of scope but not out of the blast radius.** The
  confined-orchestrator path (Mode B, SL-198/199/206) depends on the nomination
  arm. A change to nomination semantics reaches it.
- **`OQ-1`** — What should a non-worktree subagent be allowed to do? Full
  pass-through, or a repo-root floor? What threat is being modelled now that
  dispatch is out of scope? (Inherited from IMP-401 `OQ-1`.)
- **`OQ-2`** — Does the fix belong in `resolve_target` as a new verdict, or in
  the nomination grant as a broadened population? The former is a smaller
  diff; the latter reuses proven, fail-safe machinery.
- **`OQ-3`** — Do IMP-269 and IMP-342 close as duplicates of this slice, or do
  they carry residue? IMP-269 (2026-07-05) reports the identical defect for
  `/fork` subagents and poses the same open question. IMP-342 reports the
  narrower symptom — the Bash arm blocking read-only `doctrine` CLI reads from
  delegated research subagents. Both plausibly discharge here; confirm at
  reconcile rather than assuming.
- **`OQ-4`** — Does the wall keep claiming adversarial containment? The MCP
  bypass says that claim is not currently true on the Claude arm.

## Verification / closure intent

- **By test** — the existing `resolve_target` / `decide` unit suites stay green
  unchanged (behaviour preservation for the `Orchestrator`, `Jail`, and
  nomination arms), plus new cases pinning whatever the fourth arm becomes,
  including the `isolation: none`-shaped input the old rule was written to
  reject.
- **By agent** — a live probe on the Claude harness: a plain (non-worktree)
  subagent runs `Bash` and edits a file within the repo; a worktree subagent
  still cannot write outside its worktree. The control matters as much as the
  case — this surface has a documented history of reasoning that did not
  survive probing.
- **By human** — the stated guarantee of the wall is accurate and the posture
  documentation matches it.

## Summary

## Follow-Ups
