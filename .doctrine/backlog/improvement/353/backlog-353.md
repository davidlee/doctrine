# IMP-353: WITHDRAWN_STATUSES has no anti-drift guard against per-kind vocabularies

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

SL-233 PHASE-05 needed a "is this record still a usable adoption target?"
predicate for `design apply`'s adopt disposition, and no existing knowledge
predicate answers it:

- `is_hidden` and `is_terminal` both **misclassify**. An `answered` question is
  hidden *and* terminal, yet it is exactly the right adoption target. A
  `superseded` decision is no more hidden than an accepted one, yet it must not
  be adopted.

So PHASE-05 added `WITHDRAWN_STATUSES` in `src/knowledge.rs` — a named
cross-kind constant listing the statuses that disqualify a record from adoption,
with the distinction documented.

**The gap.** It is hand-maintained and **not derivable** from the per-kind
status vocabularies (`DECISION_STATUSES`, `QUESTION_STATUSES`, …). When a kind
gains or renames a status, nothing catches a `WITHDRAWN_STATUSES` member that no
longer appears in any vocabulary, or a newly-added withdrawal-shaped status that
was never added to the set. Same drift shape as `integrity::KINDS` membership,
which is likewise advisory rather than enforced.

**Fix.** A guard test asserting every `WITHDRAWN_STATUSES` member appears in at
least one kind's vocabulary. That catches the rename/removal half cheaply. The
addition half (a new withdrawal-shaped status nobody registered) is not
mechanically detectable and stays a review concern — say so in the test's
comment rather than implying the guard is total.

The PHASE-05 worker deliberately did not add this test, to keep that phase's
VA-1 "green and UNCHANGED" reading clean. Filed rather than folded in.
