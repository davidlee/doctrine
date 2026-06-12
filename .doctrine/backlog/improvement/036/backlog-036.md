# IMP-036: inspect full-corpus scan should tolerate a malformed sibling entity, not abort corpus-wide

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Finding

`relation_graph::build_relation_graph` (SL-046 PHASE-03) walks every
`integrity::KINDS` entity and calls each kind's `relation_edges` reader. If **any
single** entity's TOML fails to parse, the whole scan returns `Err` and
`doctrine inspect <ANY-ID>` aborts — one corrupt entity anywhere in the corpus
takes down every inspect, regardless of whether it is related to the queried id.

Surfaced by the SL-046 PHASE-04 real-corpus smoke: two legacy pre-canonical-ref
entities (SL-003 `supersedes = [2]`, ADR-002 `related = [1]`) each aborted the
full scan. The data was fixed (commit `6eb5796`), but the **fragility remains**:
the next malformed entity reproduces it.

## Why deferred (not in SL-046)

SL-046's design (§5.4 step 6) explicitly scoped diagnostics/validation out to the
`validate` / SL-048 layer — the relation graph is a read surface, not a validator.
A v1 inspector reasonably assumes a parseable corpus. Hardening was a real-but-
out-of-scope robustness improvement; the user chose to backlog it rather than
expand SL-046 scope. Non-blocking for SL-046 close.

## Desired behaviour

The scan should degrade gracefully on a per-entity parse error: skip the
unparseable entity (it contributes no edges / no node), optionally surface it as a
diagnostic (a note line on the human render, or a `--strict` opt-in that restores
the hard-fail), and complete the inspect for every well-formed entity. The queried
entity's own parse error may still be a hard error (you asked for it specifically),
but an unrelated sibling's must not be.

## Pointers

- engine: `src/relation_graph.rs` `build_relation_graph` (the `outbound_for` `?`
  in the edge pass is the abort site)
- related: SL-046 (origin), SL-047 (actionability — also rides the scan, same
  fragility), SL-048 (writer / validation layer — natural home for a corpus-
  validity diagnostic)
- pattern: `mem.pattern.entity.free-text-ref-not-forward-validated`
