# Implementation Plan SL-249: Knowledge facet write seam

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Eight phases in two movements. `PHASE-01` and `PHASE-02` are the wire fix —
together, exactly what would have prevented the SL-248 loss. `PHASE-08` and
`PHASE-03` through `PHASE-06` build the edit surfaces the corpus measurement
demands. `PHASE-07` is the governance amendment, and its landing point is the
plan's one open question (below).

Execution order is the array order in `plan.toml`, and `PHASE-08` sits third —
ids are immutable and never renumbered, so a phase added during revision takes
the next free id wherever it belongs in the sequence.

`DEC-165` — the ruling that objective 4 does not gate objectives 1–3 — is what
makes that split available, and `R6` warns it erodes under convenience: Phase A
is small, the facet work is adjacent, and *just add the table while we're here*
is how the ordering is lost.

## Sequencing & Rationale

### Why the prose payload precedes the refusal

The design presents objective 3 (the inert-key refusal) and the prose half of
objective 2 as one phase. They are split here, and the order is the reason.

`PHASE-02`'s refusal, on a `cp-` subject carrying `body`, names
`dispose.create.body` as the key the caller actually wanted. If the refusal
ships first, that remedy names a key that does not exist — the message is
aspirational, and the agent it refuses has nowhere to go. So the slot is built
first and the sink is made loud second. The dependency runs one way and the
plan follows it.

### Why the boundary is structural rather than disciplinary

`R6`'s mitigation in the design is an exit criterion asserting that nothing
Phase A ships references a symbol from the facet table, with §10 noting *the
cheapest review is still a grep*. A grep over a symbol nobody has written yet
asserts very little; it is the vacuous-absence shape.

The plan gets the property for free instead. `facet_fields` is authored in
`PHASE-03`, so a reference to it from `PHASE-01` or `PHASE-02` would not
compile. The criterion that remains (`PHASE-01/EX-6`, `PHASE-02/EX-6`) is
therefore about the phase's own **diff** — a subject that exists and can be
read — rather than about the absence of a thing nobody has authored.

### Why the table and the tripwire share a phase

Not theme. `Cargo.toml` sets `warnings = "deny"` with `unused = "deny"`, which
makes `dead_code` a hard build error — verified during planning, not assumed.
An item with no production consumer therefore cannot land alone, and the
project's established alternative is to stage the whole chain behind
`cfg_attr(not(test), expect(dead_code, reason = …))`, where every link carries
its own attribute and they all retire together when the first production caller
arrives.

That is real friction to buy nothing. The `doctor` tripwire is the cheapest
genuine consumer of `facet_fields` — it reads the table, needs no write seam,
and is owed by the slice anyway — so pairing them keeps the phase honest with
no staged attributes at all.

One residual is expected rather than discovered: each field's *shape* has no
reader until `PHASE-04`'s writer needs it. If that trips `dead_code`, the
staging attribute is the sanctioned answer for exactly that field, and it
retires in `PHASE-04`.

### Why the kind-blind verb goes early and alone

`PHASE-08` was added in revision, splitting the prose tier out of what was the
plan's largest phase.

`OQ-1` settled the surface by **tier**: invariant fields kind-blind, `[facet]`
kind-dispatched. The kind-blind half — title, tags, `--body` — depends on
nothing this slice builds. It rides `dep_seq` and `entity::write_body` exactly
as `memory edit` does, needs no table, no write posture and no per-kind
dispatch, and so has no reason to wait behind `PHASE-03`.

Shipping it third buys three things: `PHASE-04` shrinks to the kind-dispatched
half it is actually about; concept records get their whole edit surface at once,
since a concept carries no facet by design and its content is its prose
(`DEC-172`); and the prose tier stops being a passenger in a phase whose risk is
concentrated in a shared writer's posture change.

### Why `settle` is its own phase

`PHASE-04` could absorb it — it is another verb over the same seam. It is kept
separate because the design's own review left a live question against it
(§10 press item 2): `DEC-178`'s case was partly that a settlement is a coupled
multi-write, and after `RV-349`'s `F-2` it is one write of one document, which
is what `knowledge edit question` also is. The remaining case is `DEC-062`'s and
stands on its own — but it is now the *whole* case rather than the larger half
of one.

A phase boundary is where that gets answered deliberately. Folded into
`PHASE-04`, it gets answered by whoever is mid-implementation, or not at all.
`PHASE-05/VA-1` makes it an obligation.

### How the criteria are written

Two rules, applied throughout, and both are reactions to what this slice has
already cost.

**No criterion carries a count.** Every count in this design moves: the `four`
occurrences become one when the REV lands; the `I10` matrix grows with every
wire key; the facet slot total changes the moment a kind gains a field. A
criterion pinned to a number is an amendment waiting to happen, and `EN-`/`EX-`/
`VT-` ids are immutable, so amendments append rather than replace. The criteria
therefore bind **identities and equalities** — the union equals the serde key
set, the retained set equals the row, total occurrences equal the allowlist's
sum — which hold at every count.

**Closure is proved by a generator, never by an inventory.** A precise
inventory reads as exhaustive and is not; this slice has now paid for that
three times, and `R4`'s five review rounds are all the same error. So where a
criterion needs *and nothing was missed*, it names the mechanism that fails on
the miss: the matrix is generated by iterating both vocabularies rather than
written cell by cell (`PHASE-02/EX-4`), the table pins compare derived sets
rather than typed lists (`PHASE-03/EX-2`…`EX-4`), the canary iterates
`kinds::RECORD` (`PHASE-07/VT-1`), and the `dead_code` denial is what proves
nothing was left behind on the source side.

The corollary is `R10`: a generated matrix is trimmed by deleting a loop, which
is visible in review, where a hand-written one is trimmed by deleting rows
nobody misses.

**Behaviour preservation is asserted about the diff, not the result.** The
design names two suites that must stay green *unchanged* — the knowledge
round-trip suite (`I1`) and `doctrine risk set`'s (`I8`). A suite edited to stay
green passes, so the criteria (`PHASE-03/EX-9`, `PHASE-04/EX-9`,
`PHASE-04/VA-1`) require `git diff` to show them unmodified. Passing is not the
claim; passing unedited is.

## Notes

### Where objective 4 lands — settled

**The REV lands in `PHASE-07`**, not at reconcile. Settled by the user on
2026-08-08; `PHASE-07/EN-2` no longer holds the phase.

The design says the amendment lands at reconcile, in §3 and §5.3. Planning
probed `revision apply` and resolved §6's stated unknown — it auto-lands only
`status` rows and *surfaces* introduce/create/modify/move/prose rows for manual
handling — so a prose-heavy amendment is hand-authored at whichever stage owns
it, and the choice is about *which stage*, not about how much work it is.

The deciding argument is that `D9`'s canary is **code**. Authored in a phase
while the prose lands at reconcile, it is red for the whole interval, and no
phase may end red. The canary is `R4`'s *control*; `R4` is the risk this slice
has already recurred on once via `SL-159`; and a control left red across a
reconcile cycle is one somebody disables — which is `D9`'s own stated failure
mode. Shipping the control red would be `R4` recurring through the very
mechanism built to catch it.

The two rejected readings, recorded so they are not re-derived: moving objective
4 wholly to reconcile honours `ADR-013` literally but puts code authorship in a
stage that does not otherwise write code; splitting — fixture tests in the
phase, the live-corpus assertion at reconcile — is defensible but cuts a small
test across two stages for no gain now that the whole amendment moves.

**A plan does not outrank its design.** The departure is therefore carried into
reconcile as a wording item (`PHASE-07/EX-11`) rather than absorbed silently.
Reconcile settles design.md's §3 and §5.3 to say what actually happened.

### Resolved during planning, worth not re-deriving

- `entity::write_body` creates an absent file under both `BodyMode`s
  (`PHASE-01/EN-3`) — §10 press item 4's first claim, now evidence.
- `doctor_checks.rs`'s `*_findings(root) -> Vec<Finding>` is the tripwire's
  precedent (`PHASE-03/EN-3`) — press item 4's third claim.
- `revision apply` does not auto-apply prose rows (above) — §6's unknown.
- The `dead_code` denial is real and is a hard error, not a warning.

### Owed at close, not by a phase

The design's closure criteria include *`IMP-403` leads 1 and 2 are demonstrably
closed; leads 3–5 carry their own follow-up items*. Leads 1 and 2 are what the
phases build. Leads 3–5 become backlog items, which is harvest work rather than
phase work — recorded here so `/close` picks them up instead of them expiring
with the slice.

The same applies to the two inquiries the design left open into reconcile
(`inq-7`, `inq-9`) and to `D8a`'s correction of `DEC-168`'s recorded rationale.
All three are `PHASE-07/EX-10`'s business if the amendment lands in a phase, and
reconcile's if it does not — they move with the landing point, not
independently of it.

### Deliberately not in the plan

- **No refactor phase.** `DEC-179`: the edit transaction is already extracted —
  `memory`, `backlog` and `spec` all ride `dep_seq` and `entity::write_body` —
  and what stays bespoke is each verb's flag set, which shares no field with the
  others. This slice adds a caller and one parameter.
- **No `src/knowledge.rs` split.** `R8` records that the module grows on both
  axes and that splitting it is a real improvement and a different slice. Doing
  it here would put a layering refactor in front of the data-loss fix.
- **No missing-key mirror** of the tripwire (`PHASE-03/EX-8`). Cheap, tempting,
  and needs a migration story this slice does not owe.
