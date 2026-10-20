A reconciliation brief is a snapshot from audit time. Two ways it goes stale or
short, both seen at SL-284 reconcile (RV-424):

1. **Pre-drafted REV payloads predate fix-now commits.** REV-074's `modify`
   statement for REQ-480 was drafted during execution; the audit then landed a
   fix-now (F-5) changing the very fallback wording the statement described, and
   the brief said "apply REV-074 as raised". Applied verbatim, the spec would have
   described pre-fix behaviour. Before applying, diff each `modify` statement and
   prose payload against the audit's fix-now commits, and land the corrected text
   (record the correction in the REV narrative).

2. **A falsified decision rarely stands alone.** The audit caught DEC-377
   (superseded by DEC-391) but missed DEC-373 and DEC-375, accepted decisions that
   DEC-377 amended and that carried the same npx claims in their bodies. Decision
   amendments are often body notes ("Amended by DEC-NNN"), not relation edges, so
   `inspect` does not show them. Re-run the REV's own `doctrine search <term> -k all`
   after landing and read every `accepted` DEC hit; amend in the record's existing
   idiom, or supersede when the whole rule moved.

Also: a REV cannot target a DEC (`revises` targets are SPEC/PRD/REQ/ADR/POL/STD) —
see [[mem.fact.revision.no-knowledge-record-target]]. When writing a successor
decision, restate every surviving clause of the old one, not just the falsified
ones.
