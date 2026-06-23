# Notes SL-145 — Backlog relation source parity

Durable close-out harvested from the (disposable) phase-01 sheet and the RV-146
reconciliation audit. Narrative only; structured truth lives in the TOMLs.

## Outcome

RFC-003 Axis A closed by the smallest legal-set widening: BACKLOG
(ISS/IMP/CHR/RSK/IDE) added to the `sources` of two `RELATION_RULES` rows in
`src/relation.rs` — `governed_by` (target gate `Kinds(GOV)` unchanged) and the
existing `related` `[SL, RFC]` AnyNumbered row (D1: extend, not a new row). A
backlog item can now structurally express "governed by ADR-010" and "related to
RFC-003" through `doctrine link`, with the validator accepting it and `inspect`
rendering the derived inbound reciprocal. Shipped `feat 73c56a8`, merged to
`edge` via `4f490374`.

## What made it a one-file change

Both seams were already kind-generic: `backlog::relation_edges` returns
`item.tier1` (every legal row, canonical order — no per-label branch), and
`run_link`/`run_unlink` → `append_edge` carry **no source-kind allowlist**.
`validate_link` is the sole legality gate, so widening the table is the whole
code change (ADR-010 D2 `sources: SET`; ADR-004 outbound-only). No consumer
reacts (D3, RFC-003 Layer-1 — graph-effect deferred).

## Golden churn (all intended — behaviour-preservation gate held)

4 + 1 flips: `read_block` (backlog `governed_by`/`related` now emit; kept an
`IllegalForSource` demo via a `requirements` row), VT-2
`sources_match_shipped_accessors` `Related` set + its doc-comment, the `lookup`
`is_none`→`is_some` flip, the new `validate_link` positives + a backlog
target-gate negative — plus the `relation_graph` VT-4 ISS fixture authoring the
newly-legal `governed_by` axis (its "one edge of every legal axis" contract).
`just gate` exit 0. R1 grep for backlog-kind label special-casing outside
`relation*.rs`: empty.

## Verification

- VT-1/VT-2: unit goldens + full root suite green, monotonic-widening invariant
  proven by the untouched suite staying green.
- VH-1 e2e (fresh fork binary): `link CHR-024 governed_by ADR-010` + `related
  RFC-003` → persisted `[[relation]]` rows, outbound on `inspect CHR-024`,
  derived inbound (`governs`/`related`) on the targets, `unlink` round-tripped to
  0. Re-confirmed at audit: CHR-024 carries no residual rows.

## Audit (RV-146)

6 findings, all terminal: F-1..F-5 `aligned` (1:1 conformance with design
D1/D2/D3 and plan EX-1..EX-7), F-6 `fix-now` — tidied the stale SL-095 doc
comment at `relation.rs:1717`. No blocker. Reconciliation brief empty by
construction: no design/governance artifact needed amendment. See
`review/146-reconciliation-review-of-sl-145/`.

## Off-slice fix (landed here, `dfc68a39`)

Worker/coordination forks couldn't compile because `web/map/dist/` (the
RustEmbed `#[folder]` in `src/map_server/assets.rs`) is gitignored and not
fork-provisioned. Added `web/map/dist/**` to `.worktreeinclude` — self-healing
for every future fork. Pattern already in doctrine memory
(`mem.pattern.dispatch.worker-fork-missing-gitignored-embed`).

## Open / not in scope

- F-5 (a non-RV review outlet for backlog) stays open against RFC-003 **Axis B**
  (D2) — `reviews` is `RV`-only; Axis B's `references` role grammar owns it.
- Axes B/C/D of RFC-003 remain future work, gated on the RFC's ratifying ADR.
