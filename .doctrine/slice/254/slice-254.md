# Collapse dispatch onto one subprocess arm

## Context

`EVD-023` records Anthropic's June 15 update: Claude Agent SDK, `claude -p`, and
third-party app usage still draw from subscription usage limits. That falsifies
the billing half of `ADR-011`'s founding premise —

> claude's `Agent` tool has no env channel, and `claude -p` is
> Anthropic-API-billed (not subscription) and harness-specific … **For claude
> the only viable backend is the in-session `Agent` tool**, which exposes no env
> seam and no exec wrapper.

`DEC-202` takes the consequence: run the claude worker as a confined
subprocess, like codex and pi, and delete the apparatus that existed only
because it could not be one.

### One premise, four mechanisms

The four things being removed are not independently motivated designs. They are
one workaround wearing four hats, each traceable to a clause of `ADR-011` D3:

| mechanism | the clause it exists for | replacement |
|---|---|---|
| disk marker as sole worker identity | D2/D4 — `Agent` has no env channel | `--setenv DOCTRINE_WORKER 1` in the bwrap prefix |
| `SubagentStart` stamp hook | D3 — the marker needs a writer and `Agent` has no exec wrapper | orchestrator stamps before exec, as on pi |
| `worktree pretooluse` confinement wall | D3 — *"OS confinement: none — `Agent` is not a subprocess to wrap"* | nested bwrap, as on pi |
| gated `worker_commit` MCP tool | a hook-confined in-session worker cannot self-commit | a clone has a writable `.git` |

`ADR-011` D4 lists three enhancements as "codex/pi-only until a free claude env
backend lands" — env-arm worker identity, per-worktree env delivery, and nested
bwrap. A subprocess spawn **is** that backend and takes all three at once.

### Why this is cheaper than its blast radius suggests

The confinement boundary already exists and is already factored.
`scripts/pi-spawn-confined.sh:120-127` is an eight-token `PREFIX` array with a
macOS `sandbox-exec` sibling beside it. The claude-specific delta is rebinding
`$HOME/.pi` → `$HOME/.claude` and changing the exec target. The work is not
building confinement; it is deleting the thing that stood in for it.

### Not a capsule slice

`RFC-025` already records this end state — *"the capsule target uses uniform
sandboxed subprocess workers (`claude -p` / codex / pi) rather than in-session
subagents"* — and its roadmap step 2 is "Establish uniform headless subprocess
workers", step 3 raw-clone provisioning. This slice brings that forward for the
**incumbent** arm only.

It is therefore **not** governed by the capsule programme's evidentiary bar. The
microVM turn (`DEC-190`, the conformance-suite split; `DEC-191`,
confinement-as-per-front-profile; `EVD-015`, the Firecracker spike) concerns the
*capsule authority boundary*. This slice asks only for parity with what the pi
arm does in production today. Importing capsule-grade assurance here will stall
it for no gain — see `R1`.

### Provenance

Originates from `IDE-024` (2026-06-30), which independently reached the same
shape including the self-commit consequence. Supersedes the abandoned `SL-247`,
whose `DEC-152` and `DEC-154` both pre-argue this direction: `DEC-152`'s
rationale argues the wall is not earning its keep, and `DEC-154`'s residual
names assert-at-spawn as the successor to infer-at-deny — which is what clone
provisioning does.

## Scope & Objectives

**One shippable change: dispatch has one worker-spawn shape — a confined
subprocess in a clone — on every harness.**

1. **Spawn `claude -p` as a confined subprocess** under the pi arm's bwrap
   prefix, with the macOS `sandbox-exec` sibling kept at parity.
2. **Provision workers as clones, not linked worktrees.** A clone's writable
   `.git` is what lets the worker self-commit and dissolves the orchestrator's
   import-the-diff trade.
3. **Delete the in-session apparatus** — the four `worktree pretooluse` hook
   matchers, the `SubagentStart` stamp, marker stamping, and the
   `claude-force-subprocess-dispatch` config key, which has no second mode left
   to select.
4. **Collapse the skills.** `/dispatch-agent` and `/dispatch-subprocess` merge;
   `/dispatch`'s arm-routing branch goes with them.
5. **Land the governance.** `ADR-011` Context + D3, and the `SPEC-021`
   requirements that encode two arms. See *Reconcile & closure complexity* —
   this is the larger half of the slice, not a tail.

### Constraints

- **`POL-002`** — harness-specific knowledge stays out of the engine core. The
  collapse should *reduce* harness branching, and any residue that must stay
  belongs at the skill/script tier, not in the binary.
- **Two `PreToolUse` hooks are `memory surface`, not confinement.** Only the
  four `worktree pretooluse` matchers and the `SubagentStart` stamp are in
  scope. Deleting the memory-surfacing hooks would be a silent regression of an
  unrelated capability.
- **Behaviour-preservation gate** — the codex/pi arm is production today. Its
  existing suites are the proof that the collapse did not disturb it.
- The change must not pre-empt `ADR-020`'s capsule cutover, only shrink it.

## Non-Goals

- **The capsule contract.** No capsule provisioning, admission, conformance, or
  microVM work. `SPEC-030`'s requirements are untouched.
- **`REQ-335` / `FR-007`, the confined-orchestrator mediated-write tier.** It is
  `pending` and `RFC-025` holds it so deliberately — the capsule contract is its
  successor. It stays pending.
- **Retiring the Claude plugin delivery channel** (`IMP-400`) and the
  per-worktree `.claude/` question — `SL-247`'s deferred companion legs remain
  deferred.
- **Solo worktrees.** `/worktree` for non-dispatch isolation is untouched;
  `REV-046`'s cutover intent already commits to preserving them.

## Affected surface

Coarse and provisional — the exact touch-set is `/design`'s job.

- `src/worktree/` — `pretooluse.rs` (1113), `subagent.rs` (671), `jail.rs`
  (2218). ~4000 lines, **not all deletable**: `jail.rs` also holds the bwrap
  argv builder, which the subprocess arm wants to keep and may want to own.
  Separating the reusable core from the hook-serving shell is a design question,
  not a scoping one.
- `src/dispatch_config.rs`, `src/dtoml.rs` — the
  `claude-force-subprocess-dispatch` key.
- `.claude/settings.json` — four `worktree pretooluse` matchers, the
  `SubagentStart`/`Stop` nominate/denominate pair, `WorktreeCreate create-fork`.
  Whatever `doctrine install` seeds must move with it.
- `scripts/pi-spawn-confined.sh` — the `PREFIX` array, generalised past `pi`.
- Skills: `dispatch` (201), `dispatch-agent` (223), `dispatch-subprocess` (72),
  `worktree`. Plus `install/dispatch-mechanics.md`.
- `tests/` — 13 `e2e_worktree_*` files.

## Risks, assumptions, open questions

- **`R1` — assurance-bar creep.** The likeliest failure is a reviewer applying
  the capsule programme's evidentiary standard to an interim simplification.
  Mitigation is stated up front in *Not a capsule slice*: the bar is parity with
  today's pi arm, not `SPEC-030` conformance.
- **`R2` — the billing premise is a pause, not a settlement.** `EVD-023` records
  Anthropic "working to update the plan", with notice promised before anything
  takes effect. Accepted knowingly: `DEC-202` rests on the subprocess arm being
  the better shape independently, not on billing. Billing removed a blocker; it
  is not the reason. The warned window makes this a monitored risk, not a bet.
- **`R3` — mid-cutover posture on the incumbent.** `RFC-025`'s discipline is
  that "every slice but the last lands beside the incumbent worktree arms … so
  the repo never sits in a half-cutover state." This slice deliberately breaks
  that by *removing* an arm. Accepted in `DEC-202` on the grounds that it
  shrinks the eventual capsule slice 5 rather than growing it — but it is the
  slice's sharpest governance tension and design should not soften it.
- **`A1` — clone self-commit.** Assumed: a clone's writable `.git` dissolves the
  pi arm's standing "worker cannot self-commit → orchestrator imports the
  working-tree diff" trade. `IDE-024` reached the same conclusion independently.
  **Not yet verified** — confirm before it becomes load-bearing for objective 2.
- **`A2` — `claude -p` reaches the worker's needs.** Tool-surface scoping,
  structured hand-back, and MCP availability under `-p` are assumed sufficient.
  `inq-6` of `SL-247` established that tool availability is definable per agent
  definition or per invocation; whether that holds under `-p` is unconfirmed.
- **`OQ-1`** — Does `jail.rs`'s bwrap argv builder become the subprocess arm's
  home, or does the script keep owning the prefix? Bears on how much of
  `src/worktree/` survives, and on `POL-002`.
- **`OQ-2` — inherited from `SL-247`'s `OQ-3`.** Do `IMP-269` (same defect for
  `/fork` subagents) and `IMP-342` (Bash arm blocking read-only `doctrine` CLI
  reads from research subagents) discharge here? Add `IMP-334` (arm
  `isolation: worktree` omission costs a full worker cycle), `IMP-337` (worker
  absolute-path reads silently hit the primary tree), and `IMP-407` (doctor leg
  naming the hook-activation blocker) — all five are plausibly dissolved rather
  than fixed. Confirm at reconcile; do not assume.
- **`OQ-3`** — one arm or two rungs? If some environment cannot run `claude -p`,
  does the in-session arm survive as a degraded rung, or is it simply
  unsupported? `DEC-202` chose deletion; design should confirm nothing depends
  on the fallback.

## Reconcile & closure complexity

Surveyed at scoping because on this change the governance landing is the larger
half. Nothing here is a tail item.

**Requirements that encode two arms** — both `active` in `SPEC-021`, both
needing disposition through a `REV` per `ADR-013`:

- **`REQ-288`** (`FR-002`) — *"Arm routing is deterministic:
  `claude-force-subprocess-dispatch` forces `/dispatch-subprocess`; otherwise
  the env-marker (`.claude/` presence) selects `/dispatch-agent` vs
  `/dispatch-subprocess`…"* Its entire premise is a choice between two arms.
  With one arm it does not narrow — it **retires**.
- **`REQ-291`** (`FR-005`) — *"Per-harness enforcement altitude is a uniform
  contract with honest non-uniform reach … claude attains base==B by placement …
  marker-only identity, no pre-dispatch baseline-verify, and a fail-open
  `SubagentStart` stamp."* The non-uniform reach *is* the requirement. Under the
  collapse the reach becomes uniform and the requirement's point dissolves —
  a rewrite, not an edit.

**Spec responsibilities** carrying the same content in prose: `SPEC-021`
responsibilities 2 and 5 (`spec-021.toml:16,19`). A `REV` `modify --target
SPEC-021` amends spec prose directly and is surfaced-for-manual at apply
(`mem_019f181b394a7eb1be9ffb401de8f5d3`) — budget for hand-finishing.

**Silent-rot hazard.** `SPEC-021` declares
`plugins/doctrine/skills/dispatch-agent/SKILL.md` as a `[[source]]` anchor.
Deleting that skill leaves a dangling anchor and **`spec validate` will not
catch it** (`mem_019f999f39fe7820a0831f4413a539b4`). `SPEC-012` and `SPEC-022`
both carry `pretooluse.rs` / `subagent.rs` in source-anchor comments. These must
be swept deliberately; nothing will fail red.

**Governance:** `ADR-011` Context + D3 (the falsified premise), and probably
`ADR-006` §D2b — which `SL-247`'s `inq-7` declined to amend *precisely because*
it expected a cutover to rewrite the section. This is that cutover arriving
early. Precedent for the shape is in `ADR-011` itself: D5 and D6 already read
"AMENDED — FALSIFIED (SL-064 §8)" as in-place amendments.

**Does this ride `REV-046` or its own?** `REV-046` is `proposed`,
`approval=none`, and gates the capsule **cutover** (slice 5). `RFC-025` says to
approve and apply it "when there is an exact migration plan rather than an
aspirational delete list, which is at slice 5 and not before." This slice is not
slice 5. Recommendation: **its own `REV`**, so `REV-046` stays clean for the
capsule cutover. Settle at design.

**Orphaned obligation from `SL-247`.** Its Follow-Ups carried *"At reconcile —
contribute the post-capsule finding to `RFC-025`"* (`DEC-154`'s sibling
disposition, `inq-7`, user 2026-08-06). `SL-247` is terminal and will never
reconcile, so that obligation is homeless. It transfers here — and it is now
better served, because this slice actually *performs* the retirement the finding
was to be about. Carried in *Follow-Ups*.

## Verification / closure intent

- **By test** — the codex/pi suites stay green unchanged (behaviour
  preservation: that arm is production and must not move). The 13
  `e2e_worktree_*` files are triaged into keep / retarget / delete rather than
  deleted wholesale.
- **By test** — a worker spawned as `claude -p` under the bwrap prefix cannot
  write outside its clone. This is the control that matters: it is the guarantee
  the deleted wall was claiming, now held by the OS instead of a fail-open hook.
- **By agent** — a live dispatch phase driven end-to-end on the claude harness
  through the subprocess arm, including self-commit from the clone (`A1`).
  **Evidence lands in an authored sink** — `notes.md` or an `EVD` — never the
  gitignored scratchpad (`mem_019fd1d862887d42b7a1f88c28fd28a7`).
- **By human** — `ADR-011`'s corrected text and the `SPEC-021` requirement
  dispositions accurately describe the shipped arm, and no `[[source]]` anchor
  points at a deleted file.

## Summary

## Follow-Ups

- **At reconcile — contribute the post-capsule finding to `RFC-025`.** Inherited
  from `SL-247` (`inq-7`, user 2026-08-06), which was abandoned before its
  reconcile could discharge it. Deliberately deferred to reconcile for the same
  reason as before: until the arm is actually gone the finding is a prediction.
- **At reconcile — disposition the five plausibly-dissolved backlog items**
  (`OQ-2`): `IMP-269`, `IMP-342`, `IMP-334`, `IMP-337`, `IMP-407`, plus
  `IMP-401` and `IDE-024` themselves. Confirm; do not close by assumption.
