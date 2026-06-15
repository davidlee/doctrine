# Implementation Plan SL-072: Doctrine Map Server: local web graph explorer

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Eight phases, strictly sequential. The plan builds the map server from its
error-model foundation outward through assets, markdown lookup, and the
graphviz bridge, then assembles the HTTP surface, wires the CLI, drops in
the browser placeholder, and closes with a quality gate.

Each phase is file-disjoint where possible (see module layout in design §1:
`error.rs`, `state.rs`, `assets.rs`, `markdown.rs`, `shell.rs`, `routes.rs`,
`open.rs` are separate files). The only shared write target is
`src/map_server/mod.rs`, which accumulates `pub mod` declarations as modules
are added — each phase adds its own line.

## Sequencing & Rationale

**PHASE-01 Foundation** must come first. MapServerError and its IntoResponse
impl are the error substrate every handler and helper returns through.
AppState and Config define the shared state shape. The DotRenderer trait
declares the sole abstraction seam. Cargo.toml changes unblock compilation.
Without this phase, nothing else compiles.

**PHASE-02 Asset serving** is next because it has no internal dependencies
beyond the error model. RustEmbed + content_type_for is a self-contained
concern. It also forces the web/map/ directory to exist early (even with
placeholder files), which the RustEmbed proc macro needs at compile time.

**PHASE-03 Markdown lookup** depends on the error model (MapServerError
variants) and the catalog's entity kind registry (`integrity::KINDS`,
`kind_by_prefix`). It is the most path-sensitive module — the adversarial
review surfaced several sharp edges here (REQ → 501 via dedicated variant,
IO error discrimination, honest ownership claims). Implementing it early
with thorough tests prevents these from becoming route-level bugs.

All kinds use the same `kind.dir` + `kind.stem` path convention — no separate
memory path helper is needed. Knowledge records (ASM/DEC/QUE/CON) live at
`.doctrine/knowledge/{assumption,decision,question,constraint}/NNN/record-NNN.md`,
which is exactly the `kind.dir`/`kind.stem` pattern. The `entity_md_path`
function is self-contained in `map_server::markdown`; no catalog or memory
module changes are required.

**PHASE-04 Graphviz bridge** depends on DotRenderer trait and the
process-failure error variants (ToolUnavailable, CommandFailed, Timeout).
The FakeDotRenderer provided here becomes the test double for route
tests in PHASE-05. The real process spawn is isolated in shell.rs and
conditionally tested — if `dot` is absent, those tests skip rather than
fail.

**PHASE-05 HTTP routes** is the integration point. It assembles the router
and all seven handlers, consuming assets, markdown, shell, and state.
This is the largest phase by test count — the route integration test table
(design §8.2) covers every endpoint's happy path, error path, and content
type. The test_app fixture reuses catalog `test_helpers` (seed_slice,
seed_requirement, etc.) from SL-071.

**PHASE-06 CLI entry** wires the outer shell: clap args, root detection,
server startup, URL construction, and browser open. It depends on the
router being complete (PHASE-05) because `serve()` constructs the router
and binds the listener. URL construction is pure and tested independently;
browser-open failure is non-fatal per the design's revised contract.

**PHASE-07 Browser placeholder** is last among implementation phases
because it exercises the Rust server end-to-end but doesn't affect Rust
compilation. It's verified manually (VH-1) plus an automated check that
the embedded assets are reachable (VA-1). The placeholder is deliberately
minimal — the Rust server is the SL-072 deliverable; full interactive UX
is follow-up.

**PHASE-08 Gate** is the quality close. It runs the full suite, clippy,
fmt, and just gate. The agent verifications (VA-1, VA-2) are design-
conformance checks: no duplicated graph policy in route handlers, module
deps point downward per ADR-001, and the slice scope still matches the
implemented reality.

## Phase Dependency Graph

```
PHASE-01 ──┬── PHASE-02 ──┐
           ├── PHASE-03 ──┤
           └── PHASE-04 ──┤
                          ├── PHASE-05 ── PHASE-06 ── PHASE-07 ── PHASE-08
                          └── (PHASE-07 also depends on PHASE-05 for API endpoints)
```

PHASE-02, 03, and 04 are parallelizable in principle (file-disjoint beyond
mod.rs lines), but the plan keeps them serial to avoid merge friction on
mod.rs and to let each phase's tests inform the next.

## Notes

- The `src/map_server/mod.rs` file is the only shared write target across
  phases. Each phase that adds a module appends its `pub mod` declaration.
  Phases should use minimal, non-overlapping additions.
- Test helpers from `catalog::test_helpers` (SL-071) are reused — no new
  fixture infrastructure.
- The `webbrowser` crate is the sole new dependency. All other crates
  (tokio, axum, tower, http-body-util, rust-embed) are already workspace
  deps, just commented out.
- `just check` (root package only) is the fast inner-loop variant during
  development; `just gate` (workspace) is the commit gate.
