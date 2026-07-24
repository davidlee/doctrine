# DEC-011: Retain CON; abandon CON-to-INV rename

**Decision.** The epistemic kind **constraint (CON)** is retained. The
RFC-009 D2 draft position — replace CON with **invariant (INV)** — is **abandoned**,
not deferred. CON stays a first-class shipped kind (`CONSTRAINT_KIND`,
`RecordKind::Constraint`, prefix `CON`, seed CON-001).

**Provenance.** Records the observed outcome of **SL-160** ("Replace CON with INV")
reaching status `abandoned` (2026-07-03). SL-160 was carved out of SL-159 precisely
because its semantic shape was unsettled (design.md §2, "design not yet run"); it
was abandoned before a `/design` pass resolved that question. No positive written
rationale accompanied the abandonment — this record reconstructs it so the reversal
is legible as a decision, not only as a dead slice.

**Why (reconstructed from SL-160 design §2).** The rename dragged an unresolved
vocabulary question — CON's `waived` status has no clean dual for an invariant.
An invariant that no longer holds is *violated*, not "relaxed"; "relaxed" implies
the requirement was loosened, not that the property failed. Three candidate framings
(`relaxed` / drop-the-waiver / a violation-model) were all live and none clearly
won. Meanwhile the rename was **destructive** (prefix, tree dir, reservation
namespace, seed, ~17 hardcoded touch-sites incl. a panic-grade `scan.rs` dispatch
arm) — high cost against an unsettled, low-marginal-value framing. The engineering
call landed on: not worth it; keep CON.

**The live thread it leaves (routes to RFC-009 D3).** Framing (c) in SL-160 §2 is
the durable insight: the "property was violated" signal plausibly belongs on an
**EVD-`disputes`-CON edge** (SL-159's supports/disputes vocabulary), *not* on a
status-vocab rename. So the expressive need INV was reaching for is better served by
an **edge**, which is exactly RFC-009 **D3** (record/entity edges) — not a kind
rename. If D3 lands EVD-`disputes`-CON, the INV motivation dissolves entirely.

**Consequence for RFC-009.** D2's "INV replaces CON (decided in draft)" is
**superseded by this record**: CON retained, INV not adopted. The `EVD-disputes-INV`
pairing named throughout RFC-009 is moot — read it as `EVD-disputes-CON`, deferred
to D3.

Related: [[RFC-009]] (D2/D3), SL-159 (EVD/HYP + supports/disputes landed), SL-160
(abandoned), CON-001, ADR-013 (a kind-catalog change would have emitted a Revision —
none now needed).
