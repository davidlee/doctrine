# ISS-348: No verb reports the section fingerprints adopt_authored demands

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What happens

Hand-edit `design.md` while a design run is open, then run any mutating verb.
Doctrine refuses correctly and names the remedy:

```
Error: design.md has been edited outside this run — the watermark says `bc3c…`
and Doctrine reads `bd58…`. Ordinary mutation is refused against prose the
snapshot no longer describes; re-adopt the document with an `adopt_authored`
declaration naming its exact current fingerprint.
```

The **document-level** half of that is actionable: the refusal prints the
fingerprint Doctrine read, so the caller can paste it back. The **per-section**
half is not. `adopt_authored` also requires a complete and exact marker map —
every section id, each naming the fingerprint the caller read for it — and
nothing reports those values.

Submitting the map empty returns a count and nothing else:

```
Error: adopt_authored's marker map is not complete and exact: 10 missing,
0 unknown, 0 mismatched
```

No ids, no expected values. `doctrine design show` and `show --full` print only
the **stored** (pre-edit) fingerprints, truncated to 12 hex chars — by definition
the wrong values for every section that moved, which is exactly the set the
caller needs.

## Why it costs what it does

The only way through is to reconstruct the digest rule from source. That means
reading `authored_sections` in `src/commands/design.rs`, then `parse`,
`marker_lines`, `per_line` and `unescape_line` in `src/design_run/document.rs`,
to recover: *body is the bytes from the line after the marker to the next
marker's start, minus one trailing newline, per-line unescaped, sha256* — and
then reimplementing it outside the binary to produce the ten values.

Observed on `SL-253` at design run revision 79 → 80: roughly six tool calls of
source reading plus a throwaway script, for what should be one read. The
replication happened to be verifiable — the six unchanged sections reproduced
their stored 12-char prefixes exactly, a positive control on the rule — but an
agent that did not think to construct that control would not know whether its
values were right, and a wrong map is indistinguishable from a wrong rule.

This is a **refusal that names a remedy the caller cannot execute**, which is the
class of defect the design-run refusals are otherwise good at avoiding.

## Candidate fixes, cheapest first

1. **Have the refusal enumerate the missing/mismatched ids with the fingerprints
   Doctrine read.** It has already computed them — that is how it decided the map
   was inexact — so this is surfacing a value that exists rather than computing a
   new one. Smallest change, and it fixes the error at the point of use.
2. **A `--authored` flag on `design show`**, printing current authored digests
   beside the stored ones, untruncated. Useful beyond this refusal: it makes
   drift inspectable rather than only detectable.
3. **Accept `adopt_authored` with `sections` omitted as *adopt what you read*.**
   The document-level fingerprint already pins the exact bytes being adopted, and
   the section map is derived from those same bytes, so the map is not adding
   information — it is asserting that the caller decomposed what it adopted. That
   may be the point, and if so this option should be refused on that ground and
   the reasoning recorded; but it is worth deciding deliberately rather than
   inheriting.

Options 1 and 2 are complementary and neither forecloses the other. Option 3 is a
semantics question about what the marker map is *for*, and should be settled
before either is built, since a yes makes both optional.

## Provenance

Captured as a friction observation during `SL-253`'s design run (`/design`,
stage `reviewing`, revision 79 → 83) — record
`.doctrine/observations/records/1b/019ff82b-8c60-7860-a57b-56463bde8d1b.toml`,
which carries the raw session detail this item summarises.
