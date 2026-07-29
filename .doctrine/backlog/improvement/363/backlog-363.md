# IMP-363: Report invalidation of the integrated review and lock acceptance

`design_run::run::invalidation_rows` derives `evidence_invalidated` and
`review_invalidated` rows from a before/after set difference rather than from
whoever moved the content — explicitly so that "a new way of moving a fingerprint
cannot forget to report what it killed" (its own doc comment).

SL-233 PHASE-12 added two more pieces of content-bound evidence that a section
edit kills — the integrated adversarial pass (`ReviewGroup::integrated`) and the
lock acceptance (`ReviewGroup::acceptance`), both current only while their
`ContentCoverage` matches `SectionGroup::fingerprints()`. **Neither produces a
row when it dies.** The caller discovers it at the next lock attempt, through the
gate refusal.

So the module's stated principle now holds for two of four kinds of
content-bound evidence. Closing it needs two `ChangeEvent` members (the
vocabulary has headroom: `DESIGN_EVENT_NAME_BYTES` is 32 and the longest current
name is 27 B) plus their before/after currency capture alongside the existing
`live_evidence` / `live_reviews` snapshots.
