# ISS-306: knowledge show/inspect render no inbound reciprocity, contradicting using-doctrine.md

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`using-doctrine.md` § Relating entities states:

> Storage is **outbound-only** — you link from the source side and reciprocity
> is derived (ADR-004); `inspect` / `show` render both directions.

For **knowledge records** the second clause does not hold. After
`doctrine link EVD-006 supports QUE-200`:

| view | result |
|---|---|
| `knowledge show EVD-006` / `inspect EVD-006` | `supports: [QUE-200]` — outbound, correct |
| `knowledge show QUE-200` / `inspect QUE-200` | **nothing** — no `supported_by` line |
| `knowledge show QUE-200 --json` | `relationships` carries outbound labels only |

Not new, and not caused by SL-241: `knowledge show DEC-048` likewise does not
render `supported_by: EVD-002`, an edge authored in July.

`src/relation.rs` documents the intent explicitly — `Supports`: *"inbound renders
`supported_by` (SL-159 PHASE-02)"*, `Disputes`: *"inbound renders `disputed_by`"*
— so the doc and the enum comments agree with each other and the knowledge render
disagrees with both.

## Why it matters

A question record is the natural place to ask "what evidence bears on this?", and
that is exactly the direction that does not render. Found while discharging
SL-241 PHASE-05's VA-5 (*"`knowledge show QUE-200` and `QUE-201` render the
linked EVD records in BOTH tiers"*), which is not satisfiable by the Tier-1 edge
alone. The workaround there was to populate each question's own `[evidence]`
`supports`/`contradicts` citation block — which works, and is arguably the right
home for a *curated* citation list, but it means the authoritative edge and the
rendered citation are maintained separately and can drift.

## Scope to settle first

Whether the fix is to the render or to the doc. Check whether other kinds (SL,
ADR, REQ) do render inbound and knowledge is the outlier, or whether the
reference doc overstates across the board — that decides whether this is a
one-kind gap or a documentation correction. `doctrine explain` /
`relation census` may already expose the inbound direction; if so, the doc should
name the verb that does it rather than claiming `show`/`inspect` do.

## Related

- ADR-004 — relations stored outbound-only, reciprocity derived.
- ADR-010 — relation modelling; inverse spellings are derived render-text.
- SL-241 PHASE-05 T7 — where this was found.
- IMP-170 — UX review of relation-authoring CLI surfaces (adjacent).
