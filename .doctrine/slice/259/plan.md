# Implementation Plan SL-259: Truthful apply

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See glossary.md § reference forms. -->

## Overview

Six phases, one per unit of the design's four legs, ordered so that each phase
ends green and conformant rather than merely compiling.

The design is organised by class, not by issue, and this plan keeps that shape
deliberately. Seven of the eight originating items are instances of three
classes; planning them as eight fixes would reproduce exactly the failure the
design's `sec-1` names — `ISS-315` recurring three times, `ISS-450` still there
to be found by dogfooding.

## Re-verification against the current tree

`sec-9` `R0` is a standing instruction, not a caveat: three of this design's
factual claims failed external checking in `RV-365` and none was caught by its
author. Every file:line, symbol and count the design asserts was re-probed
before this plan was written. **Every path and symbol resolves.** Three results
are worth carrying forward, and they are recorded as entrance criteria rather
than left here as prose.

**One count is wrong.** The design says nine `UnknownKeys::SilentlyDropped`
contract rows; there are **eight** — `AcceptanceDeclaration`, `StageDeclaration`,
`DischargeDeclaration`, `AdoptAuthored`, `TraversalDeclaration`, `CreateRecord`,
`ReviewPolicyDeclaration` and `ApplyRequest`. The likely source of the nine is a
neighbouring doc comment counting `ApplyRequest::WRITER_ACTS`. Nothing in the
design's argument turns on the number, so this is a correction to carry, not a
premise to reconcile — it is `PHASE-03` `EN-2`.

**One encapsulation claim needs a footnote.** `sec-5` says no production code
outside `ChangeLog` reads `.rows`, and that is true. But three *test* sites do
(`snapshot.rs:767`, `:816`, `commands/design.rs:2878`), and the `StoredRow`
change will touch them. Containment holds; the work is slightly wider than the
claim reads. `PHASE-02` `EN-3`.

**One seam is not where the decision implies.** `DEC-247` asks that a term's
constructed `ValueKind` be checked against its declaration, but
`PayloadTerm::admit` does not have the event in scope — `ChangeEvent::ordered`
does, and already walks `payload_terms()`. The check needs a seam that sees
both. `PHASE-05` `EN-3`.

The research baseline reports drift (`design.md` added after it was stamped).
That drift is the design itself, which this plan read in full; the `pi` research
scripts remain broken, so no refresh round was run. The slice's seven
`design-target` selectors were checked against the tree and every one still
names a live path.

## Sequencing & Rationale

The legs are *not* planned in their numbered order, and the reason is a
dependency rather than a preference.

**`PHASE-01` first, because it changes behaviour without changing a type.**
Leg 2 adds emission at two act stores and removes `declare_node`'s create/update
branch split. It is the smallest unit, it closes two issues, and it settles
where rows come from *before* `PHASE-02` changes what a stored row is. Doing it
in the other order would mean writing new emission sites against a type that is
about to move.

**`PHASE-02` merges leg 4's storage and its disclosure into one phase, on
purpose.** They read as two units — `StoredRow` in `change_log.rs`, the
disclosure in `envelope.rs` — but splitting them would leave a phase boundary at
which a degraded read exists and is not disclosed, which is precisely what
`STD-003` forbids. A phase must end conformant, not merely green.

**`PHASE-03` through `PHASE-05` decompose leg 3 by axis, in the order the axes
depend on each other.** The key axis (`DEC-244`) is the mechanism the other two
ride: once refusal is driven off `payload_contract`'s inventory, the state axis
(`DEC-246`) is a second rule on the same admission surface rather than a rule
plus an exception, which is `sec-3`'s own framing. The value axis (`DEC-247`) is
last of the three because it touches `change_log.rs`'s `PayloadTerm` surface,
which `PHASE-02` has just rewritten.

**`PHASE-06` is leg 1, and it lands last precisely because it is leg 1.** The
hoist's whole content is *every check that can be hoisted, is*. Running it
before `PHASE-03`..`PHASE-05` would hoist a check set those phases then extend,
and the exit criterion would be satisfied at the moment it was written and false
by the end of the slice. Leg 1 is the last phase because it is a statement about
all the others.

## What this plan deliberately does not do

`PHASE-06` `EX-5` is an exit criterion written in the negative, which is unusual
enough to justify here. `DEC-250` was amended during `RV-365` from a claim about
a differential refusal between the two passes to **hoisting on principle, with
no witness behind it** — `F-2` found no reachable submission that clears pass 1
and fails pass 2, because pass 1 stands in provisional ids of the real shape
precisely so that both passes refuse identically.

A planner working from the pre-review framing would write a test that
manufactures such a case. That test would pass, and it would prove only that the
manufacture worked. Recording the prohibition as a criterion is cheaper than
trusting each future reader to re-derive it from `sec-6`.

Two other things stay out of scope by the same discipline. `ISS-361` remains
**open** against the residual late-check window: `EVD-028` ruled out its reported
mechanism and `EVD-029` names a suspect, but nothing ties its single witness to
either, and closing it on a plausible story is the species of untruth this slice
exists to remove. `IMP-446` remains open and trigger-bound — the stored
`Declaration` path is guarded by the retired-member roster alone, and `DEC-249`
forbids widening the `DEC-251` floor to help, so that is a residual to state
rather than a gap to quietly close.

## Verification posture

Every phase pins at the tier that breaks, not at the nearest convenient unit.

Two rules recur across the phases and are worth stating once. **A refusal's
reason is pinned, never merely its exit code** — a refusal that fires for the
wrong reason is a test passing for the wrong reason, and the same rule binds
`PHASE-02`'s disclosure, which pins the `why` rather than the bare fact.
**Snapshot compatibility is pinned at `snapshot::parse` over a literal legacy
fragment**, never as a round-trip over the inner type: a round-trip proves the
type reads what the type writes, which is not the question. The precedents to
copy are named in `PHASE-02` `VT-1`.

Two phases carry a criterion that is discharged by *not* changing something.
`PHASE-02` `VA-1` is the behaviour-preservation gate `AGENTS.md` requires when
shared machinery moves: `ChangeEvent`'s existing suites must be green
**unchanged**, and a diff against them is the failure, not the evidence.
`PHASE-02` `EX-6` protects `change_log.rs`'s two `const _: ()` proofs, which look
redundant and are not — `is_subset(&EMITTABLE, &READABLE)` is the only reader of
`EMITTABLE` in `src/`, so deleting it passes `cargo check` and fails
`cargo test --bin doctrine`. A per-compilation-unit trap with no attribute that
substitutes.

## Notes

`ChangeEvent::ordered` sorts an undeclared payload key to `usize::MAX` rather
than refusing it — the key-axis sibling of the `ValueKind` defect `PHASE-05`
repairs, at the same seam. It is out of scope here and not in the slice's
originating set; capture it as backlog rather than widening `PHASE-05`.
