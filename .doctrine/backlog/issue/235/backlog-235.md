# ISS-235: vt9 surface test non-hermetic on ambient CLAUDE_PROJECT_DIR

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`memory::ambient_surface_tests::vt9_no_discoverable_root_emits_nothing` asserts
"no discoverable root ⇒ emit nothing" by passing a fake, non-canonicalizable cwd.
But `discover_surface_root` (memory.rs) falls back to the ambient
`CLAUDE_PROJECT_DIR` env when the cwd fails to resolve, then surfaces from that
root's live memory corpus (deduped by the runtime seen-set). So vt9 reds whenever
the test process inherits a `CLAUDE_PROJECT_DIR` pointing at a real root whose
corpus happens to surface a memory for vt9's input (session `s9`, changed
`src/x.rs`). It passes in a bare env (regression capture at coord tree = 0
failures) but false-reds inside a dispatch worker fork whose gate inherits the
harness `CLAUDE_PROJECT_DIR` — a corpus/seen-set-dependent flake across fork
contexts. This blocks the claude dispatch arm's `worker_commit` gate on deltas
100% unrelated to memory surfacing.

Discovered during SL-204 PHASE-04 (RFC-011 case-note
`SL204-a15d-P04-vt9-gate-falsered`). Fixed opportunistically in that phase's
worker delta (point the cwd at a rootless `tempdir` so discovery resolves it and
finds no marker, never reaching the env fallback — hermetic regardless of
environment). This item records the defect + the broader signal: `worker_commit`'s
binary full-suite gate should run the funnel's B-vs-S differential (which cancels
persistent/ambient reds) rather than a bare pass/fail, and sibling ambient tests
should pin an empty corpus / guard the env.
