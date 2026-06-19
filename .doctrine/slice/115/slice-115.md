# Decompose main.rs: relocate orphan runners, extract cli arg modules

## Context

`main.rs` is 6456 lines (~4800 production). The 2026-06-19 architecture audit
found its *direction* is fine — it sits correctly at the top of the command tier
— but it carries two loads it should not:

1. **Stranded command logic** that never got a home module, even though the home
   module already exists: `run_link`/`run_unlink` (belong in `links.rs`),
   `run_needs_edge`/`after_edge`/`after_remove`/`after_prune` (belong in
   `dep_seq.rs`), `run_supersede` (belong in `supersede.rs`), plus `run_validate`,
   `run_inspect`, `write_class` + `worker_guard`. `write_class` already has its
   own test module, signalling a separable unit.
2. **The entire clap surface** — one `enum Command` plus ~25 subcommand enums
   (lines ~240–2800) and a ~960-line dispatch match — all in the one file, so
   every new kind grows `main.rs`. The `commands/` folder (2-line `mod.rs`, only
   `map.rs` inside) is a stalled extraction that already hints at the intended
   shape.

This is a cohesion/altitude problem, not a coupling-direction one. The moves are
mechanical and low-risk.

## Scope & Objectives

- Relocate the orphan `run_*` runners into their already-existing owning modules
  (`links.rs`, `dep_seq.rs`, `supersede.rs`, etc.); `main.rs` calls into them.
- Extract the clap arg-parsing enums into a `cli/` folder, split by command
  domain to mirror the command groups (decide the exact partition in `/design`).
- Leave the top-level dispatch match in `main.rs` (it belongs there) but shrink it
  to pure routing once the runners and enums move out.
- Resolve the ambiguity between the centralised dispatch match and the vestigial
  `commands/` folder — design picks one convention and records it.

Closure intent: `main.rs` production LOC materially reduced; no orphan `run_*`
remains in `main.rs` that has an existing owning module; clap enums live under
`cli/`; the dispatch pattern is documented; existing CLI behaviour and suites
unchanged (behaviour-preservation gate).

## Non-Goals

- Changing CLI behaviour, command names, args, or output.
- Reworking the dispatch *mechanism* (still clap-derive + a routing match).
- Touching the kind modules' internals beyond receiving the relocated runners.

## Summary

Shrink the `main.rs` monolith by relocating stranded `run_*` logic into its
existing home modules and lifting the clap arg enums into a `cli/` folder —
mechanical, behaviour-preserving decomposition that ends the "every kind grows
one file" pressure.

## Follow-Ups

- Settle the `commands/`-folder vs centralised-match convention so future kinds
  follow one pattern.
