# Map focus on memory refs; onboarding command

## Context

Doctrine lacks good human onboarding. It ships a rich agent-memory corpus
(`memory/`), and the map explorer (`doctrine map serve`, `web/map/`) already
renders memory entities with human-readable titles and addresses them at
`#/focus/mem_<32hex>`. But two gaps stop a human landing on a browsable
onboarding view:

1. **`--focus` rejects memory refs.** `validate_focus` (`src/commands/map.rs:26`)
   admits only canonical entity ids (`SL-001`) or bare numerics. A memory key/uid
   is refused:

   ```
   $ doctrine map serve --focus mem.signpost.doctrine.overview
   error: focus must be a numeric id or canonical entity id (e.g. 1 or SL-001)
   ```

2. **No custom command / no key-addressable entry point.** A newcomer has no
   one-liner that opens the map focused on the onboarding memory; the URL requires
   knowing the opaque `mem_<hex>` uid.

Key→uid resolution already exists (`memory::resolve_inspect_uid` +
`MemoryRef::parse`; the scope originally cited a non-existent
`build_memory_key_map` — see design.md § Reuse seam). Node titles already render
readable. This slice exposes memory refs to `--focus` and adds the onboarding
command — nodes stay human-readable, the URL keeps the uid.

## Scope & Objectives

1. **`--focus` accepts memory refs.** Extend `validate_focus` (and the serve →
   initial-hash path) to accept `mem.<…>` keys and `mem_<hex>` uids, resolve
   key→uid, and seed the initial route `#/focus/mem_<uid>` so the explorer opens
   focused on that memory.
2. **Onboarding command.** Add a custom command (skill under `.agents/skills/`
   and/or a thin CLI verb — see OQ-1) that runs
   `doctrine map serve --focus mem.signpost.doctrine.overview --open` — a
   one-liner that drops a human into the onboarding graph.
3. **Node labels stay human-readable** (already true: `entry.title`); confirm the
   focused memory and its neighbours render titles, not uids, in the browser.

## Non-Goals

- The resolver-complain fix and corpus relink — that is **SL-200** (it produces
  the edges this slice's onboarding view browses).
- Making the map's URL scheme itself key-addressable (`#/focus/mem.<key>`);
  decision is uid-in-URL, human-readable-label-on-node. Not changing the hash
  grammar.
- Authoring new onboarding *content* — reuse `mem.signpost.doctrine.overview` as
  the entry memory.

## Affected Surface

- `src/commands/map.rs` — `validate_focus`, `run_serve`, `MapServeArgs`.
- `src/map_server/` — initial-focus → hash seeding, if focus resolution lives
  server-side (`state.rs`, `mod.rs`).
- key→uid resolution — reuse `build_memory_key_map` (`src/catalog/hydrate.rs`);
  do not duplicate.
- `.agents/skills/<onboarding>/` — the custom command definition (OQ-1).
- Possibly `web/map/src/` — only if initial focus is seeded client-side; prefer
  server-side.

## Dependency

Relates to **SL-200** via `after` (soft ordering, **not** `needs`): file-disjoint
(`map.rs`/`map_server`/skills vs `links.rs`/`memory.rs`/`memory/**`), so the two
can be developed in parallel. SL-200 supplies the memory→memory edges that make
this slice's onboarding view worth browsing; without it, focus works but the
graph around the memory is sparse. Ordering is a value hint, not a hard block.

## Risks / Assumptions / Open Questions

- **OQ-1 — RESOLVED (design.md D1):** first-class CLI verb `doctrine onboard`,
  no flags, entry memory hard-coded as a named constant. Skill entry dropped.
- **OQ-2 — RESOLVED (design.md D2):** server/CLI-side; focus already reaches the
  hash untouched, so `web/map/src` and `src/map_server/` are not touched.
- **Assumption:** ref validation reuses `MemoryRef::parse` (classifier) and
  `memory::resolve_inspect_uid` (resolution). No new parsing.
- **Risk:** `--focus` on a non-existent memory key should error clearly at CLI
  time (before serving), matching the existing `validate_focus` error ergonomics.

## Verification / Closure Intent

- `doctrine map serve --focus mem.signpost.doctrine.overview` starts and the
  explorer opens focused on that memory (uid in URL, title on node).
- `doctrine map serve --focus mem_<valid-uid>` also works; an unknown key/uid
  errors clearly without starting the server.
- The onboarding command launches the above in one step.
- Existing `map.rs` `validate_focus` tests stay green; new tests cover memory
  key + uid accept, and unknown-ref reject.

## Follow-Ups
