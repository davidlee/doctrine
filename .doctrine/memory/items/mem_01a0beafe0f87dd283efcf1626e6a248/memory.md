# Verify judges the artefact and the account — contest a response that misdescribes its own repair

`verify` is terminal and immutable, so it certifies the **whole disposition** —
including the response's account of what was done. Two failure modes follow, and
the corpus already names only the first.

**1. The repair is incomplete.** Covered by
[[mem.pattern.review.repair-inherits-finding-scope]] and
[[mem.pattern.review.repair-closes-a-subset-of-the-stated-class]].

**2. The repair is complete but the response misdescribes it.** Not covered, and
it is the one that slips past a careful raiser, because the artefact checks out.
Verifying anyway writes a permanent, immutable, wrong account of why the artefact
has the shape it has. A later reader reconstructing the decision from the ledger
gets the superseded answer.

**The rule.** At `verify`, read the artefact and confirm the account matches it.
Where the account is wrong in its operative detail, `contest` and let the
responder re-dispose — the wrong text is then *replaced* rather than footnoted. A
post-hoc correction in the `RV` `.md` prose body is the weaker instrument: the
finding still reads `verified` against the wrong response.

**A responder's own completeness claim is a defect site.** A phrase like *"the
overclaim appeared in four places and all four are fixed"* is an enumeration
nobody re-counts — the raiser has no cheaper signal than re-running the sweep,
and the claim's confidence is what suppresses the re-run.

Evidence — `RV-371` / `SL-260`, a design-review ledger of 10 findings. Nine were
disposed and the pass read closed. A fresh critical re-read afterwards found five
defects in the repairs:

| defect | shape |
|---|---|
| `F-6`'s precedence rule | the repair resolved the arm the finding opened with and left the arm the same finding also named |
| `F-2`'s class sweep | the response claimed four sites fixed; there were five |
| `DEC-271`'s title | carried the exact claim `F-2` withdrew, in the one field append-style amendment cannot reach |
| `CON-006`'s citations | two ids off by one, one of them pointing at a live unrelated record |
| four code citations | true claims, line ranges that did not support them |

Two of the five were contested and re-disposed on this rule; the rest were
artefact-only and repaired directly. None was found by the adversarial pass.

**The broader read: a single pass does not converge.** The marginal find here came
from re-reading under different conditions, not from an additional reader — which
is a different lever from
[[mem.pattern.review.second-cheap-pass-is-triage-not-confirmation]]. Budget a re-read of
the repairs before accepting a pass as closed, and treat the repair round as a
first-class defect site rather than as cleanup.
