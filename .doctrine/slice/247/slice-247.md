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

1. **Settle the policy for the fourth arm** — *settled: DEC-152.* An ordinary,
   un-nominated, non-worktree subagent passes through unconfined; no repo-root
   floor. It was a policy question, not a bug fix.
2. **Implement it** at the seam the inquiry selected — a **three-valued
   topology input** to `resolve_target` (DEC-154), so DEC-152's grant applies
   only where the topology is positively known not to be a worktree, and
   `Unknown` keeps today's deny. Not the nomination grant (the
   `PRIVILEGED_AGENT_TYPES` fusion trap) and not a `JailPolicy` input.
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
  `project_anchor()` resolution, and `cwd_is_project_worktree`, which stops
  collapsing its three error sites into `false` (DEC-154).
- `tests/e2e_worktree_pretooluse.rs` — new. The verb is the only one of its
  stdin-payload family (`create-fork`, `stamp`, `verify-worker`) with no e2e
  coverage, and DEC-154 puts the change in the shell (`inq-8`).
- ~~`src/worktree/subagent.rs`~~ — **out**. The nomination route was rejected
  at triage (the `PRIVILEGED_AGENT_TYPES` fusion trap), so no `SubagentStart`
  role resolution is added.
- Agent definitions under `.agents/` / `plugins/doctrine/` — named, not pulled.
  Tool-surface scoping is a term in what the wall guarantees (`inq-6`), not a
  change this slice makes.

## Risks, assumptions, open questions

- **`R1` — fail-open regression. Held, but its stated cause was wrong**
  (corrected 2026-08-06, `inq-5`). The regression risk is real and DEC-154
  answers it: `Unknown` topology keeps the deny. The claim that `agent_type` is
  absent from the payload is **false** — `docs/claude/hooks.md:592` and a
  verified probe log both say the harness sends it beside `agent_id`; the
  original ✓ cited our own `PreToolUseInput` struct, which is evidence about
  the parser, not the payload. Role is therefore probably resolvable at
  deny-time, but the route dies on POL-002 regardless (it needs a closed set of
  agent-type names in the engine), so nothing downstream changes.
- **`A1` — dispatch is out of scope but not out of the blast radius.** The
  confined-orchestrator path (Mode B, SL-198/199/206) depends on the nomination
  arm. A change to nomination semantics reaches it.
- **`OQ-1` — SETTLED by DEC-152.** Full pass-through, no floor. The threat
  model is accident, not adversary.
- **`OQ-2` — SETTLED: neither.** Nomination is a trap (the
  `PRIVILEGED_AGENT_TYPES` fusion), `JailPolicy` cannot carry it, and role
  discrimination dies on POL-002. The fix is a three-valued topology input to
  `resolve_target` (DEC-154) — the smaller diff, and the one that keeps
  `Unknown` fail-closed.
- **`OQ-3`** — Do IMP-269 and IMP-342 close as duplicates of this slice, or do
  they carry residue? IMP-269 (2026-07-05) reports the identical defect for
  `/fork` subagents and poses the same open question. IMP-342 reports the
  narrower symptom — the Bash arm blocking read-only `doctrine` CLI reads from
  delegated research subagents. Both plausibly discharge here; confirm at
  reconcile rather than assuming.
- **`OQ-4` — SETTLED.** No. The wall is a guard-rail against mostly-accidental
  holes. What it guarantees is **composite** — bwrap for `Bash`, pathcheck for
  `Edit`/`Write`, tool-surface scoping at the agent definition for `mcp__*`
  (`inq-6`: the punch-through is a scoped grant, not an unguarded hole).

## Verification / closure intent

- **By test** — the existing `resolve_target` / `decide` unit suites stay green
  (behaviour preservation for the `Orchestrator`, `Jail`, and nomination arms),
  plus new cases pinning the three-valued topology input, including the
  `isolation: none`-shaped input the old rule was written to reject. Note the
  one departure from *unchanged*: widening the boolean churns the 7 existing
  `resolve_target` call sites mechanically (DEC-154). No behaviour moves.
- **By test (new leg, `inq-8`)** — a thin e2e on the `pretooluse` verb,
  `tests/e2e_worktree_pretooluse.rs`: confirmed worktree ⇒ Bash rewritten to
  the bwrap argv; confirmed non-worktree ⇒ pass-through; topology `Unknown` ⇒
  deny. Deny is stdout data, never an exit code — the verb always exits 0. The
  `Unknown` case needs no git surgery: an absent `CLAUDE_PROJECT_DIR` produces
  it deterministically. Justified because DEC-154 puts the change in the
  **shell**, the layer with no coverage at all.
- **By agent** — a live probe on the Claude harness: a plain (non-worktree)
  subagent runs `Bash` and edits a file within the repo; a worktree subagent
  still cannot write outside its worktree. The control matters as much as the
  case — this surface has a documented history of reasoning that did not
  survive probing. **Its evidence lands in an authored sink** (`notes.md` or an
  EVD record), never the gitignored scratchpad — a VA criterion over runtime
  state leaves an audit nothing to re-derive.
- **By human** — the stated guarantee of the wall is accurate and the posture
  documentation matches it.

## Summary

## Follow-Ups

- **At reconcile — contribute the post-capsule finding to RFC-025** (DEC-154's
  sibling disposition, `inq-7`, user 2026-08-06). Aim (2) — learning what
  subagents look like once dispatch is gone — lands in RFC-025's programme,
  which is where the successor reads. Deliberately deferred to reconcile:
  before the implementation lands and the live probe runs, the finding is a
  prediction. Do NOT write it at design or execute time.
- **Not taken: amending `ADR-006` §D2b.** Declined at `inq-7` — `ADR-013` routes
  it through a REV, and `ADR-020`'s cutover rewrites the section anyway. The
  accepted cost is that until then `ADR-006` states a posture `DEC-152` has
  changed, reconciled only in a reader who finds both.
