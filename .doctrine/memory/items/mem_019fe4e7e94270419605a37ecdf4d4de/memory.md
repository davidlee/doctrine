## The evidence

`SL-249` tracked this as risk `R-inventory` and it fired **six** times across
eight phases, always the same shape — a count or list carried from a prior
reading, wrong when re-derived against the tree:

1. A Finding category's touch sites: 6, not the 5 the design said.
2. `PHASE-04` staged 5 `expect(dead_code)` attributes, not the planned 3.
3. `VT-3` shipped 9 refusal cases against a mandate naming 6.
4. The paired-form census: 8 of 28 present, where a co-presence read said 12.
5. `ISS-332`: a closed-enum list said three where four ship in `src/knowledge.rs`.
6. **The reconcile ledger's own inventory** — written to hand the other five
   forward — said the undeclared cell held three files. It held **eleven**, and
   it was silent on the undelivered cell entirely. Found by `RV-351` running
   `slice conformance`, not by a careful reader.

Six in one slice is not bad luck. Inventories decay silently — nothing fails
when a count goes stale, so nothing announces it.

**#6 is the load-bearing one.** The artefact written *specifically to stop this
pattern* fell to it, and a mechanical check caught what re-reading could not.
Nobody's care level was the variable. This memory itself sat stale at "five"
for a day after the audit had found six — which is the seventh firing, and the
reason the count above is now derived from `RV-351` rather than from the
version of this memory you may have already read.

## Why the counts go wrong

- **Co-presence is not the property.** #4 came from grepping for a prefix and a
  long name in the same file, when the property was strict adjacency. A loose
  proxy always over-counts.
- **The tree moves under a locked document.** #1 and #2 were correct when
  written and stale by execution.
- **The author's own recent derivation feels authoritative.** In one task of this
  slice a `four` was *introduced* by the very sentence that removed another —
  caught only by re-running the census, not by re-reading the edit.
- **A summary inherits its source's staleness silently.** #6 and this memory's
  own drift are the same move: a document that restates another document's count
  acquires a second place for that count to be wrong, and no seam between them.

## How to apply

- Re-derive the number at the point of use, mechanically, even if you derived it
  yourself an hour ago. `grep -c` costs nothing; the count in your head costs a
  phase.
- Where a count is load-bearing across a whole phase, encode it as a test rather
  than restating it in prose — an arithmetic identity that fails loudly beats an
  enumeration that goes quietly stale.
- Beware the proxy: check that what you are counting *is* the property, not
  something correlated with it.
- Run the mechanical check *early* rather than as a closing formality. On
  `SL-249`, `slice conformance` owed nothing to anyone's recollection and was
  the only reader that found #6.

Related: [[mem.pattern.verification.guard-blind-to-its-own-residue]] — the same
slice's fifth firing, and the argument for a second differently-shaped reader.
