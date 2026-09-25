# Inquiry map tree view

## Context

ISS-299: the inquiry map (a design run's tree of open/settled design questions)
never reaches the user. SL-233 committed to making the active path, frontier,
blockers and counts visible to the user; the bounded envelope renders some of
that, but nothing delivers it. The only prompt obligation to show the map is
`conditions/initial-concerns-recorded.md` (added by IMP-467), which fires once
and has the agent rebuild the tree from its own memory — and asserts "the map
has no other viewer", which is false once a viewer exists.

Two delivery modes are legitimate and differ by project:

```
            relay (agent-driven)              sidecar (user-driven)
who runs    the agent, after a map change     the user, in a spare pane
delivery    tree pasted into the chat         user runs `design tree`
prompt      obligation: show the tree         obligation: stay quiet
```

Target rendering (user mockup):

```
sample-states  (6 answered, 7 open)
├── ● storage          one JSON doc per tree, rewritten under a lockfile
│   ├── ● disk-format  JSON, one object per tree
│   └── ○ write-model  ← next
│       └── ⊘ compaction  cauterised by storage
├── ◌ metrics          blocked by deploy
└── ● telemetry        OTLP to the collector sidecar
```

## Scope & Objectives

1. **Pure tree renderer** — run snapshot → text tree of the whole map (uncapped).
   Glyphs: `●` resolved, `○` open, `◐` deferred, `⊘` pruned (with reason),
   `◌` derived-blocked ("blocked by X"), `← next` at the traversal cursor.
   Totals header. Right-hand column = resolution/disposition text, truncated to
   width with `…`. Colour on a TTY only; plain glyphs when piped. Own module
   under `src/design_run/render/`, not grown into `envelope.rs`.
2. **Read surface** — `doctrine design show --tree` (or a `--format` value —
   design to settle) plus alias `doctrine design tree [SL-NNN]`. Slice arg
   optional: defaults to the active design run; several open → most recently
   touched, named in the header.
3. **Delivery mode config** — `doctrine.toml` `[design] map_view = "relay" |
   "sidecar"` (project default; name and default value for design). Selects the
   prompt obligation: relay → the agent shows the tree output verbatim after any
   turn that changes the map; sidecar → no relay obligation.
4. **Prompt assets** — `inquiry.md` gains the mode-dependent obligation;
   `initial-concerns-recorded.md` drops "no other viewer" and points at the tree
   output instead of a hand-built listing.

## Non-Goals

- `doctrine design watch` (repaint-in-place live view) — follow-up.
- ISS-298 (`design show --full` widens nothing) — separate issue.
- Hosting the tree in `doctrine map serve` (the web explorer).
- Re-measuring map use (CHR-065, which `needs` ISS-299) — runs after this lands.
- Any change to what SL-233 excluded: no full map injected into every turn
  envelope.

## Affected surface

- `src/design_run/render/` — new tree module; `render/mod.rs` wiring.
- `src/commands/design.rs` — `--tree` / `tree` verb, optional slice resolution.
- `src/design_run/inquiry.rs` — read-only use of node status / parent / needs.
- config loading for `[design]` in `doctrine.toml`.
- `install/design-prompts/inquiry.md`,
  `install/design-prompts/conditions/initial-concerns-recorded.md`.
- `install/design-payload-contract.md` / reference docs if the read surface is
  documented there.

## Risks, assumptions, open questions

- **Q1 — conditional prompt content.** Can shipped design-prompt assets vary on
  project config? If not, a small engine change is needed to select the relay
  obligation by `map_view`.
- **Q2 — reconcile with `--format status`.** An existing compact rendering;
  decide whether the tree complements or subsumes it.
- **Q3 — "active design run" resolution.** What counts as active (unlocked run?
  slice in `design` status?) and the tie-break when several are.
- **Q4 — glyph set and width.** Unicode glyphs vs ASCII fallback; terminal
  width detection when piped (fixed default).
- **A1** — the snapshot already carries everything the render needs (status,
  parent, needs, cursor, disposition text); no model change.
- **R1 — concurrent work.** Uncommitted changes across `src/design_run/*` from
  another agent (e.g. a new `InquiryNode::open` parameter) must land before
  implementation begins; scoping and design can proceed.

## Verification / closure intent

- Renderer: fixture maps covering every glyph, nesting, cursor, blocked-by,
  pruned-with-reason, truncation; plain vs colour output.
- CLI: `design tree` with and without a slice arg; ambiguity tie-break named in
  the header; parity with `design show --tree`.
- Config: each `map_view` value yields the right prompt obligation (and none
  leaks into the other mode).
- ISS-299 resolved on close; CHR-065 unblocked.

## Summary

Give the inquiry map a whole-tree text view, a short verb to reach it, and a
project-level choice of whether the agent relays it or the user watches it.

## Follow-Ups

- `doctrine design watch` — live repaint of the tree (backlog item to raise).
