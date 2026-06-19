# Review RV-100 — reconciliation of SL-103

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (post-implementation, self-audit; both roles via `--as`).

**Surface reviewed:** `candidate/103/review-001` — the impl_bundle (`review/103`)
3-way merged onto current `main` via `dispatch candidate create`. The raw
`review/103` base was stale (diff vs main showed large unrelated deletions —
`e2e_mcp_server.rs`, slices 026/109/117, backlog items — all base skew, not
SL-103). The candidate isolates the real delta: 7 files, +784/−28.

**Lines of attack & invariants held:**
- Contract shape — top-level `units{estimation,value}`; per-node
  `estimate{lower,upper}` + `value{value}` behind `Option`/`skip_serializing_if`
  (design §4.5; VT-1/2/7).
- Per-facet malformed isolation — `Error` diagnostic + that-facet `None`, sibling
  + node intact, **no bound coercion** (D4; VT-4).
- Unit resolution — NotFound-only default; every other read error + all parse
  errors propagate (RV-094 F-4; §5.4; VT-3).
- `from_scanned` stays pure — `units` is an input, no disk read (D2; §5.3).
- Kind-agnostic read — `[estimate]` on a non-slice TOML surfaces (VT-6).
- Behaviour preservation — hydrate/graph/map_server suites green unchanged (§6.3).
- Dispatch-deferred attestations (verify, don't assume): VA-1 structural
  non-blocking (NF-001), VA-2 vocabulary denylist.
- Governance carried to reconcile (§7): value-graph-exposure orphan REQ, SL-103 →
  REQ-280 trace, D2 unit-literalism ratification.

## Synthesis

**Closure story.** SL-103 wires the SL-101 estimate/value facets onto the
scan/catalog/graph path through a clean, additive, policy-free projection. The
audit reviewed the candidate interaction surface (impl_bundle on current main) and
holds the slice fully conformant to the RV-094-locked design:

- **Read tier (scan.rs).** `read_facets` + the generic `parse_facet` helper read
  `[estimate]`/`[value]` kind-agnostically with strict per-facet isolation — a
  malformed *present* facet pushes one `Error` diagnostic and drops *that* facet to
  `None` (no coercion), leaving the node and the sibling intact; a non-table value
  is fail-loud, not silent-absent (D4 honoured). VT-1..4 cover present/absent,
  isolation, non-table, and kind-agnostic-on-ADR.
- **Hydrate tier (hydrate.rs).** Facets carry `ScannedEntity → CatalogEntity`
  (memory → `None`); `from_scanned` takes `units` as an input and performs no disk
  read (purity preserved). `resolve_units` mirrors `coverage_store::load_config`
  faithfully — NotFound→default, parse error and non-NotFound read error both
  propagate. The directory-as-`doctrine.toml` test pins the non-NotFound arm
  (RV-094 F-4 closed in code).
- **Graph tier (graph.rs).** `from_catalog` projects both facets +
  `units` onto `CatalogNode`/`CatalogGraph`; `skip_serializing_if` omits absent
  facets from the wire. VT-5/6/7 + round-trip durability are tested.
- **Surface (routes.rs).** `/api/graph` serves `CatalogGraph` raw, so facets +
  `units` surface there — confirmed by an end-to-end test (RV-094 F-3 in scope);
  `/api/map`'s `{key,label}` DTO is untouched.
- **Dead-code cleanup (§5.6).** The now-fulfilled `expect(dead_code)` is removed
  from `estimate`/`value` `parse_optional` + `resolve_unit` and `dtoml`
  `DoctrineToml::{estimation,value}`; the SL-102-owned confidence symbols
  (`DEFAULT_*_CONFIDENCE`, `resolve_confidence`) correctly retain theirs. `cargo
  clippy` is zero-warning (no unfulfilled-expect — PHASE-03 EX-2 met).

**Deferred attestations discharged.** VA-1 (F-5) and VA-2 (F-6), carried forward
as agent-attested from dispatch, were both verified clean in this audit — VA-1 by
absence (facet types are referenced only in the facet defs, the SL-103 projection,
the SL-101 show path, and the still-dead SL-102 display module; no
dispatch/execute/audit/close predicate reads facet presence), VA-2 by checking the
shipped field vocabulary against the SPEC-001 Appendix B whole-word denylist.

**Verification.** `cargo test` green (exit 0; dispatch logged 1904), `cargo clippy`
zero-warning. `just check`'s `lint-js` step fails in the candidate worktree purely
for a missing `web/map/node_modules` — an environmental artifact of the fresh
worktree; SL-103 touches no JS.

**Standing risks / tradeoffs consciously accepted.**
- Malformed-present vs absent facet are indistinguishable on the wire (both omit
  the key); corruption is observable only out-of-band in the `Error` diagnostic
  stream — the conscious v1 contract (D4; RV-094 F-2).
- `EstimateFacet`/`ValueFacet` *are* the external `/api/graph` contract (served
  raw), so they are change-controlled types, not free-to-mutate internals (D5;
  RV-094 F-7). A serialisation-DTO seam is deferred.
- `read_facets` does a second per-entity TOML parse + carries a benign
  divergent-read window (F-4, `aligned`) — the design-recorded single-parse refactor
  is filed as **IMP-109**, not a defect.

**No blockers.** The ledger is `done · await=none`; the close-gate is clear.

## Reconciliation Brief

All governance findings are confirmed observations whose *remediation* is
reconcile's to write — audit changes no spec/governance. Per-slice prose is already
accurate (design §7 pre-records these), so the brief is governance-only.

### Per-slice (direct edit)
- None. `design.md` already documents D1/D2/§7 faithfully; the implementation
  matches it. No prose correction needed.

### Governance/spec (REV) — all tracked under CHR-011
- **RV-100 F-1 — value-graph-exposure orphan REQ.** Author a value-graph-exposure
  requirement (sibling in intent to REQ-274), choose its spec home (SPEC-020 is
  titled "Estimate graph exposure" → rename/extend SPEC-020, or mint a sibling
  spec), and bind SL-103 to it. The CLI cannot mint an un-homed REQ, so this is
  reconcile's to do, not audit's.
- **RV-100 F-2 — SL-103 → REQ-280 trace.** `scan_catalog`/`resolve_units` realises
  REQ-280 (value unit resolution, §5.4); add the SL-103 → REQ-280 requirement edge.
- **RV-100 F-3 — D2 unit-literalism ratification.** Ratify that REQ-274's "project
  unit" is satisfied by top-level reachability (units are project-wide constants
  reachable from every node via the held graph), not literal per-node duplication.
  No code change — a governance ratification.

### Harvest (filed, not reconcile's)
- **IMP-109** — single-parse refactor of the catalog scan (RV-100 F-4 `aligned`).
