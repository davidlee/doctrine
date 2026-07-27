# DEC-054: validate's two unknowns unify: ISS-257 absorbed into SL-232

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The decision

**ISS-257 is absorbed into SL-232** (`SL-232 fulfils ISS-257`) and designed as
one mechanism with RV-307 **F-36**, under a new objective 7: *`validate` renders
every undeterminable explicitly.*

Taken by the user during SL-232's design round, on the responder's
recommendation, after reading SL-230's reconciliation audit.

## Why — they are the same defect wearing two hats

`validate` has two ways of not knowing something and saying nothing:

| unknown | today | population |
|---|---|---|
| non-ancestor `verified_sha` → `commits_touching` returns `None` | binds in a let-chain, falls out, emits no finding | **67 of 115** anchored memories (reach capped at 41.7%) |
| a scope entry contributes no observable evidence | never probed at all | **59** entries across 55 memories |

The first is ISS-257 (RV-313 F-2, pre-existing in Check 2; SL-230's Check 4
inherited it). The second is F-36, one of the two blockers that split SL-232 out
of SL-230 in the first place. Building them as two mechanisms would implement the
same epistemic honesty twice — the parallel implementation A1 rejected, one level
up.

## The obligation is approved governance, not a preference

**REV-041** (done, approved) amended SPEC-007 § "Git-anchored staleness" on
RV-313 F-6. It split the sentence along the seam the evidence marked: the
five-state resolution is the **render contract**, binding `find`/`retrieve`; and

> The prohibition on **silent over-trust is surface-independent**: it binds *any*
> git-anchored staleness computation in this engine, including `memory validate`'s
> health checks. … A surface that emits findings rather than states discharges
> this by emitting a finding, not by falling silent.

That last clause is the mechanism-level anchor F-36 was missing. F-36's charge
was that DEC-020 had been *"applied normatively and not mechanistically"* — it
still needs a probe, but the obligation now rests on amended spec text rather
than on a decision record alone. ISS-257 is correspondingly a **conformance fix**,
not merely an improvement.

## What this falsifies in the inherited design

- **D11 — "`validate` keeps its existing raw seam" cannot survive.** Objective 7
  must touch the same two call sites.
- **R7's four-defect enumeration is incomplete.** The `None`-swallow is a fifth,
  the largest by population, and the only one that is **non-conformant** rather
  than merely weak. The inherited text presents that list as exhaustive
  (*"named rather than silently inherited"*), which is now false.

Both are restated in the design rather than silently corrected — the same
discipline RV-307 F-34/F-39 established after corrections repeatedly landed in
one tier while another asserted what they replaced.

## Three validate concerns, and only two unify

The inherited text blurs these; keeping them apart is load-bearing:

| concern | question | owner |
|---|---|---|
| ISS-257 — non-ancestor anchor renders as clean | *can I determine drift?* | objective 7 |
| F-36 — no contribution probe | *can I observe this evidence?* | objective 7 |
| R7 / IMP-317 — raw, `paths`-gated scope seam | *which paths do I ask about?* | objective 4, gated on OQ-2 |

## Implementation direction, not yet detail

- The ancestry guard in `commits_touching` (`src/git.rs:2493`) is **correct and
  stays** — a non-ancestor `since` over-counts a set difference, so `None` is the
  documented no-over-trust posture. The defect is in how the callers consume it.
- The correct shape already exists one module away: `src/coverage.rs:150-166`
  defines `IsStale{Fresh,Stale,Unknown}` over the **same** `commits_touching`
  seam with the contract `None => Unknown`, and `coverage_scan` degrades cells to
  `Unknown` rather than dropping them. **Ride it; do not re-invent it.**
- F-36 additionally needs the **continuation policy** F-29 identified — what a
  corpus-wide run does when one memory's surface cannot be probed. A corpus-wide
  verb must not abort the corpus for one bad row.

## Consequences

- **SL-232 is now a `validate` slice as much as a `verify` slice.** Its title
  does not say so; the scope document states it explicitly.
- **A governance gap surfaces.** `validate` appears in SPEC-007 exactly **once**
  — the sentence REV-041 added. REV-034's amendment inventory was drawn for
  `verify` before objective 7 existed and must be re-taken during design, not
  deferred to close. REQ-147's title is the retired contract verbatim, so the
  queried-surface trap of RV-307 F-39 applies to any requirement left unamended.
- **New risk R-G.** `memory_health_findings` is consumed corpus-wide, so the
  behaviour-preservation gate applies, and the tri-state must not convert a
  silent exemption into a noisy one for the 67 previously-invisible rows.

## Relations

- SL-232 (objective 7), ISS-257, RV-313 F-2 / F-6, REV-041, SPEC-007, RV-307
  F-36 / F-29, DEC-020, DEC-027, IMP-317 (kept separate).
