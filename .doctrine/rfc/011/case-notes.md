
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
