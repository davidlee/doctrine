# Implementation Plan SL-128: deliver_to config as single trunk-ref source

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases, a strict dependency chain — leaf config → gate consumer → CLI
consumer → prose. Each ends green and behaviour-preserving (default unchanged).
The design (`design.md`) is canon; this plan only sequences it.

## Sequencing & Rationale

**Why this order.** The dependency edges force it: the gate (PHASE-02) reads a
field that must exist (PHASE-01); the sync handler + verb (PHASE-03) read through
the impure `load_doctrine_toml` reader that PHASE-02 introduces; the close prose
(PHASE-04) references the verb that PHASE-03 ships. Reversing any pair leaves a
dangling reference or an unused symbol that `warnings = "deny"` rejects.

**PHASE-01 — config field (leaf).** Pure addition to `DispatchConfig`. The one
trap is the default: `#[derive(Default)]` would yield `""`, so it is replaced by a
hand `impl Default` sharing the `default_deliver_to()` fn with serde, and a parity
test pins the two together (I1). The struct already carries a **container**
`#[serde(default)]`, which fills absent fields from `Default` — so the hand
`impl Default` is the single load-bearing default mechanism and the per-field
`#[serde(default = …)]` is redundant local documentation, not a second source of
truth (G2). Deliberately introduces **no** live read of `doc.dispatch`, so the
existing `expect(dead_code)` stays fulfilled and the build stays green — the
dead-code removal is held until its real consumer lands. The new `deliver-to` key
is also documented (commented, default-valued) in both `doctrine.toml.example`
files alongside `preferred-subprocess-harness`, since it is the operator-facing
point of the slice (G1).

**PHASE-02 — gate consumer (the behaviour-preservation crux).** The const
`TRUNK_REF` is retired from the gate in favour of a config read. Two design
constraints drive the shape: the impure reader lives in the **neutral** `dtoml.rs`
(not `slice.rs`) so PHASE-03's `main.rs` consumer need not couple sideways
(codex-F2); and the read happens **only inside the `reconcile→done` gate branch**
so malformed-`doctrine.toml` ordering is unchanged for every other `slice status`
transition (codex-F3, I4). `load_conduct` becomes a thin wrapper over the shared
reader (IF3). The `expect(dead_code)` is dropped here, where the live read makes
it fire (R5). The existing `trunk_integration` suites are the proof: they stay
green unchanged (R3).

**PHASE-03 — sync read-default + verb (CLI).** The READ stages gain the config
default; the WRITE stage (`--integrate`) is untouched, because absent `--trunk`
there means *edge-only projection* — a live tested path (I2). The clap
`requires="trunk"` on `--show-journal-trunk-oid` is relaxed and its refusal test
is replaced by behaviour tests (codex-F4). The replacement test needs a fixture
that *holds a trunk row* — once `requires` is relaxed, a bare no-flag fixture
resolves `deliver_to` then hits the distinct runtime no-trunk-row error, which is
the wrong failure to assert on; the sibling `…_errors_when_no_trunk_row` test
(explicit `--trunk`) is unaffected and stays green (G3). The `deliver-to` verb is
the thin read that PHASE-04's prose will call.

**PHASE-04 — close prose.** Mechanical literal-retirement, gated behind the verb
existing. All four delivery literals route through the verb/config — including
line 68 `candidate create --base`, which codex-F1 established is a delivery
literal, not the fork-base auto-resolver. The verify-read and the `git diff`
compare share one captured shell var (`trunk=$(doctrine dispatch deliver-to)`) —
not two verb spawns (DRY, G4). The step-3a TODO that motivated IMP-124 is removed.

## Notes

- **Parallelism:** none worth it — the chain is serial by dependency. Solo
  execution, one phase at a time.
- **Behaviour-preservation gate:** PHASE-02's `trunk_integration` suites and
  PHASE-03's edge-only e2e are the proofs; they must stay green *unchanged*.
- **Out of scope (follow-ups, see scope doc):** PR/remote delivery mode;
  base+delivery unification into `git.rs::trunk_tree_ish`; converging
  `coverage_store::load_config` onto the shared `load_doctrine_toml` reader.
