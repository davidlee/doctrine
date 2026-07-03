
[dispatch; sl189-orch-claude-arm]
- `dispatch arm-spawn` auto-detects project root from CWD. Run from the SESSION
  root (the orchestrator's default cwd), `--slice N` alone wrote the arming `base`
  to the SESSION-root arming dir, not the coord tree's. Cost: a wrong arm + inode
  check + re-arm with `--path .dispatch/SL-N` + stray-base cleanup (~1 extra
  Bash round-trip + reasoning). The `--slice` flag is diagnostic-only (per its own
  help), so it does NOT redirect the write. Suggest: `arm-spawn` should resolve the
  coord tree from `--slice` (or refuse when CWD root != coord root), or the skill
  must say "always pass `--path <coord>`".
- Worker confinement blocks the RFC-011 instrumentation itself: the dispatch-worker
  cannot append to `.doctrine/rfc/011/case-notes.md` (authored `.doctrine/` write
  denied), so all worker-side friction must be relayed through the return message
  and transcribed by the orchestrator. Under-counts worker-side notes.
- `cargo test` inside a marker-stamped worker worktree false-fails the authored-write
  e2e suites (`e2e_adr_cli_golden`: 3 failures, "refusing authored write"). Not a
  slice regression — the funnel's marker-cleared `check regression diff` correctly
  reported zero new/changed. But the worker's own `cargo test` report shows red that
  needs orchestrator adjudication; a worker running the full suite can't self-clear
  the marker. Costs a reasoning step to separate env-artifact from real failure.
- `slice phase` takes `--status <S>` (not positional) — minor CLI-shape guess-miss.

[design; SL-180-sess1]
Design-surface friction, token cost:
- ADR-001 layering: had to read layering.toml A->submodule tiers A->
  tests/architecture_layering.rs to learn edges are extracted at TOP-LEVEL
  module granularity + set-deduplicated. Nowhere in boot/reference-docs states
  the gate's edge granularity, so I initially asserted a WRONG tangle-safety
  proof ("slice imports no worktree") that a transitive slice->review->worktree
  path falsified. External (GPT) inquisition caught it; verifying required
  reading the test internals. A one-line note in ADR-001 body ("edges = first
  path segment, deduped; sub-tier map governs upward-edge check only, not
  tangle") would have saved ~2 tool round-trips + a wrong claim.
- `doctrine memory retrieve <key>` positional arg rejected (needs --query or
  scope probes); boot Onboarding says "/retrieving-memory `mem.key`" which reads
  like a positional. Cost one failed call.

[inquisition; RV-216-plan-review]
VT keyword false-positive risk: the VT gate's keyword-grep mechanism over entire test_file (not just #[test] blocks) creates a structural risk where any substring in comments or unrelated production code can falsely attribute pass. The "against"/"strict" case (F-1) is concrete — these appear as English words in existing comments. The fix (double-dash prefix) works but is fragile: a future developer who writes `// assert against regression` in a test comment would re-introduce the false positive. The gate should ideally scope keyword search to test functions only, or use a token that is syntactically implausible outside test code (e.g. `fn test_against_strict`).

[inquisition; RV-216-contiguity]
`append_edge`/`append_relation_row` has no contiguity-awareness. The method name "append" accurately describes what it does, but the invariant it violates (same-label contiguity) was added later (SL-176). This is a classic instance of a storage invariant being added without updating the write seam — the write seam was correct when written but became wrong when the invariant tightened. ISS-058's fix should either group-on-write or sort-on-write; both approaches have tradeoffs (group-on-write preserves partial hand-ordering; sort-on-write is simpler but destroys any intentional intra-label ordering).

[worktree+preflight+execute; ISS-058-direct-fix]
* Worktree setup was smooth — `doctrine worktree fork` handled everything including provision.
* Preflight → execute flow worked well for a well-scoped, pre-diagnosed bug with a clear fix sketch in the issue body.
* One friction point: `doctrine worktree land --fork` expects a branch name, not a path. CLI error `no-such-fork` was confusing (the fork directory existed).
* Clippy `unwrap_used` + `expect_used` both denied required switching from index-based collection to `array.iter().cloned()`. Minor but worth noting.
* Test label selection: `References` for SLICE_KIND requires a `Role`; had to switch to `Related` (label-only, writable for SLICE). This was discoverable but not immediately obvious from the issue sketch.

[dispatch; sess-restart-2026-07-03]
Session RESTART did NOT fix the claude-arm isolation:worktree failure. Post-restart
re-dispatch of SL-187 PHASE-04: worker landed in the coord tree (toplevel
`.dispatch/SL-187`, no `.worktrees/agent-` segment), hard base-guard aborted cheap
(~18.7k tok). Confirmed persistent, restart-surviving harness limitation: the Agent
tool's `isolation: worktree` is a no-op in this build — `.claude/worktrees/` exists
but stays empty, no WorktreeCreate hook fires, no `dispatch/agent-*` branch created.
Hook config verified correct (plugin doctrine@0.1.0 hooks.json: `WorktreeCreate:* ->
doctrine worktree create-fork`); `create-fork` + `worktree fork --base` both work when
invoked directly. So the break is squarely the harness not emitting WorktreeCreate for
isolation:worktree spawns — not doctrine. Token cost of the whole diagnosis-through-
restart-through-re-confirm loop is the RFC-011 signal here: a harness capability that
silently degrades to no-op (rather than erroring) burns a full spawn + guard + restart
before it's provable. The base-guard-in-worker-prompt pattern is what keeps each failed
attempt cheap (~15-19k) instead of a full wasted worker.

[audit; sess-restart-2026-07-03]
Dispatch coord tree carries a STALE review corpus: `.dispatch/SL-187` (dispatch/187,
forked at 757b71db) sees RVs only through RV-216, while trunk (edge) has RV-232
committed + RV-233 in-flight (SL-180). `dispatch status` confirms trunk moved 82
commits past the fork-point. Consequence: an audit RV minted from the coord tree
would allocate RV-217 and COLLIDE with 16+ already-allocated RVs (217..233). The
review-ledger parent-tree caveat ("review verbs refuse a worktree/fork-resolved
root; run from the main tree or merge the fork first") is not just a root-resolution
nicety — it is what keeps RV allocation coherent against the trunk corpus. RFC-011
signal: a self-applied (non-dispatch) phase still incurs the FULL dispatch-integration
tax at conclude — refresh-base (82-commit merge) → prepare-review → candidate →
integrate — before the audit can even allocate an id. The code delivery (PHASE-04)
was cheap; the integration ceremony is where the coordination cost lives, and it is
unavoidable because the RV namespace is trunk-global.

[IMP-103; preflight→execute] IMP-103 was 2 doc-comment edits. The preflight
surfaced the dependency chain (IMP-103 after IMP-101 after ISS-024) and
confirmed the `after` is advisory (no `needs`). Body files for both IMP-101
and IMP-103 were empty templates — had to reverse-engineer the actual task
from the title + SL-121 design + memory corpus. Token spend: ~7k to discover
what amounted to a 2-line change. The memory corpus was the right place
(mem_019edd33d3b273928001e6c867cb2de5 nailed the problem statement), but
having it in the IMP body would've saved several round-trips.

[phase-plan/execute; sl181-postcompact-resume]
Compaction summary said REV "to author" and "revision list EMPTY", but the
prior (compacted) session had already: authored+approved REV-018, flipped
SL-181 → audit, AND run /audit to a clean RV-232 (0 findings). Re-deriving true
state cost ~6 probe commands (revision list/show/paths, slice status, git
show, review show) because the summary lagged the on-disk reality. Also found
3 uncommitted residues the prior session left mid-transition: slice toml status
bump, and two slug symlinks (REV-018, RV-232) the `*/new` verbs create but the
author's path-scoped `git add` of the numeric dir missed. Cost: summary-vs-disk
drift forces defensive re-verification on every post-compaction resume.

[reconcile; SL-180-recon-c8e8]
Two frictions, both cost a re-derivation step:
1. Brief-surface mismatch. Audit's reconciliation brief listed a `plan.toml EX-4`
   edit, but the /reconcile skill names ONLY design.md + slice-NNN.md as per-slice
   direct-edit surfaces (plan criteria are EX-/VT- immutable-append). Had to stop
   and surface the tension to the user rather than blindly execute the brief. A
   brief item can name a surface the writer skill won't touch — the two skills
   don't cross-check. Cost: one AskUserQuestion round.
2. Prose vs registry duality. Brief item "clear the spurious undelivered" reads as
   a design.md §6 prose edit, but `slice conformance` reads the SELECTOR REGISTRY
   (slice-180.toml via `slice selector`), not §6 prose. The load-bearing fix was
   `slice selector rm`; the §6 edit is only the human mirror. Easy to edit the
   prose and leave conformance still red. The brief should name the registry verb,
   not just the prose line.

[dispatch-agent; sess-sl190-drive]
Claude-arm dispatch worked cleanly after the harness/hooks update — WorktreeCreate
forked each worker at the explicit base, PreToolUse confinement held, funnel
imported the live worktree delta each phase. Zero token waste on the spawn
mechanism (contrast prior session where the hook produced no fork).
Recurring worker friction (PHASE-03 worker self-reported): the worker initially
pointed Read at the shared-checkout ABSOLUTE path (/workspace/doctrine/src/...,
on edge, behind the fork) while Bash relative paths hit its worktree copy — the
two disagreed on content/line numbers, costing several tool calls to diagnose.
Mitigation for future worker prompts: instruct workers to prefix EVERY file op
with the worktree root and never touch /workspace/doctrine/* directly.

[close; sl190-imp127-trunkrow-gate]
IMP-127 direct-land escape is now INCOMPLETE against the SL-126 close-integration
gate. Prior memory (mem.pattern.dispatch.split-lineage-close-conflict-direct-land)
says `done` "waves through" a never-journal-integrated bundle — STALE. Current gate
(ledger.rs trunk_integration): journal ABSENT/zero-rows → NotDispatched (passes);
journal HAS rows but none target trunk → Blocked("no trunk row"). SL-190's
dispatch/190 journal carries review/190 + phase/190-01..06 rows (funnel-projected),
so direct-land trips Blocked even though main genuinely contains the reviewed code
(gate-green, byte-identical to reviewed tip 0eca671a). Neither integrate path can
write the trunk row: candidate path needs an admitted close_target (IMP-127 blocks);
legacy plan_trunk_row sources phase/190-06 tip which CANNOT ff main (split lineage —
the exact reason the candidate merge existed). Net: a split-lineage dispatched slice
is UN-CLOSEABLE by any verb once its phase rows are journaled — the direct-land
escape must also hand-write a trunk row, or IMP-127's fix must add a
"record-completed-integration" path. Cost: full topology re-derivation + code read to
prove no verb suffices. Recommend IMP-127 scope note + a memory correction.

[audit; sess-clear-187-integrate]
Post-session-clear resumption of a dispatched slice's audit paid a recurring
runtime-state tax. Three distinct frictions, each a re-derivation step:
1. STRAY UNCOMMITTED LIFECYCLE EDIT. The coord worktree (.dispatch/SL-187) held an
   uncommitted `status = "audit"` flip on slice-187.toml (prior session halted mid-
   transition). refresh-base HARD-REFUSES over a dirty coord tree. Resolution: commit
   the flip (never discard). Cost: one diff + one commit to unblock.
2. RUNTIME PHASE FLAGS LOST. All four phase-NN.toml runtime flags read `planned` after
   the clear (disposable state didn't survive), so `sync --prepare-review` AND
   `slice conformance` both refuse with "recorded row for PHASE-NN, which is not a
   completed phase". The delivery was genuine (commits + registry end-oids matched);
   only the flags were gone. Restoring = re-flip all four `completed` from the verified
   registry+commit evidence. RFC-011 signal: the disposable runtime tier and the
   committed evidence tier disagree after a clear, and the refuse-message points at the
   flag, not the root cause (state loss), costing a registry+git cross-check to trust
   the re-flip.
3. CONFORMANCE EDGE-MIRROR SPLIT-BRAIN. `slice conformance` read from the edge tree
   flagged cli.rs + e2e_prompt_resolve_golden.rs `undeclared`, but those selectors ARE
   declared design-target on the delivery (a299cf27) — invisible on edge only because
   the authored slice-187.toml hasn't integrated to trunk. Distinguishing real drift
   from this pre-integration artifact required reading the selector list from the
   candidate surface AND git-showing the declaration commit. A conformance read is only
   trustworthy from a tree whose authored slice state matches the delivery.
Also: `doctrine check gate` (→ just gate proxy) emitted a thin captured output (2 test
binaries) that was easy to misread as the full suite; had to run `just gate` explicitly
to confirm the real 3703/0. The proxy's cadence vs the full-suite command is not
self-evident from the output.

[preflight; iss206-pf1]
ISS-206 preflight: issue framed the fix as "touches INV-2/INV-3 semantics — design call". Real, higher-value find surfaced only after reading design.md §L95-101/L133-134 against INV-2 (L271-273): the SL-186 design *narrative* already claims same-slot user override ("the user wins the same slot", "a user edit at the same slot wins") but attributes it to the *provenance tiebreak* — which is ordering-only, never suppression. So the design is internally inconsistent with its own INV-2 ("only replaces suppresses"), and the resolver faithfully implements INV-2. Cost: ~2 file reads (design + hymns.rs) to promote the issue's "correctness smell" into "design contradicts stated intent". Cheap, but the issue body could have cited the design line that promises override, saving the cross-read.

[slice; IMP-197→SL-191]
`doctrine slice selector add` writes the `[[selector]]` array AFTER the
`[[relation]]` block in the slice toml. A later `doctrine link` then refuses
to append (F1: "typed table `[selector]` authored after `[[relation]]`"),
forcing a manual TOML re-home of the selector block above the relations.
Order-of-authoring coupling between two write verbs on the same file — the
selector-add verb should home its block above the relation array (or link
should tolerate a trailing array-of-tables). Cost: one Read + one Edit +
re-run. Suggest: seed relations first, selectors last — or fix the verb.

[dreaming; dream-2026-07-03-a]
`doctrine link <src> related <target>` refused on a shipped-corpus memory
(`mem.pattern.doctrine.close-drift-discharge-rec`, in `shipped/`, ADR-005 tier)
with "shipped/ is read-only — record a version in items/ first". The error
message is clear and actionable, but `memory list --orphans` doesn't
distinguish shipped-tier orphans (structurally can't gain relations without a
promote step) from items/-tier orphans (a plain missed-link). Cost: two failed
`link` calls before recognizing the pattern; a `--tier` column on the orphans
table would let a dreaming pass skip shipped rows up front instead of
discovering the refusal per-row. Separately: found one corpus memory
(mem.pattern.doctrine.plan-va-scoping) with a body that is ONLY the title —
no content. `memory validate` does not flag empty bodies as a finding; filed
CHR-035 rather than fabricate content.

[preflight; rfc013-confidence-fable-0703]
- User pointer "transcript & design docs from SL-186,SL-187" — no transcript files exist under .doctrine/slice/{186,187}/; only rfc/013/transcript.md. Cost: one find + one ls round-trip disambiguating. Slice dirs carrying no session transcripts is fine, but the RFC's References could name transcript.md's location explicitly.
- memory_search "hymns prompt cascade" surfaced mostly dispatch-worker memories (scope overlap via 'worker/prompt' terms); only 1/20 rows cascade-relevant. BM25 term collision between dispatch-prompt and cascade-prompt vocabularies.

[spec-tech; rfc013-confidence-fable-0703]
- `doctrine link SPEC-023 related RFC-013` refused — SPEC may only author `governed_by`. Edge had to be authored from the RFC side. Skill/glossary don't state per-kind writable labels; one wasted round-trip. A hint in the error listing the legal direction (RFC→SPEC `related`) would have saved it (it did list legal labels for SPEC — good — but not the inverse option).
- Scaffolded spec-023.toml comments say `parent — (SPEC-NNN or PRD-NNN); subtype-aware` while SPEC-017/skill text imply tech parent is SPEC-only; harmless but momentarily confusing.

[route; spec003-inq-0703]
Boot onboarding names `/retrieving-memory`, but the installed skill is
`/retrieve-memory`; resolving that spelling mismatch cost a small
cross-check before the required orientation memories could be retrieved.

[retrieve-memory; spec003-inq-0703]
Exact onboarding keys (`mem.signpost.doctrine.overview`,
`mem.signpost.project.orientation`) are easiest to consume via
`memory retrieve --query <key>`, but the command returns "1 of 30" because the
query is still lexical rather than an exact-key retrieval surface.

[inquisition; spec003-inq-0703]
The inquisition skill requires a ledger for durable subjects, but a spec target
needs CLI discovery to confirm the facet and target support. `review new --help`
shows facets are lifecycle-aspect names and accepts a generic `--target`, so
SPEC-003 can be arraigned under `--facet design` with `--raiser inquisitor`.
After opening RV-236, `review prime` refused because priming is slice-selector
only ("needs a slice target") even though `review new` accepts SPEC targets.
That turns "open + prime" into "open + record failed prime" for spec/ADR/design
inquisitions. Another small mismatch: `--raiser inquisitor` stamps the label,
but `review raise --as inquisitor` is rejected; the mechanical role remains
`--as raiser`.

[canon; spec003-inq-0703]
SPEC-003 descent review exposed a canon ambiguity: PRD-012 says a technical
spec descends from product intent, SPEC-017 makes `descends_from` optional for
unfilled tech specs, and `spec validate SPEC-003` accepts an active context spec
with null descent. Resolving whether root context specs are exempt required a
finding/follow-up rather than a confident one-line sentence.

[notes; spec003-inq-0703]
The notes skill assumes a slice-owned unit ("find owning slice" / `notes.md`),
but this was a spec-level RV with no slice owner. The durable record naturally
lived in RV-236 synthesis plus IMP-237, not slice notes; determining that still
cost a skill read. Also backlog item paths use `backlog-NNN.*` even under
`.doctrine/backlog/improvement/NNN/`, not `improvement-NNN.*`.

[record-memory; spec003-inq-0703]
`memory record` needed git anchor capture and failed inside the sandbox with
`.git/index.lock: Read-only file system`; escalation was required for a normal
local memory. After record, `memory show` and `memory list --filter/tag` found
the item, but ranked `memory find --command "doctrine review prime"` did not
surface it even at limit 30 because broad shipped command-scope signposts won
the ranking. Command-scope alone is weak for fresh local gotchas; add a tag and
expect list/key retrieval if precise surfacing matters.
[inquisition/retrieve-memory/route; spec-023-rfc-013-inquisition-01]
Procedural friction: boot onboarding says `/retrieving-memory`, but the available skill is `/retrieve-memory`; route skill says to run `backlog list`, while the boot command table exposes backlog subcommands without `list`. This forces CLI-shape verification before substantive review work.
[inquisition/consult; spec-023-rfc-013-inquisition-02]
Review priming friction: `doctrine review prime RV-237` refuses a SPEC target because priming only knows slice selectors, while the inquisition skill says durable doctrine subjects such as design/spec artifacts should be tried on an RV and then primed. The review must proceed unprimed with manual `show`-sourced evidence.
[inquisition/backlog/review; spec-023-rfc-013-inquisition-03]
Review role-label friction: `review new --responder respondent` records a custom responder label, but `review dispose --as respondent` is refused; `--as` accepts only role-axis tokens (`raiser`/`responder`). The help text says cooperative role assertion but does not warn that custom labels are display-only for `--as`.

[/design SL-192; sess-opus-192-design]
- Detailed handover (reading list + pre-enumerated watch-points + banked
  algebra) collapsed the clarifying loop to a single confirm-question. High
  signal-per-token: the design skill's full multi-turn clarify was mostly
  formality because the hard forks were pre-resolved upstream (SPEC-023 D2/D3,
  RFC-013). Handover quality directly bought token efficiency.
- Codex external pass earned its cost: 2 real catches (Sidecar.model presence —
  bare defaulted Vec loses omitted-vs-empty; singleton-degeneration wording
  conflated behaviour-preserved with tests-unchanged when ~12 struct-literal
  sites take a forced type migration). Also 1 over-reach (proposed collapsing
  same-root pins, which would break a legitimate distinct-subtree intersection)
  — triage against source truth mattered, don't apply reviewer findings blind.
- Minor CLI friction: `relation census <ID>` rejects a positional id (wants a
  flag); cost one wasted call. Low.
[record-memory; spec-023-rfc-013-inquisition-04]
Memory-recording inefficiency: I recorded `mem.fact.review.prime-slice-target-only` before probing for an existing scoped memory. The recorder suggested `mem.fact.doctrine.review-prime-slice-target-only`, which already captured the same fact, forcing a duplicate show + supersede cleanup. Future record-memory flow should run `memory find` for the proposed title/key first when the fact is likely already known.

[execute; SL-192-p01 solo-exec]
- Atomic BTreeSet type-flip (Option→set on 2 structs) rippled to ~17 sites; the
  long red window between authoring goldens and last migration is inherent, not
  incidental — new-diagnostics stream made it cheap to chase remaining sites
  without full recompiles.
- Minor friction: (a) shell is zsh — `${PIPESTATUS[0]}` empty after a pipe, had
  to re-run `check gate` to capture exit; (b) `perl` absent in jail, fell back to
  Edit replace_all for batch `model: Some(..)→BTreeSet` migration; (c) edge HEAD
  advanced under me twice mid-session (concurrent agents) — expected, cost one
  extra `git rev-parse` to reconfirm the fork base.
- Zero design ambiguity: design §3/§4 carried every decision incl. the accepted
  cross-root alpha boundary; no /consult needed. Signals a well-shaped design→plan.

[reconcile-REV; ISS-206/REV-019 session]
Reading a requirement's *statement* cost several dead tool calls. `doctrine spec
req` has only `list` (roster: id/label/kind/status — no statement). No `spec req
show`; no top-level `req` (it's `rec`, easy misfire). `spec req list --json`
rows omit the statement/title field too. Ended up grepping `.doctrine/requirement/NNN/requirement-NNN.toml`
directly for `title =`. For a REV that targets requirements by statement content,
"show me REQ-NNN's statement" should be one call. ~6 wasted calls.

[audit; SL-192-audit-238]
Solo-fork audit hit real process friction (SL-192, RV-238):
 (1) `review new` SUCCEEDS on a solo fork worktree but `review raise`/`dispose`/
     `status` HARD-REFUSE it (IMP-024: turn baton lives in parent-tree gitignored
     state). So `review new` mints an RV on the fork that can never be driven from
     the fork — a mint-then-strand trap. Either `new` should refuse too (fail
     early) or all baton verbs should tolerate the fork. Cost a full create+raise
     round-trip + a stranded RV-238-on-fork to discover.
 (2) No documented solo-fork audit path. Review must run from the parent tree
     (IMP-024), but the parent tree lacked BOTH the delivered code (fork-only
     commits) AND the completed-phase runtime state (.doctrine/state is
     gitignored/fork-local; `worktree land` git-merges code but carries no runtime
     state). Dispatch has the candidate-branch projection for exactly this; solo
     /execute has no analog. Net resolution: land fork->edge FIRST (ledger's
     "merge the fork first"), then manually reconstruct phase state on edge
     (`slice phase ... completed` x2 + `slice status audit`) — steps no verb
     performs and which aren't written down. Worth a solo-fork audit runbook or a
     `worktree land`-carries-state / candidate-analog feature.
 (3) Running the gate on edge surfaced an UNRELATED pre-existing edge blocker
     (`.doctrine/dispatch/` over-broad gitignore, commit 61eae2ce, shadows the
     committed dispatch ledger + fails the classification parity test) — captured
     ISS-207. Auditing on the shared edge tree couples a slice audit to whatever
     else is red on edge; the fork gate (exit 0) was the clean SL-192 evidence.

[reconcile+close; SL-192-close-238]
Close pre-check `doctrine check gate` is repo-global (full test suite), so a
single unrelated red test (ISS-207, `.doctrine/dispatch/` gitignore parity)
couples close-of-SL-192 to repo-wide gate health. SL-192 correctness was
independently proven (fork gate exit 0 + edge conformance 4/4), yet close
required a documented-override judgment call rather than a clean green. Every
close on edge will hit the same forced judgment until ISS-207 is fixed. The
terminal transition itself (`slice status done`) correctly gates only on RV
blockers, not the test gate — so the friction is skill-level (pre-check) vs
binary-level (transition) divergence, not a hard block. Reconcile itself was
clean: F-1 unlink one verb, F-3 deferral one documented decision.

[revision (REV-020 enacting RV-236 penance); rv236-followup]
Three friction points surfaced enacting a governance correction from an
inquisition:
1. Inquisition trust cost. RV-236 F-1 was a false positive that consumed a full
   verification pass to refute: it read ADR-004's `superseded_by` flag as proof
   of dead authority without checking the superseder's topic (ADR-012 = dispatch
   topology, unrelated) or that the active corpus still cites ADR-004 as live.
   Root cause was a fixture supersede from SL-155, not the flagged citation.
   A cheap guard ("is the superseder on-topic? do peers still cite the target?")
   would have saved the round-trip. Recorded as
   mem.system.governance.no-fixture-supersede-on-live-adr.
2. No supersede reversal verb. `doctrine supersede` is one-way and refuses an
   already-superseded OLD; `unlink` only touches tier-1 `link` edges, not the
   lifecycle supersede. Reverting the bad edge required hand-editing two TOMLs
   (status + superseded_by + supersedes). A `doctrine unsupersede`/`revision`
   reversal path would keep the operator off raw TOML.
3. No typed backlog↔REV provenance edge. IMP-237 (origin) could not link to
   REV-020: backlog `references` targets exclude REV, and REV rejects inbound
   `references`. Provenance survived only as rationale prose. The REV↔origin
   relationship is real (ADR-018 originates_from) but unauthorable across this
   kind pair.

[backlog; fable-viz-thread-0703]
Dup-survey friction: `backlog list` (no filter) appends an `overrides:` block of
dangling soft-edge notices (8 lines, edges to absent SL ids) on every listing —
noise tokens during a duplicate survey; belongs behind a flag or in `doctor`.
Also: concurrent-commit surprise — user/background commit ("doctrine", 46bd61ef)
scooped the freshly created IMP-241 files while agent was still authoring; agent's
path-limited commit then carried only the toml delta. Harmless here (path-limited
per AGENTS.md), but worth knowing the window exists.

[preflight; IMP-241-pf-a]
Card IMP-241 names substrate as "pure functions over the existing
ActionabilityView (src/priority/view.rs)". Reading the code, ActionabilityView
is the THIN web projection (id/title/score/rank/blockers + needs/after edges) —
it lacks the score-component decomposition (leverage/optionality/base, on the
PriorityGraph maps) AND the provenance reasons (EvictedEdge/CycleDegraded, only
emitted on the explain path, never attached to ActionabilityView). Roughly half
the finding catalogue (decomposition, provenance, β-perturbation) cannot be
computed from ActionabilityView alone. An implementer who took the card's
substrate literally would build the wrong seam. Cost: one extra read pass of
surface.rs/channels.rs to discover the true substrate is PriorityGraph. Low, but
the card's precise-looking file cite (`view.rs`) is a mild false anchor.

---

[dispatch (claude arm); SL-193-edge-0321]

Drove SL-193 (2 phases) end-to-end via /dispatch (claude arm, Opus/Opus). Full
funnel worked, but several friction points cost significant tokens (~190k in
avoidable re-dispatches):

1. **F5 design gap caught only at funnel verify, not design.** The locked design
   named the sidecar projection "forward-step 4" (append-last). But `install`'s
   step 3 (`install_agents_for`) renders `.doctrine/agents/*.md` from the resolver
   BEFORE step 4 wrote the sidecar → non-idempotent install (first pass doubles,
   second suppresses). Caught by an existing e2e (`install_refresh_is_stable`) at
   the regression-diff verify beat — i.e. AFTER a full worker run (~85k tokens).
   The adversarial design pass even asserted "no step-count regression / step 4
   breaks nothing" — the exact claim that was false. Cost: 1 full PHASE-01
   re-dispatch. A design-time check "does any install step render from the corpus
   the new step mutates?" would have caught it for ~0 tokens.

2. **Base-drift trap: authored/selector commits between arm-spawn and import
   invalidate the worker's fork base (`verify-worker: wrong-base`) → forced
   re-fork.** Hit TWICE. (a) Committing the F5 design-note on `dispatch/193`
   advanced coord HEAD off the worker's base → had to re-dispatch PHASE-01 at the
   new base. (b) A missing design-target selector forced `slice selector add`
   mid-drive; committing it advanced HEAD again → re-dispatch PHASE-02 (~106k
   tokens). LESSON: all authored corrections (design amendments, selector fixes)
   must land BEFORE arm-spawn, never interleaved between arm and import — the
   funnel binds worktree-base==--base==coord-HEAD rigidly. The skills don't warn
   that an authored commit mid-batch strands the in-flight worker. Worth a
   memory + a red-flag in /dispatch.

3. **Selector authoring gap vs plan.** plan.toml PHASE-02 VT-3/VT-4 mandate tests
   in `tests/e2e_prompt_resolve_golden.rs`, but that path was not in the slice's
   design-target selector set → `import --slice` refused `undeclared-scope`.
   Selectors should be cross-checked against plan VT `test_file`s at /plan time
   (a `slice selector doctor` that reads plan test_files would catch it).

4. **Phase-completion flip location is silent + misleading.** The prepare-review
   completeness gate reads the COMPLETED-PHASE set from the PRIMARY worktree
   (`dispatch.rs:1873/1900`, `registry_completeness(&primary,&primary)`). But the
   natural move during dispatch is `slice phase --status completed` in the COORD
   tree — which makes the coord rollup show "2/2" (looks done!) yet leaves the
   gate seeing 0/2 → `registry incomplete: recorded row ... not a completed phase`.
   Cost a debug detour to discover the gate reads primary. The conclude cadence
   never states "flip phase completion in the primary tree." Fix: either the
   funnel record-boundary should flip primary phase status, or the conclude doc
   must say where to flip.

5. **`prompt explain` does not apply replaces-suppression** (prints the raw
   ranked active set), but design EX-2/VA-1 say "explain shows framework
   SUPPRESSED." A worker following the design literally would assert against the
   wrong verb. Correct verb is `prompt resolve` (applies the replaces graph).
   Design-vs-impl wording mismatch — should be corrected at reconcile.

6. **Reconcile-via-full-install pollutes.** `install` runs `execute_plan` (base
   materialization) unconditionally after "Proceed?", so running it on the coord
   tree to backfill 5 sidecars would also write unrelated base files. Worked
   around by running the producer into a throwaway temp root and relocating the 5
   producer-generated `.toml` (byte-identical — `replaces` is corpus-independent).
   A standalone `prompt project` / `install --only-hymns` verb (design OQ-2,
   deferred) would remove the workaround.

[/audit; SL-193-audit-239]
Auditing a DISPATCHED slice needs the impl bundle materialised to build+test
independently, but a fresh detached worktree on review/193 won't compile: the
RustEmbed `#[folder]` root `web/map/dist/` is a gitignored BUILT asset absent
from the branch (E0599 `Assets::get` not found + "folder does not exist"). Cost:
one failed `cargo build` + diagnosis + manual `cp -r` of web/map/dist from the
primary tree before the build succeeds. A dispatch/audit worktree-prep step (or a
`doctrine worktree fork`-style provision that seeds gitignored embed roots) would
save the round-trip. Second friction, already known (F-2): `slice conformance`
run from edge reads `undeclared` because the declaring selector + F5 design
amendment live only on the impl bundle, not edge — conformance-against-edge is
structurally red for any dispatched slice until /close lands the bundle. The
audit must review the bundle surface, not edge, to read conformance truthfully.

[design; SL-194-ext-review-abc]
- codex mcp model override `gpt-5.2-codex` 400s on a ChatGPT account ("not
  supported"); dropping the override → default GPT-5.5 worked. Cost one round-trip.
- backlog `new` creates BOTH `NNN/` dir and a `NNN-slug` symlink; at commit
  time the double entry needed an `ls` to confirm before path-limiting (shared
  index, must exclude another agent's untracked IMP-242). Minor: a note that the
  slug is a symlink alias would save the inspection.

[plan; SL-191-plan-a] `doctrine slice show <id>` renders scope MD only — it does
not surface the authored plan phases/criteria. Verifying the just-authored
plan.toml therefore required Reading the raw file (against the "read via show, not
raw files" guardrail, because show has no plan projection). A `slice show --plan`
or a plan projection would keep plan authoring inside the guardrail.

[dispatch; dispatch-SL194-p01]
New top-level verb has scope-coupling beyond the nominal "edit cli.rs":
adding a `Command::Findings` variant forced (a) a Read-class arm in the
exhaustive `write_class` match in `src/commands/guard.rs`, and (b) two
cli.rs help-tree assertions (reports-family membership + verb census
47→48). guard.rs was not in the slice's design-target selectors, so the
funnel `import --slice 194` scope-check would have refused it
(`undeclared-scope`). Cost: worker flagged it as a seam surprise; orchestrator
dropped the `--slice` scope-check for the batch, manually vetted the 8-file
delta, imported, then declared guard.rs as a design-target selector post-hoc.
Root cause: the verb-registration surface (cli enum + guard write_class +
help-tree/census asserts) is not discoverable from the plan's "cli match arm +
members list" wording — a checklist of "files a new verb touches" would have
pre-declared guard.rs and saved the round-trip.

[dispatch; SL-191-P02-golden-halt]
PHASE-02 added install/hymns/model/adherence/low.md (a new model-band trait key
per D2/D3). This changed `prompt model-keys` output 2→3 keys, breaking the e2e
golden tests/e2e_prompt_resolve_golden.rs::vt2_model_keys_exact_relative_keys
(pins exactly 2 keys). The funnel regression-diff caught it (HALT, correct).
TWO cost sources:
1. Worker-suite-scope: orchestrator prompt told worker to run
   `cargo test --bin doctrine install` — too narrow. The affected golden is a
   tests/ integration binary, invisible to a --bin doctrine filter. Violates
   mem.pattern.dispatch.worker-prompt-run-full-suite. Worker reported green;
   funnel diff (full suite) is what caught it. Belt worked; the worker's local
   gate gave a false-green. Prompt should mandate the regression-relevant suite
   incl. tests/ e2e when a phase mutates the embedded corpus.
2. Selector/base collision: the golden file is not a design-target selector, so
   import refuses undeclared-scope. Adding a selector mid-phase advances the
   coord branch (B→B'') and breaks the in-flight worker's verify-worker base==B'
   invariant. A foreseeable-golden-coupling that plan-time selector enumeration
   missed. Recovery requires re-basing the phase on a B'' that carries the
   selector — i.e. the selector set must be complete before a phase that will
   touch a coupled golden. Lesson: `slice selector doctor` / plan-stage should
   flag corpus-mutating phases whose downstream goldens aren't in the selector set.

[execute; IMP-183-01] Lenient deserialization for Doc structs — adding estimate/value
fields to shared Doc structs (governance::Doc in particular) broke catalog scan
outbound_for → relation_edges, which parses the full Doc. Had to add lenient
deserializers (estimate/value) so malformed facets don't block entity reads.
The 4 failing tests were all ADR-seeded catalog scan tests. Diagnosis required
tracing scan_entities → outbound_for → relation_edges → read_doc to find the
serde parse that was failing. A comment on Doc documenting the tight coupling
between show and relation_edges would help future agents.

[dispatch; SL-191-P02-recovery-phantom-gate]
Nearly spent a promote(edge→main)+refresh-base+selector-add+re-fork ceremony on
a recovery, on the false premise that funnel `import --slice` refuses an
undeclared-selector path (tests/e2e_prompt_resolve_golden.rs). Memory check
(mem.fact.conformance.rev-only-slice-undeclared) corrected it: undeclared-scope
is an AUDIT-time `slice conformance` report (dispose `aligned`), NOT an
import-time gate. The only import path-belt is R-5 (.doctrine/.claude). Real
root cause was mem.pattern.dispatch.worker-prompt-run-full-suite: P02 worker
prompt scoped the gate to `--bin doctrine install`, so the worker never ran the
tests/ e2e golden, never saw it red. Fix = full-suite mandate + declare the
golden as a 4th worker file; land it, dispose undeclared `aligned` at audit.
Token cost of the phantom: this whole detour + the halted funnel round.

[dispatch; SL-191-P04-isolation-worktree-fell-back-to-coord]
PHASE-04 Sonnet worker: Agent `isolation:worktree` lost the git repo-lock race
under a busy shared clone (7 stray `agent-*` worktrees, 4 live dispatches
SL-185/186/192/194) and silently fell back to the Bash-cwd worktree — a textbook
ISS-034 / mem.signpost.doctrine.dispatch-claude-arm-wrong-base instance. BENIGN
variant: because orchestrator cwd was parked at COORD@B (not main), the fallback
landed at the CORRECT base b77396cc, so the worker's base-guard
(`merge-base --is-ancestor B HEAD`) passed legitimately and the delta was sound.
Cost: the delta materialised in the coord working tree (not an isolated tree), so
the automated `worktree import` R-5 belt was bypassed; orchestrator recovered by
hand — R-5 by `git status --porcelain` (exactly the 2 declared files, no
`.doctrine/`/`.claude/`, no untracked), full `check regression diff --base B`
(clean), branch-point guard (HEAD==B), single path-limited commit → 22833fbe.
Token/complexity note: the worker's confused self-report ("isolated worktree
/workspace/doctrine/.dispatch/SL-191" — that IS the coord tree) cost a verify
detour to disprove; the missing `worktreePath` footer is the memory's documented
red flag that no isolated tree was created. Serial single-worker funnel + parked
coord cwd + base-guard makes the coord-fallback recoverable, but it is NOT
isolation — a concurrent second worker in the same coord tree would collide.

[/dispatch conclude + /audit; SL-194-conclude-9785ea03]
- `dispatch sync --prepare-review` clap usage line lists all three stage selectors
  (`--prepare-review --integrate --show-journal-trunk-oid`) as "required", so a
  first invocation with only `--slice N --prepare-review` errored demanding the
  other two. They are mutually-exclusive stages, not co-required — the required-args
  error mis-signals. Cost one retry + a --help read. A clap group (one-of) or a
  clearer usage string would remove the false "missing required arg" halt.
- `link ... --intent <role>` is wrong; the flag is `--role` (originates_from etc).
  The `link --help` "intent role refining a references edge" prose primes `--intent`
  but the flag is `--role`. Cost one retry. Minor naming mismatch (prose says
  "intent", flag says "role").
- Untracked backlog dirs need `git add` before a path-limited `git commit -- <dir>`
  (commit pathspec only matches tracked/staged paths). Expected git behaviour, but
  the path-limit-the-commit discipline (AGENTS.md) collides with new-file capture —
  one `add` then `commit -- <paths>`. Not doctrine's fault; noting the two-step.

[/close; SL-194-close-9785ea03]
`check gate` failed RED at close pre-check on a cordage `denylist` test —
`root = env!("CARGO_MANIFEST_DIR")` bakes the BUILD-TIME abs path into the test
binary. The cached `target/debug` binary was compiled on the host
(`/home/david/dev/doctrine/...`), which doesn't exist in the jail
(`/workspace/doctrine`), so the scan's self-guard walked 0 files and panicked.
`touch tests/denylist.rs && cargo test -p cordage --test denylist` (force
recompile → correct CARGO_MANIFEST_DIR) → 3/3 green. Slice-independent (SL-194
doesn't touch cordage). Cost: a spurious RED gate at a close gate, ~2 min to
prove environmental. Root cause: stale cross-environment target cache + env!
compile-time path embedding in a filesystem-walking test.

[preflight + backlog; iss-209-preflight]
- backlog show ISS-209 returned the template path (.doctrine/backlog/issue/209/ISS-209.md) but the actual file is at `backlog-209.md`. CLI `show` output is correct but the template body path hint was wrong — had to use `paths` to find the real file. Minor friction.

[/dispatch + funnel verify beat; SL-191-P05-verify-caught-worker-misattributed-regression]
PHASE-05 (Opus worker, arm-spawn ritual → CLEAN isolation this time: real
`worktreePath` footer, isolated tree — contrast P04's ISS-034 coord-fallback under
contention; the difference was contention clearing + using the full arm-spawn
ritual rather than bare isolation:worktree).

Funnel-integrity WIN, zero marginal cost: the worker self-reported ALL e2e reds as
"ambient worker-marker refusals" and shipped. The funnel's regression-diff beat, run
on the UNMARKED coord tree, found exactly ONE was NOT ambient —
`import_from_worktree_happy_stages_tracked_and_untracked`: the phase's new post-apply
prove gate ran in a happy-path e2e whose temp coord had no justfile → baked
`just prove` failed → import halted. The worker had patched the sibling UNIT helper
(`worker_tree_touching_seed`) but missed the e2e integration binary (an undeclared
file it dismissed under the marker theory).

Hazard identified: the worker's "e2e red == ambient marker" heuristic is CORRECT for
most e2e reds in a marked worktree but LAUNDERS a real gate-induced regression. A
worker running the full suite in its own (marked) tree is structurally blind to
gate-behaviour regressions surfacing in e2e binaries — only the unmarked-coord verify
beat distinguishes. Direct vindication of the funnel doctrine "self-reports are
advisory; trust only the verify beat." Orchestrator reconciled the coupled e2e
test-golden inline (same additive prove-override the worker used in the unit helper) —
same coupling class as P02's golden (import-scope-belt), surfacing one funnel round late.

---

[dispatch; SL-191-P06]

PHASE-06 (Sonnet arm, corpus knowledge refresh: hymn-cascade README + shipped
memory + VT-1). Arm-spawn ritual again gave clean isolation (real worktreePath
footer). Sonnet authored the content faithfully — the trait-space model, seal/
expose symmetry, and provenance-as-ordering-only all correct on first pass; VA-1
content review needed no correction. Distilling the *delivered* model (with the
forward-intent set-valued parts explicitly fenced off) into the worker prompt
paid off — the risk was Sonnet reintroducing model-identity framing, and it
didn't.

TWO worker-contract / import-mechanism gaps surfaced, both orchestrator-absorbed:

F1 — worker-mode marker guard refuses `memory record --global` (and `memory
sync`) INSIDE the worker tree ("worker fork (signal: marker): refusing authored
write"). The canonical shipped-corpus scaffold path is unavailable to a
dispatch worker; the worker hand-authored memory.toml/.md + minted a uid +
symlink by hand, and correctly declined to route around the sandbox guard. Cost:
the master never passed the real CLI `memory validate` in-tree — the orchestrator
had to run sync+validate+show post-import as the true proof (VT-1 lint_master had
already proven schema/INV/scope-floor). A corpus-authoring phase is a poor fit
for the current worker contract; either exempt `--global` authoring from the
marker guard or make corpus authoring an orchestrator beat.

F2 — the claude-arm import (`run_import_from_worktree`) DROPS untracked symlinks.
The new master's `mem.<key>` alias symlink (an authored corpus artifact, git-
tracked for every other master) was NOT in the gathered working-tree delta;
only the uid dir's two regular files came across. Orchestrator recreated +
staged the symlink. Silent gap — no error, just a missing file that would have
shipped a keyless master.

Funnel verify beat earned its keep AGAIN (third phase running): the regression
diff on the unmarked coord caught a real new failure — `e2e_memory_sync::
sync_populates_...` — that the worker's `cargo test --bin doctrine` structurally
could not see (that form runs only the in-binary unit tests, skipping every
`tests/` integration binary). Root cause was a brittle test, not the delta: the
"writes planned" guard `!contains("0 new,")` is a substring false-positive once
the corpus count ends in 0 (grew 29→30 with this master). Same coupled-golden
class as P02/P05, surfaced one funnel round late, fixed inline (anchor on the
": " count prefix). Reinforces: a worker running `--bin doctrine` is blind to
integration-binary regressions; only the coord verify beat (full suite) sees
them. Worker-contract note: the worker SHOULD run the full `cargo test`, not
`--bin doctrine`, but the funnel does not depend on it doing so.
[slice; sl-197-scope] Creating SL-197 from an IMP backlog item. The `link` command's error message lists legal labels as ["references", "references", "references", ...] with duplicates — the three `references` rows differ by role (implements/originates_from/concerns) but the error message doesn't differentiate them, forcing a second guess-then-error cycle to discover --role is required. Mild token waste.

[dispatch/close (orchestrator-author); SL-191-conclude-2026-07-04]
prepare-review bailed "conformance registry incomplete: recorded row for
PHASE-01..07, which is not a completed phase". Root cause: phase-completion
sheets are per-worktree runtime state, and `registry_completeness` reads the
COMPLETED-set from `git::primary_worktree(root)` (state.rs:906 / dispatch.rs:1901),
while an orchestrator-AUTHOR drives `slice phase --status completed` from the COORD
worktree cwd. So all 7 completions landed on coord's sheets; primary's stayed
stale (01 in_progress, 02–07 planned) → every recorded boundary row read as
"Extra" (completed-set empty). record-boundary DOES write primary (source_deltas
rooted on primary), so the boundaries ledger was fine — only the completion flags
diverged. Fix was a manual replay: `slice phase 191 PHASE-NN --status completed -p
<primary>` ×7. Token/complexity cost: a full mechanism dig (state.rs completeness
reader + dispatch.rs gate) to discover the primary-vs-coord asymmetry, then a
7-command reconciliation. For a WORKER-driven dispatch this never bites (orchestrator
flips completion from root); it is specific to orchestrator-AUTHOR phases run with
cwd in the coord tree. Candidate remedies: (a) `slice phase` completion could
also stamp the primary registry the way record-boundary does; (b) prepare-review
could read the completed-set from the dispatch tip like it reads the ledger; (c)
document the orchestrator-author "flip completion with -p <primary>" step.

[phase-plan; SL-195-P02-a]
Live-probed `claude plugin list` + `marketplace list` output formats during
phase-planning because the F-4 exact-match parser design needs the token shape,
and notes.md captured `marketplace list` (`Source: Directory (/abs)`) but NOT
`plugin list`. One extra live shell-out round-trip. Minor; ground truth now in
the phase sheet. Root cause: PHASE-01 notes recorded the marketplace-list format
(it was the R4 lever) but the plugin-list format was only relevant to PHASE-02's
F-4 fix, so it wasn't carried forward. Handover reading-list pointed at the seam
(install.rs:527-533) but not the runtime format.

[inquisition; SL-196-ext-2026-07-04]
External adversarial review (codex mcp) is high-signal but every finding needed
independent line-number re-verification before integration — codex cites precise
`file:line` that mostly held, but the trust model (CLAUDE.md: subagents/reviewers
hallucinate params) mandates re-reading each site. Cost: ~8 targeted Reads to
ground-truth 4 findings. Not incidental complexity — inherent to external review —
but the token floor of an inquisition is "review output + full re-verification",
not "review output" alone. Cheap win observed: the design's own precise `:line`
citations (RelationEdge:735, hydrate:278) made verification O(1) lookups vs
open-ended grep — dense line-citation in a design pays back at inquisition time.

[dispatch/close (orchestrator-author); SL-191-close-2026-07-04]
Orchestrator-author close required repeatedly resolving WHICH TREE is canonical
for each write — a recurring token tax with no in-flow signal:
- Slice lifecycle (audit/reconcile/done), the RV ledger, notes/design/selector
  reconcile edits → all EDGE (parent tree); the review verbs even refuse a
  worktree-resolved root (review-ledger §6). But `slice status` auto-detects root
  from cwd, so running it from the COORD tree silently wrote the coord copy —
  producing an edge=ready / coord=audit split I had to detect and realign with
  `-p <edge>`. A `slice status` that warned "you are transitioning a lifecycle on a
  dispatch worktree, not the parent" would have saved a dig.
- The integrate topology: `main` EXISTS but had DIVERGED from edge (edge +31 / main
  +2 from a parallel slice's landing), so the AGENTS.md pre-dispatch `git fetch .
  edge:main` promotion is WRONG at close (non-ff) — close integrates the admitted
  close_target onto CURRENT main via ff-CAS, independent of edge. This isn't stated
  where the close operator is looking; the close skill's step 3a shows the candidate
  workflow but not "do NOT promote edge→main here."
- `git rev-parse --short A B C` aborts the whole command if ANY ref fails to
  resolve ("Needed a single revision"), which misled me into briefly believing
  `main` was absent. Single-ref checks are reliable; multi-ref --short is a footgun.
- ISS-030 verify (a) `git diff --quiet HEAD` false-failed on an unrelated
  concurrently-STAGED `flake.nix` from another agent (shared index) — the check is
  whole-tree, so any parallel agent's staged file trips it even though the pure-ref
  integrate never touched the checkout. (b) journal-trunk-oid is the trustworthy
  signal. A shared-index repo makes (a) noisy for orchestrator-author close.
Net: the close mechanics are sound and fail-closed, but an orchestrator-author (no
worker) pays a large navigation tax the worker-driven path never sees.
