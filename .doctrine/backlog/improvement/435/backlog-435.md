# IMP-435: Rationalise boot snapshot size against its universality contract

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why now

The boot snapshot is ~30KB (~8–9k tokens) and every agent pays it. For an
interactive session that is a one-off. For the tiered subagent topology in
IMP-434 (driver → orchestrator → planner/worker) it is paid once per agent, per
spawn, across a whole slice's implementation — and it cannot be avoided: only
the built-in Explore and Plan agents skip `CLAUDE.md`, and there is no
frontmatter field or per-agent setting to change which agents skip it
(`docs/claude/subagents.md`, "What loads at startup"). Subagents reach it via
the `CLAUDE.md` `@`-import backstop, not the `SessionStart` hook, whose matcher
is `startup|clear`.

It is a cost problem rather than a context-pressure problem — 9k against a 250k
per-agent band is ~3.6% — but it is paid tens of times per slice.

## The principled cut

`boot.md`'s own Model band section states the contract:

> This boot sector is **universal** — model-agnostic by construction. […] No
> per-model, per-role, or per-harness content is baked into this snapshot.

Two sections violate it outright:

| section | bytes | why it doesn't belong |
|---|---|---|
| `DOCTRINE_BIN → the coord build` | 2,456 | per-role (dispatch coordinator) *and* per-stage (close-time) lore |
| `Instrumentation: capture friction as an observation` | 2,269 | contains a literal per-role/per-context routing table |

~4.7KB, ~16% of the snapshot, removable on the file's own declared principle
rather than on convenience. Both belong in the hymns cascade, pulled on demand
by role — the mechanism `boot.md` already names.

Section census at time of writing (bytes, `.doctrine/state/boot.md`):

    5010  Routing & Process        4079  Accepted ADRs
    2456  DOCTRINE_BIN             2269  Instrumentation
    2123  When to retrieve what    1660  Commands
    1469  Tooling & Dev Workflow    881  Structure
     870  Memory                    869  Model band

## Harder calls (decide, don't assume)

- **Accepted ADRs (4,079)** — a title index of 20 ADRs. Load-bearing for
  routing? Or would a count plus "run `doctrine adr status`" serve, given every
  agent has the CLI?
- **Routing & Process (5,010)** — genuinely universal for a seat that routes,
  dead weight for a worker that has already been routed. But `boot.md` is one
  file shared by the whole agent tree; no per-agent variation exists. The only
  lever is moving role-specific content into the hymns cascade, which is exactly
  what the Model band section prescribes.

The general shape: **boot carries what every seat needs; the hymns cascade
carries what a role needs.** Each section that fails that test is either moved
or cut.

## Verification

Byte-count the snapshot before and after; assert no routing regression by
checking that `/route` still resolves correctly from the trimmed snapshot alone.
`doctrine boot --check` must stay green.
