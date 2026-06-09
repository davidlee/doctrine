# Implementation Plan SL-032: Worker-mode CLI guard and trunk-ref id allocation with reseat

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-032 mechanises three near-independent additions plus one reuse from the locked
design (§5): the **worker-mode guard** (D2a), **trunk-ref id allocation** (D3),
the **`validate` + `reseat`** backstop, and the **memory-record worktree warning**
(ADR-006 amendment). It is the prerequisite SL-031 (the orchestrator funnel)
consumes — so each phase's VT criteria assert the *conformance surface* a caller
trusts (refusal-under-worker, trunk-aware mint, detect/reseat, the warning), not
only internal mechanics.

The slice touches shared entity-engine machinery only at one point — the injected
`scan` seam in `materialise`. That is deliberate: the design rides the existing
seam rather than building an allocation framework, which keeps the
behaviour-preservation gate (the existing numeric suites, green unchanged) cheap
to satisfy.

## Sequencing & Rationale

The four phases are ordered by **dependency and risk**, matching design §9:

1. **PHASE-01 — Worker-mode guard.** First because it is SL-031's hardest
   dependency and a pure leaf: a `write_class` classifier and one `main()` gate,
   no engine touch. Landing it early unblocks the funnel's most load-bearing
   contract while carrying the least integration risk. Its exhaustiveness (no
   wildcard) is the whole safety argument — a future write verb cannot ship
   unguarded because omitting its class fails the build.

2. **PHASE-02 — Trunk-ref id allocation.** Second because it carries the
   engine-gate risk (R-1): it widens shared machinery. Sequencing it before the
   backstop lets the behaviour-preservation gate (existing numeric suites green
   unchanged) act as a hard checkpoint before anything depends on the new
   allocation path. The pure `next_id` extraction is what makes the union
   unit-testable without disk.

3. **PHASE-03 — `validate` + `reseat`.** Third because `reseat` needs PHASE-02's
   trunk-aware free-id pick to choose a non-colliding default target. It is the
   detect-and-repair backstop for the residual offline/unpushed-collision case the
   best-effort trunk allocation cannot prevent — so it logically follows the
   allocation it backstops.

4. **PHASE-04 — Memory-record worktree warning.** Last because it is the most
   independent: a shared `is_linked_worktree` helper plus one non-blocking call
   site. It has no dependency on the earlier phases and the least blast radius, so
   it sequences naturally at the tail.

Phases 1, 2, and 4 are genuinely separable; only 3 has a hard upstream dependency
(on 2). The ordering therefore optimises for unblocking SL-031 early (1) and
clearing the shared-machinery risk gate (2) before the dependent and the
independent work.

## Notes

- **OQ-6 (design §6) — `validate` rule set.** v1 is the design §5.2 (a)/(b)/(c)
  set; it may tighten during PHASE-03. It is *not* load-bearing for SL-031, so it
  does not block this plan; treat any tightening as a PHASE-03 design touch
  (`/design`), not a silent plan change.
- **Behaviour-preservation gate.** The slice changes shared entity-engine
  machinery (PHASE-02). The existing numeric allocation suites are the proof and
  must stay green unchanged — this is an exit criterion, not an aspiration.
- **Pure/imperative split.** The guard reads `DOCTRINE_WORKER`, the trunk scan
  reads git, the warning reads worktree context — all impure, all confined to the
  shell. The engine gains only data params (`trunk_ids`), never effects.
