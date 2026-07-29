When the bug you are guarding against is **"the stored content was not updated"**,
an assertion over a digest, fingerprint, or revision **cannot fail on it**. Those
fields are updated by the very code path that forgot the content, so they agree
with the new world while the content still holds the old one.

## The instance that proves it

SL-233 PHASE-06 found `adopt_authored` (`src/design_run/run.rs:281-295`) updates
a section's `fingerprint` and never its `body` — `DerivedInput.authored_sections`
is a `BTreeMap<DesignId, Fingerprint>` (`run.rs:64`), so the pure layer is handed
digests without bodies and cannot adopt prose even in principle. A later
`materialise` then renders the stale body over the user's hand edit and
re-baselines the watermark to the reverted bytes.

The landed test over that exact path,
`adopt_authored_crosses_divergence_and_rebaselines_alone`
(`tests/e2e_design_state.rs:281`), hand-writes new prose over the document,
adopts it, and asserts **two** things: the watermark re-baselined, and the
section's fingerprint moved. Both hold *while the prose is silently reverted*.
The test is well-written, well-named, and structurally incapable of catching the
defect it appears to cover.

## The rule

For any "did the content actually change / actually persist" property, the
oracle is **the bytes**:

- read back the stored value (or the file) and compare it to the expected
  content, byte for byte;
- a digest/fingerprint/revision assertion is a *supplement*, never the oracle;
- be most suspicious where the digest is computed from the **new** input rather
  than from the **stored** value — that is the shape that guarantees agreement.

## How to spot it in review

Ask: *if the implementation updated only the metadata and dropped the payload,
would this test still pass?* If yes, the test is asserting the wrong thing. This
is the same family as the anti-theatre rule that five distinct wrong inputs must
not collapse into one `is_err()` — a test whose assertions cannot distinguish the
failure from success is one test wearing a name it has not earned.

Related: [[mem.pattern.doctrine.tdd-loop]].
