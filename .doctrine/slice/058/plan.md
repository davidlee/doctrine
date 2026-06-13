# Implementation Plan SL-058: Finish the relation surface: fix stale scaffold templates, migrate their entity fallout, add agent guidance

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-058 is conformance cleanup, not redesign (design §1). SL-048 migrated tier-1
relations to the `[[relation]]` idiom but left the scaffold templates emitting
the old typed `[relationships]` axes, so entities are born malformed. The plan
fixes the source (templates), cleans the fallout (entities), closes the
detection gap that hid the fallout (parser + guard), and adds the agent guidance
that would have prevented hand-authored drift. Three phases.

## Sequencing & Rationale

The ordering is driven by one hard constraint — **each phase ends green** — and
the design's dependency graph (§5.1, §5.4).

- **PHASE-01 (templates) first.** Fixing the source stops new malformation
  before we touch the existing fallout; otherwise concurrent authoring keeps
  minting bad entities while we clean. The phase is self-contained: the six
  template edits, the re-embed, the black-box scaffold assertion, and the
  kind-specific template-level guard all concern the *born shape* and stay green
  on their own (the corpus is not touched). Putting the template guard here means
  the root regression — a future stale template — is caught from this point on.

- **PHASE-02 (detection-gap closure + entity migration) is one phase, not two,
  because the two are entangled for green-ness.** Hardening `view()` to see
  inline-comment headers is precisely what makes the latent backlog fallout
  *visible* to the corpus invariant — do it without stripping the entities and
  the invariant goes red. Equally, removing the `name == "056"` hardcode and
  adding the slice no-`[relationships]`-header assertion only land green once
  SL-056 is stripped. So the hardened parser, the entity strip (backlog +
  SL-056), the edge-preserving `link` of IMP-045, and the strengthened corpus
  invariant must arrive together. The phase opens with a re-scan (EN-2): the
  fallout list already grew 7→10 during design, so the actual set is discovered
  at execution and measured against the D1 cutover rule before committing to
  link+strip over a one-shot migrator.

- **PHASE-03 (guidance) last, and independent.** It points at the corrected
  surface, so it follows the fix; but it touches no code path the earlier phases
  do (memory, `using-doctrine.md`, `plugins/` skill sources), so it carries no
  ordering risk of its own.

The TDD grain within each phase is red/green/refactor: PHASE-01 writes the
scaffold-output test against the stale templates (red) then fixes them (green);
PHASE-02 hardens `view()` (red — latent items surface) then strips the entities
(green). The behaviour-preservation gate (SL-048 / SL-046 / relation / cordage
suites) rides every phase unchanged — the proof the machinery is untouched.

## Notes

- Build/use the fresh dev binary, never the stale installed one
  (`mem.pattern.build.jail-target-redirect`); re-embed needs a crate touch
  (`mem.pattern.embed.rustembed-recompile-and-symlinks`).
- Leftover `.worktrees/` registrations are a false-RED confound (touch
  `tests/*.rs` before trusting a corpus walk) — out of scope, user-owned.
- D1 cutover rule (design §7) governs PHASE-02's mechanism choice; revisit if the
  re-scan finds populated non-backlog fallout or >1 populated migrated key.
