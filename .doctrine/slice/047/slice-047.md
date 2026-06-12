# Cross-kind actionable survey/next/explain/blockers CLI

## Context

Slice **2 of 3** in the graph-relations work (PRD-011 / SPEC-001). SL-046 (the
spine) projects every entity kind into a `cordage` graph and exposes the
related/inbound relation view. This slice builds the **derived priority CLI** on
top of that graph: the operator-facing "what should I look at next, and why?"
surfaces SPEC-001 names but that do not yet exist —
`survey` / `next` / `explain` / `blockers` (and `inspect` if not folded into
SL-046's query).

**This slice broadens intent beyond current canon** and is therefore **blocked on
a spec revision**. PRD-011 today scopes the *actionable* channel to backlog
lifecycle only (resolution = promoted/terminal, FR-006); the agreed intent is
that **all kinds rank as actionable work** (slices, requirements, specs, … with a
work-like lifecycle). Realising that needs:
- a **PRD-011 revision** — extend the actionable channel + survey/next semantics
  to all kinds (or carve which kinds are "actionable"), and
- a **SPEC-001 revision** — D10 (survey/next sort keys) and the policy/adapter
  actionability mapping currently assume backlog lifecycle.

Until those land, this slice cannot be designed against settled canon.

## Scope & Objectives

1. **`survey`** — the importance lens over all active items across kinds:
   `authored-priority → actionability → consequence → deterministic fallback`
   (SPEC-001 D10). High-priority **blocked** items float high with a blocked
   badge.
2. **`next`** — the do-now lens: filter to actionable, order by `order_key`
   (dep-topology → seq rank → fallback, D9). Blocked items absent.
3. **`blockers <ID>`** — direct blockers by default; `--transitive` walks the
   chain both directions (D11; REQ-073).
4. **`explain <ID>`** — always walks to root cause; renders the structured
   reasons (order_key, predecessors, evictions) as the "why" surface (D11).
5. Honour the derived-output invariants: deterministic (REQ-077), disposable
   cache stamped with policy version (REQ-094), never authored back (REQ-078),
   terminal/promoted excluded from default active output (REQ-075), cycle
   diagnostics degrade rather than emit a false order (REQ-076).

## Non-Goals

- **No relation/inbound query or all-kind graph projection** — that is SL-046,
  the prerequisite. This slice consumes that graph; it does not build it.
- **No new authored relation schema** — slice 3 (SL-048).
- **No `cordage` core change** (SPEC-001 D1) — `order_key`/`ordered`/`explain`/
  `reachable` consumed as-is.
- **No item-level authored-priority scalar** (PRD-011 OQ-001, owned by PRD-009) —
  if still unbuilt, survey's authored-priority slot stays empty (D10 explicitly
  allows this). No behaviour rests on it.
- **No architectural-trigger actionability mask** (IMP-026 / D6) — separate,
  prerequisite-bound.

## Affected Surface

- New policy/derived-surface module(s) (likely `src/priority/…` per SPEC-001
  diagram) — channel semantics: eligibility, actionability, consequence, render.
- The SL-046 all-kind adapter — consumed; may need actionability node-attributes
  (lifecycle/resolution per kind) added at the projection boundary.
- `src/main.rs` — four new CLI verbs + handlers.
- `crates/cordage/` — consumed, not modified.
- Per-kind lifecycle readers — to classify actionable/terminal across kinds (the
  policy mapping the revision defines).

## Risks, Assumptions, Open Questions

Blocking dependency:
- **Canon revision is a hard prerequisite** (PRD-011 + SPEC-001). Designing/
  building before it = building against stale canon (route gate).

Open questions (design-time, after the revision):
- **Which kinds are "actionable"?** A pending requirement or proposed slice is
  work; an accepted ADR is not. The revision must define the per-kind
  actionable/terminal lifecycle mapping (policy, not core — D2 boundary test).
- **Cross-kind `consequence`** — inbound reference weight across kinds; PRD-011
  OQ-002 deferred knowledge_record (still unbuilt). v1 = existing reference/
  lineage edges only.
- **CLI shape consistency** with SL-046's query verb (shared render/column model).

Assumptions:
- SL-046 lands first and exposes a graph + node attributes sufficient for the
  actionability policy.
- Determinism posture (no clock/RNG/map-order) inherited from cordage + adapter.

## Verification / Closure Intent

- `survey` and `next` **diverge** correctly on a ranked-but-blocked item (top of
  survey with badge; absent from next) — the D10 feature test.
- `blockers --transitive` and `explain` surface the full chain; list rows show
  only direct blockers (D11); toggling display depth never changes ordering.
- A `dep` cycle degrades with a diagnostic, never a false topological order
  (REQ-076).
- Deterministic output under input permutation (REQ-077).
- Actionable ranking spans **all admitted kinds**, not just backlog (the
  broadened intent) — proven over a seeded multi-kind corpus.
- Behaviour-preservation: `backlog order` and cordage suites stay green.

## Follow-Ups

- Architectural-trigger actionability mask (IMP-026 / SPEC-001 D6) once its
  file-set sources are built.
- Item-level authored-priority scalar (PRD-011 OQ-001 / PRD-009) — fills survey's
  empty slot when it lands.
- knowledge_record consequence seam (PRD-010, unbuilt).
