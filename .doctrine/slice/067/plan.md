# Implementation Plan SL-067: Tags command surface: backlog beachhead (add/remove, list column, colour)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

The backlog `tags` axis is shipped but dead: seeded `[]`, surfaced only by
`backlog show`, never written, never listed, never coloured. This slice gives it
a command surface on the backlog **beachhead**, riding the shared `listing.rs`
machinery so the column + colour generalize to other kinds later rather than
forking a bespoke path. Two phases, split along the **producer/reader** seam.

## Sequencing & Rationale

**Why two phases, producer before reader (PHASE-01 → PHASE-02).** The render
work (PHASE-02) needs tagged data to assert against. PHASE-01 ships the write
verb, so PHASE-02's colour/column tests seed their fixtures through the real
verb rather than hand-authoring TOML — the round-trip is exercised end-to-end,
and the two phases touch largely disjoint surfaces (PHASE-01 the write path +
JSON projection in `backlog.rs`; PHASE-02 the paint model in `listing.rs` + the
table column wiring in `backlog.rs`). The split is along a natural data-flow
boundary, not an arbitrary size cut.

**PHASE-01 — producer.** One atomic verb, `backlog tag <ID> [TAGS]...
[--remove/-d <TAGS>...]` (D3), built on the `set_backlog_status` edit-preserving
recipe: parse the ref, require the item exists, mutate `tags` in place via
`toml_edit`, stamp `updated` from the injected clock. The substance lives in
three pure helpers and one invariant:

- `normalize_tag` is the **single write chokepoint** (D4): trim → lowercase →
  charset `[a-z0-9_:-]` (colon for namespacing, e.g. `area:backlog`) → non-empty,
  else bail naming the token. Modelled on `resolve_slug` as the lone charset wall.
- `fold_filter_tag` is a **deliberately separate, lenient** fold for the `-t`
  filter — trim + lowercase, **no charset reject**. It cannot route through
  `normalize_tag`: a filter that matches nothing must succeed silently, never
  error. "Single chokepoint" governs the write only; the two folds diverge by
  design.
- The write is a **sorted-array set-replace**: `new = (current ∪ adds) ∖ removes`,
  stored sorted. The **no-op guard compares as sets, not ordered vecs** — a
  hand-authored unsorted `current` whose logical content is unchanged must not
  write+stamp spuriously. The idempotency test seeds an unsorted store precisely
  to pin this.
- **F-1 refuse** on an absent `tags` key — a malformed (hand-edited) file is
  refused, never `insert`ed into (a tail-insert lands inside a trailing subtable
  = corruption). All well-formed items seed `tags = []`.

PHASE-01 also closes a producer-side read gap (§3.1): `list --json` omitted tags
entirely (only `show`/`show --json` carried them). `BacklogRow` gains a flat,
**unconditional** `tags` field — JSON rows are stable and never visibility-gated,
unlike the dynamic table column that PHASE-02 adds.

The `dep_seq` append seam is the nearest precedent but is **not reused**: it is
relationships-scoped (tags is top-level), append-only (no remove), and dead-code.
The F-1 refuse pattern is borrowed; the sorted-array *replace* is genuinely new.

**PHASE-02 — reader/render.** The paint model can't address tokens today —
`paint_cell` paints the whole cell one hue. A tags cell is a join of N values,
each wanting its own colour, so a new `ColumnPaint::PerToken { split, render }`
variant is added to the shared `listing.rs` (lower layer, ADR-001 — generic,
reusable by any future tags column). `paint_tag` renders a colon-segment chip
(segments hued by a stable `segment_hue` hash, colons white); the palette
excludes Red **and** BrightRed (reserved for adverse status in `status_hue`) and
Black/White (background / colon separator).

The column wiring lives in `backlog.rs`: `tags` joins `BL_COLUMNS`, with
**dynamic default visibility** (D2) — shown by default only when ≥1 *visible*
row is tagged. "Visible" is load-bearing: `any_tagged` is computed on the final
displayed set, after both `retain` **and** the `--kind` filter, and once, so the
column layout is uniform across `--by id` blocks. An explicit `--columns` request
overrides the dynamic logic verbatim. The const `BL_DEFAULT` stays 4-column;
`tags` is spliced into a locally-built `effective_default`, never the const.

**The byte-clean invariant is the slice's sharpest test (F1).** Nothing in the
type system couples the column's plain `cell` extractor and the PerToken `split`;
both read `tags`, but only by convention. PHASE-02 carries a guard test asserting
the **property** — `strip_ansi(coloured) == plain == cell(r)` over multi-tag,
colon-namespaced, and empty-segment rows — not a proxy like "ANSI is present".
This preserves SL-053's zero-ANSI-under-`color=false` plain path.

## Notes

- **Behaviour-preservation gate.** SL-053 VT-2 (zero ANSI under `color = false`)
  and the existing `list`/`show` goldens are the proof the shared colour
  machinery is unchanged — they must stay green unmodified.
- **Assumptions carried from design §9** (A1 sorted/set semantics; A2 empty
  colon-segments tolerated; A3 lenient filter does not reject charset; A4 palette
  size/hues an impl choice at P2; A5 intra-array comments not preserved; A6 the
  verb normalises only adds/removes, the first write self-heals ordering not
  casing) are accepted, not open work — they do not gate either phase.
- **Out of scope** (slice non-goals): filter-by-tag (shipped), tags on non-backlog
  kinds, a tag registry/vocab/rename/colour-config, tag-axis relations. The
  shared-machinery design keeps the cross-kind door open as a follow-up.
