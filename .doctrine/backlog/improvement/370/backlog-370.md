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

`src/commands/design.rs` derives the same digest kind at more than one seam.

**Corrected 2026-07-30, after RV-324's raiser contested F-5's response.** The
census below is re-run at `540aed95`; the version this item was created with
described the tree before SL-233 PHASE-15 landed.

- ~~the **authored watermark** — by the named helper `authored_fingerprint()`
  (`:265-266`) and independently inline at `:1406`~~ — **CLOSED, incidentally.**
  PHASE-15's F-2 remediation (`56de92d3`) had to derive the post-write watermark
  from bytes read back off disk; spelling it twice was the mechanism of the
  self-certification defect, so the inline derivation became
  `authored_fingerprint(&body)` (now `:1505`). This was a side effect of F-2, not
  work on F-5 — EX-12 fenced that — but the leg is genuinely closed.
- **section-body fingerprints** — independently at `:293` (adoption's authored
  read) and `:1352` (`section_digests`), while legacy region fingerprints reuse
  the named helper at `:1082`. **This is the live surface, and now the whole of
  it.**
- `:1129` — **NEW since this item was written, and disclosed rather than left to
  be found.** `entry_digests`, added by PHASE-15's F-4 remediation (`875df3bb`),
  digests an imported `OQ-*` entry's headline. It is a new *kind*, not a second
  spelling of an existing one, so it does not widen the duplication — but it is a
  third inline `Fingerprint::new(crate::git::sha256(..))` and so widens the census
  any fix must cover. Additive and lawful under PHASE-15 EX-12 / S4, which permit
  a new precomputed digest input and forbid merging two existing derivations.

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

## It IS a standalone decision — the entanglement is void

**Struck 2026-07-30.** This section argued that DEC-105's borrow of
`src/review.rs::with_turn`'s lock-and-CAS shape pointed at the same extraction as
F-5's shared-primitive instinct — shared write-guard machinery owning both the
locking and the fingerprinting — so the two should be settled together.

**DEC-100 supersedes DEC-105 and voids that premise.** There is no lock, no
compare-and-swap, and no `LockGuard` extraction: PHASE-15 `EX-13`/`EX-14`/`EX-15`
are discharged by recorded infeasibility under `EX-5`, and `src/review.rs` is
untouched. So there is nothing left to sequence this against, and F-5 is a
standalone decision about digest-derivation single-sourcing. That is a
simplification — it removes the only stated reason the fix shape was thought to
be entangled with anything else.

## Sequencing

SL-233 PHASE-15 (RV-324 remediation) **deliberately excludes** this — its EX-12 is
a scope fence, so an unrequested consolidation there would pre-empt the owner's
decision and contaminate the phase diff. Settle at slice close.
