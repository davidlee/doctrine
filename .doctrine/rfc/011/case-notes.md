
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
