# IMP-440: Lint doctrine evidence for records whose subject is a neighbouring repo

`oubliette:ADR-003` clause 1 — evidence whose subject is oubliette's own
machinery is minted in oubliette's corpus, not doctrine's — is prose on both
sides and enforced by nothing. Its own *Verification* section says so. This is
the cheap half of making it more than prose.

## Shape

Scan doctrine's `EVD` bodies and facets for markers that a record's subject is
the neighbouring repo's mechanism rather than doctrine's choice — oubliette paths
(`probe/`, `host/`, `capsule-*`), the retired `microvm-spike` checkout name, a
probe script name — and report each hit with the record id and the matched
marker. Report, not refuse: `EVD-025` legitimately cites oubliette while its
subject stays doctrine's selection, so a hit is a question for a human, not a
verdict. The retired-checkout-name and `..`-climb cases from `ADR-003` clause 2
*are* mechanical and could fail rather than warn.

Existing evidence records already matching the markers were surveyed at capture
time: `EVD-006..011`, `EVD-013`, `EVD-015..020`, `EVD-022`, `EVD-024`. Most are
expected keeps; the point of the lint is that no one currently has to look.

Where it hangs is open — `doctrine doctor`, a `knowledge validate` leg, or its
own verb. Prefer an existing gate over a new one.

## Boundary

**Do not build before `CHR-070`.** A lint enforces a rule; doctrine has not
adopted the rule yet, only oubliette has, and a check that cites a neighbouring
repo's ADR as its authority is the wrong dependency direction. `CHR-069` is the
content repair a lint would have caught. `oubliette:CHR-012` names this as
doctrine's to build.
