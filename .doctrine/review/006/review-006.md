# Review RV-006 — reconciliation of SL-046

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation **conformance** audit of SL-046 (cross-kind relation graph
spine) against `design.md` §5, ADR-001/004/010, and REQ-091/074/077/078/079. All
four phases landed and green; this reconciles the shipped behaviour to canon and
dispositions every gap before close.

**Delivered:** PHASE-01 `Projection<K>` leaf (`5bedbe1`) · PHASE-02 relation vocab
+ 6 per-kind accessors + `outbound_for` (`ff1745c`) · PHASE-03 all-kind scan +
overlays + `inspect` (`c86eea5`) · PHASE-04 `inspect` CLI + render + `--json`
(`c316d6f`). Driven via `/dispatch` (serial, one worker per phase, 3-way import
onto a moving shared `main`).

**Lines of attack / invariants held:**
- **REQ-091** (the slice's acceptance): node ids are adapter-minted **opaque**
  cordage ids (via `Projection`); **every** rendered edge traces to an authored
  outbound relation (no synthetic edges); diagnostics re-map opaque ids → doctrine
  canonical refs + label names. Probed by VT-4 (`inspect_req091_…`) + real-corpus
  smoke.
- **ADR-004 §3/§5** outbound-only: inbound is **derived** from `in_edges`, never a
  stored reverse field. `supersedes`-inbound renders "superseded by" with NO
  `superseded_by` read. Probed by VT-1/VT-4 (structural) + VT-4 C8/R3.
- **ADR-010 D2** code-authoritative vocab: 13 `RelationLabel` = 11 overlay-backed +
  `Drift`/`DecisionRef` (target-unvalidated, no overlay, always dangle).
- **ADR-001** layering: `relation_graph`(engine) → `projection`(leaf) → cordage; no
  cycle. **C3** dedupe, **C5** scan-order determinism (REQ-077), **C7** never
  `graph.provenance()`, **I2** direct-only one-hop.
- **Real-corpus behaviour** beyond the seeded goldens — the place latent gaps hide.

**Where bodies were buried:** the full-corpus scan reads *every* entity, so the
inspector's robustness is hostage to the whole corpus's parseability — exercised
below.

## Synthesis

**Closure story.** SL-046 ships the cross-kind relation graph spine: a
`Projection<EntityKey>` over a separate cordage `Graph`, 11 `Reject`/`Unbounded`
reference overlays keyed by relation label, and a `doctrine inspect <ID> [--json]`
read surface that derives inbound from `in_edges` and groups outbound from the
per-kind `relation_edges` accessors. All four phases landed green; `just check`
(fmt + lint + 1000 lib tests + e2e) passes clean on the committed HEAD with zero
clippy warnings.

**Acceptance discharged.** REQ-091's three criteria hold and are tested
(`inspect_req091_ids_remapped_and_edges_authored`, VT-4): node ids are
adapter-minted opaque cordage ids the caller never sees; every rendered edge
traces to an authored outbound relation (the scan emits only what `outbound_for`
yields — no synthetic edges); the view re-maps opaque ids back to canonical refs
(`key_of`→`canonical_id`) and overlays to label names (`label.name()`). ADR-004 is
honoured structurally — inbound is recomputed from `in_edges` every query and the
`supersedes` reciprocal renders "superseded by" with **no** `superseded_by` read
(VT-1/VT-4; the C8/R3 fixture proves a lone stored `superseded_by` yields no
inbound). ADR-010 D2's 13-label vocab, ADR-001 layering, and the C3/C5/C7/I2
caveats are each covered by a dedicated test. Real-corpus smoke confirms the
end-to-end derived reciprocal (`inspect SL-002` → `inbound: superseded by:
SL-003`) and the live inbound from the newly-authored IMP-036 (`inspect SL-046` →
`inbound: slices: IMP-036`).

**Findings (both terminal, non-blocking).**
- **F-1 (minor → follow-up):** the full-corpus scan aborts every `inspect` if any
  single entity is unparseable. Per design (validation scoped to validate/SL-048),
  but a real fragility — owned by **IMP-036** (skip/note a malformed sibling,
  hard-fail only the queried id, optional `--strict`).
- **F-2 (minor → fix-now):** pre-existing pre-canonical-ref bare-int relation data
  (SL-003, ADR-002) the scan made fatal. Reconciled in `6eb5796`; corpus now
  parseable, the two latent `slice show` / `adr show` breakages repaired as a
  by-product. Not a SL-046 defect — surfaced and fixed by it.

**Standing risk / tradeoff consciously accepted.** Until IMP-036 lands, `inspect`
assumes a parseable corpus; a future malformed entity reproduces the corpus-wide
abort. Accepted for v1: the relation graph is a read surface, not a validator, and
the design deliberately deferred diagnostics. No `blocker` outstanding; the slice
is reconciled and close-ready.

**Note (canon hygiene, not a finding).** `design.md` §5.3 says "~9 entries" in one
place and "~11 overlays" elsewhere for the overlay map; the shipped count is 11
overlay-backed labels. The `~` marks both as approximate and the PHASE-02/03
commits + this synthesis record the authoritative 13-label / 11-overlay split, so
no design edit is forced — flagged here for the SL-048 vocab-table work that will
make the count exact.
