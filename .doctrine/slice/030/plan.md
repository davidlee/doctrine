# Implementation Plan SL-030: Policy entity kind (POL)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases deliver the policy kind via the design's central move (D1): share the
*mechanism* across governance kinds, keep the *identity* per kind. The order is
dictated by one hard constraint — the **behaviour-preservation gate**. Migrating
shipped ADR code onto a shared spine is only safe if a real net catches surface
drift, and the design's adversarial pass (Codex MAJOR-6) showed the existing adr
unit tests are not that net: they exercise the very symbols the extraction moves
and never capture stdout. So the gate must be *built first*, as black-box tests,
before a line of production code moves.

## Sequencing & Rationale

- **PHASE-01 — tests before extraction.** Pin `adr show`/`adr status`/`adr list`
  at the CLI surface (stdout + JSON + error text). `tests/e2e_list_conformance.rs`
  is *parse*-conformance only (flags parse, exit 0, generic `{kind,rows}`
  substring) — it does not pin `adr list`'s byte-exact rows, so list gets its own
  golden over a populated tree (hide-set, ordering, prefix). Tests-only, no
  production change. This is the safety net every later phase holds green. VT-2/VT-3
  deliberately mutate render fns (show/status AND list) to prove the net bites — a
  net that never goes red proves nothing.

- **PHASE-02 — extract and migrate, behaviour-identical.** Pull the shared
  compute/io + shell wrappers into a command-tier `governance.rs` parameterized by
  `GovKind`, and thin `adr.rs` down to its descriptor. This is the highest-risk
  phase (it touches shipped code) and the reason PHASE-01 exists. It carries no new
  user-visible behaviour: success is the PHASE-01 goldens passing *unchanged*. The
  two sharp edges the design caught live here — `parse_ref` must keep ADR's exact
  two-case strip (a case-insensitive shortcut would silently widen the accepted
  forms), and `show_json`'s dynamic key forces a hand-built map, not the `json!`
  macro. Doing this before POL exists avoids the worst state: a half-extracted
  spine with one kind thin and one kind fat.

- **PHASE-03 — the actual feature.** With the spine proven, POL is a thin kind:
  descriptor, clap enum, scaffold, templates, and the three install surfaces. The
  install wiring is its own exit criterion because the project's blanket-ignore
  gitignore makes a new authored tree *silently uncommittable* until negated — a
  known trap, so it gets an explicit test rather than trust. The policy prose body
  reuses tuned prior art, with its frontmatter stripped to honour the storage rule.

- **PHASE-04 — make policies visible where governance is read.** Project
  `required` policies into the boot snapshot beside accepted ADRs. Deliberately
  *small*: the design ruled the boot error≡empty marker collapse and the
  supersession-vs-status gap pre-existing and shared with ADR, so this phase
  inherits and documents them rather than reworking boot. Their fixes are recorded
  as slice follow-ups, not smuggled into this slice.

Dependency chain is linear: 01 (net) → 02 (extract) → 03 (POL rides the spine) →
04 (boot reads POL). Each phase ends green and `just check`-clean; nothing later
is needed to make an earlier phase correct.

## Notes

The three follow-ups surfaced by the adversarial passes — a real governance
**tag reader** (`--tag` is inert today, ADR included), a **`policy supersede`**
verb to enforce the supersession invariant, and **boot error/empty
disambiguation** — are explicitly out of scope here and carried in `slice-030.md`
§Follow-Ups and `design.md` §6. They are parity-with-ADR gaps, not regressions
this slice introduces.
