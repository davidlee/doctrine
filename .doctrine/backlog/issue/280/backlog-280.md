# ISS-280: review contest records no durable rationale

`review contest` is the raiser's verb for rejecting a responder's disposition.
It flips a finding's `status` to `contested` and **stores nothing about why**.

The finding table carries `id`, `status`, `severity`, `title`, `detail`,
`disposition`, `response`. There is no field the contest's reasoning can land
in. `review verify` has the same shape — its `--note` is likewise not
persisted.

## Why this matters

ADR-007 makes the ledger the durable, diffable record of an adversarial review.
A contest is the most load-bearing move in the protocol: it is the raiser
asserting the fix is wrong or incomplete, and it is what forces another
revision. Losing its reasoning means the ledger records *that* a round happened
but not *what it established*.

## Evidence — RV-323, observed 2026-07-29

Round 2 contested F-1 and F-3. Both contests were substantively right and both
drove a full sketch revision:

- **F-1** — the repair silently altered prose the caller declared, and its
  justification conflated trimming with framing.
- **F-3** — "the body's first ATX heading" was shipped as a total function when
  it is partial.

Neither sentence exists anywhere in `.doctrine/review/323/review-323.toml`. The
reasoning survived only because it came back in the reviewer subprocess's chat
summary, which is ephemeral. Reading the committed ledger afterwards shows two
findings that went `answered → contested → answered` with no record of what the
contest said. A later auditor cannot reconstruct why rev 2 became rev 3, and the
responder's round-2 `response` is the *only* account of the contest — written by
the party the contest was against.

The prior handover already noted this as "unfiled: `review contest` has no
durable rationale field". This is that filing, with evidence.

## Second instance — RV-346 round 6, observed 2026-08-08

Larger, and it sharpens the *shape* of the fix rather than just re-confirming
the gap. An external reviewer (codex) took one turn on the SL-248 design review
and contested **six** findings at once: `F-24`, `F-25`, `F-28`, `F-29`, `F-30`,
`F-32`. Every contest was substantively right and every one drove a real
revision — `F-29`'s in particular was a compile failure the reviewer reproduced
(`rustix::process` is feature-gated) that no amount of reading would have found.

None of the six reasons is in `.doctrine/review/346/review-346.toml`. They
survived only because they came back in the MCP call's return value, and the
responder had to copy them out by hand into a scratch file before the value
scrolled out of context. That is a manual, lossy step standing between an
adversarial round and the record of what it established.

Two things this instance adds to the RV-323 evidence:

- **The flat history is the binding constraint, not a nice-to-have.** RV-323
  reached `rounds 17`; RV-346 is at `rounds 121` across 37 findings, several of
  which have gone `answered → contested → answered` more than once. `F-28` alone
  was raised in round 4 and contested in rounds 5 and 6. A single overwritable
  contest note would already have lost two of its three reasons. The per-round
  record is the shape to build.
- **The asymmetry is visible to the reader, not just to the protocol.** Reading
  RV-346 today, every finding carries the responder's account of the contest —
  written by the party the contest was against — and nothing from the raiser.
  On a ledger this size that is the difference between a durable adversarial
  record and a durable record of one side of it.

The workaround this session used (hand-copying the reviewer's summary into the
next `dispose --response`) is not a mitigation worth documenting as practice: it
puts the raiser's reasoning inside a responder-owned field, which misattributes
it in exactly the way the ledger's role split exists to prevent.

Captured live as an observation: `.doctrine/observations/records/d7/`.

## Sketch of a fix

Add a raiser-owned, append-only field per round — symmetric with the
responder-owned `disposition`/`response` pair the finding already has. The
existing `--note` flags on `contest` and `verify` are the natural carriers; they
are accepted and discarded today, which is the smallest possible gap to close.

Note the finding's history is currently flat: one `disposition` + one `response`
overwritten per round. RV-323 reached `rounds 17` on 5 findings, so a per-round
record is the more useful shape, but a single durable contest note would already
be a large improvement over none.

Related: [[mem.signpost.doctrine.review]], ADR-007.
