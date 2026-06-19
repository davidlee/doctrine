# Implementation Plan SL-111: Hoist kind identity to a leaf kinds module to break relation layering cycles

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases that introduce a leaf, then move two independent consumers onto it.
The whole change is mechanical and compile-enforced; the behaviour-preservation
gate (design §9) is the proof at every boundary — no assertion or expected value
moves. The slice is deliberately narrow (`relation.rs`-only; `relation_graph`
behavioral edges deferred, design D5/§8 R4).

- **PHASE-01 — Leaf `kinds` module.** Create `src/kinds.rs` with the 20 prefix
  consts + `GOV`/`BACKLOG`/`RECORD` groupings, pinned by a membership test. A
  standalone leaf with no dependents — it compiles and is verifiable in isolation.
- **PHASE-02 — Re-key the relation engine.** Drop the 20 `&crate::<cmd>::*_KIND`
  aliases, re-type the rule table to `&str`, source from `kinds::*`. This is the
  slice's reason to exist: the 7 confirmed cycles break here, proven by an empty
  closure grep with the relation suite still green.
- **PHASE-03 — Single-source the command prefixes.** Re-point each command
  `*_KIND.prefix` onto `kinds::<X>`, removing the parallel `"SL"`-lives-twice copy,
  and ripple the one `relation_graph.rs:1615` test reader.

## Sequencing & Rationale

**Why a leaf-first split (PHASE-01 alone).** Both consumers (engine, commands)
depend on the vocabulary existing; nothing depends on them yet. Introducing and
pinning `kinds` first de-risks the two re-key phases and gives the membership
invariant (INV-1) a home before any caller relies on it. The phase is small but
real — a drift in `GOV`/`BACKLOG`/`RECORD` silently changes relation legality, so
the pin earns its place.

**Why the engine re-key precedes the command re-point (PHASE-02 before -03).**
The two re-keys are mutually independent — `relation.rs` reading `kinds::SL` does
not require the `slice::SLICE_KIND` const to also read it, and vice-versa. They
are ordered to deliver the headline outcome first: PHASE-02 is what breaks the
cycles and unblocks SL-112's precondition (for the confirmed `relation.rs` edges
— engine-crate readiness for `relation_graph` stays contingent, design D5). Its
exit is the measurable objective: the closure grep goes empty. PHASE-03 is the
cleanup that satisfies the *single-source* objective (no parallel literal); it is
separated so its verification (a literal-duplication grep) is distinct from the
de-cycling proof rather than entangled with it.

**The intermediate state between -02 and -03 is acceptable.** After PHASE-02 the
prefix `"SL"` lives in both `kinds.rs` and (still) each command const — the
parallel copy the slice removes. That is a transient, compiling, green boundary;
PHASE-03 closes it. Each phase ends green; no phase leaves the tree broken.

**Scope discipline.** No phase touches `relation_graph`'s behavioral upward edges
(`:82/:423/:652`) or hoists `dir` — that is the deferred `KindCore` widening
(design §8 R4), recorded for SL-112 to adopt deliberately. PHASE-03 touches
exactly one `relation_graph` line (the `:1615` test reader), which rides the
rule-table type change, not a behavioral edit.

## Notes

- **Behaviour-preservation gate is primary.** Every phase keeps the existing
  suites' assertions unchanged; the readers take mechanical accessor edits only.
  If any expected value needs to move, that is a signal the change is not in fact
  behaviour-preserving — stop and `/consult`.
- **Type-safety net.** Every stale `.prefix`-on-`&str` site is a compile error,
  not a silent path (design R3) — the compiler enforces ripple completeness across
  PHASE-02/03.
- **Authoritative reader set** (design §5.5, external pass §10): matchers
  L477/621/858/927 + `sources_match_shipped_accessors` (L1096) +
  `target_spec_matches_design` (L1267-1403) + `relation_graph.rs:1615`. The first
  three land in PHASE-02, the last in PHASE-03.
