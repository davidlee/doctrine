A finding carries two claims: **something is wrong here** (the observation) and
**do this about it** (the prescription). They fail independently, and the second
fails far more often than its confidence suggests — a reviewer who has correctly
localised a defect has usually not paid for the design work the repair needs.

`SL-244` hit this four times in one design stage:

- an issue's first ruling — the defect was real, the proposed fix was one of four
  candidates and not the one taken;
- an external review finding that correctly identified an underivable value and
  prescribed routing it through the wrong type;
- a self-review finding that correctly found duplication and prescribed removing
  one of the two sources, when the two were a requirement and its evidence and
  had to *both* stay;
- a self-review finding whose own prescription was reversed by a later finding in
  the same batch (see
  [[mem.pattern.feedback.rule-findings-in-dependency-order]]).

Each was caught only because the finding went up for a ruling before anything was
integrated. Practice:

1. **Split the finding when you report it.** State the observation and the
   prescription as separate claims with separate confidence. "I'm sure of the
   defect, less sure of the repair" is the honest and common position.
2. **Never integrate on the finding's authority**, however obviously correct the
   repair looks. Obviousness is what a wrong prescription feels like from the
   inside.
3. **Verify the observation against the tree, not the prose.** Two of the four
   above were settled by reading the incumbent implementation, which said
   something neither the finding nor the design had noticed.
4. When you are the finder and a ruling rejects your prescription while accepting
   your observation, that is the mechanism working. Do not re-argue the
   prescription; the observation was the contribution.

Related: [[mem.pattern.feedback.rule-findings-in-dependency-order]],
[[mem.pattern.testing.assert-bytes-not-digests]].
