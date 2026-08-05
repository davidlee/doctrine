# Notes SL-247: Usable non-worktree subagents

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage (exploring, 2026-08-05)

Discharges `explore.triage`. Evidence: `research/research.md` (✓-marked rows
verified by the assembling agent).

### Motivations (user, 2026-08-05)

Two, both to be served **as cheaply as possible**:

1. Stop subagents being broken now, while working toward RFC-025.
2. Help answer what subagents look like once dispatch is gone in its current
   form — while still supporting worktrees that cannot knock *mostly
   accidental* holes in the primary tree.

Motivation 2 changes the read of "interim and deletable": this is not throwaway
work, it is a **cheap probe of the post-capsule shape**. The code is deletable;
the finding must not be.

### Constraining governance

- **ADR-020** — ADR-011 stays authoritative until REV-046's cutover gates are
  met. Interim work sits under the incumbent; ADR-020 is not a revision
  candidate.
- **ADR-006 D2b** — the harness does not confine workers to their worktree and
  raw-tree confinement is not CLI-stoppable. Why the worktree Bash wall cannot
  simply be dropped.
- **RFC-005** — states the threat model, but for *dispatch workers*
  (`subagent-orchestrator-design.md:83`, `:94`). It also already concedes the
  wall's trust basis is weak (`rfc-005.md:636`, `agent_id` absence as "an
  unauthenticated tell… a named residual").
- **POL-002** — bounds harness knowledge in the engine.
- Checked and **not applicable**, with reasons: PRD-015, SPEC-012, SPEC-021,
  CON-003, CON-005, RFC-022, SL-199's design. See `research.md` Thread 1.

### Shaping decisions (carried into the inquiry map)

- **The threat-model vacuum is real.** No authority says what a non-dispatch,
  non-worktree subagent may do. This is a finding, not a research gap — the
  `Reject` arm was a dispatch-motivated choice that incidentally caught every
  ordinary subagent.
- **`OQ-4` settled by the user**: the wall is a guard-rail against *mostly
  accidental* holes, not adversarial containment. The `mcp__*` bypass
  (RSK-225) already makes the stronger claim false on the Claude arm.
- **The nomination route is a trap** — `PRIVILEGED_AGENT_TYPES` (`jail.rs:122`)
  is deliberately fused as both nomination-eligibility *and* `decide_agent`'s
  privileged deny-set (`jail.rs:113-121`, RV-258 F-1). Broadening it to win a
  PassThrough would simultaneously forbid jailed callers from spawning that
  type. This **inverts** `slice-247.md`'s `OQ-2`, which read nomination as the
  machinery-reusing option.
- **The `mcp__*` punch-through is a scoped grant, not a leak** (user,
  2026-08-06). Dispatch workers are given just enough tool surface to commit,
  and tool availability is definable per agent definition or per `Agent`
  invocation. So objective 4's statement of the guarantee is **composite** —
  bwrap for `Bash`, pathcheck for `Edit`/`Write`, tool-surface scoping at the
  agent definition for `mcp__*` — and that last one is a design lever this
  slice should name even if it does not pull it.
- **`JailPolicy` is ruled out** as a carrier — layout precondition confines it
  to `.worktrees/<name>`, it is consulted only under `Target::Jail`, and
  `deny_unknown_fields` makes any schema change breaking.

### Risks

- **`R1` (from scope, ✗ CORRECTED 2026-08-06)** — the ✓ was wrong. It cited
  `pretooluse.rs:98-107`, which is our `PreToolUseInput` struct: evidence that
  the code does not *parse* `agent_type`, not that the harness does not *send*
  it. `docs/claude/hooks.md:592` lists `agent_type` beside `agent_id` as the
  pair added whenever a hook fires inside a subagent, and
  `mem.fact.claude.pretooluse-agent-carries-spawner-id` (verified) preserves a
  probe log carrying it. Narrow residual doubt: that probe was on matcher
  `Agent`, so Bash/Edit/Write is corroborated by docs but not by our own log.
  Role is therefore probably resolvable at deny-time — but the route dies on
  POL-002 anyway (it needs a closed set of agent-type names in the engine), so
  the correction changes the record, not the design. See DEC-154 § notes.
- **`R2` (new)** — no e2e coverage of the `pretooluse` verb exists at all, so
  the live-probe verification leg carries unusual weight (`inq-8`).
- **`A1` (from scope)** — dispatch is out of scope but not out of the blast
  radius; the confined-orchestrator path depends on the nomination arm.

### Appetite

RFC-025's mechanism census marks this whole surface DELETE
(`mechanism-census.md:40`, `:44`, `:67`). Smallest patch that works is the
correct engineering answer, not a compromise.

### Open questions

Live in the design run's inquiry map (`doctrine design show 247`) — `inq-1`
through `inq-8`. Not restated here; the run is the source of truth.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-05 · design/inquiring · e771a5da

### Produced

- DEC-152 — disposes `inq-1`. Minted hollow; facets filled and then corrected
  2026-08-06 (the floor was cheap, and the rationale that decided it is
  environment-dependent — see the record).
- DEC-154 — disposes `inq-5`.
- ISS-318 — design-run defect found while driving this slice.
- Observation records `019fd226-…` and `019fd22a-…` (RFC-011 instrumentation).
- `research/` (runtime, gitignored — not a durable sink; its findings are
  carried in the triage section above).

### Learned

- The `PRIVILEGED_AGENT_TYPES` fusion trap — recorded in the triage section
  above, **memory candidate** at close. It is not in any durable sink outside
  this file, and `research.md` is disposable.
- Stale `~/.cargo/bin/doctrine` narrows `apply` payloads silently — see
  ISS-318 § Related. Also a memory candidate.

### Open

- `inq-7`, `inq-8` — blocking, live in the design run. Read them
  there (`doctrine design resume 247`), not from a copy here.
- Slice `OQ-3` — whether IMP-269 and IMP-342 close as duplicates of this
  slice. Settle at reconcile, not before.
- IMP-400 plus IMP-401's per-worktree `.claude/` legs — deferred to a second
  slice, not yet cut.
