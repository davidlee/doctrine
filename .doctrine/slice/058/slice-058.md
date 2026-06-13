# Wire the structural relation surface end to end (read, write, scaffold, guidance)

## Context

SL-048 ("the cut") migrated tier-1 relations to the uniform `[[relation]]` row
idiom and built the machinery ahead of consumers: `RELATION_RULES` (the legal
`(source, label)` table, single source of truth — SPEC-018, ADR-010), the
generic `read_block` parser, and the `append_edge`/`remove_edge` writer in
`src/relation.rs`. SL-046 shipped the all-kind graph reader. But for the
slice/backlog kinds the wiring never closed: relations are **write-only in
practice** today. Authoring a relation means hand-editing TOML, and nothing
displays it.

Four already-captured backlog items name the gaps:

- **ISS-010** (load-bearing) — the read path returns empty. `inspect` outbound
  and `slice show` relationships render nothing though `[[relation]]` rows exist
  and corpus `validate` is clean. Diagnosed reader-side, not render: the spec
  kind's typed axes (`descends_from`/`members`) display fine — the generic
  `[[relation]]` row path does not. That contrast is the diagnostic wedge.
- **IMP-048** — no `link`/`unlink` CLI verb. The writer
  (`append_edge`/`remove_edge`) exists but is unsurfaced, so the only way to
  author a structural relation is by hand.
- **ISS-009** — the `slice new` scaffolder still emits the stale
  `[relationships]` reserved-table comment, not the post-SL-048 `[[relation]]`
  idiom. (Reproduced live in this slice's own scaffold.) Sibling scaffolders
  (spec, backlog) to be checked for the same staleness.
- **IMP-049** — no agent-facing guidance (skill / memory / docs) on how and
  when to relate entities structurally.

## Scope & Objectives

Close the structural relation surface end to end. One coherent change across
read, write, and scaffold; agent guidance layered on top.

1. **Read (ISS-010)** — repair the slice/backlog `[[relation]]` read path so
   authored rows surface in `slice show` (and `inspect` outbound, and any other
   render that reads `tier1_edges`/`targets_for`). Diagnose first: legality-drop
   in `read_block`, `tier1_edges` not reached, or the slice source omitted from
   the corpus graph. Render is already correct — do not touch it.
2. **Write (IMP-048)** — surface a uniform `link`/`unlink` CLI verb over the
   existing `append_edge`/`remove_edge` machinery, gated by the table's
   `LinkPolicy` (write-strict: refuse dangling / illegal-kind / non-`Writable`
   triples per SPEC-018).
3. **Scaffold (ISS-009)** — emit the `[[relation]]` idiom from the `slice new`
   scaffolder (and any sibling scaffolder carrying the same stale comment).
4. **Guidance (IMP-049)** — agent-facing support: how/when to relate, the legal
   vocabulary pointer (`RELATION_RULES`), the `link` verb. Skill and/or memory
   and/or docs. May split into its own follow-up slice if it sprawls — decided
   at `/design`.

## Non-Goals

- Reshaping the relation model — this completes wiring, not redesign. The
  storage shape, `RELATION_RULES`, tiers, and outbound-only policy (ADR-004,
  ADR-010) are fixed.
- Touching the render layer (`format_show`/`show_json`) — proven correct.
- Tier-2 / tier-3 (typed / payload / free-text) edge handling beyond what the
  generic read/write path already covers.
- Cross-corpus prose-only relation gaps (IMP-016, IMP-035) — separate items.
- SL-057 (formal VT verification) — parked; resumes after this slice.

## Affected Surface

- `src/relation.rs` — `RELATION_RULES` :252, `read_block` :494, `tier1_edges`
  :547 / `targets_for` :558, `append_edge` :716 / `remove_edge` :735, module
  note :24-37. (Authority; ride it — no parallel impl.)
- `src/slice.rs` — `relation_edges` :1185 (read path returning empty),
  `format_show` :1201 / `show_json` :1254 (render — read-only reference), the
  `slice new` scaffold path.
- Per-kind `relation_edges`: `backlog.rs` :769, `governance.rs` :253, `rec.rs`
  :406; `spec` typed axes are the working contrast case.
- `src/relation_graph.rs` — corpus relation graph feeding `inspect` outbound.
- CLI command surface for the new `link`/`unlink` verb.
- Scaffolder source(s) emitting `[relationships]`.
- Skill / memory / docs surface for IMP-049.

## Risks, Assumptions, Open Questions

- **Behaviour-preservation gate**: SL-048 / SL-046 + relation / cordage suites
  must stay green unchanged.
- ISS-010 root cause is unconfirmed pending diagnosis (three candidate sites).
  The fix shape — and whether it's one-line or structural — follows the
  diagnosis. SL-058's own toml `[[relation]]` rows become a free render fixture
  once the reader is fixed.
- `validate` is read-tolerant: it confirms rows are legal but does not prove
  rendering — not a sufficient test oracle on its own.
- OQ: does IMP-049 belong in this slice or split out? (decide at `/design`)
- OQ: which sibling scaffolders carry the stale `[relationships]` comment?

## Verification / Closure Intent

- Authored `[[relation]]` rows surface in `slice show` and `inspect` outbound —
  a black-box CLI assertion over SL-058's own rows (or a fixture).
- `link`/`unlink` round-trips a structural edge and refuses an illegal triple
  (write-strict); `unlink` removes it; corpus `validate` stays clean.
- `slice new` scaffold output contains the `[[relation]]` idiom, not
  `[relationships]`.
- Behaviour-preservation suites green unchanged.
- Agent guidance landed (skill/memory/docs) per the IMP-049 disposition.

## Follow-Ups

- SL-057 resumes (`/design`) after this slice.
- Possible IMP-049 split-out (decided at `/design`).
