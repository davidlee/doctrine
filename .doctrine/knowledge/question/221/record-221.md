# QUE-221: Pin backlog after's cross-kind target refusal?

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

`backlog after <ITEM> <TO>` gates its target through `require_item`, so a
cross-kind target (`SL-154`, `QUE-219`) is refused on every leg today. SL-238 §6
routes all three legs through the kind-neutral `commands::dep_seq` operations, so
that refusal **goes away** — deliberately changed behaviour, landing in
PHASE-07/PHASE-08.

The question: should its before-state have been pinned in PHASE-02, alongside
`--prune`'s vocabulary and `after --remove`'s target gate?

**The case against, which is why PHASE-02 did not.** §7's named characterisation
set is exactly three items — `--prune`'s vocabulary, `after --remove`'s target
gate, and the two interim tests from `74b773690`. PHASE-02's `EX-1`/`EX-2` name
neither this refusal nor `backlog after`'s append leg. §7 lists `backlog after
accepts a cross kind target on every leg` under *new behaviour*, i.e. an ordinary
red-first test in the phase that changes it, not a before-state pin.

**The case for.** It meets §7's own definition of deliberately changed behaviour
("each needs its before-state pinned first, so the change is visible as a
change"), and the cost is one test against an existing fixture idiom. Without it,
PHASE-07/08's widening is visible only as a new green test — the classic
always-was-going-to-pass shape §7 exists to prevent.

**Bound.** A one-test addition to `tests/e2e_dep_seq_verbs.rs` if accepted;
PHASE-07 authors the superseding assertion either way, so the cost of answering
"no" is zero and the cost of answering "yes" late is that the pin has to be
written against a tree where the behaviour has already changed — i.e. it cannot
be written at all. **So this is answerable only before PHASE-07 lands.**

Raised as `D4` in PHASE-02's runtime sheet
(`.doctrine/state/slice/238/phases/phase-02.md`, `## Decisions`), deliberately
not decided by the planning agent: it is a scope call, and scaling the phase
down or up is the owner's.
