A design's line count is a bad phase-sizing input when much of the document is
**review archaeology** rather than implementation instruction.

`SL-248` is the worked case. Its scope reasoned a band of *~1000 design lines
over 6–9 phases* (~110–170 lines/phase) from two prior slices: `SL-233` ran 16
phases on 781 lines and left holes, `SL-244` ran 8 on 3459 and was uncomfortably
coarse. The design then locked at ~5100 lines. Applied literally the band gives
roughly 34 phases, which is absurd — because after nine adversarial review
rounds the bulk of those lines are *why the previous version was wrong* (what
`F-25` cost, why *in full* was false three times), not instruction to an
implementer.

**Size on the evidence instead.** A design with a `Code impact` / evidence table
already states its own test counts per surface and usually says they are
indicative for phase sizing. That count tracks work; the line count tracks how
contested the design was. `SL-248` sized ten phases on ~160 pure tests plus 23
executed rows (~18 tests/phase) and landed inside the scope's own phase band
even though it was five times over the line band.

**The tell** that a design is archaeology-heavy: sections that narrate their own
correction history, findings cited by id in prose, and a residuals section that
explains what earlier drafts got wrong. When you see it, re-derive the band from
the evidence table and say so in `plan.md`, because the scope's stated line band
is now misleading to the next reader.

Related: [[mem.fact.doctrine.cli-source-of-truth]] for reading the design's
premises rather than recalling them, and [[mem.pattern.doctrine.core-loop]].