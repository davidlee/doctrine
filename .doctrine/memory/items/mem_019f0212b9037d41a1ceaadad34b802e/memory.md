# prepare-review full re-run is not idempotent until the PHASE-05 gate

`commit_boundaries` (the ISS-039 ledger splice at the top of `prepare_review`,
`src/dispatch.rs`) **is** content-idempotent: it re-serialises the parsed working
ledger to canonical TOML, splices it into the tip tree, and compares the candidate
**tree** oid to the current tip tree — identical content ⇒ identical tree ⇒ no
commit, no ref advance (design SL-154 F1).

But a **full second `prepare_review` run is NOT idempotent** at PHASE-04:

- the first run already created `review/<slice>` + `phase/<slice>-NN` via
  **zero-oid CAS**; the second run's deterministic re-projection hits those refs →
  `RefCas::Moved` → rows flip **Failed**, the run `bail!`s "stale refs";
- `with_journaled_projection` commits the journal **twice** (Pending pre-apply,
  then the post-apply status) — both differ from the committed Verified journal, so
  each **advances `dispatch/<slice>`** and the recovery commit persists the **Failed**
  rows over the Verified ones. Pre-existing, design-acknowledged behaviour, pinned by
  `e2e_dispatch_sync.rs` `stale_review_ref_is_reported_not_clobbered` (EX-5) and
  `refused_row_persists_failed_status_in_committed_journal` (VT-4).

The clean re-run only exists once **PHASE-05** adds the projection-source guard +
completeness gate that **halt BEFORE the ref projection** (design F1) — a halt
creates no refs, so the operator's `record-delta → commit-ledger → re-run` collides
with nothing.

## How to apply

- **Verifying `commit_boundaries` idempotency:** assert at the **commit grain** —
  exactly one `ledger: boundaries` commit after two runs
  (`git log --grep "ledger: boundaries" dispatch/<slice>`) and a **stable committed
  blob oid** — NOT a full-rerun `dispatch/<slice>` tip-equality assertion (it will
  fail on the journal churn). This is the SL-154 PHASE-04 VT-2 ruling.
- **Don't diagnose the journal churn / Failed-rewrite on a re-run as a bug** — it is
  the absence of the PHASE-05 gate, not a regression.

See [[mem.pattern.dispatch.prepare-review-plumbing-desync-reverts-journal]] (the
plumbing-advance / journal-revert cousin). Born SL-154 PHASE-04, anchor `d82ec4b7`.
