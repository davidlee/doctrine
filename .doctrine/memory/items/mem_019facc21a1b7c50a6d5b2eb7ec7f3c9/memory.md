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

## What `body` is, exactly (SL-244)

Step 4 above says `sha256(expected_body)` without defining `body`, and getting
it wrong reads as a parser disagreement rather than as your own arithmetic.

Framing (`document.rs` module doc): each block is **the marker line, a newline,
the body verbatim, then ONE newline**, and blocks concatenate with no separator.
So:

    body = <everything after the marker line> minus exactly ONE trailing newline

Interior sections and the **last** section follow the same rule — the trap is
that the file's final newline is that section's framing newline, not part of its
body. A naive line-split leaves it attached and only the last section's digest
disagrees, which looks like an EOF-handling bug in Doctrine and is not.

Two more, cheap to know:

- `git::sha256` is plain hex sha256 of the bytes; the document digest is over
  the whole file, unframed.
- `design show` **truncates fingerprints to 12 hex chars**. Compute the full
  digest yourself; never paste the displayed value into `adopt_authored`.

Positive control before trusting any of it: recompute an *unchanged* section's
digest and check it equals what `design show` displays. If the untouched ones
match and only the edited one differs, the parse is right.
