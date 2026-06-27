# RFC-011 case notes — token-inefficiency & incidental complexity

Running log. Each entry: context · friction · root cause · token cost (rough).
Captured during informal subagent-orchestration of SL-166 PHASE-04/05
(orchestrator = main agent, workers = subagents in the shared worktree).

## 2026-06-27 — SL-166 orchestration session (orientation)

- **Runtime phase-state is per-worktree, but handover asserted "PRIMARY-rooted".**
  Handover (SL-166) said lifecycle/registry verbs "resolve to the PRIMARY
  registry" even from the fork. False for phase status: `.doctrine/state/` is
  gitignored and per-worktree, so the worktree's PHASE-03=`completed` flip never
  reached primary (primary still showed `planned`). Cost: ~2 extra tool calls to
  reconcile primary-vs-worktree state before trusting either. Root cause:
  handover conflated authored-registry writes (record-delta → committed TOML)
  with gitignored runtime state. Both are "doctrine verbs" but route to different
  tiers. A worker onboarding cold would mis-target lifecycle flips.

- **CLI command-shape guesses cost round-trips.** `doctrine paths SL-166`
  (suggested in boot.md "useful commands") → `unrecognized subcommand 'paths'`.
  `doctrine slice status SL-166` → wants `<ID> <STATE>` (it's a setter, not a
  reader) AND a numeric id (`SL-166` → "invalid digit"). `doctrine status 166`
  → "unexpected argument". Three failed invocations before finding phase status
  via raw `grep` of the runtime toml. Root cause: id-form inconsistency (some
  verbs take `166`, prose/commits take `SL-166`) + reader/writer overload on
  `status` + a stale "useful commands" hint in boot.md. Each miss = one wasted
  call + its error payload in context.

- **Handover is large (105 lines) and duplicated across two copies** (primary
  `.doctrine/slice/166/handover.md` stale @ PHASE-03, worktree copy fresh @
  PHASE-04). A `diff` was needed to discover which was current. Cost: ~1 large
  read + 1 diff. Root cause: handover.md committed into the tree (so it forks
  with the branch) while also being a per-phase mutable doc — two live copies,
  no freshness marker except mtime.

## PHASE-04 (g1) — worker friction (subagent report + orchestrator)

- **Bin-crate test invisibility.** `cargo test --lib` produced NO output: the
  dispatch/corpus_guard code lives in the BIN crate (`src/main.rs`), while the
  workspace also has a lib crate (`cordage`). A worker ran `--lib`, saw nothing,
  briefly assumed its tests didn't exist. Correct form: `cargo test --bin doctrine
  <filter>`. Root cause: mixed bin+lib workspace with the primary logic in the bin
  — non-obvious to a cold worker. Cost: ~1 confused iteration. Candidate fix: a
  `just test` recipe that targets the right crate, documented in AGENTS.md.
- **Self-referential audit test.** The VA-1 verb-set audit test does
  `include_str!("dispatch.rs")` and counts call sites — it miscounted (7 vs 1)
  because the test's OWN assertions name the guarded symbol. Worker had to scope
  the scan to `split("#[cfg(test)]").next()`. Inherent to grep-the-source-as-a-test
  patterns; ~1 extra iteration. Not a doctrine/dispatch issue — a test-design
  hazard worth a memory.
- **Orchestrator overhead (positive note).** Delegation was clean: the worker's
  single final message (commits + coverage map + deviations + gate tail + friction)
  was enough to verify-and-flip without re-reading its diffs. The PHASE-02/03 notes'
  layering-split guidance transferred accurately, so the worker wired g1 directly
  with no design back-and-forth. Main-thread cost to verify: 2 calls (gate + grep)
  + the lifecycle flip. This is the efficient path; the expensive parts were all
  in orientation (above), not execution.

## Cross-phase: warm-agent reuse unavailable

- The Agent tool docs advertise `SendMessage` to continue a prior subagent with
  its context intact. For PHASE-05 (same terrain as PHASE-04: doctrine.toml load,
  `--allow-corpus-clobber` arg, dispatch verbs) reusing the warm PHASE-04 worker
  would have skipped a full re-orientation (~tens of k tokens). But no
  `SendMessage` tool was exposed/loadable in this harness — `ToolSearch` found
  none. Fallback: spawn fresh + hand-carry the prior worker's file:line findings
  into the new prompt. Net: re-orientation partly avoided by manual context
  transplant, but the clean "resume the warm agent" path was not available. Root
  cause: tool surface mismatch between the Agent tool's described capability and
  the loadable tool set. Cost: orchestrator must curate a context packet by hand
  per phase instead of one cheap continue-call.

## PHASE-05 (enable+parity+docs) — worker friction + a governance coherence gap

- **STALE DESIGN PREMISE (highest-cost finding).** PHASE-05 EX-1, design §5.3, and
  the slice all specify enablement as "set `authoring-branch` in `doctrine.toml`
  in a dedicated *commit*." Reality: SL-146/ISS-055 (`a0acf0eb`, merged the SAME
  DAY the SL-166 design was written) moved config to `.doctrine/doctrine.toml`,
  which is **gitignored and never tracked** (the `.doctrine/*` ignore + whitelist
  excludes `doctrine.toml`; repo-root `doctrine.toml` is also ignored at
  `.gitignore:11`). So a "dedicated enabling commit" is **impossible as written** —
  config is deliberately environment-local now. The design carried a pre-SL-146
  mental model. Worker cost: the single most expensive investigation of the phase
  (git-log both paths, gitignore whitelist analysis, dtoml.rs resolution, tracing
  to the main worktree's live config because the worktree has none). Root cause:
  two same-day slices with a contract overlap; the later design didn't reconcile
  against the just-merged config relocation. This is the canonical RFC-011 shape:
  a worker burns a large fraction of its budget reconciling a doc against a moved
  target rather than doing the work. **Mitigation candidate:** design lock should
  re-grep the touched subsystem's constants/paths at lock time, not author-time.
- **Phantom test target.** Plan EX-2 + design §9 name `e2e_dispatch_close`; no such
  target exists (close-integration tests live in `e2e_dispatch_sync` +
  `e2e_dispatch_lifecycle`). Worker grepped to locate. And `e2e_dispatch_lifecycle`
  is itself one of the foreign SL-165-dirty files the worker was told not to touch —
  runnable but a confusing overlap. Cost: ~1 grep + momentary "am I allowed to run
  this?" Root cause: criteria named a test target that was never created / renamed.
- **Config absent from the worktree entirely.** "First inspect the current
  doctrine.toml" returned nothing — the gitignored config doesn't exist in a fresh
  worktree, only in the main worktree. A worker reasoning purely from its own
  worktree cannot observe posture state at all. Root cause: env-local config +
  worktree isolation interact badly for any phase whose criteria reference live
  config. Cost: extra hop to the main worktree.

[SL-166 P3-5 C drive complete @ 148k]


[audit; SL-166-rv180]
Token-inefficiency / incidental complexity during /audit of SL-166 (RV-180):

1. CLI id-form inconsistency wasted a call: `slice conformance SL-166` errors
   "invalid digit found in string" (wants bare `166`), but `slice show SL-166`
   accepts the prefixed canonical id. Same session, adjacent verbs, opposite
   id-form rules. The boot guardrail says "cite the durable prefixed id
   everywhere" — but half the CLI surface rejects it. Had to retry with `166`.

2. No read-only "current lifecycle state" affordance on `slice status`: the verb
   is transition-only (`slice status <ID> <STATE>` required), so `slice status
   166` errors on a missing positional instead of printing state. Discovering the
   legal states + current state needed a `--help` round-trip.

3. Closure-seam two-hop surprise: handover said "parked for /audit", but the
   slice was in `started`, and `slice status 166 reconcile` refused
   ("reachable only across the closure seam → reconcile from audit"). Had to
   flip started→audit→reconcile as two explicit hops. The /audit skill never
   states it must first advance started→audit; an agent reading only the skill
   would not know the lifecycle hop is its job.

4. review-ledger.md path not resolvable from the advertised skill base dir: the
   /audit skill says "read review-ledger.md" and the skill base was
   `/home/david/doctrine-edge/doctrine/skills/audit`, but the file is not there —
   had to `find /` to locate it (`install/review-ledger.md` + the installed
   `.doctrine/review-ledger.md`). A relative pointer that doesn't resolve costs a
   search.

5. Direct tension between two authoritative instructions: the prior agent's
   handover said "run /audit from the worktree so it reads the fresh handover",
   but review-ledger.md §6 says "review verbs refuse a worktree/fork-resolved
   root — drive from the parent tree." Resolved by reading the handover content
   once (cheap) and driving all ledger verbs from primary — but reconciling the
   contradiction cost reasoning tokens. The handover's "run from worktree"
   premise was simply wrong for the ledger half of the work.

6. Pre-land audit friction: `slice conformance` returns "incomplete" because the
   fork is unlanded and source-deltas bind at land. So the one mechanical
   drift-signal the audit skill leans on (undeclared/undelivered/conformant
   algebra) is unavailable for the whole pre-land audit window — the audit runs
   blind on path-conformance and must defer it to a close-time re-run. The
   audit→land ordering means the strongest mechanical signal arrives after the
   audit verdict, not before it.

[SL-166-rv180 audit complete @ 115k]


## [/reconcile; SL-166-recon-a]

Reconcile pass was clean — brief was fully structured (3 per-slice items, REV:None
explicit), so zero re-derivation. Token cost ~all in reading the RV ledger
(review_show returns the full 9-finding payload + brief + synthesis in one block —
efficient) and the two target files.

One incidental friction: the Read tool renders TOML escaped quotes (`\"`) WITHOUT
the backslash, but the Edit tool matches raw file bytes (which DO contain `\"`).
Editing an `EX-1` criterion line containing `\"refs/heads/edge\"` forced a
defensive `sed | cat -A` re-read to confirm exact escaping before Edit would
match. Minor (~1 extra tool call) but recurs on any TOML string-with-quotes edit.
Mitigation a worker can't fix; flagging as a harness/Read-vs-Edit fidelity gap.

No other inefficiency — the audit→reconcile seam held: discovery was complete,
write surface was unambiguous, no /consult needed.

## [/close; SL-166-close-a]

Close pre-checks surfaced real token cost in one place: the per-phase completion
state. `slice list` showed 2/5 because phases 03/04/05 were executed+completed in
the FORK worktree's own gitignored runtime state, which never propagates to the
primary tree. Reconstructing this took ~4 investigative tool calls (slice list →
phase sheets → per-phase status grep) before the picture was clear. The fork-solo
execution model leaves the primary's runtime phase rollup stale by design, but the
close skill's "confirm X/X complete" pre-check gives no hint that a stale sub-X/X
rollup is the EXPECTED pre-land state vs. genuinely-incomplete work — an agent must
already know the fork-runtime split to avoid mis-reading 2/5 as dropped phases.

Second: `slice status <ID>` has no bare query form (it is a setter requiring
<STATE>), so there is no cheap "what lifecycle state + phase rollup" one-liner;
status comes via `slice list | grep`. Minor friction, recurs every close.

## [/close; SL-166-close-b]

Detour during close: an orphan case-note stub `[SL-166 audit - ]` (an empty
entry header, no body — present in the working tree from session start, then
re-appeared after I committed) made the tree unclean, so `worktree land`
fail-closed with `land-refused: tree-unclean`. Cost: a diff → diagnose →
complete-the-stub → commit cycle (~3 tool calls) before the land could proceed.

Two compounding factors worth flagging for RFC-011:
1. The instrumentation directive itself (append a `[skill; id]` header per skill
   use) is prone to leaving half-written empty headers if an entry is started
   but not filled — and any such fragment then blocks the land gate. The
   instrumentation tax can directly obstruct the workflow it instruments.
2. `worktree land`'s clean-tree precondition is whole-tree, so an unrelated
   dirty file (instrumentation notes, another agent's fmt reflow) blocks a
   land that touches entirely disjoint paths. Correct fail-closed posture, but
   the coupling means shared-tree noise serializes landings.

Minor git ergonomics (not doctrine): `git rev-parse --short <A> <B>` with two
args errors `Needed a single revision` even when both resolve individually —
cost one redundant re-run. Harness/git, not a doctrine surface.

[SL-155-close reconcile -> close agent @ 141k]
