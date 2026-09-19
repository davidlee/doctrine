Cross-kind inbound relation ordering follows `EntityKey`'s **derived `Ord`** —
prefix compared **lexically**, then the numeric id — and NOT `RecordKind::ALL`'s
declaration order. The sort site is `src/catalog/scan.rs:85`.

So `CPT-903` sorts **before** `DEC-901`: `CPT` precedes `DEC` lexically, and the
ids are irrelevant until the prefixes tie.

## Why this is a trap

The comment adjacent to the sort (from SL-050) reads as though `RecordKind::ALL`
declaration order governs. It does not. An agent writing a golden for a
multi-kind inbound set from that comment will write the kinds in declaration
order and get a red that looks like an implementation bug.

**Derive the expected order from a real run, then assert it — never hand-write a
cross-kind order from a declaration list.** In SL-246 PHASE-04 a mandated
mutation battery caught exactly this: a hand-written golden had the kinds in
declaration order and was wrong.

Within one prefix the ids sort numerically, so 998, 999, 1000, 1001 — not
canonical-string order.

See [[mem.pattern.doctrine.tdd-loop]].
