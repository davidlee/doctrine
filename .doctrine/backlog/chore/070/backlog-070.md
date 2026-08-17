# CHR-070: Doctrine-side rule for who mints evidence about a neighbouring repo

`oubliette:ADR-003` *Evidence has one home, and it says what it does not prove*
settles which corpus mints an evidence record when two doctrine-governed repos
observe the same machinery: the subject decides, not who ran the probe and not
who wanted the answer. Oubliette mints evidence about oubliette's mechanism;
doctrine keeps evidence whose subject is doctrine's own selection (an adoption
cost, a comparison between confinement shapes) and cites across by id.

**The rule binds one side only.** It is authored in oubliette's corpus. Doctrine
has no statement of it, so a doctrine agent minting the next `EVD` off a probe
run has nothing to read, and `CHR-069`'s two repairs are the shape of what
happens without one.

## What is owed

A doctrine-side analogue — the ownership test, the citation form (entity id plus
repo-relative path, never a `..` climb, never a retired checkout name), and the
no-migration position on records that predate the rule. Whether it also carries
`oubliette:ADR-003` clause 4's two probe disciplines (*assert both directions*;
*name what the run deliberately does not prove*) as general evidence disciplines
is open — they are stated there as applying to records in either corpus, and
doctrine's `EVD` facet has nowhere to put either.

## Where it landed

**Its own ADR — `ADR-022` *Evidence ownership between peer corpora*, `proposed`.**
This item discharges when that is accepted.

Folding it into `SL-257` (*Authority floor and assurance profile*, `design`) was
the owner's initial suggestion and was rejected on scope: `SL-257` is scoped to
the conformance-verdict seam and governed by `ADR-020`/`POL-002`, neither of which
reaches corpus ownership of an `EVD`. The `references(concerns)` edge to `SL-257`
stays, because `QUE-217` reads doctrine's side of the disputed figure and
`CHR-069`'s repair wants to be done before that slice reasons off an uncontested
8.31 s.

`ADR-022` deliberately does **not** carry `oubliette:ADR-003` clause 4's two probe
disciplines (*assert both directions*; *name what the run deliberately does not
prove*). They are general evidence-quality disciplines, not peer-corpus questions,
and their home is `SPEC-019`'s `EVD` facet — which today holds `datum` /
`provenance` / `confidence` and has nowhere to put either. Tracked separately.

## Boundary

`IMP-440`'s lint enforces this rule and should not be built before it exists.
`CHR-069` is the content repair the rule's absence produced. Oubliette's
`CHR-012` tracks the same debt from the other side.
