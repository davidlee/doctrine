# IMP-430: Per-section read verb for design sections

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

There is no `design show --section <id>`, and `Declaration.body` is **whole-section**.
So changing a few lines of one section means:

1. recovering the section's current body (no verb emits it — it must be scraped
   from `materialise` output or raw state);
2. scripting a marker-split to isolate it;
3. re-emitting the **entire** section body in the payload.

Measured cost: a ~90 KB payload to change a paragraph.

## Why it matters

This is the dominant per-revision token cost of the managed design run, and it is
pure overhead — the bytes re-sent are byte-identical to the bytes already stored.
It scales with document size, so it gets worse exactly as a design matures, and it
biases agents toward fewer, larger, less reviewable edits.

It also interacts badly with `ISS-320` (*re-adopting an edited `design.md` needs a
section map nothing emits*): the hand-edit escape hatch is gated on fingerprints
that no verb reports, so the cheap path out of the expensive path is also closed.

## Evidence

- `019ffa30-2aec` — no per-section read verb; every section edit round-trips whole body
- `019ff661-d3db` — surgical edit forces full section-body resubmission

## Shape of a fix

Two halves, independently useful:

1. **Read** — `design show --section <id>` emitting the stored body verbatim
   (plus its current fingerprint, which also serves `ISS-320` / `ISS-348`).
2. **Write** — a patch-shaped declaration act. The cheapest honest form is
   anchored replace (`{ section, expect_fingerprint, find, replace }`), which
   refuses on a moved anchor rather than silently mis-patching.

(1) alone removes the scrape step and is worth landing first; it is also the
prerequisite for any (2) that wants to verify before writing.

## References

- `IMP-390` — envelope reports state, not what to do next (the adjacent affordance gap)
- `ISS-320`, `ISS-348` — the section-fingerprint reporting gap
- `RFC-011` — token-efficiency benchmarking (this is a headline cost)
