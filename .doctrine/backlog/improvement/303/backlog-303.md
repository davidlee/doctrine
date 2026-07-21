# IMP-303: Bind admitted close_target OID to its audit RV at the close gate

Surfaced by the codex adversarial review of REV-030 (F-2), as a **pre-existing**
gap — it predates SL-212 and affects today's clean Doctrine close_target merge
equally.

## The gap

`integrate --trunk` on the candidate-active path requires only that a current
`close_target` admission *exists* (`dispatch.rs:2172-2189`); admission stores the
governing `--review` as **optional metadata** and never verifies that the named RV
actually reviewed the exact `admitted_oid` (`dispatch.rs:1653`). So the combined
tree that reaches trunk is *inspectable* (a named candidate ref) but not provably
*inspected*: a close_target created/admitted after audit can land a combination no
RV covered.

ADR-012 D5 requires "audit … against the actual review units … before they
integrate," but the mechanism doesn't bind the admitted OID to that audit. This is
the weaker half of the "trunk honesty" claim — inspectable ≠ inspected.

## Direction

Make the audit→admitted-OID relationship mechanical (or an explicit close gate):
`integrate` (or `admit`) should require the admitted `close_target` OID to be the
OID an audit RV disposed. SL-212 (operator-ingested merges) raises the stakes —
the ingested combination is more likely to differ from anything reviewed — so this
should land before or with SL-212's close path, not after.

Relates: REV-030, ADR-012 D5/D6, SL-212, IMP-127.
