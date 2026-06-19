---
seq: 0009
scope: spec
target: PRD-011 OQ-001 / REQ-054 (FR-006) / SPEC-001 D10
confidence: high
reversible: yes (proposal only; no spec/backlog change — read-only analysis)
---
## What
The graph-derived worklist (`doctrine survey` / `next`) is the single most
team-facing payoff of the topology — "what should we work on, in what order." Its
ordering contract is fully designed: SPEC-001 **D10** fixes the precedence as
`authored-priority → actionability → consequence → deterministic fallback`
(`.doctrine/spec/tech/001/spec-001.md:265-295`). The **top slot — item-level
authored priority** (a head-tail scalar) — is the one place a human asserts "this
outranks the derived order."

That slot is **designed, requirement-minted, and unbuilt.** The code ships the
precedence contract with the slot permanently empty: `survey_for_map`
(`src/priority/surface.rs:135`) sorts `actionability → consequence desc → id`, and
the module header (`:14-17`) states plainly "the v1 authored-priority slot is EMPTY
(PRD-009 OQ-001 unbuilt)." SPEC-001 ties it to a real requirement —
`spec-001.md:282`: "the item-level head-tail scalar (PRD-011 OQ-001,
FR-006/`REQ-054`)" — and `:292-295`: "survey's authored-priority slot is supplied
by the OQ-001 item scalar, still open … until OQ-001 lands, `survey` orders by
actionability → consequence → fallback with the slot empty."

The gap: this requirement-backed capability has **no work-intake presence** — a
`backlog list` scan for priority/authored-priority/OQ-001/REQ-054/head-tail/pin
finds nothing tracking it. It exists only as an open OQ buried in PRD-009/PRD-011
and a citation in SPEC-001. So the highest-precedence input to the team worklist is
invisible to the very planning surface it would feed — it will not surface in
`next`, will not be scheduled, and risks indefinite latency despite being
design-complete down to the precedence contract.

Secondary (minor): an ownership cross-reference wrinkle — `spec-001.md:293` calls
OQ-001 "owned by **PRD-009**", while `:282` cites "**PRD-011** OQ-001, FR-006/
REQ-054". One of the two PRD attributions is stale; worth reconciling so the OQ has
a single owner.

## Options
1. **Capture OQ-001 as a backlog improvement now** (don't build) — give the
   designed slot a tracked work item so it enters `survey`/`next` and can be
   sequenced. Tradeoff: zero design cost, just makes invisible work visible; the
   build decision stays yours.
2. **Scope it straight to a slice** — D10 already fixes precedence and REQ-054
   exists, so this is closer to execution than discovery. Tradeoff: moves faster,
   but commits to building before you've decided the head-tail model's UX (verb
   shape, persistence tier of the scalar).
3. **Leave deferred** — keep it an open OQ. Tradeoff: zero effort; but the team's
   most-wanted lever (pin priority) stays unavailable and untracked, and the empty
   slot keeps reading as "priority is fully automatic" when the design says it
   shouldn't be.

## Recommendation
Option 1: capture it as a backlog improvement, citing REQ-054 / FR-006 / SPEC-001
D10 / PRD-011 OQ-001, so the slot becomes tracked work that `next` itself will
surface. Rationale: the design is done (precedence fixed, requirement minted), so
the only thing missing is *visibility and sequencing* — exactly what backlog
capture provides. Promote to a slice (Option 2) once you've settled the head-tail
model. Fix the PRD-009/PRD-011 ownership cross-ref as a one-line correction
whenever SPEC-001 is next touched.

Decisions deferred to YOU:
- (a) **build or keep deferred** — is leaving authored priority unbuilt a conscious
  v1 scope cut, or just untracked drift? (the design reads "intended, not yet
  built".)
- (b) **the head-tail model** — is authored priority a binary head/tail pin, a
  small ordinal, or a full numeric rank? (SPEC-001 says "head-tail scalar" —
  confirm the intended shape and where the scalar persists: authored toml vs
  runtime.)
- (c) **OQ-001 owner** — PRD-009 or PRD-011? (reconcile the conflicting citations).

## Next doctrine move
```
# read the precedence contract + the empty slot (read-only):
sed -n '260,296p' .doctrine/spec/tech/001/spec-001.md
doctrine spec show SPEC-001        # D10 + REQ-054/FR-006 membership

# capture the designed-but-untracked slot (NOT executed — fence forbids transition):
doctrine backlog new improvement "Build the item-level authored-priority slot \
  (head-tail scalar) feeding survey/next — precedence already fixed by SPEC-001 \
  D10, requirement REQ-054/FR-006, PRD-011 OQ-001; slot currently empty in \
  priority/surface.rs" --tag area:priority --tag area:spec
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — the build is gated on decision (b) (the head-tail model), which a
speculative diff would prejudge. The precedence comparator that would consume the
scalar already exists at `src/priority/surface.rs:137-141`; authored priority slots
in as the first `.cmp` term ahead of `act_rank`.
