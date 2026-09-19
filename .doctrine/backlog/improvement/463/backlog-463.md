# IMP-463: Measure design-review cost per stage

Nothing durable measures what a slice costs in tokens, wall-clock, or human
minutes, per lifecycle stage (design, review round, implementation, audit). The
available figures are anecdotes: one external review round of about 13 minutes,
one reviewer that read 45,105 tokens and produced no ledger action.

**Why it matters.** `RFC-026` proposal `P10` (route each severe design-review
finding to the instrument that can settle it) carries a three-slice trial. The
trial can show whether the convention operates and whether repeated argument
falls; it cannot test the economic claim — that total effort through accepted
implementation falls — until cost is measured. A saving at design that reappears
at implementation or audit is invisible today.

**Shape, as small as possible.** Per slice and per stage, from what the harness
already records (session logs, ledger turn timestamps, commit times). Prefer a
script over a schema. `RFC-011` (dispatch token efficiency) already gathers
friction observations for the dispatch path; check what it measures before
building anything.

**Guard.** Review ledger `RV-353` found that 61% of one programme's artefact was
measurement apparatus. Keep this to the three numbers.
