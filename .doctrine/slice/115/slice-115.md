# Decompose main.rs: relocate orphan runners, extract cli arg modules

## Context

`main.rs` is 7264 lines. The 2026-06-19 architecture audit found its *direction*
is fine — it sits correctly at the top of the command tier — but it carries two
loads it should not:

1. **Stranded command shells** — command-tier `run_*` runners that never moved
   out of `main.rs`: `run_link`/`run_unlink`, `run_needs_edge`/`after_edge`/
   `after_remove`/`after_prune`, `run_supersede`, `run_validate`, `run_inspect`,
   the estimate/value runners, and `write_class` + `worker_guard`. (Design
   correction: these are command-tier shells; their *data/policy* already lives at
   the right tier — `dep_seq`(leaf), `supersede`(engine), `relation`(engine) — so
   the shells land in `commands/` modules that call *down*, **not** in the
   leaf/engine data modules. `run_link` in particular is the structural-relation
   verb and has nothing to do with `links.rs`, which is wikilink extraction.)
2. **The entire clap surface** — one `enum Command` plus ~25 subcommand enums and
   a large dispatch match, all in the one file, so every new kind grows `main.rs`.
   The `commands/` folder (`map.rs`, `serve.rs`) already demonstrates the intended
   shape: a command-tier module pairing clap `Args` + its `run_`.

This is a cohesion/altitude problem, not a coupling-direction one. The moves are
mechanical and behaviour-preserving.

## Scope & Objectives

- Relocate the orphan `run_*` shells into command-tier `commands/` modules that
  call *down* into their existing data/policy modules (`commands/relation.rs`,
  `commands/dep_seq.rs`, `commands/supersede.rs`, `commands/validate.rs`,
  `commands/inspect.rs`, `commands/facet.rs`, `commands/guard.rs`).
- Move each kind's clap subcommand enum **and** its dispatch arm into the kind's
  own command-tier module behind a `dispatch(cmd, color)` entry, so a new kind no
  longer touches `main.rs`. Lower-tier-backed surfaces (coverage, estimate/value,
  map) get `commands/` shells instead.
- Move the top-level `Command` enum + the thin dispatch match into
  `commands/cli.rs`; `Cli` + the shared leaf-only clap bundles (`CommonListArgs`)
  stay at the crate root (design §5 F-C — they are inert to the gate and must not
  enter `commands/`). `main.rs` reduces to a ~250-LOC orchestration entrypoint.
- Convention resolved (design §1): one `commands/` folder, no parallel `cli/`.

Closure intent: `main.rs` materially reduced (~7264 → ~250 LOC; the residual is
`Cli` + shared leaf-only clap bundles + `main()`, design §5 F-C); no relocated
`run_*` or `enum *Command` remains in `main.rs`; clap surface lives under
`commands/`; the convention is documented; the ADR-001 layering gate stays green
(no new accepted violation, tangle baseline unchanged); existing CLI behaviour and
suites unchanged (behaviour-preservation gate — `tests/e2e_*` goldens untouched).

## Non-Goals

- Changing CLI behaviour, command names, args, or output.
- Reworking the dispatch *mechanism* (still clap-derive + a routing match).
- Touching the kind modules' internals beyond receiving the relocated runners.

## Summary

Shrink the `main.rs` monolith by relocating the stranded `run_*` shells into
command-tier `commands/` modules that call down into existing data/policy, and
lifting each kind's clap enum + dispatch into its own module — mechanical,
behaviour-preserving decomposition that ends the "every kind grows one file"
pressure. Convention unified on `commands/`.

## Follow-Ups

- **SL-129 / IMP-067** own the shared id→path core (`entity::id_path`) inside the
  four resolvers this slice relocates. **Sequence SL-129 → SL-115** (design R1):
  both edit `main.rs` and the kind modules; consolidating id→path before the move
  keeps SL-129's 93-site inventory valid. (`after` edge pending user confirmation.)
