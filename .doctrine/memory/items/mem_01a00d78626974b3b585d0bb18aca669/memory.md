# A golden can pass because of the bug it fails to name

A characterisation golden proves the behaviour it asserts. It does **not** prove
its own fixture is valid — and where the code under test applies one rule to
every kind, an invalid fixture is *invisible*, because the code never consults
the vocabulary the fixture violates.

SL-238 PHASE-07 replaced `--prune`'s hardcoded `status == "resolved" || status ==
"closed"` with a per-kind routed classifier. Two SL-105-era goldens went red.
They looked like regressions. They were not: both set a **slice**'s status to
`resolved` — a *backlog* word, outside ADR-009's slice vocabulary
(`proposed…reconcile` / `done` / `abandoned`). They had only ever passed because
the old probe applied one hardcoded table to every kind alike. **The tests were
asserting the cross-kind leak the slice existed to remove.**

## The rule

When a change replaces a cross-cutting hardcoded rule with a per-kind (per-type,
per-tenant, per-locale) routed one, treat every red golden as a **fixture
suspect first**:

1. Read the fixture's literal values against the *real* vocabulary of the entity
   it constructs — not against the assertion, and not against what makes the test
   pass.
2. If the fixture is invalid, repair the **fixture** and leave the asserted
   behaviour alone. Say so in the commit: a fixture edit that looks like an
   assertion relaxation is the most expensive thing to explain at audit.
3. Only if the fixture is sound is the red a behaviour change — and then it must
   be one the design names, or the design owns it before the code does.

The failure mode this prevents is the expensive one: adjusting production code
until an invalid fixture goes green, thereby *re-implementing* the defect.

## Why a prose reviewer will not catch it

The fixture and the assertion agree with each other. Only the *third* party — the
kind's own vocabulary, which lives in a different file — disagrees, and nothing
in the test forces a reader to fetch it. Same seam as
[[mem.pattern.testing.grep-for-the-pin-before-characterising]]: a claim about the
tree that nobody re-derives at the moment it matters.

## The corollary that saves the diagnosis

When such a golden goes red, its STOP condition ("halt if a golden reddens for a
reason the design does not name") does **not** fire — the reason *is* named, it
just arrives from the narrowing direction rather than the widening one. A design
that says "`done` and `answered` become prunable" is also saying "a cross-kind
status word stops working", and usually never writes the second sentence.
