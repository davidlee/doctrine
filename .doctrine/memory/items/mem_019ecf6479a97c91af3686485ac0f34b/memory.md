# SL-076 PHASE-01 worker findings: visibility surprises

Three items not captured in the design/plan surfaced during implementation:

1. **SegmentKind needed `pub(crate)` + Serialize** — adding Serialize to
   `ConceptMapDiagnostic` transitively requires SegmentKind (a variant field
   type) to implement Serialize. The phase sheet said SegmentKind didn't need
   promotion, but the Rust compiler enforced otherwise. This is a mechanical
   follow-on from T5, not a design gap.

2. **Dead-code gates for forward-declared symbols** — Three
   `ConceptMapMutationError` variants (`NodeCollision`, `MissingDsl`,
   `InvalidToml`) and `rename_node_in_dsl` are only exercised by test code in
   PHASE-01; route consumers land in PHASE-02. To keep clippy clean:
   ```rust
   #[cfg_attr(not(test), expect(dead_code))]
   ```
   Follows the precedent set in `catalog/graph.rs`. The gates can be removed
   in PHASE-02 when the route code exercises them.

3. **CM arm merged with REQ in outbound_for** — clippy `match_same_arms`
   forced merging `"CM" => Ok(Vec::new())` with the existing `"REQ" => Ok(Vec::new())`
   arm rather than adding it as a standalone arm before REV. Both are empty-return
   arms with identical bodies, separated because REQ is "empty forever" and CM
   is "empty for now" (entity-ref follow-up may add edges). The existing
   `#[expect(clippy::match_same_arms, reason = "...")]` on the knowledge-records
   arm is a precedent for keeping arms separate when their futures diverge.
   Adding the CM arm to the REQ arm is pragmatically fine (CM to REQ coupling
   is weak), but if the arms' futures diverge, split them with the same expect
   attribute pattern.

4. **remove_edge_from_dsl removes only first match** — matches the design
   contract in plan.md: "removes one matching line (first match in line order)".
   Multiple identical edge lines require multiple POST calls.
