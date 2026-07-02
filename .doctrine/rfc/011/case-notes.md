
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
