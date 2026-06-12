# Requirement status visibility: spec req roster + standalone drift read

## Context

SPEC-002 / PRD-013 (Requirement Reconciliation Engine) shipped its compute and its
write paths — SL-042 (observe substrate) and SL-044 (reconcile writer + closure
gate), both `done`. The derived-read machinery exists as library code:
`coverage::composite()` (the per-requirement fold) and
`coverage::drift(authored, &composite) → Verdict{Coherent, Divergent(reason),
Indeterminate}` (SL-042 A·P3, realising `REQ-110`/`REQ-111`), over
`coverage_scan::scan_coverage`.

But nothing **surfaces** those reads to a user. `drift()`'s only callers are
*write* paths: `doctrine reconcile` (a per-requirement prompt, SL-044 design §5.4
step 1) and the `slice status` closure gate. There is no read-only command, and
`doctrine spec req` exposes only `add` / `status` — no `list`. `spec show` prints a
requirement's authored status inline (`slug · kind · status`) but buried in verbose
per-requirement prose, not a scannable roster, and answers neither "which of this
spec's requirements are implemented?" nor "where does authored status diverge from
observed coverage?".

Consequence, observed on doctrine's own corpus: every SPEC-002 requirement
(`REQ-108`..`REQ-116`) is still `status = pending` despite the engine being shipped,
and that drift is invisible because no read exposes it. `REQ-111` ("surface drift as
a derived read … *reported* as drift") is only half-discharged — the fold exists, the
report surface does not — and its authored status is itself `pending`.

This slice is the read-surface follow-on to the SPEC-002 A→B roadmap
(`mem.signpost.spec-002.slice-roadmap`): pure surfacing over existing seams. It
descends from **SPEC-002** / **PRD-013** and completes the user-facing half of
`REQ-110` / `REQ-111`. No new engine, no new store.

## Scope & Objectives

1. **Requirement roster** — `doctrine spec req list <SPEC>`: a scannable table of a
   spec's members — `REQ-NNN`, membership label (`FR-/NF-NNN`), kind, authored
   status. Rides the shared listing column model (SL-037, `src/listing.rs`) and the
   uniform list/show/filter/render contract (SL-025): `--columns`, `--json`.

2. **Standalone drift read** — a read-only command surfacing, per requirement,
   composite observed coverage and the `drift()` verdict (authored vs observed:
   Coherent / Divergent(reason) / Indeterminate). Completes the user-facing half of
   `REQ-110`/`REQ-111`. **No authored write.** Command placement and scope unit are
   `/design` questions (see OQs).

3. **Reuse only.** Both read paths ride `coverage::composite`, `coverage::drift`,
   `coverage_scan::scan_coverage`, `requirement::ReqStatus`, and the listing column
   model. The `coverage` / `reconcile` engine is not modified.

## Non-Goals

- **No status writes.** `doctrine reconcile` and `spec req status` already author
  status; this slice only reads.
- **Not the corpus dogfood run.** Moving SPEC-002's `pending` requirements to their
  true status is `CHR-002`, downstream of this read surface.
- **No spec/requirement-text revision vehicle** (`IDE-003`).
- **No mass-divergence Drift Ledger** (`IMP-022`) — that is a separate kind.
- **No new coverage-recording surface** — writing coverage entries is SL-042's
  concern, untouched here.

## Affected surface (concrete)

- `src/coverage_view.rs` *(new leaf)* — the derived read: ref dispatch, `CoverageRow`
  materialisation, the `observed_state` classifier, column model + JSON rows.
- `src/spec.rs` — `spec req list` subcommand + render (requirement column model here,
  mirroring `SPEC_COLUMNS`) **+** one new `pub(crate) member_reqs` seam for the fan.
- `src/main.rs` — wire the top-level `coverage` leaf + `spec req list`.
- `src/listing.rs` — generic `Column`/`select_columns`/`render_columns` **reused
  unchanged** (`mem.pattern.listing.column-model-extension`: pre-materialised typed row,
  `const` non-capturing `fn(&R) -> String` extractors, JSON per-kind typed). No edit.
- `src/coverage_scan.rs` — **+1 additive** `scan_coverage_batch` (one walk, bucket by req);
  `scan_coverage` behaviour unchanged.
- `src/coverage.rs` — **+** terse `Verdict::label()` display helper; the pure fold untouched.
- `src/requirement.rs` — reuse `load` / `ReqStatus` / `ReqKind` / `CoverageStatus`.

## Risks, assumptions, open questions

- **OQ-1 (placement).** RESOLVED (design D1) → top-level `doctrine coverage <ref>`, a
  noun-as-verb leaf. Not `doctrine drift` (name-collides with SL-009's slice-rollup `⚠`),
  not `spec req drift` (reads wrong for a polymorphic spec ref).
- **OQ-2 (scope unit).** RESOLVED (design D2) → polymorphic `<ref>`: `REQ-NNN` single
  row, spec ref (`PRD-/SPEC-NNN`) fans over members in `order`.
- **OQ-3 (roster columns).** RESOLVED (design D3) → `spec req list` stays authored-only;
  the observed/verdict join lives **solely** in `doctrine coverage`. Tier split by
  command boundary is the strongest defence of the `NF-001`/`REQ-114` wall — no shared
  output path to blur, and the roster stays scan-free.
- **RSK (perf).** RESOLVED (design D4) → a new additive `scan_coverage_batch` collapses a
  spec fan to **one** corpus walk (not N), bounding the `RSK-006` amplification; staleness
  stays one-HEAD-resolve. Accepted for v1 (no cliff).
- **ASM.** `composite()`/`drift()` are pure and read-only by construction, reusable
  as-is for a read path with no signature change.
- **Behaviour-preservation gate.** The SL-044 `NF-001` no-derivation import-edge
  proof and the existing coverage/reconcile suites must stay green unchanged — this
  slice adds a read, it does not re-route the write wall.

## Summary

Surface what the reconciliation engine already computes. Two read-only additions —
a requirement roster and a drift read — over existing pure seams, discharging the
user-facing half of `REQ-110`/`REQ-111`. Descends from SPEC-002 / PRD-013.

## Follow-Ups

- `CHR-002` — reconcile SPEC-002's own `pending` requirements once this read lands.
- `IDE-003` — requirement/spec-prose revision vehicle (distinct; not this slice).
