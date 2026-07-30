# IMP-370: Single-source design digest derivations by kind

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Carries RV-324 F-5 (minor), dispositioned `follow-up` because the **fix shape is
the owner's reserved decision**, not because the work is large. The owner leans
toward one shared primitive over a kind-named sibling but called it "purely on
vibe" and reserved it for SL-233's close. Capturing it here so the deferral is
owned rather than lost.

## The finding

`src/commands/design.rs` derives the same digest kind at more than one seam:

- the **authored watermark** — by the named helper `authored_fingerprint()`
  (`:265-266`) and independently inline at `:1406` (materialise's re-baseline);
- **section-body fingerprints** — independently at `:293` and `:1264`, while
  legacy region fingerprints reuse the document-named helper at `:1030`.

The module's own comment at `:263-264` says these "must not spell it twice". The
derivations are output-equivalent today; a future normalisation or
domain-separation change would make read, adoption, import, declaration and
materialisation disagree with **no compiler signal**. STD-001 (single-source named
constants) is the governing standard.

## Two things this is NOT

**Not PHASE-07's fragment digest.** `fragment_section` (`design.rs:1521`) emits a
bare `crate::git::sha256` because `FragmentReceipt::digest` is typed `String`.
That is asset identity on the wire — a different kind from the authored-document
watermark `Fingerprint`. Reusing `authored_fingerprint` there would be
wrong-by-kind. Recorded as SL-233 PHASE-07 sheet F-19; verified again during
RV-324 adjudication.

**Not a pure/shell violation.** RV-324 verified the split intact with a positive
control: `rg -n 'sha2::|Sha256::|sha256\(' src/design_run/ -g '!prompt.rs'` exits
1 while the control finds the shell sites. The pure core never hashes, as
designed. This is duplication *within the shell only*.

## Why it may not be a standalone decision

DEC-099 (which supersedes DEC-092) has `materialise` borrow
`src/review.rs::with_turn`'s lock-and-CAS shape. That borrow and F-5's
shared-primitive instinct point at the same extraction — shared write-guard
machinery that owns both the locking and the fingerprinting. Worth deciding
together rather than twice; if they converge, this item folds into that decision
instead of being resolved on its own.

## Sequencing

SL-233 PHASE-15 (RV-324 remediation) **deliberately excludes** this — its EX-12 is
a scope fence, so an unrequested consolidation there would pre-empt the owner's
decision and contaminate the phase diff. Settle at slice close.
