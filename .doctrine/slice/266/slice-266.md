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

Reference for feel — a screenshot of *hydra*, a friend's MIT Rust tool built
from the user's original idea. Not a spec; its vocabulary is not ours:

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

1. **Pure tree renderer** — a rendering of the full turn envelope (DEC-303):
   the whole map, uncapped. Marks `●` resolved, `○` open, `◌` derived-blocked,
   `◐` deferred, `⊘` pruned; provenance letter; `*` blocking; `← cursor` /
   `← pinned`; legend line (DEC-306, DEC-308). Right-hand text: question, or
   `needs <ids>`, or disposition record id + title / note. Wraps with a hanging
   indent, never truncates; `--color` (DEC-307). Own module under
   `src/design_run/render/`, not grown into `envelope.rs`.
2. **Read surface** — `doctrine design show --format tree` plus shorthand
   `doctrine design tree [SL-NNN]` (DEC-304). The slice is optional on `tree`
   only: latest non-locked run of a non-terminal slice by snapshot mtime, named
   and marked chosen in the header (DEC-305). The tree is a rendering of the
   turn envelope projected at `Detail::Full`, which gains the whole map
   (DEC-303); line anatomy, wrapping and marks per DEC-306/307/308.
3. **Delivery mode config** — `doctrine.toml` `[design] map_delivery =
   "relay" | "sidecar"`, default `relay` (DEC-309). Relay: a map-changing
   `design apply` ends with a line telling the agent to show the user the
   `design tree` output verbatim before ending the turn; sidecar: no line
   (DEC-310). The tree output itself names the `doctrine design tree SL-NNN`
   command.
4. **Prompt assets** — `inquiry.md` gains one mode-neutral sentence naming
   `design tree` as the user's view of the map (fragment digests stay
   config-independent); `initial-concerns-recorded.md` drops the hand-built
   listing and "no other viewer" in favour of `design tree` output (DEC-310).

## Non-Goals

- `doctrine design watch` (repaint-in-place live view) — IMP-472.
- Reasons on pruned/deferred nodes — IMP-473.
- Changed-since-revision marks on the tree (DEC-308).
- ISS-298 (`design show --full` widens nothing) — separate issue.
- Hosting the tree in `doctrine map serve` (the web explorer).
- Re-measuring map use (CHR-065, which `needs` ISS-299) — runs after this lands.
- Any change to what SL-233 excluded: no full map injected into every turn
  envelope.

## Affected surface

- `src/design_run/render/` — new tree module; `render/mod.rs` wiring.
- `src/commands/design.rs` — `ShowFormat::Tree`, `tree` verb, run resolution,
  record-title lookup.
- `src/design_run/render/envelope.rs` — whole-map field at `Detail::Full`.
- `src/design_run/inquiry.rs` — read-only use of node status / parent / needs.
- `src/dtoml.rs` — `[design]` config (`DesignConfig`).
- `design apply` output path — relay line (DEC-310).
- `install/design-prompts/inquiry.md`,
  `install/design-prompts/conditions/initial-concerns-recorded.md`.
- `install/design-payload-contract.md` / reference docs if the read surface is
  documented there.

## Risks, assumptions, open questions

- Q1–Q4 settled in the design run: DEC-303..DEC-310. `--format status` is
  complemented, not subsumed (status carries counts only).
- **A1 (revised)** — the snapshot carries everything except record titles,
  which the command shell reads and passes into the projection (DEC-306).
- **R1 — concurrent work.** Uncommitted changes across `src/design_run/*` from
  another agent (e.g. a new `InquiryNode::open` parameter) must land before
  implementation begins; scoping and design can proceed.

## Verification / closure intent

- Renderer: fixture maps covering every mark, provenance letter, blocking,
  nesting, cursor/pin, blocked-by, record titles and notes, wrapping at narrow
  and piped widths; plain vs colour output.
- Envelope: `Detail::Normal` never carries the map; `Full` carries all nodes.
- CLI: `design tree` with and without a slice arg; chosen run named in the
  header; parity with `design show --format tree`.
- Config: relay → a map-changing apply ends with the relay line, a non-map
  apply does not; sidecar → never; unknown value refused.
- ISS-299 resolved on close; CHR-065 unblocked.

## Summary

Give the inquiry map a whole-tree text view, a short verb to reach it, and a
project-level choice of whether the agent relays it or the user watches it.

## Follow-Ups

- `doctrine design watch` — IMP-472.
- Pruned/deferred reasons — IMP-473.
