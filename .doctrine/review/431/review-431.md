# Review RV-431 — plan of SL-287

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Commitments pass over SL-287's locked design, commitment-ledger.toml,
plan.toml, plan.md, scope, and DEC-415 through DEC-420. Compare extraction
against the full design, then compare each satisfying exit criterion against
the quoted intent. Check exact quote fidelity, both coverage directions,
proof substance and proportionality. RV-429 is read only to confirm that no
demonstrate/probe/control response requires a review-source commitment row.

Verification is document inspection plus read-only TOML/link and literal-string
checks. Logical failure scenarios are identified as inference. This pass
does not assess implementation, runtime behaviour, sequencing, phase sizing
or test reachability; those last three belong to the sequencing reviewer.
Writes are restricted to RV-431. Conclude this raiser pass and leave findings
for the responder; do not dispose findings or change the plan or design.
