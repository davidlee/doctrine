# Implementation Plan SL-166: Dispatch corpus-loss guards

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases land the three corpus-loss guards (design §5) plus their config
surface and posture enablement. The shape is **foundation → universal guard →
posture guards → enable + prove**:

- **PHASE-01** lays the shared substrate: the optional `[dispatch]
  authoring-branch` field and the named-constant refusal tokens (STD-001).
  Inert alone — no guard is wired, so behaviour is byte-identical (INV-2).
- **PHASE-02** ships **g3**, the always-on 3-way clobber gate. It needs no
  posture config, so it lands independently and first among the guards.
- **PHASE-03 / PHASE-04** ship **g2** then **g1**, both gated on the PHASE-01
  config.
- **PHASE-05** enables the posture for this repo and proves INV-2 parity.

## Sequencing & Rationale

**Why g3 before g2/g1 (PHASE-02 ahead of the posture guards).** The internal
pass thought g3 was pure forward-insurance; the RV-176 external pass (F-2) proved
otherwise — `advance_pure_ref` is a plain CAS and `plan_edge_row` is not ff-gated,
so the `--edge` integrate leg can today advance to a non-descendant tree with **no
FF guard** ([[mem.fact.dispatch.edge-advance-leg-not-ff-gated]]). g3 is the only
thing closing that live path, and it is posture-independent, so it earns the front
of the queue: earliest protection, smallest dependency surface (only PHASE-01's
constant).

**Why config first (PHASE-01).** g1 and g2 both branch on `authoring-branch`; the
field and the STD-001 constants are their common prerequisite. Keeping it a
standalone phase keeps the inert-by-default proof (INV-2) clean — the config can
land and ship without changing a single advance.

**Why g2 (PRIMARY) is PHASE-03, not first.** g2 kills the witnessed SL-164 chain
at its root, but it depends on both PHASE-01 (config) and its own tri-state
`last_corpus_commit` seam. The fail-closed correction (RV-176 F-1) is the load
the phase carries: a set-but-unresolvable `authoring-branch` must **refuse**, not
silently disable the primary guard. The ADR-001 layering check (pass resolved
values into `coordinate()`, never the loader) is an exit criterion, not an
afterthought — confirmed by a module-graph assertion (VA-1).

**Why g1 is PHASE-04.** g1 converts the unenforced "stay off the buffer" etiquette
into a mechanism refusal. Its scope was the F-4 finding: guard **only** verbs that
advance `deliver_to`/`edge`; `candidate create`/`admit` mutate no integration ref
and stay unguarded (OQ-3 resolved at design, enumeration confirmed here).

**Why enablement is last (PHASE-05).** Turning the posture on is a separate,
reversible config commit (design §5.3) — it must follow working guards, and it
must be flanked by the INV-2 parity re-run so we prove single-branch dispatch is
untouched with the posture **unset** before we switch it **on** for this repo's
edge/main split.

## Notes

- **TDD per phase:** every guard is a pure predicate over injected git readings
  with an impure thin-shell wiring point; red/green/refactor on the bare-repo test
  substrate ([[mem.fact.git.remote-mutation-seam]]).
- **Behaviour-preservation gate:** the existing dispatch suites
  (`e2e_dispatch_sync`, `e2e_dispatch_close`, the `git` units) are the proof —
  green unchanged except where a guard is the direct subject (design §3, §9).
- **Open at /phase-plan (mechanical confirmations design deferred):**
  - *PHASE-02 plumbing.* `advance_row` is currently passed as a bare `fn(root,
    row)` into `with_journaled_projection`; threading the `allow` set and computing
    `base = merge-base(new, cur)` per leg means promoting it to a capturing closure.
    Confirm the closure seam keeps the existing replay/CAS classification intact.
  - *PHASE-02 cost (R2).* Prefer `diff --name-only base cur` minus `diff
    --name-only base new` (both over `.doctrine/`) — the same clobber set as the
    design's per-path `blob_at` compare, with two diffs and **no** per-blob reads;
    bounds the 4816-path catastrophe case. EX-1/EX-5 admit either mechanism.
  - *PHASE-03 signature.* `coordinate(root, slice, dir)` gains a resolved-posture
    parameter (`Option<{authoring_branch, deliver_to}>` values — never the config
    loader). That values-in shape is what the ADR-001 module-graph check (VA-1)
    asserts on.
  - *PHASE-04 enumeration.* Confirm the exact `sync`/integrate candidate-active
    arm set against `dispatch.rs`; `admit`/`create` stay out by the F-4 principle.
- **Lineage:** design §10 (RV-176 dispositions), slice §Scope (Model B),
  ADR-012 D4 (no Revision — D6 acquitted).
