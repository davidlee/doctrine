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

[/audit SL-163; audit-sl163 @ ~60k]
Two friction sources, both cost retries / a user round-trip:

1. CLI id-form inconsistency. `doctrine slice conformance SL-163` (prefixed,
   the form boot/AGENTS mandate everywhere in prose/commits) errors `invalid
   digit found in string` — the verb wants a BARE `163`. Same for `slice
   status`/`record-delta`. But `review new --target` wants prefixed `SL-163`.
   An agent fresh off the "always cite the prefixed canonical id" rule pays 2
   failed calls discovering which verbs are bare-only. The error is also
   opaque (a parse-int failure, not "expected bare slice number"). Cost: ~2
   redundant invocations each time a lifecycle/slice verb is first reached.

2. Review-on-worktree refusal has no upstream signpost. The /audit skill body
   never warns that review verbs refuse a worktree fork (IMP-024) — that
   constraint lives only in review-ledger.md §6 (parent-tree caveat), read
   late. When a slice was developed in an isolated worktree (the normal case
   for code slices), the audit cannot create its RV there; the slice code +
   lifecycle status live only on the branch while review state must live on
   the parent tree (edge). Resolving the split (promote branch -> edge so the
   parent tree carries code + `audit` status before the RV opens) required a
   user consult — the skill offers no recipe for "slice was built in a
   worktree, now audit it." A one-line pointer in /audit ("if the slice lives
   on a branch/worktree, land it on the parent tree before opening the RV —
   review verbs refuse forks, IMP-024") would have saved the whole detour.

[SL-163 ? -> audit @ 140k]


[/reconcile + /close SL-163; reconcile-close-sl163 @ ~70k]
Three friction sources across the reconcile→close pass:

1. Stale boot signpost: `doctrine paths SL-163` (listed verbatim under boot's
   "useful commands" as `doctrine paths SL-123`) errors `unrecognized
   subcommand 'paths'`. No such verb. First instinct for "list slice files"
   wastes a call; recovered via `find .doctrine -name '*163*'`. Either the
   snapshot recipe is stale or the verb was renamed/removed without updating
   the signpost agents are told to trust.

2. id-form rejection AGAIN at close (dup of IMP-189 / prior audit note):
   `slice phases SL-163` and `slice status SL-163` → `invalid digit found in
   string`. Recurs every lifecycle verb first-reach; the prior audit note
   already flagged it, this pass re-paid ~1 call. Confirms the friction is
   per-session-recurring, not one-and-done — the fix (bare-vs-prefixed parse
   error message, or accept both) keeps earning.

3. Solo slice reaches /close with runtime phase status never flipped. Rollup
   showed `0/2` / both sheets `planned` though the work was implemented AND
   audited faithful (RV-181 verified, just check green). Solo (non-dispatch)
   implementation never moved phases planned→in_progress→completed in runtime
   state, so close's "confirm X/X complete rollup" pre-check fails on a slice
   that is substantively done. Cost: ~4 investigation calls (where does phase
   state live? why 0/2? which verb flips it?) then a manual
   `slice phase 163 PHASE-0N --status completed` per phase — each emitting a
   `phase-binding capture skipped … no code_start_oid stamped` warning because
   the phase never entered in_progress under the binding. The close skill body
   assumes phases arrive pre-flipped (the /execute path); it offers no recipe
   for "solo slice, audited-done, runtime phase status stale → reconcile the
   rollup before transition." A one-line pointer in /close would save the
   detour, and the binding-capture warning is noise in this legitimate path.

[SL-163 reconcile -> close @ 73k]

[inquisition; SL-168-RV183]
`doctrine review prime RV-183` aborts with `Is a directory (os error 21)` when a
slice selector points at a directory (`tests/`). The selector fileset hasher
(SL-147 PHASE-05) assumes file paths; a dir-valued design-target selector — a
legitimate, common shape — kills the prime outright instead of skipping/walking
it. Prime is "optimization, not gate" (review-ledger §2), so the inquisition
proceeded uncached, but the failure is non-obvious and cost a help-read +
retry to diagnose. Token cost: ~1 extra round-trip. Fix candidate: hasher should
walk or skip dir selectors, not error.

[/audit; SL-168-audit-2026-06-28]
Candidate review-surface worktree (`dispatch candidate create --worktree`) does
NOT carry gitignored derived embed assets — specifically `web/map/dist/` (the
built map frontend, embedded via RustEmbed `#[folder="web/map/dist/"]` in
src/map_server/assets.rs). With the folder absent, the RustEmbed derive degrades
to a struct with no `get()` → `Assets::get` E0599, and the *whole binary +
test bin* fail to compile. So a fresh audit worktree cannot build/test the slice
out of the box; the auditor must manually `cp -r web/map/dist` from a provisioned
tree first. Cost: a full failed build+test+clippy cycle and the investigation to
distinguish env-gap from slice defect. Worktree provisioning (dispatch fork /
candidate create) should hydrate gitignored derived embed roots, or the funnel
should fail loudly with a provisioning hint rather than a deep E0599.
Secondary: background Bash `${PIPESTATUS[0]}` after a `| tail`/`| grep` captures
the filter's exit, not cargo's — the harness "exit 0" notifications were
misleading; had to read the raw output to see the real compile failure.

[/audit→repair; SL-168-audit-2026-06-28]
Replaying a dispatched impl bundle onto a moved trunk for repair: the squashed
`review/<slice>` single commit cherry-picks cleanly with `--no-commit`, but two
non-obvious frictions surfaced. (1) Anchoring a corpus scanner to `.doctrine/**`
(F-3 fix) silently invalidated unit-test fixtures that wrote `.md` files at the
temp-root — they fell outside scope and either failed or passed vacuously; the
real fix was as much in the test fixtures (move them under `.doctrine/`) as in
the scanner. A scan-scope change must be co-reviewed with its fixtures.
(2) `cargo fmt --check` flagged files I had reverted to trunk's version
(policy/standard) — trunk itself is fmt-stale under the active rustfmt edition,
so the dispatch worker's apparent "fmt churn" was rustfmt *correcting* trunk, not
noise. An audit that dispositions "gratuitous fmt churn" should first confirm the
trunk file is fmt-clean under the gating toolchain, else it inverts the fix.

[justfile fix; 2026-06-28-A] — trivial justfile edit, no incidental complexity.

[dispatch; SL-173-close]
Agent-arm WorktreeCreate hook absent (no `.pi/hooks/` directory) — the agent arm's
`isolation: worktree` spawned the worker in an un-isolated main tree rather than a
fork at the explicit base. The base guard caught it (dirty tree + not-isolated),
preventing a wrong-base write. Had to switch to subprocess arm.
Token cost: ~3 rounds diagnosing the isolation failure, reading pi SDK docs
(no hook docs there either), and switching arms.

[dispatch; SL-173-worker-timeout]
Pi subprocess arm worker hit 300s timeout during final commit text generation
on deepseek-v4-pro. The commit had already succeeded (the `git commit` bash call
returned), but the model was still rendering the hand-back report text when
timeout killed the RPC pipe. The commit survived and was importable.
Token cost: ~2 rounds verifying the commit existed and wasn't orphaned.

[dispatch; SL-173-record-delta]
`doctrine slice record-delta` expects bare numeric id (173), not canonical
(SL-173). First attempt with SL-173 failed. The `--help` text says `<ID>` but
actual behaviour requires un-prefixed number. Token cost: 1 round.

[reconcile; SL-173-noop]
No-op reconcile pass works cleanly when all findings are tolerated/withdrawn.
The reconcile skill's no-op gate is well-documented — append outcome and hand
off. No friction points.

[dispatch; SL-172-0628]
- pi-RPC spawn hang: fifo holds pi stdin open, so pi does NOT self-exit on
  `agent_end` — it idles until the keepalive `sleep`/`timeout` expires (burned a
  full 1200s window on the first PHASE-01 spawn before kill). Fix: orchestrator
  must poll worker stdout for the `agent_end` event and kill pi. Encoded in a
  reusable spawn script. The skill's pi template ("agent_end gives typed
  completion") implies this but doesn't show the kill-on-event loop; a worked
  example would save every subprocess-arm user the same rediscovery.
- `worktree fork` is orchestrator-classed and refuses when its auto-detected
  project root (from CWD) lands inside a worker-stamped worktree:
  "refusing authored write `fork` — workers return a source delta". The
  orchestrator's interactive CWD drifts into worker forks during inspection, so
  the spawn must `cd` to the orchestrator root (or pass `--path`) before fork.
  Latent footgun — earlier spawns only worked by CWD luck.
- Worker prompt friction: `cargo test --lib` is wrong for a binary crate (no lib
  target). Self-inflicted in the first prompt; worker still committed but its
  self-verification was unsound. Funnel verify (orchestrator-side, correct cmd)
  is the real safety net — reinforces "trust the funnel verify, not the worker's
  self-check."
- DESIGN DEFECT surfaced (for audit/reconcile): design SL-172 §5.2 specifies
  `est_cost(est: Option<&EstimateFacet>, ...)`. That names a facet type in
  `priority/graph.rs`, tripping the NF-001 non-blocking tripwire
  (tests/e2e_estimate_non_blocking.rs) — the existing architecture routes facets
  into graph.rs via the local `EntityFacets` struct precisely so graph.rs never
  names facet types. The worker implemented the design faithfully and hit the
  wall. Resolved by deviating from the design signature (route via
  `Option<(f64,f64)>` bounds). The design should have caught this; the design
  template / inquisition pass doesn't cross-check signatures against authored
  architectural tripwires.

[audit; SL-170-RV188-audit]
Conformance triage was the token-heavy beat: `slice conformance` gives the cells
but not the *cause*, so disambiguating undeclared (foreign-slice interleave vs
authoring byproduct vs noted incidental) required manual cross-referencing of
boundaries.toml code_start_oid + git log + selector list. A `conformance --why`
that annotated each cell with its boundary/provenance origin would collapse ~4
investigative tool calls into one. Captured the recurring gotcha as a memory so
the next auditor skips the rediscovery. Minor: system doctrine binary predates
this slice's `verify-vt` verb — had to switch to ./target/debug (expected for a
slice that adds a verb, but a one-line "dogfood via build target" hint in the
audit handover would save the failed call).

[audit; SL-172-RV-189-audit]
Candidate-worktree provisioning gap. `dispatch candidate create --worktree`
produces a fresh worktree that lacks gitignored build inputs — here
`web/map/dist/` (RustEmbed `#[folder]` for map_server). The bin fails to compile
(`Assets::get` not found) until the assets are copied in, so the audit suite
run is blocked on a manual `cp -r web/map/dist <cand>/web/map/dist` before
`just check`. Token cost: a full failed-compile cycle + diagnosis before the
real audit work could start. The candidate-create provisioning step should
seed (symlink/copy) gitignored build artifacts the bin embeds, or the audit
skill should document the copy-in as a known pre-step for slices that don't
themselves touch web assets.

[reconcile; SL-172-RV-189]
Brief F-3 said "reconcile REQ-310 status to reflect the delivered aggregation" — but
REQ-310 was already `active` and the deferral lived in *prose*, not lifecycle. So the
"status" verb in the brief was a mis-cue; the real action was a `modify` (prose) row.
Cost: a re-read of SPEC-020 to confirm no hidden `deferred` status. Reconcile then hit
a genuine semantic fork the brief flattened: SPEC-020 already delegates aggregation
caller-side (§110/D3), so "lift the v1 aggregation deferral" risked falsely relocating
aggregation into the facet schema. Required a user round-trip to pick reading A-truthful
(narrow, not blanket-lift). Signal: audit briefs should disambiguate status-vs-prose and
flag when a "lift" interacts with an existing caller-side delegation, so reconcile
doesn't rediscover the fork. Also hit a transient `.git/index.lock` (concurrent agent) —
one retry cleared it.

[route+spike; imp004-dB3-bwrap]
D-B3 confinement spike was low-friction. Nested bwrap inside the outer NixOS
jail Just Worked (unpriv userns nesting allowed; bwrap 0.11.2). The whole
confinement is ~12 lines: `--ro-bind / /` then `--bind "$D" "$D"` re-grants rw to
only the worker tree; `--dev/--proc/--tmpfs /tmp`; `--die-with-parent`. No token
friction worth flagging — riskiest assumption (nested userns) was discharged by
one 8-line probe before any file was written. Residual unknown deferred to live
dispatch: whether pi needs a writable $HOME dot-dir beyond --session-dir.

[dispatch (pi/subprocess arm); SL-171-drive-2026-06-28]
Friction encountered during SL-171 two-phase pi-arm dispatch:
- pi-spawn.sh shipped committed-broken (line-17 `}m -rf "$D"` syntax error, fbade28c) —
  `bash -n` fails; cost a halt + user confirm + fix-commit before any spawn could run.
- pi-spawn-confined.sh (D-B3 spike) had two latent bugs that only surface in-jail:
  (1) relative `$D` → `bwrap --bind` can't mkdir mountpoint under `--ro-bind / /`
      (needs absolute path); (2) `--ro-bind / /` starves pi's `~/.pi` config → pi can't
      write its runtime lock → ships wrong (Google) key to OpenAI → 401. Needed
      `--bind $HOME/.pi` rw. Both cost a failed spawn + log-dig each.
- Confined arm CANNOT let the worker self-commit: a linked worktree's git object store
  lives in the main tree's `.git` (ro-bound), so `git commit` fails RO. Worker did all
  the work but left it uncommitted; orchestrator had to import the WORKING-TREE diff
  (`git diff B -- src | git apply --index`) instead of cherry-picking a commit. This is
  a structural property of the confined arm, not a one-off — worth a skill note.
- Worker self-reports unreliable: PHASE-01 worker claimed "green except 3 pre-existing
  worktree::marker failures" but (a) missed a real NF-001 allowlist tripwire failure it
  caused, and (b) the "3 marker failures" were its own DOCTRINE_WORKER=1 env (later fixed
  on trunk b02d2ff5). Orchestrator coord-tree verify is the only trustworthy gate.
- Orchestrator verify footgun: `cargo test 2>&1 | tail -40` truncates a 3366-test run to
  the last 40 lines — hid the true pass/fail picture; had to re-run with full capture to a
  file + grep all `test result:` lines. Don't tail-pipe the verify.
- Installed `~/.cargo/bin/doctrine` is stale (lacks `slice verify-vt`); had to fall back to
  the coord tree's freshly-built `./target/debug/doctrine` for the conclude VT gate.
- Plan VTs carry prose `expects` only (no structured `test_file`/`keywords`) → verify-vt
  reports every VT UNCHECKABLE. Non-halting, but the S3 gate provides zero real coverage
  signal for this slice.

[audit; SL-171-audit-rv190]
- Independent S3 verification of a dispatched slice required building the review
  surface in a fresh `git worktree`, which fails to compile: `web/map/dist/` is a
  gitignored built frontend artifact, so RustEmbed `#[folder=web/map/dist/]` over a
  missing dir drops `Assets::get` → E0599 in map_server (unrelated to the slice
  under audit). Cost: one full doomed compile + diagnosis before recognising it as
  environmental. Fix: `cp -r` the dist from the main tree into the worktree. The
  corpus already had memories for this (worktree-embed-gate, worker-fork-missing-
  gitignored-embed, coord-worktree-missing-build-artifacts) — but the audit skill
  doesn't surface them at the "build the review surface" step, so I rediscovered it.
  Candidate fix: audit/dispatch worktree provisioning should auto-copy gitignored
  build artifacts, or the skill should signpost the provision step.
- `git checkout review/171 -- ` (path-limited read of a ref into a fresh tree) with a
  shell-dropped empty pathspec silently switched the PRIMARY worktree off `edge` —
  the one move AGENTS.md forbids hardest. Caught via `git worktree list`. Inspecting
  a ref should never risk a branch switch; reaching for `git show`/`git diff <ref>`
  or a dedicated worktree avoids it.
- verify-vt returned all 10 VTs UNCHECKABLE (prose-only plan VTs). The conclude S6
  gate exited 0 on a dead signal — it neither confirmed PHASE-01's strong coverage
  nor flagged PHASE-02's genuine test gap. The gap was only found by hand-reading the
  diff for added test fns. A green-but-inert gate is a token/attention trap. (→ IMP-209)

[dispatch (pi arm); SL-176-drive-2026-06-29]
Regression baseline fingerprint drift mid-drive. Captured `baseline-B'` clean at
PHASE-02 start (fp 56536277). After spawning the pi worker + applying its diff,
`check regression diff --base B'` hit INV-8 cache-miss: the run-fingerprint had
drifted (env/exe component) so the freshly-captured baseline no longer matched.
fingerprint() is source-independent (argv+env_worker+marker+current_exe), so the
clean and patched trees compute the SAME fp at any given moment — but it differed
between the B'-capture call and the later diff call within one drive. Recovery cost
a full extra suite run: reverse-apply patch → re-capture clean baseline under the
now-current fp → re-apply → diff (green). Incidental complexity: the orchestrator
must capture the baseline IMMEDIATELY before the diff (same env window), OR the
fingerprint needs to be stable across a drive. Capturing right after the prior
phase's commit (as the funnel cadence implies) is NOT safe if anything perturbs
current_exe/env between then and the verify beat. ~1 wasted ~25s suite run per
occurrence + the reverse/reapply git surgery.

[dispatch (pi arm); SL-176-drive-2026-06-29 PHASE-03]
Two worker-verification gaps the funnel had to catch by hand:
1. MISSING TESTS read as green. Worker implemented the burndown production code
   correctly but added ZERO new test fns (count 32→32). The mandated 6 VT-3 fixtures
   were absent. The `slice verify-vt` keyword gate (test_file + keywords ["Fulfils",
   "burndown"]) would FALSE-PASS because those keywords appear in production code +
   comments, not tests — the gate checks presence-in-file, not presence-in-test. And
   `check regression diff` cannot catch missing tests (absence ≠ new failure). Only an
   orchestrator eyeball (git diff test-fn count) caught it. Cost: a continuation worker
   spawn to add fixtures.
2. HOLLOW self-reported green. Worker reported "check quick green". But `check quick`
   is UNCONFIGURED here ⇒ an owned no-op (exit 0). So the worker's lint self-check was
   vacuous. The real `check gate` (clippy -D warnings) found 7 errors (type_complexity,
   unnested or-patterns, let-else, sort_by_key, doc backticks) the worker introduced.
   The dispatch funnel's verify beat is `regression diff` (behaviour) only — it does NOT
   run clippy, so lint rot lands silently unless the orchestrator runs `check gate`
   per-phase. Fixed inline (orchestrator sole-writer, behaviour-preserving nits).
Takeaways: (a) verify-vt keyword gate is satisfiable by prose — it is NOT proof the
fixtures exist; a "new test fn count must increase" check would be stronger. (b) the
funnel should run `check gate` (or at least clippy) as part of the verify beat, not
just `regression diff` — behaviour-green ≠ landable. (c) worker prompts must name a
REAL green command; "check quick" is a no-op trap when unconfigured.

[dispatch/SL-176-PHASE-04; orchestrator-editorial-migration]
Two mechanism gotchas cost a debug+re-run cycle during the corpus migration:
1. scoped_from→originates_from via unlink+link is WRONG — both spellings parse to
   Role::OriginatesFrom (transitional alias), so unlink --role scoped_from no-ops and
   link adds a DUPLICATE. Class-1 wire-rename must be a pure on-disk value substitution,
   not engine unlink+link. Cost: 1 corrupted entity (SL-117), caught by inspection.
2. append_relation_row's F1 guard refuses to append a [[relation]] when a typed
   array-of-tables ([[selector]]) sits AFTER the relation array. One fulfil-target slice
   (SL-138) needed its [[selector]] blocks re-homed above [[relation]] before the bulk
   driver could complete. The migration op-runner aborted mid-batch on this; idempotent
   re-run after the fix completed. Lesson: pre-scan fulfil-target slices for trailing
   typed tables BEFORE running an append-driven migration.
Net: driving a bulk migration through the engine's own write seam (vs hand text-surgery)
is the right call for toml validity + contiguity, but the alias + F1 edges need an
up-front pass, not discovery mid-run.

[conclude SL-176; verify-vt-gate-divergence]
At funnel conclude, `check gate` (build+tests) was green but `slice verify-vt 176`
exited 1 on 3 VT existence-gate FAILs — two distinct gates, no single "is it done"
signal. Cost: a full re-derivation of where each mandated keyword/test actually
lived (P04 VT-1/VT-2 oracles never written; P03 VT-2 keyword pointed at the wrong
file — "fulfilled by" is owned by relation.rs, not relation_graph.rs). The
existence gate checks keyword presence but cannot tell a genuinely-missing test
from a mis-located mandate; disambiguating that took ~6 source probes. A
`verify-vt` that emitted the grep target + nearest matching symbol per FAIL would
have collapsed most of that. Handover had flagged the P04 oracle as an open
judgment, which saved re-discovering the question itself.
