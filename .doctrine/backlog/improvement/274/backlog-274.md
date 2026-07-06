# IMP-274: Triage skill: backlog grooming via pairwise value/estimate calibration and deduplication

## Intent

A new `/triage` skill for grooming open backlog items. It interrogates the
description of each untriaged item, deduplicates and links related items,
benchmarks and estimates human attention cost (estimate facet) and value/risk
(value facet), and transitions the item to `triaged`. When no `open` items
remain, it finds old and potentially stale non-terminal backlog items for
re-triage.

## Core workflow

1. **Survey open items** — `backlog list --status open`. For each: inspect
   metadata, read prose body, check existing value/estimate.
2. **Deduplicate & link** — `doctrine search` for near-duplicate intent; inspect
   candidate neighbors; link with `doctrine link` or `backlog needs`/`after`.
3. **Validate/calibrate** — benchmark the estimate (attention cost) and value
   against the description. If data looks suspicious, enter pairwise comparison
   mode to recalibrate against anchor items (see below).
4. **Transition** — `backlog edit <ID> --status triaged` once the item's facets
   and links pass scrutiny.
5. **Stale fallback** — when no `open` items remain, scan non-terminal items by
   `created`/`updated` age, flag stale candidates, and prompt for re-triage.

## Pairwise comparison technique

When value or estimate data looks weak, wrong, or suspicious, the skill uses
pairwise comparisons to recalibrate. The technique:

- Selects 5-10 high-information-yield pairs (items missing data, near-ties,
  suspicious orderings, anchors vs unknowns).
- Asks narrow pairwise questions (higher value? larger effort?).
- Translates each answer into concrete `doctrine value set` / `doctrine estimate
  set` commands.
- Prefers relative calibration over false precision. Uses Fibonacci-ish anchors
  (1, 2, 3, 5, 8, 13, 21) unless the repo defines a local convention.
- Never invents new schema, entities, or TOML records — uses existing CLI only.

## Placement

Per POL-002, the skill lives under `plugins/doctrine/skills/triage/SKILL.md`
as an installable marketplace skill, not in `.pi/skills/`.

## Preflight findings (2026-07-06)

- **187 open items**, all with value + estimate set. Only 2 triaged (IMP-053, IMP-099).
- `backlog list` table doesn't surface estimate/value columns — that's IMP-246.
  `backlog inspect` does per-item.
- `created`/`updated` timestamps exist; oldest open items from 2026-06-10 (~26d).
- `doctrine survey` and `doctrine findings` provide priority-graph context for
  pairwise selection.
- `doctrine explain <ID>` explains an item's priority derivation.

## Open design questions

1. Bulk vs selective: triage all 187 open items or focus on suspicious/weak data?
2. Transition gating: auto-transition after satisfactory interrogation, or
   require explicit human approval per item?
3. Pairwise mode as primary workflow or alternate mode?
4. Stale threshold: open > 30d? updated > 14d? Action on stale: re-triage,
   tag `stale`, prompt close/archive?
5. Deduplication depth: BM25 titles+slugs sufficient, or also inspect prose
   bodies?
