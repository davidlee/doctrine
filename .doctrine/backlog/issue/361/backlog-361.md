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

## Code reading, 2026-08-15 — the reported mechanism is not at the parse site

Read before designing anything. The mechanism as reported **cannot occur where the
title implies**, and hunting it there will waste a session.

`apply` (`src/commands/design.rs:1495`) parses first and propagates with `?`:

```rust
let prior = read_snapshot(root, slice)?;
let request: ApplyRequest =
    serde_json::from_str(payload).context("parse the apply payload as JSON")?;
let digest = crate::git::sha256(payload.as_bytes());
match design_run::run::admit(&prior, &request.envelope, &digest) ...
```

A top-level parse failure returns **before** `admit`, before any write. And
`admit` (`src/design_run/run.rs:207`) is **pure** — it reads `prior.receipts` and
writes nothing, so no receipt exists to lock anyone out of a submission that never
parsed.

### The likely reconstruction

Two known behaviours compose into the reported symptom:

1. **The top-level parse succeeded.** `ApplyRequest` carries
   `#[serde(flatten)] envelope` and therefore cannot carry
   `deny_unknown_fields` (`ISS-333`), and nine of the twelve wire structs lack the
   attribute outright (`SL-251` design `sec-2`). So a malformed payload can parse,
   apply, and bump the revision. Whatever reported a parse error was **downstream**
   of the top-level parse — a nested `from_str`, or a refusal the agent read as one.
2. **The resubmission reused the `submission_id`.** `admit` returns
   `Refusal::SubmissionReplayed` when a known submission id arrives with a
   *different* payload digest (`run.rs:218-227`) — which is exactly what a
   *corrected* payload is. This is the DEC-083 / DEC-086 idempotency guard working
   as designed, not a bug.

If that reconstruction holds, the novel defect is neither of those. It is that
**`SubmissionReplayed`'s refusal does not name its remedy** — mint a fresh
`submission_id` — so a caller holding a correction reads a deliberate guard as a
dead end. That belongs with `IMP-390`'s fourth candidate (refusals naming
remedies), and it is cheap.

### What to do

Do **not** design a fix from the title. Reproduce first:

1. Submit a payload with a deliberate nested type error and a fresh `submission_id`.
   Record whether the revision bumps and what the error text is.
2. Resubmit the corrected payload with the **same** id. Confirm the refusal is
   `SubmissionReplayed` and read its wording.
3. Resubmit with a **fresh** id. If it lands, the lockout is the guard plus a
   missing remedy, and this issue reduces to that.

Only if step 1 shows a revision bump on a *top-level* parse failure is there a
distinct defect here.

### Consequence for this item's weight

The 5.0 value anchor was set on the reported description ("outranks everything if
it reproduces"). This reading makes the severe reading less likely, so the anchor
is superseded at 3.0 — the residual is a real but bounded refusal-wording gap plus
an unreproduced tail.
