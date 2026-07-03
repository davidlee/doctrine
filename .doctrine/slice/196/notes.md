# Notes SL-196: Per-edge relation descriptor

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design stage (2026-07-04)

**Status:** design authored + internal adversarial pass integrated + narrowed.
Status = `design`. **Not yet locked** — awaiting external GPT inquisition, then
`/plan`. Commits: `e8ab9ef` (scope), `778d587` (design), `43fe87a` (adversarial
integration).

**Shape:** `descriptor` = optional free-text cell on the `references:concerns`
`[[relation]]` row, riding the SL-176 `Degree` seam verbatim (per-row
`descriptor_bearing` column, `validate_link` reject-gate, excluded from
`(label,role,target)` identity, reject-on-differ append). Purely additive — no
migration, no `--notes` rename, no template change.

**Key decisions:** ride Degree seam (D1); scope to the single directed
link-authored label (D2); outbound-only render, no inbound index (D3); descriptor
on hydrated `CatalogEdge` (D4, novel — no degree precedent there, R3).

### Adversarial pass — the big catch (F0)

Initial design assumed `contextualizes` was a descriptor home. **It isn't:**
`contextualizes` is `CM`-source and authored via **concept-map DSL** lines
(`source > rel > target`, `concept_map.rs:1550`) — a write path `link`/`append_edge`
never touches. Narrowed to `references:concerns` only (the confirmed
`link`→`append_edge`→`Tier::One` home). contextualizes descriptor → follow-up
(DSL grammar change).

### Cross-slice: SL-197 has a latent wiring bug (F0b — ACTION for next agent)

The **driver** (concept records saying annotated things) needs `CPT` to be a legal
`references:concerns` source. **SL-197's scope wrongly assumes** CPT auto-inherits
`References(concerns)` by riding `kinds::RECORD`. It does **not**: the
`references:concerns` `sources` array is **hand-enumerated**
(`relation.rs:417`, `[SL, RFC, ISS, IMP, CHR, RSK, IDE, ASM, DEC, QUE, CON, EVD,
HYP]`), not `RECORD`-splatted — the source comment even admits the tail is kept in
sync *manually*. **SL-197 must add `CPT` to that array explicitly**, or it ships a
concept kind that can't be *about* anything. Flagged via `related` edge
SL-196↔SL-197 + design OQ-5. **Route to the SL-197 agent.**

### Durable-gotcha candidate (→ /record-memory)

"The `references:concerns` (and similar) relation `sources` sets are
hand-enumerated in `RELATION_RULES`, NOT `RECORD`-splatted. Adding a RECORD-family
kind does NOT auto-join those source-sets — each must be edited explicitly." Bit
SL-197's design; would bite any future kind-addition. Not yet recorded (design
stage; capture at plan/execute or now if convenient).

### Gate

No code touched yet (design stage) — `doctrine check gate` N/A.

### Foreign working-tree noise (bystander note)

Session opened with `SL-160` flipped `proposed→abandoned` in the working tree by
another agent (not SL-196 work); left untouched. Multiple agents active
(SL-195/197 files also dirty). All SL-196 commits were strictly path-limited to
`.doctrine/slice/196/`.
