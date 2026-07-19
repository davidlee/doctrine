# ISS-232: Path-surface PreToolUse hook never fires in live harness

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

`SL-205` / `IDE-032` shipped ambient memory surfacing via two `PreToolUse`
hook wires in `plugins/doctrine/hooks/hooks.json`:

- `Bash` → `doctrine memory surface` (command surface)
- `Read|Edit|Write` → `doctrine memory surface` (path surface)

Both entries are marked `done`/`resolved`. In real Claude Code sessions, only
the `Bash` wire actually fires. The `Read|Edit|Write` wire has never fired in
production use — the path surface has been silently dead since it shipped.

## Evidence (2026-07-19 session, project plugin `doctrine@doctrine` v0.24.0,
active per `installed_plugins.json`)

- `.doctrine/state/mem-surface.log` holds ~90 entries across many historical
  sessions: 86 are `"surface":"command"` (Bash-triggered), only 5 are
  `"surface":"path"` — and all 5 came from manual `echo '{...}' | doctrine
  memory surface` CLI invocations during this investigation, none from an
  actual harness-triggered Read/Edit/Write. Zero real path-surface firings
  exist anywhere in the log's history.
- Manually piping a realistic hook envelope to `doctrine memory surface`
  works correctly and returns real `additionalContext`:
  ```
  echo '{"session_id":"probe-check","cwd":"/workspace/doctrine","tool_name":"Read","tool_input":{"file_path":"src/backlog.rs"}}' \
    | doctrine memory surface
  # => surfaces 3 memories, exactly as designed
  ```
- Doing a live `Read` tool call on the same file (`src/backlog.rs`), and
  separately on `AGENTS.md`, `src/main.rs`, `src/memory.rs`, produced **no**
  `additionalContext` system-reminder and **no** new log line in
  `mem-surface.log`.
- A live `Bash` tool call in the same session, same event, same plugin, fired
  correctly and logged/surfaced on the first try — confirming the harness
  *does* invoke plugin `PreToolUse` command hooks in general; the gap is
  specific to the `Read|Edit|Write` matcher (or to non-Bash tools).

## ROOT CAUSE — CONFIRMED (2026-07-19, deterministic)

**Ours, not the harness.** The hook *does* dispatch for Read/Edit/Write and *does*
receive a complete, valid JSON envelope on stdin. The defect is in
`doctrine memory surface`: `probe_for` (`src/memory.rs:9545`) wraps the
harness-supplied `tool_input.file_path` into `ScopeProbe::Path(PathBuf::from(fp))`
**verbatim**. Claude Code sends `file_path` **absolute**
(`/workspace/doctrine/src/x.rs`); the downstream path-scope match expects a
**cwd-relative** path (`src/x.rs`), so it matches nothing and surfaces 0. The
command surface (Bash) keys on the command string and is unaffected — which is
exactly why 86 of 91 log entries are `command` and the `path` surface has been
silently dead since ship.

Deterministic repro (same file, only the path form differs):
```
echo '{...,"tool_input":{"file_path":"/workspace/doctrine/src/backlog.rs"}}' | doctrine memory surface  # ⇒ 0 (absolute — what the harness sends)
echo '{...,"tool_input":{"file_path":"src/backlog.rs"}}'                      | doctrine memory surface  # ⇒ 1 (relative — what manual probes used)
```

**Why it hid / why the Evidence above misframes it:** every manual probe (and the
unit-test fixtures, `~src/memory.rs:9927`) used *relative* paths, which silently
sidesteps the bug — so "manual works, harness doesn't fire" read as a harness
issue. It is not: the hook fires and gets good stdin (captured live: absolute
`file_path`). Confirmed via a stdin-capture marker hook after loading it with a
real process restart (plugin `hooks.json` is NOT hot-reloaded; `/hooks` is the
positive control for what's actually loaded).

Theories investigated and **retracted**: command-string dedup, stdin non-delivery,
`|`-alternation matcher bug, stale-process/no-reload. All exonerated by
positive-controlled tests.

## Fix (small, single-site)

- Relativize `file_path` against `discover_surface_root(cwd)` (already computed at
  `src/memory.rs:9587`) before building the probe in `probe_for` — accept an
  absolute *or* relative incoming path.
- Test hardening: the synthetic-stdin fixtures (`~src/memory.rs:9927`) use relative
  paths — the same blind spot that let this ship green. Fixtures must use
  **absolute** `file_path`, as the live harness sends. Red-first with an absolute
  path.
- `plugins/hooks.json` needs **no** change (dedup was a red herring).

## SL-205 reconciliation

The path surface never functioned live since ship, and the relative-path fixtures
masked it. Note against `SL-205` exit criteria on close.

## Resolution (2026-07-19) — fixed in `2a924b77a`

`probe_for` (`src/memory.rs`) now relativizes the incoming `file_path` against the
discovered surface root before building the `ScopeProbe::Path`: a relative path
passes through unchanged; an absolute path under the root is stripped to its
root-relative form (`/workspace/doctrine/src/x.rs` ⇒ `src/x.rs`); an absolute path
outside the root ⇒ `None` ⇒ surface nothing (INV-2 fail-open, no canonicalize of a
possibly-nonexistent `Write` target). Root discovery was reordered ahead of the
probe to thread the root in as data (the probe stays pure).

Verified live against the deterministic repro — absolute `file_path` (the form the
harness sends) now surfaces identically to the relative form. Red-first regression
tests (`iss232_absolute_file_path_surfaces_like_relative`,
`iss232_absolute_file_path_outside_root_surfaces_nothing`) close the fixture blind
spot by exercising the absolute form directly. `plugins/hooks.json` unchanged (the
dedup / matcher theories were red herrings).

**SL-205 exit-criteria reconciliation:** the path-surface exit criterion was
attested on relative-path fixtures that never exercised the live (absolute) form,
so it passed green while the surface was dead in production. The criterion is now
genuinely satisfied; the gap was a test-representativeness defect, not a design
defect in SL-205's surface pipeline.
