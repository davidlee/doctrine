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
4. `doctrine design adopt SL-N --dry-run --diff`.

**Step 4 is the readout.** The engine derives the section map from the
document itself (`SL-261`), and the dry run writes nothing, so the report *is*
the parser's decomposition: a `section_fingerprint_changed` row for each
section whose parsed body differs from the held one, an `unchanged` line for
the rest, a `reordered` line if marker order moved, and a `head:` line for a
whitespace-only preamble. `--diff` shows the held body against the body the
parser read, per changed section. A parse refusal (unknown, missing or
duplicate marker; the grammar rows) is printed as itself. Nothing is
hand-computed, and the evidence is end-to-end through the real binary.

**Probing a document you did not change.** Right after `materialise` the
document matches the watermark, and `adopt` short-circuits to its no-op line
before parsing — so a probe of materialise's own output reads nothing. Prepend
one blank line (a whitespace-only head moves no section body) and run the dry
run: every section `unchanged` plus the `head:` line means every body parsed
back byte for byte. The e2e suites' `parser_readout` helpers are this idiom.

Two corollaries worth keeping:

- **Acceptance is the interesting outcome, not refusal.** Most incumbent
  defects here are silent recoveries — a preamble dropped, a duplicate
  last-won, an unheld marker ignored. The demonstration is a dry run that
  *succeeds* where it should have refused.
- **A dry run never advances the revision**, so probes can be repeated freely.
  A real `adopt` does advance it; a later `design apply` against a stale
  `known_revision` fails on the conflict guard and looks like a negative
  result. See [[mem.pattern.testing.assert-bytes-not-digests]] for the sibling
  trap on the assertion side.

Related: [[mem.pattern.design.locate-incumbent-before-specifying]].
