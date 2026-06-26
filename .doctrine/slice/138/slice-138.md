# Relation-transitive walk for inspect

## Context

`blockers --transitive` walks dep edges transitively via cordage's
`reachable`. There is no equivalent for relation edges. You can't ask
"show everything shaped by ASM-001, transitively" or "show the full
descends_from tree rooted at PRD-001."

The cordage graph already builds with relation overlays — the walk
machinery exists. This is a flag on `inspect`, reusing the same
`reachable` infrastructure.

RFC-001's thesis explicitly names transitive impact query (proposal 0003)
as a consumption surface gap.

## Scope & Objectives

> **Scope amended 2026-06-26** (design lock). Direction-selectable (inbound is
> the primary blast-radius direction), depth cap default 5, multi-label
> `--labels`, and a new depth-bounded cordage primitive. See `design.md` for the
> authoritative decisions; this scope is reconciled to it.

```bash
# Blast radius — what transitively depends on ADR-005 (inbound; the default both also shows it)
doctrine inspect ADR-005 --transitive --direction inbound

# Both directions (awareness view, default), all labels, depth 5
doctrine inspect ADR-005 --transitive

# Full derivation closure outward, unbounded
doctrine inspect PRD-001 --transitive --direction outbound --max-depth all

# Narrow to specific labels (comma-separated)
doctrine inspect SL-047 --transitive --labels governed_by,references
```

### Implementation

- Add `cordage::reachable_bounded` (depth-tagged + `truncated`); `reachable`
  re-expressed over it, behaviour-identical. (The existing `reachable` carries no
  depth — the cap and truncation indicator require the new primitive.)
- `relation_graph::transitive_from` — reuse `build_relation_graph_from` + the
  `require_minted` gate; per-overlay × per-direction `reachable_bounded`.
- Direction-selectable: `inbound` (`Against`), `outbound` (`Along`), `both` (default).
- Per-label sections, all overlay-backed labels by default; `--labels` narrows.
- Default depth 5; `--max-depth N`; `0`/`all` = unbounded; truncation indicator.

## Non-Goals

- No indented-tree / path render (cordage returns `depths`, display drops it this
  slice — a clean follow-up).
- No cycle detection beyond what cordage already provides.
- No graph export. No actionability/priority block on the transitive view.
- No-overlay labels (`drift`, `decision_ref`) are 1-hop-only — omitted from transitive.

## Terrain

| File | Change |
|------|--------|
| `crates/cordage/src/query.rs`, `lib.rs` | New `reachable_bounded` + `Reach`; `reachable` re-expressed over it |
| `src/relation_graph.rs` | `transitive_from` + `TransitiveView`/`TransitiveGroup` + transitive render |
| `src/commands/inspect.rs` | Branch `run_inspect` on `--transitive` (relation-only) |
| `src/commands/cli.rs` | `--transitive`, `--direction`, `--labels` (+`--label` alias), `--max-depth` |

## Dependencies

- SL-137 (corpus relation query) — soft `after` (same catalog query surface)
- SL-140 (cordage traversal unification) — closed IMP-020; `reachable` is the
  clean primitive `reachable_bounded` extends
