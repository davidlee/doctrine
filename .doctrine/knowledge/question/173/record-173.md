# QUE-173: Should verification invalidation be digest-based rather than git-derived

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The question

SL-230 detects a stale attestation by asking git: *have any commits touched this
memory's own item directory since `verified_sha`?* That works, needs no schema
change, and rides machinery that already exists (EVD-001).

It has two blind spots. Should we instead stamp a **content digest of the body**
at verify time, and invalidate by comparing digests?

## What the git-derived approach cannot see

- **Uncommitted edits.** `rev-list` counts commits, so a hand-edit that has not
  landed is invisible until it does. A transient window rather than a durable
  lie, but it is exactly the window in which an agent is most likely to read the
  memory back.
- **Masters.** Repo-empty orientation masters (`repo = ""`, `anchor_kind = none`,
  ADR-002) carry **no `verified_sha` at all** — verify stamps only the review
  axis for them. There is no anchor to diff against, so git-derived detection
  cannot work for the masters tree even in principle. Since masters are
  hand-edit-only (no write verb resolves the repo-root `memory/` tree), this is
  precisely the population most exposed to silent body drift.

## What a digest would cost

A new persisted field on the verification axis — a schema change, which SL-230
explicitly excluded. It would also need a decision on what exactly is hashed
(body only, or body + claim-bearing metadata such as summary/title — see the
related open question about whether `--summary` should clear verification too).

Against that: a digest is git-independent and path-independent. It works for
masters, for uncommitted state, outside a repo entirely, and it cannot be fooled
by history rewriting.

## Status

Deferred from SL-230, deliberately, not overlooked. SL-230 ships the git-derived
check in `validate` because it closes the *committed* hand-edit bypass — the one
that inverts the guardrail — with no schema change. This question owns the
residual.

Relates to: EVD-001 (the observed defect), SL-230 D4/D5 and OQ-3.
