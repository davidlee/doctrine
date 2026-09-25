# Implementation Plan SL-265: Unified doctrine show

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1. See glossary.md § reference forms. -->

## Overview

Four phases. The verb and its surface land together; a fidelity phase pins the
central byte-equivalence and refusal property through the binary; a small
guidance phase points the agent read path at the verb; a governance phase
carries the `SPEC-013` amendment and its evidence.

```
PHASE-01 router + Command::Show + explore + guard + census   (the verb, reachable)
   │
PHASE-02 byte-equivalence + refusal e2e                       (the central property)
   │
   ├── PHASE-03 guidance: using-doctrine + routing-process + boot + guardrails test
   └── PHASE-04 governance: REV + FR-006 + coverage record/verify + prose + REV done
```

`SL-265` is one top-level, kind-blind `show` verb that resolves a canonical ref
to its kind and delegates to that kind's existing `show` (design sec-1). Nothing
renders differently; the change is a surface over the twelve per-kind verbs that
already exist.

## Sequencing & Rationale

- **PHASE-01 is one phase, not two.** The router module and the clap surface
  that reaches it are one coherent change. Splitting them would leave
  `run_show()` unreached between phases — dead code under the zero-warning gate
  — with no behaviour to verify in between. The unit totality test, the
  `write_class` parse test and the census all land here because each is
  observable the moment the variant exists.
- **PHASE-02 second, and separate.** The slice's central property is a
  *comparison*: `doctrine show <REF>` against the kind's own verb, per prefix and
  in both formats. It is worth its own phase because it is the phase that can
  falsify the delegation table, and because its test binary is what PHASE-04's
  coverage check binds. Both sides are stdout from the same binary invoked
  twice, so there is no stored golden to age (design sec-7).
- **PHASE-03 and PHASE-04 both follow PHASE-02; their order is free.** Guidance
  names a verb that exists (PHASE-01) and the governance evidence binds a test
  that exists (PHASE-02). Governance is last because it is the closure-flavoured
  act: the `REV` reads `done`, and the new requirement waits on `/reconcile` for
  its `pending -> active` flip.
- **The governance phase is its own phase, not a footnote to the code.** Its
  acts are discrete and separately verifiable (`DEC-299`): the `REV` skeleton,
  the single introduce row, approve, apply, the manually minted requirement, the
  check-bound coverage cell, the hand-landed spec prose, and the `REV` close.
  The precedent is `REV-038`, which amended `SPEC-013`'s own members.

## Phase boundaries worth naming

- **The router adds no layering edge.** Its home is `src/commands/show.rs`
  (command tier): a child file inherits the `commands = "command"` tier row, so
  `layering.toml` needs no new row and the command tangle (ratcheted at 76, per
  top-level module) does not count it. A new top-level `src/show.rs` *would* need
  a tier row and trip `Unclassified`. The router calls the four per-kind
  governance wrappers rather than `governance::run_show` directly, which keeps
  the no-new-edge claim true (`RV-384` `F-8`).
- **The `--json` shorthand gets one home.** `CommonShowArgs::format()` replaces
  the four inline copies (`backlog` Show/Inspect, `knowledge` Show/Inspect). The
  kind `run_show` bodies and their goldens are untouched.
- **`REQ` is the one prefix off the uniform path.** Its own arm reaches
  `spec::run_req_show`; `PRD`/`SPEC` reach `spec::run_show`. The equivalence test
  therefore maps the reference command per group (`RV-384` `F-12`).
- **The equivalence is scoped to prefixed refs.** The router uppercases the
  prefix unconditionally (a superset of every per-kind case rule), so for every
  *prefixed* ref a kind's own `show` accepts, the bytes match. A *bare* id is
  deliberately outside the property: it resolves across every kind and refuses as
  ambiguous rather than being read in one namespace (`RV-384` `F-28`).
- **No kind is touched.** A failing equivalence is a router bug; the phase says
  so, so a worker cannot "fix" it by editing a renderer.

## Transcribed review criteria

`RV-384` (design, concluded) left four instrument-routed findings `answered`,
verified here by landing each criterion on a phase:

| finding | criterion | phase |
|---|---|---|
| `F-1` | totality test iterates the `KINDS` table, with a negative control on a synthetic unrouted prefix | `PHASE-01` EX-5 / VT-1 |
| `F-2` | governance sequences coverage record **then** verify, exiting on the cell reading `Verified` | `PHASE-04` EX-4 / VT-1 |
| `F-5` | the totality test is a `#[cfg(test)]` unit test in `src/commands/show.rs` | `PHASE-01` EX-5 / VT-1 |
| `F-13` | the coverage record binds a concrete check (`--command`/`--matcher-*`) | `PHASE-04` EX-3 / VT-1 |

## Notes

- Plan-time re-grep (2026-09-25): every path and symbol in design sec-2..sec-7
  resolves against the tree. Line numbers have drifted by at most two (e.g.
  `knowledge`'s Show dispatch is at `src/knowledge.rs:3878`, `CommonShowArgs` at
  `src/main.rs:217`, the census at `src/commands/cli.rs:2010`). No stale premise.
- Selectors were extended at plan time with `install/using-doctrine.md` and
  `install/routing-process.md`, which the locked design's code-impact table
  touches but the design-time selector set did not carry.
- The 24 equivalence fixtures were each confirmed present (`doctrine inspect`)
  before being named in `PHASE-02` EX-2.
- PHASE-01 is the largest phase. If its phase sheet shows it will not fit one
  worker context, split the `run_show` delegation (EX-1/EX-2) from the accessor
  and surface edits (EX-3/EX-4), keeping the totality test with the router.
- The `SPEC-013` prose is *not* staged as a `modify` row: a `modify` row keys on
  a live peer FK, not a prose section, so the prose rides the `revision-NNN.md`
  companion and is hand-landed (`RV-384` `F-6`, `F-17`).
- Guidance's source of truth is `install/using-doctrine.md`. The tracked
  `.doctrine/using-doctrine.md` is a legacy *projected* copy, stale against the
  source since ADR-019 made reference docs published-not-projected; it is
  deliberately not edited here (out of scope, `CHR-078`'s neighbourhood).
