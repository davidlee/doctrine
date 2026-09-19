# Implementation Plan SL-246: Entity reads carry their knowledge records

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML.

## Overview

Six phases, building inward-out: one shared data structure, then a pure
selection over it, then the render, then two callers, then a deletion. The
design's §5.1 already splits the system three ways — selection decides which
records and what to call them, rendering turns records into bytes, and the
command layer composes a subject with the rendered block — and the phase
boundaries follow that split rather than cutting across it.

The slice's real risk is not difficulty. It is that `Skip` must be byte-identical
to a surface that already exists, while the same code path grows two new levels
and a verb changes its default. So the sequencing is chosen to keep the
behaviour-preservation alarm (`C2`) armed and meaningful from the first commit to
the last.

## Sequencing & Rationale

**PHASE-01 goes first because it is the only phase that changes nothing.**
`facet_fields` is a pure refactor: two hand-written per-kind projections collapse
into one ordered table that both arms render from, and `knowledge show` does not
move a byte. That makes it the cheapest possible place to discover that the two
projections disagreed about something — and if they do, it surfaces before any
new behaviour is layered on top of them. It also establishes the instrument every
later phase relies on: from here, a red suite in `tests/e2e_knowledge_cli_golden.rs`
means `Skip` stopped being byte-identical, not that a golden needs updating.

**PHASE-02 is pure and testable without disk or CLI.** Selection is a function
from an already-derived view to a list. Deduplication, source-kind filtering, and
numeric-past-999 ordering are each a unit test here and would each be an
end-to-end golden anywhere later. Keeping it a pure function is also what lets
the design read and, later, the transitive closure be *callers* rather than
second traversals — objective 3.

**PHASE-03 is where the review's hardest lessons are cashed.** Three separate
findings across rounds 3 to 5 — a guarantee claimed from a return type, a JSON arm
with no producer, and an empty-state policy sited where its inputs were
unreachable — turned out to be one defect: there was no per-record layer. The
phase exists as its own unit so that layer is built deliberately rather than
discovered. Its exit criteria are written as shape claims (a map not a
`filter_map`; markers composed where the reference and the root are held; the
absence of the block expressed in an `Option`) because that is what the
invariants actually rest on.

**PHASE-04 before PHASE-05 is the load-bearing ordering choice.** Both are
callers of the same renderer, and `inspect` is the one that changes no existing
default. Landing it first means the composed read is proven end-to-end — all three
levels, both arms, every marker, the fixture corpus — while every existing suite
is still green. PHASE-05 then adds a second caller to a working renderer instead
of debugging a new renderer and a moved default at the same time.

**PHASE-05 is isolated because it is the scope addition.** It changes the default
output of a live `SPEC-013` golden surface — the one the managed design run is
itself driven through — so its blast radius is deliberately contained in one
phase with one whitelist. The design's §5.6 names exactly three suites that go red
on purpose across this slice; two of them are this phase's, and anything else red
is the alarm. That distinction belongs in the plan rather than in the implementer's
judgement at the moment the build breaks.

`OQ-1` lands here too. It was left open on the design for a second opinion and
answered at planning (`QUE-223`, answered 2026-09-19): disclose. After `DEC-261`
re-sited the reader into `commands/design.rs`, beside the run's own verbs,
consulting run state is a state read rather than a new dependency — the coupling
that held the question open is gone. It lands on both arms, which is `OQ-1`'s own
clause and not an embellishment: the design recorded that the answer has two
halves precisely so that whoever answered it could not answer half.

**PHASE-06 is last and is a deletion.** The deprecated `slice design` leaf, its
guard row, and its residual dispatch arm all exist because one enum variant does;
removing the variant takes them together. Doing it after the reclaimed surface has
landed means no reader ever has neither verb. It also carries the slice's two
non-test verifications — the agent's re-derivable acceptance attestation against
the real `SL-244` corpus, and the one human question the feature lives or dies on.

## Notes

**One correction the plan carries that the design does not (`DEC-262`).** §5.2
specifies a new `facet_fields(&RecordFacet)` as `C4`'s single field-order table.
Re-grepping at planning found that name and that role already occupied:
`facet_fields(kind: RecordKind) -> &[FacetFieldRow]` at `src/knowledge.rs:1028`
is already *"the single authored derivation of the per-kind field sets"*. The
tree holds three per-kind facet enumerations, not the two the design counted, so
writing §5.2 literally would add a fourth and leave `C4` true of two functions
and false of the tree. `DEC-262` rules the repair: the tier rides the authored
row, the value-bearing projection takes a different name and reads through that
row, and `DEC-150`'s ruling is untouched — only its implementation sketch moves,
exactly as §5.2 already records `DEC-150`'s own sketch moving. PHASE-01's
criteria are written against the record, not against the locked sketch. The
divergence between `design.md` §5.2/§5.6 and what gets built is deliberate and
bounded, and is a reconciliation item for `/audit`.


**Why `VA` and `VH` appear at all.** Three claims in this slice cannot be
discharged by a test and are marked by mode rather than quietly dropped. `VA-1`
on PHASE-06 verifies *absence* — no keyword mandate can express "this symbol is
gone", so it is an agent grep with a required positive control at the branch
point. `VA-2` is the acceptance claim of `DEC-151`, which is a judgement:
the attestation must name the ids surfaced, the ids excluded and why, and the
byte count at each level, because "looked fine" is the accidental default and
discharges nothing. `VH-1` is the only question that decides whether the middle
level was worth building.

**What this plan does not schedule.** `CHR-073` — re-attesting five committed
memories that document `design show` as the envelope read, one of whose thesis
inverts under `DEC-261` — is counted by the design but is not a phase. Re-attesting
a memory is its own verb and the corpus is not code. It must land with or before
the code, since a stale memory is injected into agent context; whether that
warrants a hard `needs` gate is left to close, deliberately.

**A figure this plan inherits rather than verifies.** The design's central
justification for the middle level is that `facets` costs about 30% of `full` on
the specimen. That figure lives in a runtime-tier, untracked research artefact:
it cannot be re-read and a re-run produces different numbers (`IMP-462`).
`PHASE-06`'s `VA-2` is the first and only point in this slice where the number is
put back in front of evidence, which is why its attestation is required to record
byte counts at all three levels rather than a verdict.
