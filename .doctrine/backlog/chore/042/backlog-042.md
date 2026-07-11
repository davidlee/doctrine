# CHR-042: RFC-019 Phase C entry: empirical evaluation of Phase B inference

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why

RFC-019's Phase C (elicitation queue + capture loop) carries an explicit
**entry criterion**, bound at external review 3 (2026-07-11): an empirical
evaluation of the Phase B inference layer (shipped in SL-213) against a
**real ledger over this repo's own backlog**. Question-selection machinery is
exploratory until that evidence exists. This chore *is* that evaluation.
Phase C (IMP-280) is needs-gated on it.

## Task

1. **Capture a genuine ledger.** Use the `doctrine compare` verb group to
   record honest pairwise value judgements over live backlog items
   (`doctrine backlog list` for candidates). Agent-rater rows are admissible
   (rater kind is disclosed per RFC-019 T7); judge relative *value for the
   project*, frame `equal-effort`. Use the full v2 response vocabulary where
   honest: `prefer-a` / `prefer-b` / `equal` / `incomparable`. Do **not**
   manufacture cycles or conflicts to exercise degradation — the point is
   observing rates under genuine evidence. Target enough coverage for
   propagation to matter (~20–40 rows across a connected subset, including
   some items with authored `value` facets so anchors participate).
2. **Run inference.** The pre-pass runs inside the priority scan; observe via
   `doctrine reports next` / `doctrine reports explain <ID>` (value-source
   line: provenance, bounds, rater-kind counts, residual diagnostics) on
   compared items. Use the tree-local `./target/debug/doctrine` after
   `cargo build`.
3. **Assess against the RFC's questions:**
   - *Useful?* Do bounds/projections discriminate between items, or collapse
     to near-uniform? Do projected values move `value_dim` sensibly?
   - *Stable?* Re-run twice; same merged file set ⇒ identical active rows,
     bounds, residuals, projections. Add a row; check perturbation is local
     and proportionate (P10–P15 stability contract, SL-213 design.md).
   - *Degradation sane?* How often do SCC quarantine / anchor-conflict
     residual exclusion fire under genuine evidence? Are the findings
     actionable (named exits: supersede / tombstone / edit anchor) or a wall
     of pairs?
4. **Write the report.** Findings + verdict into
   `.doctrine/rfc/019/phase-b-evaluation.md` (co-located with the RFC's
   `partial-order.md`). Verdict is one of: *entry criterion met* (Phase C may
   open), *met with caveats* (name them), or *not met* (name the Phase B
   defects — those become new backlog issues).
5. **Close the loop.** Set this item's resolution to the verdict; if met,
   IMP-280 unblocks automatically via the needs edge.

## References

- RFC-019 §Implementation path, Phase C entry criterion — `doctrine rfc show RFC-019`
- SL-213 (Phase B, done) — verification intent + design decision ledger
  (D3/D4 degradation semantics, D9/D10 projection, P10–P15 stability):
  `doctrine slice show SL-213`, `doctrine slice design SL-213`
- SL-210 (Phase A capture) — session file mechanics
