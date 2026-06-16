# Implementation Plan SL-076: Load concept maps into the Map Explorer and ship a web authoring surface

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Six phases: three Rust (engine → routes → adversarial tests), three frontend
(model+rendering → authoring → diagnostics+polish). The Rust and frontend
tracks diverge after PHASE-02 (routes exist) — PHASE-03 (adversarial Rust tests)
and PHASE-04 (frontend model+rendering) are file-disjoint and can run in
parallel. The frontend phases are sequential (each builds on the prior UI layer).

## Sequencing & Rationale

### PHASE-01: Catalog registration + concept_map.rs engine changes

Foundation. Registers CM in the entity catalog so the scan picks it up, adds the
outbound_for arm (empty), promotes concept_map.rs symbols to pub(crate), defines
the typed `ConceptMapMutationError` enum, and extracts three pure mutation
functions from the CLI shell verbs. Everything else depends on this.

The typed error is co-located with the pure functions — it's part of the engine
contract, not the HTTP layer. The `From` impl that bridges it to `MapServerError`
lives in PHASE-02 where the HTTP error type is extended.

The CLI shell verbs (`run_add`, `run_remove`, `run_rename_node`) are left
as-is — they are thin I/O wrappers and the duplication is intentional. The pure
functions are new; the CLI verbs become callers alongside the web routes.

### PHASE-02: Map server routes + error mapping

Depends on PHASE-01 (pure functions + typed error exist). Adds `MapServerError`
variants, the `From` impl, and two routes: GET (structured data + diagnostics +
dsl_hash) and POST (three actions + stale-write guard). The stale-write guard is
implemented here — GET computes SHA-256 of the DSL, POST checks optional `base_hash`.

The markdown path verification test is included in this phase because it
exercises the existing `read_entity_markdown` path with a CM entity — a latent
bug surface that the design explicitly calls out.

Route handler structure follows the thin-wrapper pattern: parse → call engine
function → map error → assemble response. No DSL semantics live in routes.

### PHASE-03: Rust adversarial + preservation tests

Depends on PHASE-02 (routes exist and respond). File-disjoint with PHASE-04
(frontend) — can run in parallel. Covers all adversarial cases from design §8:
hostile labels, TOML preservation, DSL comment preservation, collision edge
cases, malformed DSL, bad discriminators, I/O error mapping.

These tests are substantive — they don't duplicate the standard route tests from
PHASE-02; they probe the risky boundary conditions the review identified. The
preservation tests are particularly important: they prove the "don't vandalize
authored files" contract holds for every mutation path.

### PHASE-04: Frontend model, API client, and DOT diagram rendering

Depends on PHASE-02 (API surface stable). First frontend phase — builds the data
layer (cache, API client, normalizer), CM detection, DOT generation with the
escape helper, SVG wiring, hover pane, and render dispatch. At the end of this
phase, a focused CM entity renders its diagram in view mode with working hover.

The DOT escape helper contract is implemented here: `escapeStringContent` escapes
content but does not add surrounding quotes. The caller adds the enclosing double
quotes. If the existing `dotQuote` already returns quoted strings, extract the
bare-escape function as a new helper.

### PHASE-05: Frontend authoring UI

Depends on PHASE-04 (diagram renders in view mode). Builds the full authoring
surface: add edge form with autocomplete datalists, edge table with remove
buttons, inline node rename (click edge table label or SVG node), Edit/Done
toggle, and stale-write handling (base_hash sent with every POST, 409
auto-refetches).

Autocomplete datalists are rebuilt from the cache after every successful
mutation — zero extra API calls. Client-side trim validation prevents empty-field
submissions before they reach the server.

### PHASE-06: Diagnostics panel, JS tests, HTML/CSS polish, and integration

Depends on PHASE-05 (full authoring UI works). Final phase — diagnostics panel
rendering (view mode only, hidden when empty), cache invalidation, error state
rendering, CSS styling, HTML containers, JS test suite, and integration smoke.

The diagnostics panel is the last UI piece because it's read-only and depends on
the GET response shape being stable. The JS test suite covers all modules.
Integration smoke proves end-to-end: CLI creation → browser viewing → browser
editing → CLI verification.

### Parallelism opportunity

PHASE-03 (Rust adversarial tests) and PHASE-04 (frontend model+rendering) are
file-disjoint — they touch `src/` (tests) vs `web/map/` (JS). Both only depend
on PHASE-02. They can run as a concurrent batch under `/dispatch`.

## Notes

- Phase ids and criterion ids are immutable per the glossary reference forms.
- PHASE-03 and PHASE-04 can be dispatched in parallel if using multi-worker mode.
  They are file-disjoint (PHASE-03 touches `src/` tests only; PHASE-04 touches
  `web/map/` only). PHASE-03 must not add or modify `Cargo.toml` dependencies on
  the parallel track — if new test deps are needed, move them to PHASE-02.
- Do not split `concept_map.rs` into submodules during this slice — the module
  seam guard in the design is a future concern, not an implementation target.
- PHASE-06 is broad (diagnostics + tests + CSS + HTML + integration). This is
  intentional for a final polish phase, but note the breadth risk — a failure in
  any sub-area blocks the entire phase.
- The `just gate` exit criterion on every Rust phase ensures the workspace stays
  clean — clippy zero warnings, no regressions.
- The CLI shell verbs (`run_add`/`run_remove`/`run_rename_node`) are not modified
  in this slice. The CLI collision check follow-up is deferred (design §10).
- `rename_node_in_dsl` matches source/target labels by **derived key**, not
  case-insensitive label equality (the CLI `run_rename_node` uses case-insensitive
  matching — this is a deliberate semantic difference; the CLI will gain key-based
  matching in a follow-up).
- PHASE-04 VA-1 (manual smoke) is a quality check, not a hard gate — PHASE-05
  can begin after PHASE-04 VT-1 passes.

### Review-driven corrections (RV disposition)

This plan was updated after an adversarial review that identified:
- Visibility list undercount (7 → 13 symbols with fields)
- Missing-DSL graceful handling (was 500, now 200 with empty data)
- `set_dsl` drops inline comments on the `dsl` key (accepted tradeoff, documented)
- `sha2`/`hex` crate dependency verification added
- TOML field preservation added to PHASE-01 EX-8
- `graphRenderSeq` stale-render guard test added to PHASE-04
- `description` field in GET response added to EX-3
- `kindOrder.CM` entry added to PHASE-06 EX-6
- Escape helper character list made explicit (EX-6)
- Parallel Cargo.toml collision risk noted
- `ConceptMapIoError` variant added for I/O error mapping
