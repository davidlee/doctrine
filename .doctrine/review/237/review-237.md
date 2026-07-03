# Review RV-237 — design of SPEC-023

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition arraigns SPEC-023 as the durable technical-specification
hardening of RFC-013's prompt-cascade selection algebra.

Lines of interrogation:

- Does SPEC-023 faithfully preserve RFC-013's settled positions: trait-space
  classification over model identity, conjunctive-only selector grammar,
  set-valued model context, intersection targeting, root-wise specificity, and
  per-def classification with no central registry?
- Does SPEC-023 cohere with its declared ancestors, PRD-007 and SPEC-003, rather
  than smuggling source-of-truth duties into a disposable projection or breaching
  the pure/imperative split?
- Do the requirement entities say the same thing as the prose, with durable
  `REQ-NNN` ids carrying the obligations and no mobile label treated as identity?
- Are delivered-vs-forward-intent boundaries explicit enough for SL-192 to
  implement without inheriting false claims about already-delivered behaviour?
- Are prompt delivery, boot projection, dispatch classification, and install
  corpus boundaries cleanly separated, with open questions called out rather
  than inferred?

The cache was not primed: `doctrine review prime RV-237` refuses non-slice
targets because SPEC-023 has no slice selectors. Evidence is therefore gathered
manually through `doctrine <kind> show`/`inspect` outputs and the installed
review-ledger reference.

## Synthesis

Judgement: SPEC-023 preserves the central RFC-013 algebra, and no charge was
sustained against its trait-space doctrine: classification over identity,
conjunctive-only selectors, set-valued context, intersection targeting,
root-wise specificity, and per-def classification all survive the rack. Yet
three lesser corruptions were confessed under examination.

The heresy is not in the algebra; it is in the record-keeping around the
algebra. Dispatch consumption is named in prose but absent from structured
interactions. Corpus-loader ownership is asserted by responsibility and prose
while the source anchors omit `src/install.rs`. The `prompt check` cadence claim
is written as if already delivered, while local evidence shows no current
`doctrine check`/`just check` feed.

Sentence:

- IMP-238 shall reconcile SPEC-023's interaction graph so dispatch consumers
  are either structured (`SPEC-012`/`SPEC-021`) or removed from the prose claim.
- IMP-238 shall make corpus-loader ownership explicit by source anchor or by
  a clearer delegation to SPEC-009.
- IMP-238 shall settle the `prompt check` cadence claim: wire and cover it, or
  mark it as target/follow-up rather than delivered fact.

Verification expected: `doctrine spec show SPEC-023`, `doctrine inspect
SPEC-023`, and `doctrine coverage show SPEC-023` must agree after correction;
the check-cadence claim must be backed either by configuration/code evidence or
by explicit forward-intent wording.

Tolerated residue: `review prime` remains unavailable for a SPEC target; this
is a tooling mismatch recorded in RFC-011 case notes, not a SPEC-023 defect.
