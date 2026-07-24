# REV adding forward-intent to a retrospective spec must be pending introduce-only

Retrospective specs (e.g. the dispatch tech specs SPEC-021/022) describe *shipped*
behaviour — coverage is reconciled from evidence, never inferred ahead of code. A
Revision that wants to add forward-intent to such a spec must ship **`pending`
introduce-only requirements**: the dual-posture rule expressly permits planned
requirements as long as they stay distinguishable from verified ones. The slice
that implements the mechanism flips them `pending → active` at reconcile.

**What you must NOT do:** amend an *active* requirement to describe unbuilt
behaviour. That presents target behaviour as already shipped and breaches the
retrospective charter. Those §-prose / active-requirement modifies defer to a
**ship-time sibling Revision** authored at slice close, when the evidence exists.

**Why:** the charter's whole value is that `active` means "verified, shipped".
Editing an active requirement forward silently converts intent into a false claim
of coverage.

**How to apply:** when scoping a REV against a retrospective spec, split the
payload — `introduce` rows (new `pending` FRs) go in the pre-code REV; `modify`
rows against active requirements are held for a sibling REV at ship time. If a
change row targets an existing active requirement, that is the tell you are over
the line.

**Precedent:** REV-030 declined to amend SPEC-022 ahead of the code. REV-032
(zero-rescue, RFC-016 Cluster 2) followed the same rule after adversarial review
RV-300 F-1/F-3: it dropped the 4 active-requirement `modify` rows
(REQ-287/293/294/318) and kept 6 `pending` introduce rows, deferring the modifies
to a ship-time sibling REV.

Related: [[mem.pattern.doctrine.spec-prose-requirement-drift]] (sweep §-prose when
a requirement changes), [[mem.fact.revision.spec-prose-modify-target]] (modify
mechanics), [[mem.fact.conformance.rev-only-slice-undeclared]].
