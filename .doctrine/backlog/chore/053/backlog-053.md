# CHR-053: RFC-025 cleanup pass after the capsule spike

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Lodged by **SL-241 PHASE-06 (EX-10)** so these do not survive only as prose in a
slice that is about to close.

RFC-025 was written before the spike ran. The spike answered several of the
things it left open and contradicted at least one of its assumptions, so the RFC
now carries statements the evidence has moved past. This is the pass that brings
it current.

## What needs doing

1. **Fold in the spike evidence.** `.doctrine/rfc/025/evidence/` holds the
   result — `README.md` (the verdict and its seven limits), `matrix.md` (the
   sixteen hazard rows), `guards.md`, `measurements.md`, `results-c3.tsv` (the
   authority), and the archived phase sheets. `.doctrine/rfc/025/go-no-go.md` is
   the scoped verdict. The RFC body still reads as if none of this exists.

2. **The census B1 note.** `mechanism-census.md:40` marks B1 (the on-disk worker
   marker) **DELETE for dispatch** with a `†` pointing at a note
   (`:179`) that qualifies it against *"the uniform-subprocess model this census
   assumes"*. The claude arm's worker self-commits through the gated
   `worker_commit` tool, so that assumption no longer holds uniformly. The note
   and the verdict need reconciling against both arms.

3. **RT-3 — the topology decision** (`red-team.md:68`), open and marked
   *design-change*. The token-efficiency invariant needs it, and it gates its own
   dependents (escalation, and RT-6's notification-channel auth discussion).

4. **RT-9 — the forensic-archive storage-tier ruling** (`red-team.md:151`),
   marked *design-change (small)*. The spike settled what the archive artifact
   *is* per mechanism (EVD-008, EVD-010) but not which tier it lands in.

## What this is NOT

- **Not** the allowlist / QUE-204 follow-on work (D-P05-17) — that is
  **IMP-397**, and it is much larger. (This bullet previously read *"already
  placed as a follow-on slice"*. It was **decided** to be one by DEC-129, not
  placed as one: no slice and no work item existed, so the largest piece of
  forward work the spike found surfaced in no `backlog list` and no `doctrine
  next`. Lodged as IMP-397 at SL-241 close, 2026-08-03.)
- **Not** the REV scoping for the census DELETE rows — that is **CHR-054**.
- **Not** a re-run of anything. The rig is complete and its results are banked;
  this is document work.

## Related

- **SL-241** — the capsule spike rig, which produced the evidence.
- **RFC-025** — the subject.
- `.doctrine/rfc/025/go-no-go.md` — the scoped verdict, and its § 5 outstanding
  work list, which this item does not duplicate.
