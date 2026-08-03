# EVD-008: Both ingestion mechanisms preserve the worker's commit identity into quarantine

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Scope first — what this is NOT

**No row in the C3 matrix scores forensic retention.** § 5.4's hazard set targets
admission, not archival; there is no hazard whose observable is "the worker's
history survived". So this record does not report a scored result, and must not
be cited as one. It reports what the sixteen rows incidentally establish about
retention, which is weaker but not nothing.

## Datum

Under **both** mechanisms the worker's own commit survives ingestion as a
first-class object, and the pipeline demonstrably reads it:

- Every ancestry refusal cites the capsule-side tip **computed trusted-side from
  the transferred objects** — H1's four cells (light/heavy × fetch/bundle) each
  record `S=<oid> is NOT a descendant of B=<oid>`, with distinct per-cell S
  values. The parent could not compute that without holding the worker's commit.
- H3's four cells likewise name the *specific* merge commit inside `B..S`
  (`merge commit(s) in B..S: c9392831…`), which is a walk over transferred
  history, not a flag.
- F-P05-28's diagnosis independently verified the quarantine's object store on
  the M-B arm: all four packs pass `verify-pack`, `multi-pack-index verify` is
  OK, and **no object damage** was found even after three killed harvests.
- On a quarantine killed before any transfer, in-place resume returns *the same
  OID as a fresh clone* (`d7462447…`, both legs exit 0) — § 5.6's idempotence,
  observed.

## What it bears on

RFC-025 commits to retaining worker commit history as forensic evidence, and
QUE-200 uses that invariant to rank its candidates. On the evidence here the
invariant is **satisfied by both candidate 1 (fetch into quarantine) and
candidate 2 (bundle)** — so forensic completeness does **not** discriminate
them. It only discriminates against **candidate 3 (tree materialization)**,
which forfeits history by construction. The probe supplies no measurement
against candidate 3: it was deliberately not rigged, on QUE-200's own
recommendation ("skip materialization unless both fail something"), and neither
fetch nor bundle failed anything that would reopen it.

The one place the mechanisms differ on the forensic axis is **what the archive
artifact is**, not whether history survives. QUE-200's candidate 2 argues the
bundle "doubles verbatim as the forensic archive artifact"; the cost of that
convenience is measured separately in EVD-010.

## Related

- [[safe-capsule-ingestion-mechanism]] — QUE-200, the question this informs.
- EVD-010 — the trusted-side cost of the bundle-as-artifact story.
- SL-241 PHASE-05; `~/capsules/probes/c3/results.tsv` rows H1, H3, H15.
- Finding F-P05-28 (quarantine object-store forensics; idempotent resume).
