# Review RV-349 — design of SL-249

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

External adversarial pass over SL-249's design at design-run revision 48
(`dr-019fd6b6`), taken at the `drafting → reviewing` boundary and before the
lock gate. The subject is `.doctrine/slice/249/design.md` — ten sections,
`sec-1`…`sec-14` — read against the slice scope (`SL-249`), the twelve rulings
it cites, and the code it commits to touching.

### What the design claims

SL-249 closes a two-headed data-loss hole: the knowledge structured tier has no
write verb (facet population is 35/142 on decisions, 0/38 on `QUE`'s answer
triple), and the design-run disposition wire has no slot for a new record's
content, so on SL-248 six dispositions sent record prose as `Declaration::body`
and had it silently discarded. The design adds one authored per-kind facet-field
table, a pure `plan_facet_edits` / `apply_facet_edits` seam over the existing
`src/facet_write.rs`, six per-kind `knowledge edit <kind>` subverbs plus a
kind-blind `edit <ID>` and a `settle` verb, a `body`+`facet` payload on
`CreateRecord` written at DEC-086 step 5, a typed refusal for wire keys inert at
their subject's kind, and a governance amendment (`SPEC-019`, `PRD-010`) from
four record kinds to seven.

### Lines of attack

1. **§ 5.1 D1's two pins.** P1 (union of `facet_fields` ≡ `RawFacet`'s serde key
   set) compares *sets*, so a field listed under two kinds is invisible to it;
   P2 is per-kind and would not notice a field on an extra row that also
   validates there. The design concedes the gap depends on `validate_facet`'s
   arms reading disjoint field sets — an argument about current code, not a
   property the pins enforce. Is a third pin needed, and what is its cheapest
   shape?
2. **§ 5.4 path B / D6 ordering.** `settle` writes facet evidence, then the
   status token, across two files, deliberately non-atomic. The argument
   optimises for a crash between the writes. Does a *refusal* inside
   `set_record_status` after the facet write invert the argument — leaving
   settlement evidence a record never earned?
3. **DEC-177's remaining justification.** The doctor tripwire was ruled when the
   read path's tolerance was the only control. D5 now validates create payloads
   at admission and the CLI validates at `plan_facet_edits`, leaving hand-edits
   as the sole producer of an inert key. Does the tripwire still earn its place,
   and is § 5.4 path E's read-only posture the right one?
4. **F-1 / `KeyPosture` and the corpus.** DEC-170 refuses an absent facet key
   rather than creating it, resting entirely on `A3` — that all seven
   `install/templates/knowledge-*.toml` seed every field of their kind, verified
   for exactly one. `R5` and `R7` state the blast radius. Is the posture right,
   is the mitigation (a Phase B test) sited early enough, and does `R7`'s
   migration consequence belong in the REV or in this slice?
5. **The phase boundary (`DEC-165`, `R6`).** Phase A ships the wire fix with no
   facet anywhere; the exit criterion is that nothing it ships references a
   symbol from the facet table. Is that boundary actually clean, or does the
   `body`-at-step-5 work in Phase A already need something Phase B owns?
6. **Governance application.** `SPEC-004` edit-preservation, `DEC-088`'s reserved
   route to `accepted`, `ADR-001` layering (the table and seam live in
   `src/knowledge.rs`, which also carries the CLI — `R8`), `STD-001` single
   spelling, `POL-002` (DEC-176's canary as a project-local test, never a
   `validate` rule), `ADR-013` (the REV lands at reconcile). Misread, weakly
   applied, or asserted rather than shown?
7. **Verification honesty (§ 9).** The behaviour-preservation gate names two
   suites that may not be edited. Do the nine invariants `I1`…`I9` have test
   shapes that would actually fail on the defect they name, and is any closure
   criterion unfalsifiable as written?
8. **Level errors and naming.** D4 keeps `body` as the wire's prose key on the
   ground that SL-248 was a level error, not a collision. Does objective 3's
   refusal actually make the level error loud enough to justify the reuse?

### Ground rules

- The design doc is the subject; the twelve `DEC-` records carry the reasoning
  it cites and are in scope where the design leans on them.
- Findings go on this ledger via `doctrine review raise RV-349 …`, one finding
  per defect, severity owned by the raiser.
- A pass that finds nothing is a result worth stating plainly. Do not invent
  findings to fill a quota.
- Do not edit the design, the slice, or the design run. Raise; the author
  disposes.
