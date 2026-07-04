
[reconcile→close; SL-198-recon-c63c1397]
Coord/primary state-split cost the reconcile stage real tokens. Slice lifecycle
status is AUTHORED (slice-198.toml), yet the dispatch-executed+audited slice
advanced its status only in the coord tree's copy; the primary tree sat stale at
`ready`, so /reconcile had to walk ready→started→audit→reconcile by hand before
the closure seam would accept the reconcile transition (audit→reconcile). Same
class as the phase-completion split hit last session. Signpost: a dispatched
slice's authored lifecycle status + phase completion both need explicit
primary-tree catch-up before audit/reconcile/close verbs behave. A
`dispatch sync --lifecycle` (fold coord-tree authored status/phase flips back to
primary) would erase this recurring hand-walk. Also: candidate `admit` needs a
`candidate create`-recorded candidates.toml row — a hand-built merge+branch -f
(as done for the review surface last session) leaves `candidates(none recorded)`,
so admit refuses on provenance; close must run a fresh `candidate create`.

[/plan; SL-199-prelock-review-a1]
Pre-lock hostile review (codex GPT-5.5) over the transactional surface cost real
tokens on two verification detours the plan text should have pre-empted:
- Judging finding severity required disambiguating a two-tier storage subtlety the
  plan/design never state outright: boundaries.toml is COMMITTED on the coord branch
  (`dispatch/<NNN>`) yet GITIGNORED on trunk (.gitignore `.doctrine/dispatch/`), so
  "atomic commit" residue is poison, not disposable runtime. Had to read
  run_record_boundary + .gitignore + storage model to rank it. Plan naming the
  storage tier of each committed output up front would save this.
- The plan named the WRONG reuse seam for the coord resolver (worktree_for_ref /
  live_worktree_for_ref — single-hit first-match) vs its own design's enumerate
  intent (list_worktrees). Verifying the mismatch cost a source read. A plan that
  cites the exact fn it reuses (and its arity/return) is cheaper to review.
- Net: both "blocker" findings collapsed to ONE root cause (server commit must be
  working-tree-free via commit_tree/scratch-index, which the repo already has) once
  the seams were read. The design asserting the mechanism (not just "commits
  server-side") would have pre-answered the reviewer.

[doctor→fix; SL-198-agent-conformance-off-by-one-c3e1b]
New AgentConformance check (SL-198) silently invisible in rendered doctor
output: render_findings had a hardcoded 8-bucket array for 9 categories,
dropping ordinal-8 findings. The error exit happened but the human-readable
output never showed the category. Root cause: ordinal-to-bucket mapping was
manually synced with no compile-time tie to CATEGORIES_BY_ORDINAL length.
Suggestion: the array size should be const-generic off CATEGORIES_BY_ORDINAL
or a BTreeMap to eliminate the mapping error class entirely.

[dispatch; SL-199-armswitch-a7]
FINAL ROOT CAUSE (both earlier conclusions in this entry were WRONG — see the
correction trail as a cautionary tale): the orchestrator (me) spawned the two real
`dispatch-worker` Agent calls WITHOUT the `isolation: worktree` parameter. Without
it, the Agent tool runs the subagent IN-PLACE in the orchestrator's current
worktree (the coord tree) — the `WorktreeCreate` hook never fires, no fork is
minted, and the doctrine pretooluse confinement jails the subagent to its cwd (the
arming dir) with src/ read-only → base-guard `src-readonly`, no delta. NOTHING was
actually broken: the hook fires, the binary forks, the jail is fine, CC 2.1.198 is
fine, the dispatch-worker agent-def is fine. Proof: three control spawns that DID
pass `isolation: worktree` ALL forked correctly to `.worktrees/agent-<id>` at
HEAD==B on `dispatch/<name>` — general-purpose from the primary root (Passthrough,
detached at primary HEAD), general-purpose from the arming dir (Fork @ B), AND
dispatch-worker from the arming dir with a trivial prompt (Fork @ B). The isolated
variable was never agent-type/model/hook/jail — it was the missing flag. The
dispatch-agent SKILL's spawn template DOES list `isolation: worktree`; I dropped it.
COST: ~5 wasted worker/probe spawns + a very long multi-hypothesis diagnosis
(claude-arm-unavailable → stale-hooks → jail → enabledPlugins → agent-def model
override → agent-type) + TWO wrong case-note conclusions before the real cause. The
whole detour is avoidable with ONE cheap invariant BEFORE deep diagnosis: on any
"worker landed in the coord tree / src read-only" symptom, FIRST re-read the Agent
call and confirm `isolation: worktree` is present — it is the single necessary
condition for the fork. A worker in the coord tree almost always means the flag was
omitted, not that the hook is broken. Suggestion: `dispatch arm-spawn` (or a
pre-spawn lint) could emit a reminder that the very next Agent spawn MUST carry
`isolation: worktree`, since the arming ritual is inert without it.

---

[dispatch; SL-199-phase01-b1]

PHASE-01 worker (claude arm, `isolation: worktree` present) forked cleanly at
B and produced a correct single-file delta (`src/worktree/create.rs`; focused
suite 61 pass, clippy clean). But its `worker_commit` self-commit REFUSED with
`commit-gate-red`: the gate runs the FULL `check commit` e2e suite from INSIDE
the worker-marked fork (`.doctrine/state/dispatch/worker` present), and three
non-hermetic ADR goldens (`adr_status_*` in `tests/e2e_adr_cli_golden.rs`)
invoke `doctrine adr status` — an authored write the worker-write guard
(`src/commands/guard.rs`) correctly refuses under the marker
(`worker fork (signal: marker): refusing authored write`). So `worker_commit`'s
own gate is self-incompatible with the marker it sets: the confined self-commit
path cannot pass a gate that includes marker-hostile authored-write goldens.

IMPACT: the claude-arm self-commit path (the newer mechanic) false-negatives;
no commit lands. RECOVERY was cheap and correct: the orchestrator fell back to
the documented `worktree import --from-worktree <fork>` claude-arm path (help
string: "since ro-.git blocks the worker's self-commit and the tree persists
post-return"), scope-belted with `--slice 199` (create.rs is a design-target),
then ran the AUTHORITATIVE `check regression diff --base B` verify in the COORD
tree — which has NO worker marker, so those same ADR goldens PASS there. Worker
self-reports are advisory; the funnel verify beat is the honest gate. Net: the
false-negative cost one import-fallback decision, not a phase.

ROOT-FIX (user-flagged, deferred to a backlog item, not this phase): the
`adr_status_*` e2e goldens should skip under `DOCTRINE_WORKER=1` (they exercise
authored writes that are legitimately guard-refused inside a worker fork). More
generally: `worker_commit`'s gate should either run marker-aware (exclude
authored-write goldens) or the goldens should be hermetic to the marker — else
every claude-arm self-commit trips the same wire.

---

[dispatch; SL-199-phase01-vt5-b73bbb93]

INCIDENT: PHASE-01's design (EX-3 one-shot consume: the create-fork Fork arm
unlinks the arming `base` slot BEFORE fork_core) had a knock-on the plan never
flagged — it broke a PRE-EXISTING e2e test in a DIFFERENT file
(tests/e2e_worktree_create_fork.rs::name_collision_refuses_on_both_arms, VT-5).
That test armed ONCE then Forked TWICE from the same arming dir, asserting the
2nd Fork hits fork_core's `fork-refused`. Under one-shot consume the 2nd Fork
finds base=None → positional-¬base → `missing-base`, never reaching fork_core.

WHY IT SURFACED LATE: the worker was scoped to a SINGLE file (src/worktree/
create.rs, its lone design-target). It could not touch the e2e test — correctly.
So the phase's own delta broke a test the worker was structurally unable to fix.
The funnel VERIFY beat (`check regression diff --base B`) caught it as
`new: unknown::name_collision_refuses_on_both_arms` — a HALT. The gate worked:
the worker's advisory self-report (61 unit tests green) said nothing about the
cross-file e2e suite; only the coord-tree regression diff (markerless, full
suite) is honest.

RESOLUTION (no orchestrator-inline edit, no fresh spawn): widened scope by one
file + RESUMED the existing worker via SendMessage (agentId == fork suffix,
retains full phase context, ~100k tok / 36s vs a cold re-spawn). Worker re-armed
between the two Fork probes (one arm per Fork under one-shot), suite 6/6. Re-
imported the fork's whole working-tree delta via `--from-worktree`.

BELT WRINKLE: the import scope belt (`--slice N`) keys on DESIGN-TARGET selectors
ONLY (`import --help`: "design-target selectors scope the import belt";
scope-relevant does NOT count, and neither does conformance). A consequential
TEST file broken by a phase is not a design target in the intuitive sense, yet
must be a design-target selector to (a) pass `--slice` import and (b) be
conformant at audit (`slice conformance` checks touched [B..S] paths against
design-target selectors → undeclared otherwise). Two shapes work: declare the
test as design-target FIRST then `import --slice`, or omit `--slice` +
manually verify the exact delta + declare-after. Took the latter (selector
didn't exist in coord at import time), then declared it → conformance
undeclared(0). LESSON: when a phase touches a pre-existing test, add that test
as a design-target selector up front in the plan's selector set — every phase
that edits tests hits this.

[quick-fix; SL-201-map-focus-2]
Two runtime bugs behind SL-201's onboard/memory-title feature, both invisible to
the unit suite because the coupling lived across the SVG DOM boundary:
- Graph click/focus nav extracted node ids from the *visible* `<text>` label, not
  the `<title>` (DOT node name). Fine while label==id (numbered entities); broke the
  moment memory labels became human titles. Cost: had to render a live DOT→SVG to
  discover Graphviz emits both `<title>`(id) and `<text>`(label). No test could have
  caught it — the id-extraction closure was inline in render.ts, untested.
- Onboard deep-link redirected to default: boot guard keyed off `state.focusId`
  (always null at boot) instead of whether the URL hash carried a focus. Latent for
  all deep-links; SL-201 was the first feature to load a focus from the URL.
Incidental friction: `localhost` vs `127.0.0.1` flaky under the jail; `pkill` absent
from nix PATH; map serve has no `--no-open`/`--port` on the `onboard` subcommand.

[/design; SL-197-cpt-kind]
`mem.pattern.doctrine.record-kind-touch-sites` (staleness: unknown) was written
at the 4-kind era (RECORD = ASM/DEC/QUE/CON). EVD/HYP + several canaries have
landed since, so its "~17 silent-drift sites" alarm is materially outdated —
most sites are now compiler-forced or canaried. Cost ~4 verification greps to
re-establish current guard state before trusting the memory for design. A
`verify`/re-anchor pass on that memory would have saved the round-trip. The
memory's core value (the scatter EXISTS, treat as grep-task) held; its severity
(silent) did not. Suggest: memories asserting "N unguarded sites" should carry a
verification anchor that re-checks on retrieval, or a staleness ping when the
cited consts change.

[backlog / rsk-227-fork; sess-intra-tier]
Forked backlog-authoring to a background worker (/fork). Worker was under
worktree-jail — every Bash on the primary tree (`echo`/`git`/`doctrine`)
returns `worktree-jail: cwd-not-a-worturtree`, and there is no backlog MCP verb
(only memory/review/worker_commit). Authoring a backlog entity is a primary-tree
corpus write reachable ONLY via `doctrine backlog new` (Bash), so a jailed fork
structurally cannot land it. ~77k subagent tokens spent to return a draft the
parent then re-authored + landed itself. Net: the fork produced a good draft but
zero direct writes; the delegation was pure overhead vs authoring inline.
Signal: forking corpus-write work to a jailed worker is a false economy — route
corpus writes to the unjailed parent, reserve forks for read/analysis.
