# ISS-302: SPEC-017 REQ-236 contradicted once anchor liveness is reported

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

SPEC-017 `REQ-236` (NF-001) states:

> Treat `[[source]]` anchors as descriptive, not currency: liveness **is not
> checked**, so a stale anchor ships silently and freshness is an authoring
> discipline.

SL-243's `doctrine spec anchors` probes path existence for every anchor and
reports the result. When it ships, the requirement asserts a fact about the
system that is no longer true.

## The distinction that matters

The requirement's **intent** survives intact: nothing *fails* because of a stale
anchor. SL-243 is report-only and non-gating by scope commitment, so freshness
remains an authoring discipline rather than an enforced gate.

Its **wording** does not survive. "Liveness is not checked" will be false, and a
requirement stating something false is worse than a missing one — a reader has
no way to tell which requirements are current.

## Recommended route

A **REV** rewording it to the surviving intent — something that says a stale
anchor never fails a gate, rather than that liveness is unobserved. ADR-013
routes governance dependency through a Revision, and SL-243 does not own
SPEC-017.

Timing: before SL-243 closes. This is an obligation the slice *incurs*, not
independent work — it is captured here so it survives `notes.md` and is visible
to anyone reading the backlog rather than the slice.

A future *gating* slice — a doctor leg (IMP-316) or a ratchet over dark loc —
would contradict the reworded requirement in turn and owes its own REV. That is
expected, not a defect in the rewording.

## Related

- SL-243 — the slice that creates the contradiction; its notes carry the same
  obligation as a plan item.
- IMP-316 — the doctor leg for anchor liveness and identifier form; the gating
  consumer that would need the second REV.
- SPEC-017 `REQ-232` — the sibling requirement, unaffected: it models the anchor
  as language + identifier + optional module and stays true.
