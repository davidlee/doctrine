Claims about `authored_sections` / `render_document` must be RUN, not read —
RV-323 killed three revisions of the SL-233 marker-grammar sketch that were
written from reading the code. The cheap way to run them, without a scratch
test file in a dispatch tree and without re-implementing the parser:

1. `doctrine slice new` + `design start` in a **throwaway project outside the
   repo** (`git init` + `doctrine install -y` is enough). One slice per
   scenario, so revisions never interleave.
2. `design apply --input p.json` with `{"declare":[{"subject":"sec-1",
   "body":"…"}]}` to seat the state, then `design materialise`.
3. Hand-write the adversarial document over `.doctrine/slice/NNN/design.md`.
4. `design apply` with `adopt_authored: {fingerprint: sha256(file), sections:
   {id: sha256(expected_body)}}`.

**Step 4 is the readout.** `adopt_authored` refuses unless every declared
digest equals what Doctrine parsed out of the document (`run.rs:263-277`), so
"adoption succeeded against digest X" *is* the assertion "the parser read body
X". Nothing re-implements the parser, and the evidence is end-to-end through
the real binary.

Two corollaries worth keeping:

- **Acceptance is the interesting outcome, not refusal.** Most incumbent
  defects here are silent recoveries — a preamble dropped, a duplicate
  last-won, an unheld marker ignored. The demonstration is an adoption that
  *succeeds* where it should have refused.
- **Watch the revision.** Each successful apply bumps `known_revision`; a
  second probe against the stale number fails on the conflict guard and looks
  like a negative result. See [[mem.pattern.testing.assert-bytes-not-digests]]
  for the sibling trap on the assertion side.

Related: [[mem.pattern.design.locate-incumbent-before-specifying]].
