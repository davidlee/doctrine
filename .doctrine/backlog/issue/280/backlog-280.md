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
