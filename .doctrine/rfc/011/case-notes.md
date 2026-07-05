
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

[dispatch/F2F3; sess-7992f474]
F2/F3 (worker_commit gate + run_gc stdout) resolution burned tokens on three avoidable confusions:

1. **cargo binary-level fail-fast masked the true blast radius.** The worker_commit
   gate runs `cargo test` (default fail-fast: stops at the FIRST failing test BINARY).
   So the operator/agent only ever saw ONE failing binary at a time; fixing it revealed
   the next. This produced a false "backlog_filter_alias is the sole blocker" narrative
   across sessions and drove ~3 wrong per-test fixes before `--no-fail-fast` exposed 91
   victims across ~30 binaries. LESSON: diagnose gate-suite failures with
   `--no-fail-fast` FIRST; never trust a single-binary failure as the whole story.

2. **A stale code comment sent the diagnosis down the wrong leg.** worker_commit.rs and
   the marker docs assert the guard "trips on is_linked_worktree alone." The actual code
   (marker.rs describe_mode) requires `is_linked && marker_present` OR env. Trusting the
   comment, I chased the env leg (.env_remove) — which the REAL marked fork ignores.
   Only a direct guard probe (marked vs unmarked fork, env on/off) revealed the truth.
   LESSON: for guard/refusal mechanics, PROBE the binary in the actual condition; a
   prose comment about a boolean predicate is not ground truth.

3. **The primary-tree sweep didn't reproduce the fork condition.** Running the suite
   under `DOCTRINE_WORKER=1` on the (unmarked) primary tree exercised only the env leg,
   not the marker leg of a real (marked) fork. False confidence that env_remove fixed it.
   LESSON: reproduce in the actual isolation posture (a marked linked worktree), not a
   convenient proxy.

Root causes were both design-shaped, not one-liners: (F2) the gate ran the full authored-
write suite inside a MARKED fork → marker leg refused ~91 fixtures; fix = clear the marker
for the trusted gate window + drop the env export. (F3) run_gc prints to fd1 = MCP JSON-RPC
channel; fix = run_gc_to sink seam. The "operator one-liner" framing in the prior finding
was an artifact of the fail-fast illusion.

[code-review; F-1-dispatch-integration]
Token-inefficiency: review required reading the full `git diff 97fa0e96..HEAD`
(12 files, ~400 lines of diff) to discover the dispatch integration failure.
No automated signal existed — the RV-249 audit ran against the wrong base
and produced a clean reconciliation. A pre-close gate that checks "are there
unmerged dispatch branches?" would have caught this at 2% of the token spend.
The review then had to reconstruct the git topology (branch --graph, merge-base,
--contains checks) before it could raise F-1 — all mechanical work that a
pre-flight dispatch-integrity check would eliminate.

[code-review; F-2-derive-drift]
Minor: had to cross-reference 7 struct definitions across knowledge.rs to
spot the inconsistent derive set. A derive-consistency lint on a closed enum's
variant structs would catch this mechanically.

[design; sl199-p05-delta-d42a1734]
Delta on a LOCKED monolithic design doc (design.md, 465 lines) to amend ~6
spots cost a full re-read of the doc + two dense state briefs (~1000 lines
combined) before a single edit. Inherent to a delta on a dense locked artifact,
but: a section-addressable design (edit §5.D without loading §5.A/§5.B) would
cut the read tax. Second: the F7→refutation that triggered this consult burned a
whole cycle because a scaffold confound (orchestrator LLM omitting the per-call
isolation:worktree arg) masqueraded as a harness ceiling ("2.1.198 doesn't honor
nested isolation"). A cheap discriminator probe (frontmatter-isolation vs
call-param) up front would have localised it in one step instead of a
multi-finding (F7→F9→F10→F11→F12) unwind. Lesson logged as the fix itself:
deterministic config rides the frontmatter/tool surface, never a per-call LLM arg.
[walkthrough; fable-wt-83ce40e3]
Inconsistent id parsing across kinds cost a retry: `rfc show 014` and
`slice paths 199` accept bare numbers, `spec show 021` refuses ("expected
PRD-NNN or SPEC-NNN") because two prefixes share the kind. Wasted one
round-trip + error output. Either accept bare NNN when unambiguous or
document the asymmetry in --help.

[dispatch; sl199-vh1-witness]
VH-1 live witness (confined dispatch-orchestrator, SL-205 smoke slice) landed
green first attempt once the harness was wired. Cost was NOT the run (~3 min,
31k orchestrator tokens) but the BRING-UP: (1) F8 red-herring diagnosis consumed
a prior session; (2) no pre-canned witness recipe — had to derive the smoke-slice
victim + coord-tree + DOCTRINE_BIN-repointed hooks from first principles; (3)
authoring a throwaway slice (new→plan→selector→phases→status ladder) + edge→main
promote just to give the orchestrator a phase to drive. A shipped `dispatch
smoke-witness` fixture (ephemeral slice + one text-delta phase, self-tearing-down)
would collapse ~15 setup steps to one and make VH-style e2e witnesses cheap and
repeatable. Also: ~29 stale `dispatch/agent-*` debris branches from prior probe
runs clutter the coord tree — a `dispatch reap --orphans` sweep is owed.

[dispatch/conclude; sl199-conclude]
Two conclude-cadence frictions hit while concluding SL-199 post-VH-1:

1. PRIMARY/COORD phase-status split-brain (idea/028). All 5 phase flips landed
   in the COORD tree runtime during the drive; primary tree showed 0/5. prepare-
   review's completeness gate reads the completed-set from the PRIMARY tree
   (dispatch.rs:1873/1901, registry_completeness(&primary,&primary)) → every
   committed ledger row read as `Extra` ("recorded row for PHASE-01, which is not
   a completed phase"). The documented fix verb `slice reconcile-phases` REFUSES
   while a live dispatch/<slice> coord tree exists — but the cadence keeps the
   coord tree until AFTER prepare-review. Mutually inconsistent for an inline-
   executed slice. Resolved by manually flipping the 5 primary phase sheets to
   `completed` (runtime, disposable) with the coord tree still present, then
   prepare-review passed. Tooling gap: the funnel record beat pushes the registry
   to primary but not the phase-status sheets.

2. verify-vt run order + tree. Cadence says run `slice verify-vt` IN THE COORD
   TREE before removal. If run on the PRIMARY tree (on edge, delta not yet
   integrated) it FAILs with misleading "keyword absent"/"test_file not found" —
   the gate scans the working tree, and edge has none of the SL-199 delta. Had to
   spin a throwaway `git worktree add --detach dispatch/199` to validate the gate
   green. Cadence should either run verify-vt before coord removal (enforced
   order) or verify-vt should default --path to a delta-bearing ref, not the CWD
   tree, so a post-removal run doesn't read as a false FAIL.

[dispatch/PHASE-04; sess-7992f474]
worker_commit self-commit STILL blocked on the claude arm after F2 — the gap is
PROVISIONING, not the gate. F2 (5a722834) fixed the commit-gate's marker leg, but
the Agent-tool `isolation: worktree` spawn yields a **plain detached** worktree:
detached HEAD at base, no `dispatch/<name>` branch, no DispatchRecord. worker_commit's
`resolve_agent` keys on a worktree checked out at `refs/heads/dispatch/<name>` →
`worktree_hits == 0` → Refused{unknown-agent}. So R1 is NOT dissolved by F2 alone:
claude-arm self-commit needs create-fork provisioning (branch + DispatchRecord)
to fire on the Agent spawn — a PHASE-05 wiring concern. PHASE-04 landed via the
proven `--from-worktree` import path (as P01–03 did).

Corollary token-sink: `worktree gc --fork <branch>` cannot reap a `--from-worktree`
worktree — it's branchless/commitless (delta was uncommitted, gathered from the live
tree), and the landed-oracle needs a fork branch to prove landing. Reap of the
Agent-isolation worktree falls to `git worktree remove --force` (delta provably
landed in the funnel commit). The funnel's "reap via run_gc" step assumes a committed
fork (pi arm); the claude `--from-worktree` arm has no branch to gc.

[dispatch/PHASE-05; sess-b73bbb93]
R1 RECONCILIATION HIT — live create-fork provisioning does not fire on the
main-thread claude arm, even WITH arming.

Setup: cwd parked at coord-root (.dispatch/SL-199), branch dispatch/199, HEAD=B
(a2623c2f). Ran `dispatch arm-spawn --path .` (base file written = a2623c2f),
then Agent isolation:worktree (subagent_type dispatch-worker). All §5.E Fork
preconditions held: coord_root ∧ dispatch-branch ∧ base-armed.

Result: worker forked DETACHED at a2623c2f, worktreeBranch=undefined, no
dispatch/<name> branch, no jail record → worker_commit refused {unknown-agent}.
Identical to §6 probe's positional-arming failure that §5.A's coord-root-cwd
discriminator was meant to fix. Consistent across P01-P05 (all used --from-worktree).

Hook IS registered: plugin hooks.json wires WorktreeCreate → `doctrine worktree
create-fork` (session settings hooks:{} is only the empty user layer). So the
discriminator runs; it took Passthrough. Candidate root causes (untested):
 (a) WorktreeCreate payload cwd ≠ coord-root (harness passes the NEW worktree
     path or session root) → discriminator sees ¬coord_root → Passthrough.
 (b) The base file arm-spawn --path . wrote is not where create-fork reads it.
 (c) Claude Code's WorktreeCreate hook cannot CREATE+checkout a branch (only
     veto) — the mechanics:154-166 "payload lacks enough to act on" caveat; then
     EX-3-as-designed (self-commit via provisioned fork) is harness-infeasible.

REFRAME: my test was the MAIN-THREAD arm (mechanics:39-42 already says the claude
Agent tool gives no base control here). EX-3 is the CONFINED-ORCHESTRATOR arm —
UNTESTED this session. The §6 probe found the confined nested spawn forked at the
armed base (cwd behaves differently for confined subagents). So EX-3's own live
experiment (nested dispatch-orchestrator subagent arms+spawns) is still owed and
is exactly VH-1. Token cost: the branchless-worktree reap + selector re-declare +
baseline re-capture recur every phase on this arm.

[phase-plan; sl199-p05-relock-rewrite]
Stale-sheet read-tax. phase-05.md existed (8.4K, prior agent) but predated the
2026-07-05 re-lock: declared "NO Rust", carried F7 (loop-infeasible) — both
refuted by the re-lock. phase-plan had to detect staleness (cross-read plan.toml
PHASE-05 vs the sheet), then rewrite wholesale rather than refine. Cost: full sheet
re-read + full rewrite. Root: a runtime sheet is not invalidated/flagged when the
authored plan it expands is re-locked underneath it — no "sheet stale vs plan"
signal. The handover DID carry the six re-locked work items, which is what made the
staleness detectable cheaply; without it the drift would have been silent.

[dispatch-agent; sl199-p05-mechanic-3way]
Pre-spawn, hit a 3-way disagreement on the claude-arm return mechanic that cost a
ground-truth probe before I could write the worker prompt:
- CLAUDE.md orchestration note: "claude arm self-commits via worker_commit MCP".
- dispatch-mechanics.md §"Two ways a worker returns": claude=worker_commit→import
  --fork; pi=--from-worktree. (STALE — this is what T5/EX-2 must rewrite.)
- dispatch-agent SKILL + `worktree import --help`: claude=--from-worktree (labeled
  "claude arm, SL-182 PHASE-05"); pi/subprocess=--fork. ← current CODE truth.
Resolution: code+skill win. Doc lags one refactor (the arms SWAPPED which import
source they use). Token cost: ~2 probes to disambiguate what a single authoritative
doc should have stated. Root cause = mechanics doc not updated when SL-182 P05
flipped the claude arm to live-worktree import; SL-199 T5 inherits the debt.

[dispatch-agent; sl199-p05-tree-unclean-selfinflict]
import --from-worktree refused `tree-unclean`: I'd appended RFC-011 case-notes to
the coord tree's TRACKED .doctrine/rfc/011/case-notes.md BETWEEN arm-spawn and
import. The import belt requires a clean coord tree at B. Committing the notes
would move HEAD off B (import needs HEAD==B AND computes the worker delta vs B, so
a moved base would try to REVERT the notes). Fix: set the file aside
(`git show HEAD:path > path`, backup first — no checkout/restore verb), import,
then re-apply + commit as a SEPARATE instrumentation commit at the clean post-import
tip. Lesson: never leave a tracked coord file dirty across an arm->import window;
hold case-notes in a scratchpad log during a batch, or commit them at a clean
boundary — don't append into the live coord tree mid-funnel. Token/step cost: one
failed import + a 4-step non-destructive set-aside dance.
