# ISS-301: Revisit compromise parent edges in the spec decomposition tree

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

Every tech spec except SPEC-003 (the context spec) currently has a `parent`.
That uniformity may be reporting more structure than the corpus actually has:
where containment was not obvious, the nearest plausible container appears to
have been chosen rather than the field left unset.

The specific case raised: **SPEC-027** (Graph projection and CLI emitter,
component) parents under **SPEC-004** (Entity engine, container). SPEC-027 is a
projection *of* the entity catalog and descends from PRD-016, a different tree
entirely. Whether it is a *component of* the entity engine, as opposed to a
consumer of its output, is the question.

## Why it matters

A parent edge is authored truth the engine round-trips but does not generate
(SPEC-006), and the registry harvests it into the decomposition tree that later
agents reason from. A wrong edge is worse than a missing one: an absent parent
is visibly absent, while a compromise parent looks like a finding and gets
cemented by everything that reads the tree afterwards.

PRD-012's principle is "a spec has **at most one** parent" — zero is lawful by
that wording, and nothing in the machinery checks for presence. `spec validate`
reports the corpus clean today and its findings vocabulary carries only
`second_parent` (multi-valued), no orphan check. So the option to leave it unset
has always been available and appears not to have been used.

## Scope when taken

- Audit the parent edges for cases where containment was inferred rather than
  determined; SPEC-027 is the known candidate, and the sweep should not assume
  it is the only one.
- For each, decide: keep, unset with a recorded open question, or re-parent.
- Consider whether a missing parent deserves an *advisory* (not hard) validate
  finding, so absence stays visible rather than becoming invisible in its own
  way. This cuts against the reason for allowing absence, so it is a real
  question rather than an obvious improvement.

## Related

- SL-243 raised this while placing a new spec: the census could not find an
  honest parent for it, and leaving the field unset was preferred to cementing
  the best available compromise.
- PRD-012 — the "at most one parent" principle and the decomposition integrity
  requirement (REQ-087).
- SPEC-006 — owns the relational spine as hand-authored TOML.
