# Audit — SL-025 Uniform DRY CLI surface

**Mode:** conformance (post-implementation, six phases shipped).
**Date:** 2026-06-08.
**Inputs:** `design.md` (canonical, §5 contracts + A-1..A-10 + OQ resolutions),
`plan.toml` (per-phase EX/VT), `slice-025.md`, ADR-001 (module layering),
`doc/slices-spec.md` (amended vocab). Behaviour observed via the **dev binary**
(`./target/debug/doctrine`) — the installed `~/.cargo/bin/doctrine` is stale and
must not be used for evidence.

## Evidence baseline

- All six phases `completed`; `slice list` rollup reads `6/6 ⚠` (the `⚠` is the
  expected `proposed`-vs-complete divergence `/close` reconciles).
- `just check` green: plain `cargo clippy` **0 warnings**, `cargo fmt` clean.
- `cargo test` green: **656 tests** (638 bin unit + 18 e2e), 0 failed.
- Behaviour-preservation suites (entity 24, registry 25, `is_divergent` /
  `is_terminal_status`) green and byte-unchanged — all SL-025 diffs to those
  paths are insertion-only.

## Conformance — EX/VT coverage

Every phase's exit + verification criteria are met and test-backed. Spot-proof:

- **Spine (PHASE-01):** `listing.rs` is clap-free + entity-free (ADR-001
  leaf-clean); `retain` matrix, two independent match domains (A-1), `build`
  `--json`-over-`--format` fold (A-9), regex pre-compile error, empty→header
  suppressed — all unit-tested.
- **adr / slice / spec / backlog / memory list+show (PHASE-02..05):** all on the
  shared `CommonListArgs` + `listing::build` + `retain` + `render_table` +
  `json_envelope` spine; prefixed ids everywhere except the memory uid exception;
  single `{kind, rows}` JSON envelope per kind (spec rows carry `subtype`, A-8;
  slice `phases` structured not rendered, OQ-1; backlog positional is a
  deprecated `--filter` alias with `--filter` winning, A-7).
- **Audit-sensitive invariants (independently confirmed in code, not just tests):**
  (a) memory uid is the canonical id, NOT routed through `canonical_id`, json
  carries `uid`; (b) boot memory section is active-only via an explicit
  `status:["active"]` predicate, decoupled from the list default (no draft leak).
- **slice markers:** three separate predicates never conflated — `is_drifted`
  (out-of-vocab stored status → `?`, never hidden), `is_divergent` (→ `⚠`),
  hide-set `{done, abandoned}`. `is_terminal_status` / `is_divergent` unchanged
  (`{done}` only).
- **Conformance (PHASE-06):** parse-conformance over all five kinds
  (`tests/e2e_list_conformance.rs`); ordering-preservation per kind with
  unsorted fixtures; OQ-3 short-flag collision audit — CLEAN (only `--kind` /
  `--type` are kind-specific, both long-only; `-p` is the root locator).

## Findings

### F-1 — backlog `list` table emits no header row — **fix now**
- **Expected** (design §5.5): "Rows present → header row … extends header to
  adr/backlog/memory." The uniform contract this slice exists to establish.
- **Observed** (`./target/debug/doctrine backlog list`): data rows only, no
  `id kind status slug title` header. adr/slice/spec/memory all emit one.
- **Evidence:** `src/backlog.rs` `format_rows` builds the grid with no header
  row; no test asserts a backlog header (the conformance test checks only the
  JSON envelope, so the gap was uncaught).
- **Disposition:** fix now — add the header (TDD: red test first) + the
  `is_empty → ""` guard so the §5.5 empty-list contract still holds.

### F-2 — backlog `is_hidden` doc-comment names the wrong kind — **fix now**
- **Observed:** `src/backlog.rs:218` documents the predicate as "the SL-025
  `spec list` hide-set"; it is the backlog hide-set (reuses `Status::is_terminal`).
- **Disposition:** fix now — trivial comment correction (s/spec list/backlog/).

### F-3 — adr `render_table` empty-guard ordering — **fix now**
- **Observed:** `src/adr.rs` builds the full header+rows grid, then checks
  `is_empty`; memory/slice guard first. Harmless but inconsistent and wasteful.
- **Disposition:** fix now — hoist the empty guard ahead of grid construction
  for consistency with the other kinds.

### D-1 — `meta::sort_and_filter` fully removed vs design prose — **design was wrong**
- **Expected** (design §5.1/§5.3 prose): a thin `meta` sort-by-id helper survives
  for the numeric kinds.
- **Observed:** the helper is fully removed (zero refs); every kind sorts inline
  via `sort_by_key`. Ordering is correct and tested for all five kinds
  (id-asc; backlog `(kind.ordinal, id)`; memory created-desc+uid) with
  intentionally-unsorted fixtures.
- **Assessment:** benign — the design *intent* (kinds own their ordering) is
  honoured; only the prose claiming a surviving helper is stale.
- **Disposition:** design was wrong → reconcile `design.md` §5.1/§5.3 prose to
  match (helper retired; kinds sort inline). No code change.

### F-4 — new e2e files use module-level `#![allow(...)]` not `#[expect]` — **aligned**
- **Observed:** `tests/e2e_*.rs` (the three new files) carry module-level
  `#![allow(clippy::expect_used, unwrap_used, tests_outside_test_module)]`.
- **Assessment:** matches every pre-existing e2e file; the gate is plain
  `cargo clippy` (bins/lib only), and not every line triggers the lint so
  per-line `expect` is infeasible. Not a violation.
- **Disposition:** aligned — no action.

## Closure-intent checklist (design §closure)

- [x] `show` resolves for all five kinds (table + json).
- [x] all list/show emit canonical prefixed ids (memory uid exempt by design).
- [x] default list hides terminal; `--all` / explicit `--status` reveal.
- [x] a shared filter base + shared renderer (human + json) back every kind.
- [x] the create-verb reconciled (`memory new` alias of `record`).
- [x] no bespoke list filter flags reintroduced (OQ-3 clean).

## Harvested durable knowledge

- Stale installed CLI footgun: `~/.cargo/bin/doctrine` lags the working tree;
  audit/verify evidence MUST use `./target/debug/doctrine`. (Already covered by
  `mem.pattern.testing.stale-cargo-bin-exe` family — confirm, don't duplicate.)
- Conformance tests that assert only the JSON envelope do NOT prove the table
  surface — F-1 slipped through exactly this gap. A uniform-contract slice needs
  a per-kind *table-header* conformance assertion, not just envelope parity.

## Closure readiness

Amber → green after F-1/F-2/F-3 fixed and D-1 design prose reconciled. No
BLOCKERs, no follow-up slice, no tolerated drift. Hand to `/close` once the
fixes land green.
