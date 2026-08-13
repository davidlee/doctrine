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
| `SubagentStart` stamp hook | D3 — the marker needs a writer and `Agent` has no exec wrapper | nothing — the marker itself goes (`DEC-207`) |
| `worktree pretooluse` confinement wall | D3 — *"OS confinement: none — `Agent` is not a subprocess to wrap"* | nested bwrap, as on pi |
| gated `worker_commit` MCP tool | a hook-confined in-session worker cannot self-commit | the orchestrator's incumbent import, as on pi |

*Row 3 re-grounded and row 4 corrected, 2026-08-13 (`DEC-213`).* Row 4 read "a
clone has a writable `.git`", which was the clone half's answer. Under the
re-scope the claude worker inherits the pi arm's answer instead: it does not
self-commit, and the orchestrator imports its working-tree diff. `worker_commit`
is still deleted — it existed only for the in-session arm — but nothing replaces
it, and `classify_import` survives as the scope belt's enforcing caller
(`DEC-204`). Row 2's replacement is likewise not "stamp before exec": with
identity on the env leg there is no marker left to stamp.

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
subprocess — on every harness.**

*Re-scoped 2026-08-13 (`DEC-213`).* This originally read "a confined subprocess
**in a clone**", and took the clone half with it. `SL-254` now stops at the arm
collapse: **the claude arm becomes a pi arm** — a confined subprocess on a
**linked worktree**, with the **incumbent import transport** the orchestrator
already uses in production. Clone provisioning and everything it forces moved to
`SL-255`. The split is safe because `DEC-207`'s identity collapse was verified
*topology-independent*; had it depended on the clone, the identity work would
have had to move too.

1. **Spawn `claude -p` as a confined subprocess** under the pi arm's bwrap
   prefix, with the macOS `sandbox-exec` sibling kept at parity.

   *Hand-back settled 2026-08-13 (`DEC-215`).* The spawn line also carries
   `--output-format stream-json`, so the collapsed arm returns a **typed
   completion event stream** at parity with the pi arm's RPC `agent_end` rather
   than prose for the orchestrator to tail. This is argv — the same tier
   `DEC-209` put the confinement prefix at — so it is a flag and no code.
   `--json-schema` is deliberately **not** adopted: nothing consumes a shaped
   hand-back today, and adopting one would be building.
2. ~~**Provision workers as clones, not linked worktrees.**~~ **MOVED OUT to
   `SL-255`** by `DEC-213`. Kept in place rather than renumbered so the
   objectives other records cite by number still resolve. The clone's writable
   `.git`, worker self-commit, `worker_commit`'s replacement by
   fetch-from-clone, and the branch-point guard's re-homing onto fetched refs
   are all that slice's. Workers here stay on linked worktrees and the
   orchestrator keeps importing the working-tree diff.
3. **Delete the in-session apparatus** — the four `worktree pretooluse` hook
   matchers, the `SubagentStart` stamp, **disk-marker worker identity**, and the
   `claude-force-subprocess-dispatch` config key, which has no second mode left
   to select.

   *Corrected 2026-08-13.* This originally read "marker stamping", which
   understated it against `DEC-202`'s own list — the marker is mechanism #1 of
   its four, and `EVD-023` already names `--setenv DOCTRINE_WORKER 1` as the
   replacement. Scope and research both drifted off the record here; see
   `research.md` cross-thread finding 7.

   *Re-grounded 2026-08-13 (`DEC-213`).* This paragraph previously argued the
   marker is **inoperable** under a clone, because `describe_mode` gates it on
   `is_linked_worktree` (`marker.rs:89`), which is false for a clone. That
   argument **does not survive the re-scope** — workers stay on linked
   worktrees, so the marker leg can fire. The marker still goes, but as a
   *chosen* deletion rather than a forced one, on `DEC-207`'s topology-
   independent grounds: worker-ness is a property of a process, the marker
   models it as a property of a tree, and nothing anywhere reads marker
   *absence* as coordination-tree identity.

   *Enlarged 2026-08-13 (design `R6`, §7.2 `D1`–`D3`).* The honest deletion set
   is wider than this list, and every addition is the claude arm's substitute
   for something `worktree fork --worker` already does. Also deleted: **`dispatch
   arm-spawn`** with its arming-dir contract, **`create-fork`'s Fork arm** (the
   `WorktreeCreate` hook entry and its Passthrough arm survive), **`worktree
   verify-worker`**, the **`Spawn`-row recorder** (`run_create_fork_and_record`),
   the **`SubagentStop` `denominate` half** beside the `SubagentStart` stamp —
   `DEC-205` found the nominate/denominate pair a closed loop with the wall's
   own gate legs, so they die by construction — and the **`worker_commit` MCP
   tool** (`DEC-204`, 1495 lines), which existed only because a hook-confined
   in-session worker could not commit. Six hook entries go, not four. Retaining
   any of the first four would leave a live fork trigger that, with the marker
   deleted, mints an unmarked and unconfined worktree — strictly worse than
   deleting it.
4. **Collapse the skills.** `/dispatch-agent` and `/dispatch-subprocess` merge;
   `/dispatch`'s arm-routing branch goes with them.
5. **Land the governance.** `ADR-011`, `ADR-006` §D2b, `SPEC-021` and
   `SPEC-012`, through one `REV` of this slice's own. See *Reconcile & closure
   complexity* — this is the larger half of the slice, not a tail.

   *Corrected 2026-08-13 (`DEC-211`).* This read "`ADR-011` Context + D3", which
   under-counts by four regions. `ADR-011` changes at **eight**: Context, `D1`,
   `D2`, `D3`'s table, `D4`, `D6`, Consequences and Verification. The two a
   Context+D3 reading loses are the load-bearing ones — `D2` is the core
   contract, whose "claude's `Agent` path has no worker env channel" is exactly
   what a confined `claude -p` subprocess falsifies, and `D6` is sixty lines of
   fail-closed altitude resting on the `SubagentStart` stamp and the marker,
   both deleted here, whose "not fail-closable" conclusion inverts under
   `DEC-208`. `D5` and `D7` are recorded in `DEC-211` as considered and
   deferred. The `REV`'s target set is four entities.

### Constraints

- **`POL-002`** — harness-specific knowledge stays out of the engine core. The
  collapse should *reduce* harness branching, and any residue that must stay
  belongs at the skill/script tier, not in the binary.
- **Two `PreToolUse` hooks are `memory surface`, not confinement.** Deleting
  them would be a silent regression of an unrelated capability. The six entries
  in scope are the four `worktree pretooluse` matchers plus the
  `SubagentStart`/`SubagentStop` nominate/denominate pair (`DEC-205`; corrected
  2026-08-13 — this read "only … the four matchers and the `SubagentStart`
  stamp", which under-counted the deletion by two and contradicted objective 3).
  The `WorktreeCreate create-fork` entry stays; only its Fork arm goes.
- **Behaviour-preservation gate** — the codex/pi arm is production today. Its
  existing suites are the proof that the collapse did not disturb it.

  *Restored 2026-08-13 (`DEC-213`).* `DEC-212` had to replace this with an
  observable-contract formulation, because the original scope moved the pi arm
  to clones too and its transport tests pin exactly what was being replaced.
  Under the re-scope the pi arm **does not move at all** — the claude arm
  changes *into* the pi arm's existing shape — so "the codex/pi suites stay
  green unchanged" is achievable as written and is the gate again. What survives
  from `DEC-212`: the confinement control (a worker under the prefix cannot
  write outside its own directory), its **skip-rather-than-pass** rule on a host
  without `bwrap`, and the VA leg's evidence landing in an authored sink.
- **No new dependency** (`DEC-209`, added 2026-08-13).
  `crates/doctrine-control` is Linux-only, sits outside every default test
  selection, and is under active development as the capsule programme's crate.
  Generalise the incumbent bwrap prefix and port **no** hardening; the hardening
  delta is filed as `IMP-428`.
- **Subscription credential, bound wholesale** (`DEC-210`, added 2026-08-13).
  `ANTHROPIC_API_KEY` would forfeit the billing property `EVD-023` establishes,
  which is this slice's whole authorisation. Bind `$HOME/.claude` the way the pi
  arm binds `$HOME/.pi` — wholesale and read-write, on both the Linux inline
  array and the macOS `--extra-rw` path. Narrowing the mount set is deferred.
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
- **Everything `DEC-213` moved to `SL-255`** (added 2026-08-13). Clone
  provisioning, worker self-commit, fetch-from-clone in place of the import
  belt, and the branch-point guard's re-homing onto fetched refs. Workers here
  stay on **linked worktrees** with the **incumbent import transport**; the
  claude arm becomes a pi arm, no more and no less. Objective 2 records the move
  at the point it was struck.
- **Worker-side MCP, and the privileged-tool binary it would need**
  (`DEC-216`, added 2026-08-13). The confined claude worker gets **no MCP at
  all**, at parity with how the pi arm already spawns (`--no-extensions`); the
  orchestrator keeps performing every privileged act. `DEC-216` settles the
  *intent* for when a worker genuinely needs one — a separate worker-tools
  binary run **outside** the confinement and reached over stdio, so the tool is
  privileged by construction — but that binary is new construction and belongs
  to `SL-255`, where worker self-commit actually arrives. Recorded here because
  deleting `worker_commit` (`DEC-204`) is what made the intent ambiguous, and
  this is the slice doing the deleting.

## Affected surface

Coarse and provisional at scoping. **Design §5.6 now holds the exact touch-set
and supersedes this list** (2026-08-13); what follows is the scoping record plus
the surfaces the survey missed.

- `src/mcp_server/worker_commit.rs` (1495) — **missed at scoping.** Deleted
  outright with the arm it served (`DEC-204`); nothing re-homes, because the
  retained import transport keeps `classify_import` as the scope belt's
  enforcing caller.
- `src/worktree/` beyond the three modules below — `marker.rs`, `create.rs`,
  `fork.rs`, `land.rs`, `inventory.rs`, `gc.rs`, `import.rs` and `mod.rs` each
  lose legs; `src/dispatch.rs`, `src/boot.rs`, `src/mcp_server/tools.rs` and
  `src/commands/observation.rs` lose the deleted verbs, hook specs and marker
  disjuncts.
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

  *Softened 2026-08-13 (`DEC-213`), and say so plainly rather than claiming it
  away.* The re-scope keeps the incumbent import transport, so the breach is now
  one axis rather than two: an arm is removed, but the funnel is not replaced
  beside it. The tension is real and remains; it is smaller.
- ~~**`A1` — clone self-commit.**~~ **MOVED OUT to `SL-255`** by `DEC-213`. The
  assumption — a clone's writable `.git` dissolves the pi arm's standing "worker
  cannot self-commit → orchestrator imports the working-tree diff" trade
  (`IDE-024` reached it independently) — is **still unverified**, and that is
  precisely why it left. Verifying it is `SL-255`'s job, not a precondition
  this slice has to clear first.
- **`A2` — `claude -p` reaches the worker's needs.** Tool-surface scoping,
  structured hand-back, and MCP availability under `-p` are assumed sufficient.
  `inq-6` of `SL-247` established that tool availability is definable per agent
  definition or per invocation; whether that holds under `-p` is unconfirmed.

  *Narrowed 2026-08-13.* Research thread 3 reports the CLI surface complete —
  tool-surface scoping, permission modes, MCP config, and a `--json-schema`
  hand-back contract all exist as flags — so the work on this leg is
  **provisioning, not capability**. That report is a researcher claim and is
  **unverified**; `A2` stays an assumption. Two of its three legs are since
  disposed: **auth** is settled by `DEC-210` on the subscription credential
  (`ANTHROPIC_API_KEY` would forfeit the billing `EVD-023` establishes), and
  **MCP availability** is moot — `DEC-204` deletes `worker_commit`, so no MCP
  tool needs provisioning. What remains genuinely assumed is the **hand-back**:
  there is no typed subagent-return equivalent under `-p`.

  *Hand-back leg discharged 2026-08-13 (`DEC-215`).* `--output-format
  stream-json` yields a typed completion event stream at parity with the pi
  arm's RPC `agent_end`, at the argv tier — a flag, not code. All three legs are
  now disposed, so what `A2` still assumes is **provisioning**: that a confined
  `claude -p` with `$HOME/.claude` bound wholesale and no MCP actually starts,
  authenticates and reaches its tools. That is precisely what the `VA` leg
  exists to establish, and nothing short of it will.
- ~~**`OQ-1`**~~ — **ANSWERED at design 2026-08-13.** Does `jail.rs`'s bwrap
  argv builder become the subprocess arm's home, or does the script keep owning
  the prefix? **Both, split at the seam the tree already draws.** `DEC-206`
  re-homes four jail primitives (`REASON_NO_BWRAP`, `have_bwrap`,
  `write_seatbelt_profile`, `REASON_PROFILE_WRITE_FAILED`) from `pretooluse.rs`
  to `jail.rs` **as the first step, before any deletion** — `jail_prefix.rs`
  imports them today, so deleting `pretooluse.rs` first breaks the surviving
  arm. `DEC-209` keeps the prefix itself at the script tier: generalise the
  incumbent `scripts/pi-spawn-confined.sh`, take **no `doctrine-control`
  dependency**, port **no** hardening (deferred to `IMP-428`). `POL-002` is
  satisfied by that split rather than strained — the argv builder stays in the
  binary and the harness specifics stay in the script.
- **`OQ-2` — inherited from `SL-247`'s `OQ-3`.** Do `IMP-269` (same defect for
  `/fork` subagents) and `IMP-342` (Bash arm blocking read-only `doctrine` CLI
  reads from research subagents) discharge here? Add `IMP-334` (arm
  `isolation: worktree` omission costs a full worker cycle), `IMP-337` (worker
  absolute-path reads silently hit the primary tree), and `IMP-407` (doctor leg
  naming the hook-activation blocker) — all five are plausibly dissolved rather
  than fixed. Confirm at reconcile; do not assume.
- ~~**`OQ-3`**~~ — **ANSWERED at design 2026-08-13 (`DEC-208`): one arm.** No
  degraded in-session rung. An environment that cannot run `claude -p` **fails
  closed at spawn**, exactly as the pi arm already does today — so the answer is
  not a new posture but the incumbent one, extended. Design confirmed nothing
  depends on the fallback: `DEC-205` found `nominate`/`denominate` to be a
  **closed loop** with `pretooluse`'s gate legs (`is_nominated` has no other
  reader), so they die by construction rather than needing a disposition.
- ~~**`OQ-4`**~~ — **SETTLED at design 2026-08-13 (design §6 `OQ-1`): the
  collapsed arm's forks stay unbound.** `worktree fork --worker` binds a fork to
  its `(slice, phase)` only when `--slice` and `--phase` are both passed and the
  dir sits under `<coord>/.worktrees/<name>`, and `bind_dispatch_record` is the
  sole writer of a `DispatchRecord` — so an unbound fork leaves no record at
  all. `scripts/pi-spawn-confined.sh` passes neither flag, making unbound forks
  the surviving arm's production posture already. The binding's only readers are
  `worker_commit` (deleted here) and MCP `dispatch_import` (which additionally
  needs a committed fork tip the collapsed arm cannot produce). The retained CLI
  import transport reads none of it, and the funnel row is named by the
  orchestrator's explicit `PHASE-NN`. **Owner's call, 2026-08-13:** leave them
  unbound. Passing the flags would write a durable record for a consumer this
  slice deliberately leaves without a producer, and would move the production pi
  arm's behaviour against the behaviour-preservation gate. The binding belongs
  with the transport, and the transport is `SL-255`'s.
- **`OQ-5` — `dispatch_import` is retained without a producer.** A knowingly
  shipped residual, not an oversight (design §6 `OQ-2`, §7.2 `D6`): the MCP
  funnel import needs a committed fork tip, no arm produces one after the
  collapse, and the funnel cadence is governed prose `DEC-211` narrowed *out* of
  this slice's `REV`. Deleting the tool would move code ahead of the spec that
  governs it, in the direction that does not fail loudly. Revisit in `SL-255`,
  which owns the transport — or widen `SL-255` if a reviewer judges the
  code/spec gap unacceptable.

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

*Settled and enlarged 2026-08-13 (`DEC-211`).* Both hedges are discharged.
`ADR-011` is **eight** regions, not Context + D3 — see objective 5. `ADR-006`
§D2b is **not** "probably": it is definite, on thread 1's scope argument, and it
carries a **second** correction the scoping survey did not see — D2b names
`IMP-065` as "the real positive-marker close", but `IMP-065` was closed
**obsolete** on 2026-07-02 via `REV-018`, superseded by confinement rather than
delivered. The ADR therefore holds a live forward-reference to a close that will
never arrive, hedging a marker-absence assumption `DEC-207` deletes outright.
Both halves re-cut together. Two further targets the survey omitted: `SPEC-021`
loses **three** responsibilities plus a half-rewrite rather than two, and
**`SPEC-012`'s responsibility prose at `spec-012.toml:18`** is a `REV` target in
its own right — "fork = create + provision + **stamp** + emit per-wt env" loses
the stamp with the marker, while "import as the belted dispatch funnel" is
unchanged because import survives the re-scope.

**Does this ride `REV-046` or its own?** `REV-046` is `proposed`,
`approval=none`, and gates the capsule **cutover** (slice 5). `RFC-025` says to
approve and apply it "when there is an exact migration plan rather than an
aspirational delete list, which is at slice 5 and not before." This slice is not
slice 5. Recommendation: **its own `REV`**, so `REV-046` stays clean for the
capsule cutover. ~~Settle at design.~~ **SETTLED at design 2026-08-13
(`DEC-211`): its own `REV`**, confirmed by research thread 1. Riding `REV-046`
would couple this slice to hostile-ingestion, interpretation-policy and
microVM-measurement evidence it does not need.

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
  write outside its own worktree. This is the control that matters: it is the
  guarantee the deleted wall was claiming, now held by the OS instead of a
  fail-open hook. It must **skip, not pass**, on a host without `bwrap`
  (`DEC-212`) — a vacuous pass is the same fail-open shape the slice deletes.
- **By agent** — a live dispatch phase driven end-to-end on the claude harness
  through the subprocess arm, concluding with the orchestrator's **incumbent
  import** of the worker's working-tree diff (`DEC-213`; self-commit belongs to
  `SL-255`). **Evidence lands in an authored sink** — `notes.md` or an `EVD` —
  never the gitignored scratchpad (`mem_019fd1d862887d42b7a1f88c28fd28a7`).
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
- **At reconcile — sweep the memory corpus for deleted claude-arm mechanisms.**
  Surfaced at design (`explore.memory`), not at scoping. At least 25 memories
  describe mechanisms this slice removes — `SubagentStart` stamping, `PreToolUse`
  jail behaviour, `WorktreeCreate` provisioning, `worker_commit` resolution,
  marker identity — several at `high` trust and `high` severity. Deleting the
  mechanism converts them to **stale-but-plausible**, which is worse than wrong:
  an agent retrieving them would act on a mechanism that no longer exists.
  `mem.signpost.doctrine.dispatch-claude-arm-wrong-base` is indexed in the boot
  snapshot, so the staleness reaches every session. Route through
  `/reviewing-memory`; retire or re-anchor rather than editing in place where the
  memory's whole subject is gone. Triage detail in `notes.md`.
