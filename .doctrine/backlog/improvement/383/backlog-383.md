# IMP-383: Self-identifying JSON head for non-list report verbs

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

Doctrine's `--json` output has two conventions and no marker separating them.
Numbered-entity `list` verbs emit SPEC-013's `json_envelope` — `{kind, rows}`.
Non-list report verbs emit a raw serialised struct: `graph` does today
(`src/commands/graph.rs`, `serde_json::to_string` over `CatalogGraph`), and
`spec anchors` will when SL-243 ships it (DEC-112).

A consumer of the second convention has to already know which verb it invoked to
know what it is parsing. Give those payloads a self-identifying head — a
top-level discriminator naming the report — so a payload says what it is without
claiming a row model it cannot satisfy.

## Why it is not in SL-243

SL-243 considered exactly this at `inq-5` and declined it *for one verb*. Minting
the convention on the new verb alone would leave `graph` as the sole
non-conforming surface — inconsistency renamed, not removed. The version worth
having covers both, and that makes it a change to an output surface other
consumers read: bounded, but breaking.

## Scope when taken

- `graph --format json` and `spec anchors --format json` gain the head.
- SPEC-013 states the convention for the non-list case. Today it fixes
  `json_envelope` for the numbered-entity `list` spine and is silent on report
  verbs, which is why `graph`'s opt-out is lawful but unwritten. This is the
  governance half, and it is what makes the next report verb obey by
  construction rather than by precedent-reading.
- Per-verb black-box goldens re-baseline (SPEC-013 REQ-204).
- Decide whether the head carries a schema version as well as a name. A
  versioning anchor is most of the argument for doing this at all if anything
  ever consumes the payload programmatically — `/spec-coverage-assessment` is the
  first candidate.

## Related

- DEC-112 — the SL-243 decision this defers from, carrying the three options and
  why B was taken now.
- SPEC-013 — CLI surface; owns the envelope invariant and the conformance regime.
- SPEC-027 — the graph projection whose raw-struct JSON is the other affected
  surface.
