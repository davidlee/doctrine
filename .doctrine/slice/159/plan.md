# Implementation Plan SL-159: Epistemic kind catalog: add EVD + HYP

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases, layered: land the **kinds**, then their **edges**, then the
**docs/memory** that describe them. The ordering follows data dependency, not
file partition — PHASE-02's `supports`/`disputes` edges target the RECORD family
that only includes EVD/HYP once PHASE-01 lands; PHASE-03 documents a catalog that
must already be coherent. Each phase ends green on `just gate`, with the existing
record suites unchanged (behaviour-preservation gate).

The governance axis is **not a phase**: per design §5 / ADR-013, the Revision is
cut after design and settled in reconciliation — it does not gate implementation.

## Sequencing & Rationale

**PHASE-01 — kinds first.** The irreducible catalog work concentrates in
`knowledge.rs` (enum, per-kind consts, status/facet match arms, `ALL` arity) with
satellite edits in `kinds.rs` (prefixes + RECORD), `integrity.rs` (KINDS rows +
count pin + stale "four knowledge kinds" comments → six), `partition.rs` (per-kind
rows), `supersede.rs`/`commands/supersede.rs` (EVD arm + stale "four record kinds"
comment), and the literal mixed-superset lists (`search.rs`, `tag.rs`,
`dep_seq.rs` — `dep_seq.rs` also updates two doc comments). SL-161 already DRY'd the predicate-bearing sites (`scan.rs`,
partition guard, `test_helpers.rs`, `dep_seq` predicate, `search` alias) — they
read RECORD and need no edit (design §2 zero-diff list). The drift canaries
(prefix-count 4→6, `KINDS.len()` 21→23, partition-cover over 6, the facet/status
known-sets) are the proof the structured checklist is complete; the headline VT-4
proves the kinds actually gate under SL-158's trinary, not merely parse. The two
e2e goldens shift with the +2 catalog and are reconciled here so the phase ends
green.

This phase is the largest. If it proves oversized at `/phase-plan` time, the
natural cut line is **catalog/facets/integrity** (kinds creatable + round-trip)
from **partition/supersede/gating** (trinary + supersession behaviour) — but they
share `knowledge.rs` match arms, so keep them together unless execution forces a
split. Flag at `/consult` rather than improvising a renumber.

**PHASE-02 — edges second.** `supports`/`disputes` are full `RelationLabel`
plumbing (codex F5), not two table rows: enum variants at the order-pin slot,
parser, inbound reciprocals, and the source/target/tier/coverage canaries. The
sole author is EVD; the target is the RECORD family (incl CON in the interim —
carries through SL-160's rename unchanged). Three pre-existing RELATION_RULES
source/target sets are hardcoded (not RECORD) and must also gain EVD/HYP:
`References(Concerns)` sources, `Shapes` target, `GovernedBy` sources. The test
canary vectors at `relation.rs:1422,1427` and the `relation_graph.rs` edge-emission
test must be updated as consumer revisions. The trap is **render**: the knowledge
display renderers hardcode `[Shapes, Spawns, GovernedBy]` (codex F4), so the edges
would be authorable-but-invisible without the `format_metadata`/`show_json` edit —
VT-3 guards exactly that. Transitions stay manual (D2): no evidence→status
automation.

**PHASE-03 — docs + memory last.** The shipped corpus (`using-doctrine.md`,
`glossary.md`, `mem.signpost.doctrine.knowledge`) describes a 4-kind catalog; it
must follow the code, not lead it. R3: the signpost is shipped memory — update,
re-embed via `cargo build`, then `memory sync` so the index matches. VA-1 carries
the doc-coherence check that no test can assert.

## Notes

- **Built on SL-158 + SL-161** (both landed on edge). EVD/HYP gate on arrival;
  the DRY'd sites pick them up via RECORD.
- **Serial with SL-160** (CON→INV, sequenced `after`): shared touch-site files,
  no parallel edits to the same lines. SL-159 lands first.
- **R1 (missed literal site):** grep `mem.pattern.doctrine.record-kind-touch-sites`
  before close; the structured canaries catch KINDS/TAGGABLE omissions, the
  mixed-superset lists (`search.rs`/`tag.rs`/`dep_seq.rs`/`relation.rs` vectors)
  still need eyes.
- **IMP-184 (DRY the remaining literal prefix sites) is out of scope** — add
  EVD/HYP in place at each; centralisation is separate work.
