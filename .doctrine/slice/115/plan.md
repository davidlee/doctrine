# Implementation Plan SL-115: Decompose main.rs: relocate orphan runners, extract cli arg modules

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases turn `design.md` into execution. The spine is the
**behaviour-preservation gate**: the CLI surface is captured as a test net
*first* (PHASE-01), then code is relocated in dependency order, asserting
byte-identity at every step. No phase changes behaviour, names, args, or output.

## Sequencing & Rationale

The order is **forced by reachability**, not preference (confirmed codex round 2,
point 4): `commands/cli.rs` is the binary's *eventual* dispatch home, but it can
only call shells that already live under `commands/` or in kind modules — it can
never call back into `main.rs` (the binary root is unimportable). So:

- **PHASE-01 — verification baseline.** Build the proof harness against the
  *current* `main.rs` before anything moves: a `--help` snapshot plus
  parse-regression tests over the parser-only contracts a `--help` diff cannot see
  (`value_delimiter`, `conflicts_with`, `value_parser`, `required_unless_present`,
  `requires` — design V7). Everything downstream asserts against this net. Nothing
  relocates here.
- **PHASE-02 — orphan shells first.** The stranded `run_*` shells must leave
  `main.rs` for `commands/` *before* the dispatch match can follow, because the
  match calls them. `commands` gains out-edges but stays a **sink** (no command
  module imports it) — so it contributes 0 tangle (design §5).
- **PHASE-03 — kind enums + dispatch (D1), batched by domain.** Each kind's enum +
  dispatch fold into the module its body actually calls. The load-bearing
  discipline is **D1a**: route to where the `run_*` lives, never the nominal kind
  name. Two known cycle-formers are pre-identified — `MemoryCommand::Sync`→`corpus`
  (stays in a `commands/` sink shell) and `SpecReq`→`spec.rs` (own-module, not
  `requirement.rs`). The gate runs **per batch**; any tangle growth past 120 halts
  the batch for restructure (never an auto-accept).
- **PHASE-04 — collapse the dispatch core.** Only once every arm is a one-liner can
  the `Command` enum + thin match move to `commands/cli.rs`. `main.rs` ends ~250
  LOC: `Cli` + the leaf-only shared clap bundles (`CommonListArgs`) + `main()` —
  these stay at the crate root because they are inert to the gate and must never
  enter `commands/` (design §5 F-C).

## Notes

- **SL-129 precedes SL-115** (`after` edge; R1). SL-129 consolidates `entity::id_path`
  on the current layout; relocating the now-thinner resolvers afterwards keeps its
  93-site inventory valid and dissolves the F-A breadcrumb concern. PHASE-01 EN-2
  gates on SL-129 having landed (or explicit user authorisation to proceed ahead).
- The per-arm D1a audit (PHASE-03 EN-2) is mandatory: codex confirmed exactly two
  cycle-formers across an exhaustive per-kind sweep (round 4), but the audit + the
  per-batch gate are the standing guard against a missed third.
- Verification modes: every phase carries VT (automated: gate + snapshot + suite)
  and VA (grep/audit checks a test cannot make). No VH — this is a mechanical,
  behaviour-preserving refactor with no human-judgement acceptance criterion.
