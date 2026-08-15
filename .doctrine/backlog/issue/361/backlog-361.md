# ISS-361: Unparseable submission applied and receipt-locked

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`design apply` reported a **JSON parse error** and nonetheless advanced the run to
a new revision. The idempotency receipt was then written against that submission,
so resubmitting the corrected payload was refused as a duplicate — the caller was
locked out of the fix by the record of the failure.

## Why it matters

This is the most severe entry in the sweep, on three counts.

1. **It breaks the failure contract.** A reported error must mean nothing landed.
   Here the error is reported *and* state moves, so an agent cannot use the exit
   signal to decide anything — the one invariant on which retry logic rests.
2. **It is unrecoverable in-run.** The receipt makes the correct payload
   inadmissible. There is no documented escape; the run is stuck at a revision it
   was told did not happen.
3. **It corrupts the ledger.** The revision that landed does not correspond to any
   payload the caller believes it sent, so the change log no longer reconstructs
   the run's actual history — which is the design run's whole reason to exist.

Related to but worse than `ISS-346` (*silently absorbs unknown payload keys*).
`ISS-346` is a success reported for a partial act; this is a failure reported for
an act that happened.

## Evidence

- `019fec3e-cf4e` — apply reported JSON parse error yet applied the payload

## Investigation needed

Only observed once, and the mechanism is not established. Before designing a fix,
establish whether the parse error and the applied payload are the **same**
submission or two — e.g. a partially-consumed stdin read, a retry inside the
client, or an error emitted after the write rather than before it. The ordering of
parse / persist / receipt in `design apply` is the place to start.

If it reproduces, this outranks everything else in `cluster:design-run`.

## References

- `ISS-346`, `ISS-333` — the silent-absorb pair
- `ISS-355` — successful apply prints no change row (why this went unnoticed)
- `DEC-024` — observation retries use caller-stable UUIDs, never content deduplication
- `QUE-219` — submission strictness against snapshot forward-compatibility
