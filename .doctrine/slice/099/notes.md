# SL-099 — Scoping & Spike Notes

## What's done

- Scoped SL-099 (read-path + data-model) and SL-100 (lifecycle verbs + skills)
  with a hard `needs` dependency: SL-100 needs SL-099.
- Compared current memory impl against `scratch/memory-spec.local.md` —
  findings in `capability-gaps.md`.
- Ran CLI UX spike (`ux-spike.md`) — mapped every existing verb and flag,
  identified missing verbs (status, edit, tag, resolve-links, backlinks,
  validate), and identified stealable patterns (knowledge status, backlog
  edit/tag).
- Scoped 7 objectives for SL-099 (see `slice-099.md`), 4 for SL-100 (see
  `../100/slice-100.md`).

## Surprises & Design Decisions

### Status enum already has 6 variants
`src/memory.rs` defines `Active`, `Draft`, `Superseded`, `Retracted`,
`Archived`, `Quarantined` — richer than what we initially proposed
(deprecated/superseded/obsolete/archived). No need to add statuses; the
surface just needs verbs to reach them. `Retracted` = "this was wrong"
(better than `obsolete`). `Quarantined` = "suspected incorrect, pending
review."

### Wikilink extraction is trivially cheap
`rg '\[\[mem\.[-a-z0-9\.]+\]\]' .doctrine/memory/` completes in ~0.007s at
193 memories. No persistence needed — compute on-the-fly for backlinks,
graph expansion, and suggested relations.

### Split decision
Initial scope had 6 objectives mixing data-model and skills. Split into two
slices:
- **SL-099**: read-path surfaces, wikilinks, backlinks, `--expand`, record
  flags, lifespan field + ageing, suggested relations, validate, verify
  `--allow-dirty`
- **SL-100**: status/edit/tag verbs, skill updates, new skill skeletons
  (maintaining, reviewing, dreaming)

Split is clean: SL-099 is data-model + query-surface; SL-100 is CLI verbs +
skills. SL-100 depends on SL-099 (needs the fields to exist before adding
verbs to manage them).

### Fields kept vs dropped
Kept: `provenance.sources`, `review_by`, `lifespan`. Dropped: `audience`,
`visibility`, `requires_reading`, `owners` — no current consumer.
`links.out`/`links.missing` not needed (on-the-fly computation).

### Lifespan ageing
`identity` > `semantic` > `procedural` > `episodic` > `working` — modulates
the recency component of the sort key. `identity` never decays; `working`
ages fastest.

## Key files

- `src/memory.rs` — Status enum (line ~77), Memory struct, record/show
  handlers
- `src/lexical.rs` — BM25 ranker, tokenizer, quantize; the suggested-
  relations engine
- `src/retrieve.rs` — find/retrieve pipeline, sort key, holdback
- `src/inspect.rs` — inspect command (refuses memory refs currently)
- `scratch/memory-spec.local.md` — spec-driver reference model
- `scratch/memory-contract.local.md` — the external decision register compatibility contract
  (future backend, not current concern)

## Stealable patterns (for SL-100)

- `knowledge status <ID> <STATE>` — minimal status transition, kind
  auto-detected from prefix
- `backlog edit <ID> --status <STATUS> [--resolution]` — richer edit
  surface with coupling
- `backlog tag <ID> [TAGS]... [-d REMOVE]...` — tag add/remove in one call

## Commits

```
d2f0ac0d plan(SL-100): add dreaming skill — proactive memory corpus maintenance posture
4761a46f plan(SL-100): scope memory lifecycle verbs and agent UX hardening; needs SL-099
199b470a plan(SL-099): trim to read-path + data-model; defer lifecycle verbs + skills to SL-100
fcfc94d8 plan(SL-099): refine scope — wikilink+edge coexistence, lifespan field, drop unconvincing fields
9d1ff0ba plan(SL-099): wikilink extraction on-the-fly — no persistence needed at current scale
e6b55a03 plan(SL-099): scope memory read-path relations and agent UX hardening
```
