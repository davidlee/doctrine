
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
